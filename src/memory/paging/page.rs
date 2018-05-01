use core::ops::Add;

use memory::PAGE_SIZE;
use super::VirtualAddr;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Page {
    index: usize,
}

impl Page {
    pub fn containing_addr(addr: VirtualAddr) -> Page {
        assert!(addr < 0x0000_8000_0000_0000
                    || addr >= 0xffff_8000_0000_0000,
                "invalid addr: 0x{:x}", addr);

        Page { index: addr / PAGE_SIZE }
    }

    pub fn start_addr(&self) -> usize {
        self.index * PAGE_SIZE
    }

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start,
            end,
        }
    }

    pub(crate) fn new_from_index(index: usize) -> Page {
        Page {
            index
        }
    }

    pub(crate) fn p4_index(&self) -> usize {
        (self.index >> 27) & 0o777
    }

    pub(crate) fn p3_index(&self) -> usize {
        (self.index >> 18) & 0o777
    }

    pub(crate) fn p2_index(&self) -> usize {
        (self.index >> 9) & 0o777
    }

    pub(crate) fn p1_index(&self) -> usize {
        (self.index >> 0) & 0o777
    }
}

impl Add<usize> for Page {
    type Output = Page;

    fn add(self, rhs: usize) -> Page {
        Page { index: self.index + rhs }
    }
}

#[derive(Clone, Debug)]
pub struct PageIter {
    start: Page,
    end: Page,
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start <= self.end {
            let page = self.start;
            self.start.index += 1;
            Some(page)
        } else {
            None
        }
    }
}