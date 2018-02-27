use super::{Frame, FrameSet};

pub trait FrameAllocator {
    type FrameSetImpl: FrameSet;

    fn alloc(&mut self) -> Option<Frame>;
    fn release(&mut self, frame: Frame);
    fn allocated_frames(&self) -> Self::FrameSetImpl;
}
