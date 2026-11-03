use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{Receiver, Sender, TrySendError, bounded};

/// Runs `Job`s on a dedicated thread and hands their results back to the caller.
///
/// [`Self::post`] never allocate because channel are bounded. This is safe for RT thread.
pub struct BackgroundExecutor<Job, Res, Data> {
    jobs: Option<Sender<Job>>,
    results: Option<Receiver<Res>>,
    handle: Option<JoinHandle<()>>,
    data: Arc<Data>,
}

pub type StatelessExecutor<Job, Res> = BackgroundExecutor<Job, Res, ()>;

impl<Job, Res, Data> BackgroundExecutor<Job, Res, Data> {
    /// Spawns the worker thread.
    ///
    /// `jobs_capacity` bounds how many unprocessed jobs may queue and
    /// `results_capacity` how many results may wait to be collected;
    pub fn new<F>(data: Data, jobs_capacity: usize, results_capacity: usize, handler: F) -> Self
    where
        F: FnMut(Job, &Data) -> Res + Send + 'static,
        Job: Send + 'static,
        Res: Send + 'static,
        Data: Send + Sync + 'static,
    {
        let data = Arc::new(data);
        Self::from_data(data, jobs_capacity, results_capacity, handler)
    }

    /// Create a new BackgroundExecutor with data already being a arc
    ///
    /// See [`Self::new`]
    pub fn from_data<F>(
        data: Arc<Data>,
        jobs_capacity: usize,
        results_capacity: usize,
        handler: F,
    ) -> Self
    where
        F: FnMut(Job, &Data) -> Res + Send + 'static,
        Job: Send + 'static,
        Res: Send + 'static,
        Data: Send + Sync + 'static,
    {
        let (jobs, jobs_rx) = bounded(jobs_capacity.max(1));
        let (results_tx, results) = bounded(results_capacity.max(1));

        let thread_data = data.clone();
        let handle = thread::spawn(move || run(jobs_rx, results_tx, thread_data, handler));

        Self {
            jobs: Some(jobs),
            results: Some(results),
            handle: Some(handle),
            data,
        }
    }

    /// Enqueues `job`.
    ///
    /// # Errors
    ///
    /// Returns `Err(job)` if the queue is full or the worker is gone, so the
    /// caller keeps ownership and can retry.
    pub fn post(&self, job: Job) -> Result<(), Job> {
        let Some(jobs) = &self.jobs else {
            return Err(job);
        };
        match jobs.try_send(job) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(job)) | Err(TrySendError::Disconnected(job)) => Err(job),
        }
    }

    /// Pops one ready result, or `None` if there is none.
    pub fn try_recv(&self) -> Option<Res> {
        self.results.as_ref()?.try_recv().ok()
    }

    /// Drains every result currently available.
    pub fn drain(&self) -> Drain<'_, Job, Res, Data> {
        Drain { executor: self }
    }

    /// Number of results waiting to be collected.
    pub fn pending_results(&self) -> usize {
        self.results.as_ref().map_or(0, Receiver::len)
    }

    /// Access the shared state with background thread.
    pub fn states(&self) -> &Data {
        self.data.as_ref()
    }
}

impl<Job, Res> StatelessExecutor<Job, Res> {
    /// See [`Self::new`] but with empty data.
    pub fn stateless<F>(jobs_capacity: usize, results_capacity: usize, mut handler: F) -> Self
    where
        F: FnMut(Job) -> Res + Send + 'static,
        Job: Send + 'static,
        Res: Send + 'static,
    {
        let stateless_handle = move |job, _: &()| handler(job);
        Self::new((), jobs_capacity, results_capacity, stateless_handle)
    }
}

impl<Job, Res, Data> Drop for BackgroundExecutor<Job, Res, Data> {
    fn drop(&mut self) {
        // Close both channels so the worker cannot stay blocked, whether it is
        // waiting on a job or stuck sending a result; then join so it is fully
        // stopped before the fields are freed.
        drop(self.jobs.take());
        drop(self.results.take());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Iterates the results ready now and any that become ready while iterating.
pub struct Drain<'a, Job, Res, Data> {
    executor: &'a BackgroundExecutor<Job, Res, Data>,
}

impl<Job, Res, Data> Iterator for Drain<'_, Job, Res, Data> {
    type Item = Res;

    fn next(&mut self) -> Option<Res> {
        self.executor.try_recv()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.executor.pending_results();
        (len, Some(len))
    }
}

fn run<Job, Res, Data>(
    jobs: Receiver<Job>,
    results: Sender<Res>,
    data: Arc<Data>,
    mut handler: impl FnMut(Job, &Data) -> Res,
) {
    while let Ok(job) = jobs.recv() {
        // A send error means the executor was dropped: stop.
        if results.send(handler(job, data.as_ref())).is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crossbeam_channel::unbounded;

    use super::*;

    /// Polls `exec` until `expected` results are collected or `timeout` elapses.
    fn collect<Job, Res, Data>(
        exec: &BackgroundExecutor<Job, Res, Data>,
        expected: usize,
        timeout: Duration,
    ) -> Vec<Res> {
        let deadline = Instant::now() + timeout;
        let mut out = Vec::new();
        while out.len() < expected && Instant::now() < deadline {
            match exec.try_recv() {
                Some(res) => out.push(res),
                None => thread::sleep(Duration::from_millis(1)),
            }
        }
        out
    }

    #[test]
    fn runs_jobs_and_returns_results() {
        let exec = BackgroundExecutor::stateless(8, 8, |j: u32| j * 2);
        exec.post(1).unwrap();
        exec.post(2).unwrap();

        let mut got = collect(&exec, 2, Duration::from_secs(1));
        got.sort();
        assert_eq!(got, vec![2, 4]);
    }

    #[test]
    fn preserves_fifo_order() {
        let exec = BackgroundExecutor::stateless(128, 128, |j: u32| j * 2);
        for i in 0..100 {
            exec.post(i).unwrap();
        }

        let got = collect(&exec, 100, Duration::from_secs(1));
        assert_eq!(got, (0..100).map(|i| i * 2).collect::<Vec<_>>());
    }

    #[test]
    fn is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BackgroundExecutor<u32, u32, ()>>();
    }

    #[test]
    fn try_recv_is_empty_at_start() {
        let exec = BackgroundExecutor::stateless(4, 4, |j: u32| j);
        assert!(exec.try_recv().is_none());
        assert_eq!(exec.pending_results(), 0);
    }

    #[test]
    fn full_jobs_queue_returns_the_job() {
        // Capacity 1, and the first job blocks the worker so the queue can be
        // deterministically filled.
        let (started_tx, started_rx) = unbounded();
        let (release_tx, release_rx) = unbounded::<()>();

        let exec = BackgroundExecutor::stateless(1, 4, move |j: u32| {
            if j == 0 {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
            }
            j
        });

        exec.post(0).unwrap();
        // Worker now holds job 0 and is blocked, leaving the queue empty.
        started_rx.recv().unwrap();

        exec.post(1).unwrap();
        assert_eq!(exec.post(2), Err(2));

        release_tx.send(()).unwrap();
        let mut got = collect(&exec, 2, Duration::from_secs(1));
        got.sort();
        assert_eq!(got, vec![0, 1]);
    }

    #[test]
    fn drain_yields_all_available() {
        let exec = BackgroundExecutor::stateless(8, 8, |j: u32| j);
        for i in 0..4 {
            exec.post(i).unwrap();
        }

        // Give the worker a moment, then drain in one shot.
        let deadline = Instant::now() + Duration::from_secs(1);
        while exec.pending_results() < 4 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(1));
        }

        let mut got: Vec<u32> = exec.drain().collect();
        got.sort();
        assert_eq!(got, vec![0, 1, 2, 3]);
    }

    #[test]
    fn backpressure_resumes_when_consumer_drains() {
        // Results capacity (2) far below the job count: the worker blocks once
        // full and must resume as the consumer drains.
        let exec = BackgroundExecutor::stateless(16, 2, |j: u32| j);
        for i in 0..10 {
            exec.post(i).unwrap();
        }

        let mut got = collect(&exec, 10, Duration::from_secs(2));
        assert_eq!(got.len(), 10);
        got.sort();
        assert_eq!(got, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn handler_may_allocate_on_the_worker() {
        let exec = BackgroundExecutor::stateless(4, 4, |j: u32| vec![j; j as usize]);
        exec.post(3).unwrap();

        let got = collect(&exec, 1, Duration::from_secs(1));
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], vec![3, 3, 3]);
    }

    #[test]
    fn drop_joins_the_worker() {
        let exec = BackgroundExecutor::stateless(4, 4, |j: u32| j);
        exec.post(1).unwrap();
        drop(exec);
    }
}
