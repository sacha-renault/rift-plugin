const SMALL: usize = 2048;
const LARGE: usize = 8192;
const EXTRA_LARGE: usize = 32768;

pub struct TaskCapacity {
    pub gui: usize,
    pub audio: usize,
    pub main: usize,
}

impl TaskCapacity {
    pub const fn same(size: usize) -> Self {
        Self {
            gui: size,
            audio: size,
            main: size,
        }
    }

    pub const fn small() -> Self {
        Self::same(SMALL)
    }

    pub const fn large() -> Self {
        Self::same(LARGE)
    }

    pub const fn extra_large() -> Self {
        Self::same(EXTRA_LARGE)
    }

    pub const fn gui_size(mut self, size: usize) -> Self {
        self.gui = size;
        self
    }

    pub const fn main_size(mut self, size: usize) -> Self {
        self.main = size;
        self
    }

    pub const fn audio_size(mut self, size: usize) -> Self {
        self.audio = size;
        self
    }
}
