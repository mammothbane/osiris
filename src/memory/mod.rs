mod area_frame_allocator;

pub use self::area_frame_allocator::AreaFrameAllocator;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    index: usize,
}

impl Frame {
    fn containing_addr(addr: usize) -> Frame {
        Frame{ index: addr / PAGE_SIZE }
    }
}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn release(&mut self, frame: Frame);
}
