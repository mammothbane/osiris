use super::{Frame, FrameSet};

mod area_frame_allocator;
mod nop_frame_allocator;

pub use self::nop_frame_allocator::*;
pub use self::area_frame_allocator::*;

pub trait FrameAllocator {
    type FrameSetImpl: FrameSet;

    fn alloc(&mut self) -> Option<Frame>;
    fn release(&mut self, frame: Frame);
    fn allocated_frames(self) -> Self::FrameSetImpl;
}
