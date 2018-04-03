use super::Frame;

pub use self::area_frame_allocator::*;
pub use self::bootstrap_frame_allocator::*;

mod area_frame_allocator;
mod bootstrap_frame_allocator;

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn release(&mut self, frame: Frame);
}
