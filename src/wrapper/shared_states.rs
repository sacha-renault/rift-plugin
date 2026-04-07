use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crossbeam_queue::ArrayQueue;

use crate::context::{AudioThreadTask, MainThreadTask};

/// Shared state for an audio plugin accessed by both the main and audio threads.
///
/// Contains bounded queues for posting tasks to the opposite thread.
/// If a queue is full when pushing a task, the push operation fails and returns
/// the unposted task in an `Err`.
pub(crate) struct SharedQueues {
    pub(crate) latency: AtomicU32,
    pub(crate) is_playing: Arc<AtomicBool>,

    /// Queues that audio / main thread can read
    pub(crate) main_thread_tasks: ArrayQueue<MainThreadTask>,
    pub(crate) audio_thread_tasks: ArrayQueue<AudioThreadTask>,
}

impl SharedQueues {
    pub fn new(task_capacity: usize) -> Self {
        Self {
            latency: AtomicU32::new(0),
            is_playing: Arc::new(AtomicBool::new(false)),
            main_thread_tasks: ArrayQueue::new(task_capacity),
            audio_thread_tasks: ArrayQueue::new(task_capacity),
        }
    }

    /// Updates the plugin's processing latency in samples.
    #[inline]
    pub fn set_latency(&self, latency: u32) {
        self.latency.store(latency, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_latency(&self) -> u32 {
        self.latency.load(Ordering::Relaxed)
    }

    /// Posts a task to be executed by the main thread from the audio thread.
    ///
    /// Returns an error if the [`Self::main_thread_tasks`] queue is full.
    #[inline]
    pub fn push_main_thread_task(&self, task: MainThreadTask) -> Result<(), MainThreadTask> {
        self.main_thread_tasks.push(task)
    }

    pub fn pop_main_thread_tasks(&self) -> Option<MainThreadTask> {
        self.main_thread_tasks.pop()
    }

    /// Posts a task to be executed by the audio thread from the audio thread.
    ///
    /// Returns an error if the [`Self::audio_thread_tasks`] queue is full.
    #[inline]
    pub fn push_audio_thread_task(&self, task: AudioThreadTask) -> Result<(), AudioThreadTask> {
        self.audio_thread_tasks.push(task)
    }

    pub fn pop_audio_thread_tasks(&self) -> Option<AudioThreadTask> {
        self.audio_thread_tasks.pop()
    }
}
