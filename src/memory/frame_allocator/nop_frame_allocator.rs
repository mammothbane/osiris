use super::FrameAllocator;
use memory::Frame;
use memory::frame_set::EmptyFrameSet;

pub struct NopFrameAllocator;

impl FrameAllocator for NopFrameAllocator {
    type FrameSetImpl = EmptyFrameSet;

    fn alloc(&mut self) -> Option<Frame> { None }
    fn release(&mut self, frame: Frame) {}
    fn allocated_frames(&self) -> Self::FrameSetImpl {
        EmptyFrameSet
    }
}
