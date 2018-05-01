use super::PAGE_SIZE;
use super::paging::PhysicalAddr;

// TODO: see if we can remove Clone derive here
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Frame {
    index: usize,
}

impl Frame {
    pub(crate) fn new(index: usize) -> Frame {
        Frame { index }
    }

    pub(crate) fn containing_addr(addr: usize) -> Frame {
        Frame{ index: addr / PAGE_SIZE }
    }

    pub(crate) fn start_addr(&self) -> PhysicalAddr {
        self.index * PAGE_SIZE
    }

    pub(crate) fn range_inclusive(start: Frame, end: Frame) -> FrameIter {
        FrameIter{
            start,
            end,
        }
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

//    fn clone(&self) -> Self {
//        Frame { index: self.index }
//    }

    pub(crate) fn set_index(&mut self, new_index: usize) {
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
