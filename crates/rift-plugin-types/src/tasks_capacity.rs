/// Configures the capacity of each cross-thread task queue, in entries.
///
/// `gui_param_changes` sizes the audio/main → GUI queue, `param_events` sizes
/// the GUI → audio parameter-event queue, and `context_menu` sizes the
/// GUI → main context-menu queue. Context-menu requests are user-driven and
/// rare, so the presets keep that queue much smaller than the other two.
pub struct TaskCapacity {
    pub gui_param_changes: usize,
    pub param_events: usize,
    pub context_menu: usize,
}

const SMALL: usize = 2048;
const LARGE: usize = 8192;
const EXTRA_LARGE: usize = 32768;
const CONTEXT_MENU: usize = 32;

impl TaskCapacity {
    pub const fn same(size: usize) -> Self {
        Self {
            gui_param_changes: size,
            param_events: size,
            context_menu: size,
        }
    }

    pub const fn small() -> Self {
        Self::same(SMALL).context_menu_size(CONTEXT_MENU)
    }

    pub const fn large() -> Self {
        Self::same(LARGE).context_menu_size(CONTEXT_MENU)
    }

    pub const fn extra_large() -> Self {
        Self::same(EXTRA_LARGE).context_menu_size(CONTEXT_MENU)
    }

    pub const fn gui_param_changes_size(mut self, size: usize) -> Self {
        self.gui_param_changes = size;
        self
    }

    pub const fn param_events_size(mut self, size: usize) -> Self {
        self.param_events = size;
        self
    }

    pub const fn context_menu_size(mut self, size: usize) -> Self {
        self.context_menu = size;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_keep_the_context_menu_queue_small() {
        let small = TaskCapacity::small();
        assert_eq!(small.gui_param_changes, SMALL);
        assert_eq!(small.param_events, SMALL);
        assert_eq!(small.context_menu, CONTEXT_MENU);

        let extra_large = TaskCapacity::extra_large();
        assert_eq!(extra_large.gui_param_changes, EXTRA_LARGE);
        assert_eq!(extra_large.param_events, EXTRA_LARGE);
        assert_eq!(extra_large.context_menu, CONTEXT_MENU);
    }
}
