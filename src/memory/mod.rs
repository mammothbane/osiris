mod area_frame_allocator;
mod paging;

pub use self::area_frame_allocator::AreaFrameAllocator;
pub use self::paging::remap_kernel;

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

    fn clone(&self) -> Frame {
        Frame { index: self.index }
    }

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter{
            start,
            end,
        }
    }
}

struct FrameIter {
    start: Frame,
    end: Frame,
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.start > self.end {
            return None
        }

        let frame = self.start.clone();
        self.start.index += 1;
        Some(frame)
    }
}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn release(&mut self, frame: Frame);
}
