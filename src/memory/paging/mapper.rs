use core::ptr::Unique;

use memory::{Frame, FrameAllocator, PAGE_SIZE};
use memory::frame_set::VecFrameSet;

use super::{ENTRY_COUNT, Page, PhysicalAddr, VirtualAddr};
use super::entry::*;
use super::table::{self, Level4, Table};

pub struct Mapper {
    p4: Unique<Table<Level4>>,
}

impl Mapper {
    pub unsafe fn new() -> Mapper {
        Mapper {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    pub fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    pub fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
        where A: FrameAllocator
    {
        let frame = allocator.alloc().expect("no free frames");

        self.map_to(page, frame, flags, allocator);
    }

    pub fn identity_map<A>(&mut self, frame: Frame, flags: EntryFlags, alloc: &mut A)
        where A: FrameAllocator
    {
        let page = Page::containing_addr(frame.start_addr());
        self.map_to(page, frame, flags, alloc)
    }

    pub fn unmap<A>(&mut self, page: Page, allocator: &mut A)
        where A: FrameAllocator
    {
        assert!(self.translate(page.start_addr()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("no support for huge pages");

        let frame = p1[page.p1_index()].pointed_frame().unwrap();
        p1[page.p1_index()].set_unused();

        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;

        tlb::flush(VirtualAddress(page.start_addr()));

        // TODO: free page table(s) if empty

        allocator.release(frame);
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A) where A: FrameAllocator {
        let p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let p2 = p3.next_table_create(page.p3_index(), allocator);
        let p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }


    pub fn translate(&self, virt_addr: VirtualAddr) -> Option<PhysicalAddr> {
        let offset = virt_addr % PAGE_SIZE;
        self.translate_page(Page::containing_addr(virt_addr))
            .map(|frame| frame.index() * PAGE_SIZE + offset)
    }


    pub fn translate_page(&self, page: Page) -> Option<Frame> {
        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(HUGE_PAGE) {
                        assert_eq!(start_frame.index() % (ENTRY_COUNT * ENTRY_COUNT), 0);
                        return Some(Frame::new(
                            start_frame.index() + page.p2_index() * ENTRY_COUNT + page.p1_index()
                        ))
                    }
                }

                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];

                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(HUGE_PAGE) {
                            assert_eq!(start_frame.index() % ENTRY_COUNT, 0);
                            return Some(Frame::new(start_frame.index() + page.p1_index()));
                        }
                    }
                }

                None
            })

        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(huge_page)
    }

    /// Scan the page table and return allocated frames. This is unsafe for several reasons:
    ///     - there could potentially be garbage in the table
    ///     - we could run out of memory doing this
    ///     - we're inferring things from memory that we maybe ought not to
    /// NOTE: we must have a heap before calling this function
    pub unsafe fn recover_frames(&self) -> VecFrameSet {
        use memory::frame_set::VecFrameSet;
        use alloc::Vec;
        use core::convert::From;

        // TODO: optimize to reduce allocation

        let p4_frames = self.p4().iter().filter_map(|e| e.pointed_frame());
        let p3s = self.p4().children();

        let p3_frames = p3s.iter().flat_map(|p3| p3.iter().filter_map(|e| e.pointed_frame()));
        let p2s = p3s.iter().flat_map(|p3| p3.children()).collect::<Vec<&Table<_>>>();

        let p2_frames = p2s.iter().flat_map(|p2| p2.iter().filter_map(|e| e.pointed_frame()));
        let p1s = p2s.iter().flat_map(|p2| p2.children());

        let p1_frames = p1s.flat_map(|p1| p1.iter().filter_map(|e| e.pointed_frame()));

        let result = p4_frames.chain(p3_frames).chain(p2_frames).chain(p1_frames).collect::<Vec<_>>();

        VecFrameSet::from(result)
    }
}
