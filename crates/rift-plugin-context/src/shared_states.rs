use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use rift_plugin_gui::{GuiEvent, GuiParamEvent};
use rift_plugin_params::AtomicF64;
use rift_plugin_types::{TaskCapacity, TaskQueue};
use rift_plugin_utils::DEFAULT_SAMPLERATE;

use crate::tasks::ParamContextMenu;

/// Shared state for an audio plugin accessed by both the main and audio threads.
///
/// Contains bounded queues for posting tasks to the opposite thread. Full queues
/// evict the oldest task; see [`TaskQueue`].
pub struct PluginSharedState {
    pub(crate) latency: AtomicU32,
    pub(crate) is_playing: Arc<AtomicBool>,
    pub(crate) samplerate: Arc<AtomicF64>,

    /// Queues that audio / main thread can read
    main_tasks: TaskQueue<ParamContextMenu>,
    audio_tasks: TaskQueue<GuiParamEvent>,
    gui_tasks: TaskQueue<GuiEvent>,

    gui_opened: AtomicBool,
}

impl PluginSharedState {
    pub fn new(TaskCapacity { gui, audio, main }: TaskCapacity) -> Self {
        Self {
            latency: AtomicU32::new(0),
            is_playing: Arc::new(AtomicBool::new(false)),
            samplerate: Arc::new(AtomicF64::new(DEFAULT_SAMPLERATE)),

            main_tasks: TaskQueue::new(main),
            audio_tasks: TaskQueue::new(audio),
            gui_tasks: TaskQueue::new(gui),
            gui_opened: AtomicBool::new(false),
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

    /// GUI → audio parameter events.
    #[inline]
    pub fn audio_tasks(&self) -> &TaskQueue<GuiParamEvent> {
        &self.audio_tasks
    }

    /// Audio/main → GUI updates.
    #[inline]
    pub fn gui_tasks(&self) -> &TaskQueue<GuiEvent> {
        &self.gui_tasks
    }

    /// GUI → main context-menu requests.
    #[inline]
    pub fn main_tasks(&self) -> &TaskQueue<ParamContextMenu> {
        &self.main_tasks
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    pub fn set_is_playing(&self, value: bool) {
        self.is_playing.store(value, Ordering::Relaxed);
    }

    pub fn samplerate(&self) -> f64 {
        self.samplerate.load(Ordering::Relaxed)
    }

    #[doc(hidden)]
    pub fn set_samplerate(&self, value: f64) {
        self.samplerate.store(value, Ordering::Relaxed);
    }

    pub fn is_gui_opened(&self) -> bool {
        self.gui_opened.load(Ordering::Acquire)
    }

    pub fn set_gui_opened(&self, is_opened: bool) {
        self.gui_opened.store(is_opened, Ordering::Release);
    }
}
