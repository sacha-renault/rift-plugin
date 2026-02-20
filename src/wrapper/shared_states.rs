use std::sync::atomic::{AtomicU32, Ordering};

use crossbeam_queue::ArrayQueue;

use crate::context::{AudioThreadTask, MainThreadTask};

const TASKS_CAPACITY: usize = 2048;

pub(crate) struct PluginSharedState {
    latency: AtomicU32,

    /// Queues that audio / main thread can read
    main_thread_tasks: ArrayQueue<MainThreadTask>,
    audio_thread_tasks: ArrayQueue<AudioThreadTask>,
}

impl Default for PluginSharedState {
    fn default() -> Self {
        Self {
            latency: AtomicU32::new(0),
            main_thread_tasks: ArrayQueue::new(TASKS_CAPACITY),
            audio_thread_tasks: ArrayQueue::new(TASKS_CAPACITY),
        }
    }
}

impl PluginSharedState {
    #[inline]
    pub fn set_latency(&self, latency: u32) {
        self.latency.store(latency, Ordering::Relaxed);
    }

    #[inline]
    pub fn get_latency(&self) -> u32 {
        self.latency.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn push_main_thread_task(&self, task: MainThreadTask) -> Result<(), MainThreadTask> {
        self.main_thread_tasks.push(task)
    }

    pub fn pop_main_thread_tasks(&self) -> Option<MainThreadTask> {
        self.main_thread_tasks.pop()
    }

    #[inline]
    pub fn push_audio_thread_task(&self, task: AudioThreadTask) -> Result<(), AudioThreadTask> {
        self.audio_thread_tasks.push(task)
    }

    pub fn pop_audio_thread_tasks(&self) -> Option<AudioThreadTask> {
        self.audio_thread_tasks.pop()
    }
}
