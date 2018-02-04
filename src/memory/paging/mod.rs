use memory::{PAGE_SIZE, Frame, FrameAllocator};
use core::ptr::Unique;

mod entry;
mod table;

pub use self::entry::*;
use self::table::{Table, Level4};

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddr = usize;
pub type VirtualAddr = usize;

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

    fn start_addr(&self) -> usize {
        self.index * PAGE_SIZE
    }

    fn p4_index(&self) -> usize {
        (self.index >> 27) & 0o777
    }

    fn p3_index(&self) -> usize {
        (self.index >> 18) & 0o777
    }

    fn p2_index(&self) -> usize {
        (self.index >> 9) & 0o777
    }

    fn p1_index(&self) -> usize {
        (self.index >> 0) & 0o777
    }
}

pub struct ActivePageTable {
    p4: Unique<Table<Level4>>,
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    pub fn translate(&self, virt_addr: VirtualAddr) -> Option<PhysicalAddr> {
        let offset = virt_addr % PAGE_SIZE;
        self.translate_page(Page::containing_addr(virt_addr))
            .map(|frame| frame.index * PAGE_SIZE + offset)
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

    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
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

        // allocator.release(frame);
    }

    pub fn map_to<A>(&mut self, page: Page, frame: Frame, flags: EntryFlags, allocator: &mut A) where A: FrameAllocator {
        let mut p3 = self.p4_mut().next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);

        assert!(p1[page.p1_index()].unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    fn translate_page(&self, page: Page) -> Option<Frame> {
        use self::entry::HUGE_PAGE;

        let p3 = self.p4().next_table(page.p4_index());

        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];

                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(HUGE_PAGE) {
                        assert_eq!(start_frame.index % (ENTRY_COUNT * ENTRY_COUNT), 0);
                        return Some(Frame {
                            index: start_frame.index + page.p2_index() * ENTRY_COUNT + page.p1_index(),
                        })
                    }
                }

                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];

                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(HUGE_PAGE) {
                            assert_eq!(start_frame.index % ENTRY_COUNT, 0);
                            return Some(Frame {
                                index: start_frame.index + page.p1_index()
                            });
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
}

pub fn test_paging<A>(allocator: &mut A)
    where A: FrameAllocator
{
    let mut page_table = unsafe { ActivePageTable::new() };

    let addr = 42 * 512 * 512 * 4096;
    let page = Page::containing_addr(addr);
    let frame = allocator.alloc().expect("no more frames");
    println!("None = {:?}, map to {:?}", page_table.translate(addr), frame);

    page_table.map_to(page, frame, EntryFlags::empty(), allocator);
    println!("Some = {:?}", page_table.translate(addr));
    println!("next free frame: {:?}", allocator.alloc());

    println!("{:#x}", unsafe {
        *(Page::containing_addr(addr).start_addr() as *const u64)
    });

    page_table.unmap(Page::containing_addr(addr), allocator);
    println!("None = {:?}", page_table.translate(addr));
}