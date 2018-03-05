use super::PAGE_SIZE;
use super::paging::PhysicalAddr;

// TODO: see if we can remove Clone derive here
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Frame {
    index: usize,
}

pub trait IFrame : Clone {
    fn new(index: usize) -> Frame;
    fn containing_addr(addr: usize) -> Frame;
    fn start_addr(&self) -> PhysicalAddr;
    fn range_inclusive(start: Frame, end: Frame) -> FrameIter;
    fn index(&self) -> usize;
    fn set_index(&mut self, new_index: usize);
}

impl IFrame for Frame {
    fn new(index: usize) -> Frame {
        Frame { index }
    }

    fn containing_addr(addr: usize) -> Frame {
        Frame{ index: addr / PAGE_SIZE }
    }

    fn start_addr(&self) -> PhysicalAddr {
        self.index * PAGE_SIZE
    }

    fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter{
            start,
            end,
        }
    }

    fn index(&self) -> usize {
        self.index
    }

    fn set_index(&mut self, new_index: usize) {
        self.index = new_index
    }
}

pub struct FrameIter {
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
