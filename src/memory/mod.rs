mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::AreaFrameAllocator;
pub use self::paging::test_paging;

use self::paging::PhysicalAddr;

pub const PAGE_SIZE: usize = 4096;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    index: usize,
}

impl Frame {
    fn containing_addr(addr: usize) -> Frame {
        Frame{ index: addr / PAGE_SIZE }
    }

    fn start_addr(&self) -> PhysicalAddr {
        self.index * PAGE_SIZE
    }
}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn release(&mut self, frame: Frame);
}
