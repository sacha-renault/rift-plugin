use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use crossbeam_queue::ArrayQueue;

use crate::context::{AudioThreadTasks, MainThreadTasks};

const TASKS_CAPACITY: usize = 2048;

pub(crate) struct PluginSharedState {
    request_restart: AtomicBool,
    latency: AtomicU32,
    main_thread_tasks: ArrayQueue<MainThreadTasks>,
    audio_thread_tasks: ArrayQueue<AudioThreadTasks>,
}

impl Default for PluginSharedState {
    fn default() -> Self {
        Self {
            request_restart: AtomicBool::new(false),
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
    pub fn push_main_thread_task(&self, task: MainThreadTasks) -> Result<(), MainThreadTasks> {
        self.main_thread_tasks.push(task)
    }

    pub fn pop_main_thread_tasks(&self) -> Option<MainThreadTasks> {
        self.main_thread_tasks.pop()
    }

    #[inline]
    pub fn push_audio_thread_task(&self, task: AudioThreadTasks) -> Result<(), AudioThreadTasks> {
        self.audio_thread_tasks.push(task)
    }

    pub fn pop_audio_thread_tasks(&self) -> Option<AudioThreadTasks> {
        self.audio_thread_tasks.pop()
    }
}
