use super::FrameAllocator;
use memory::Frame;
use memory::frame_set::EmptyIterator;

pub struct NopFrameAllocator;

impl FrameAllocator<'static> for NopFrameAllocator {
    type FrameIter = EmptyIterator;

    fn alloc(&mut self) -> Option<Frame> { None }
    fn release(&mut self, frame: Frame) {}
    fn allocated_frames(&self) -> Self::FrameIter {
        EmptyIterator
    }
}
