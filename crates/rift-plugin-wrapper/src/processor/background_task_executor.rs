use std::{sync::mpsc, thread::JoinHandle};

use rift_plugin_types::{BackgroundTask, Task};

pub(crate) struct BackgroundThread<I, O> {
    pub(crate) thread: Option<JoinHandle<()>>,
    pub(crate) sender: mpsc::SyncSender<I>,
    pub(crate) receiver: mpsc::Receiver<O>,
}

impl<I, O> BackgroundThread<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    pub fn new<F>(capacity: usize, f: F) -> Self
    where
        F: Send + 'static,
        F: Fn(I) -> O,
    {
        // Send to thread (and receive inside)
        let (ss, sr) = mpsc::sync_channel::<I>(capacity);

        // Send result from the thread, receive in audio thread
        let (rs, rr) = mpsc::sync_channel(capacity);

        let thread = std::thread::spawn(move || {
            while let Ok(task) = sr.recv() {
                let r = f(task);
                rs.send(r).expect("Enough space in the queue");
            }
        });

        Self {
            thread: Some(thread),
            sender: ss,
            receiver: rr,
        }
    }
}

impl<I, O> Drop for BackgroundThread<I, O> {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(crate) struct BackgroundTaskExecutor {
    pub(crate) task_thread: BackgroundThread<BackgroundTask, Task>,
    pub(crate) deallocator_thread: BackgroundThread<Task, ()>,
}

impl BackgroundTaskExecutor {
    pub fn new(capacity: usize) -> Self {
        Self {
            task_thread: BackgroundThread::new(capacity, |task: BackgroundTask| task.run()),
            deallocator_thread: BackgroundThread::new(capacity, |obj| drop(obj)),
        }
    }
}
