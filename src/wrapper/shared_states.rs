use std::sync::atomic::{AtomicU32, Ordering};

use crossbeam_queue::ArrayQueue;
use hug_accumulator::AudioAccumulator;

use crate::{
    context::{AudioThreadTask, MainThreadTask},
    prelude::BLOCK_SIZE,
};

const TASKS_CAPACITY: usize = 2048;

pub(crate) struct PluginSharedState {
    pub(crate) latency: AtomicU32,

    /// Queues that audio / main thread can read
    pub(crate) main_thread_tasks: ArrayQueue<MainThreadTask>,
    pub(crate) audio_thread_tasks: ArrayQueue<AudioThreadTask>,

    /// Audio accumulators
    pub(crate) audio_accumulators: Vec<AudioAccumulator<{ BLOCK_SIZE }>>,
}

impl PluginSharedState {
    pub fn new() -> Self {
        Self {
            latency: AtomicU32::new(0),
            main_thread_tasks: ArrayQueue::new(TASKS_CAPACITY),
            audio_thread_tasks: ArrayQueue::new(TASKS_CAPACITY),
            audio_accumulators: vec![],
        }
    }

    pub fn add_accumulator(mut self, acc: AudioAccumulator<BLOCK_SIZE>) -> Self {
        self.audio_accumulators.push(acc);
        self
    }

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
