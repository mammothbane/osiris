use memory::{Frame, FrameAllocator};
use memory::frame::IFrame;

pub struct BootstrapFrameAllocator {
    next_free_frame: usize,
}

impl BootstrapFrameAllocator {
    pub fn new(starting_frame: Frame) -> Self {
        BootstrapFrameAllocator {
            next_free_frame: starting_frame.index(),
        }
    }
}

impl FrameAllocator for BootstrapFrameAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        let ret = Some(Frame::containing_addr(self.next_free_frame << 12));
        self.next_free_frame += 1;

        ret
    }

    fn release(&mut self, _: Frame) {}
}
