use super::FrameAllocator;
use memory::Frame;

pub struct NopAllocator;

impl FrameAllocator for NopAllocator {
    fn alloc(&mut self) -> Option<Frame> {
        None
    }

    fn release(&mut self, _: Frame) {}
}
