use super::{Frame, FrameAllocator};

pub struct NopAllocator;

impl FrameAllocator for NopAllocator {
    fn alloc(&mut self) -> Option<Frame> { None }
    fn release(&mut self, frame: Frame) {}
}
