use std::ops::Deref;

use crate::dev_prelude::*;

#[derive(Copy, Clone)]
pub struct FrameSignal {
    write: WriteSignal<()>,
    read: ReadSignal<()>,
}

impl Model for FrameSignal {}

impl Deref for FrameSignal {
    type Target = ReadSignal<()>;

    fn deref(&self) -> &Self::Target {
        &self.read
    }
}

impl FrameSignal {
    pub(crate) fn new() -> Self {
        let base = Signal::new(());
        Self {
            write: base.write_only(),
            read: base.read_only(),
        }
    }

    pub(crate) fn install(self, cx: &mut Context) {
        self.build(cx);
    }

    pub(crate) fn tick(&self) {
        // Just set to tick change
        self.write.set(())
    }

    pub fn signal(cx: &mut Context) -> Self {
        *cx.try_data::<Self>()
            .expect("FrameSignal wasn't install in the app")
    }
}
