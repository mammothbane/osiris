use memory::{PAGE_SIZE, Frame, FrameAllocator};

use multiboot2::BootInformation;

use core::ops::{Deref, DerefMut};

mod entry;
mod table;
mod mapper;
mod temporary_page;

pub use self::entry::*;
use self::temporary_page::TemporaryPage;
use self::mapper::Mapper;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddr = usize;
pub type VirtualAddr = usize;

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

    pub fn range_inclusive(start: Page, end: Page) -> PageIter {
        PageIter {
            start,
            end,
        }
    }
}

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

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self,
                   table: &mut InactivePageTable,
                   temp_page: &mut TemporaryPage,
                   f: F)
        where F: FnOnce(&mut Mapper)
    {
        use x86_64::registers::control_regs;
        use x86_64::instructions::tlb;

        {
            let backup = Frame::containing_addr(control_regs::cr3().0 as usize);

            let p4_table = temp_page.map_table_frame(backup.clone(), self);

            self.p4_mut()[511].set(table.p4_frame.clone(), PRESENT | WRITABLE);
            tlb::flush_all();

            f(self);

            p4_table[511].set(backup, PRESENT | WRITABLE);
            tlb::flush_all();
        }

        temp_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86_64::PhysicalAddress;
        use x86_64::registers::control_regs;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_addr(control_regs::cr3().0 as usize),
        };

        unsafe {
            control_regs::cr3_write(PhysicalAddress(
                new_table.p4_frame.start_addr() as u64
            ))
        }

        old_table
    }
}

pub fn remap_kernel<A>(alloc: &mut A, boot_info: &BootInformation) -> ActivePageTable
    where A: FrameAllocator
{
    let mut temp_page = TemporaryPage::new(Page { index: 0xcafebabe }, alloc);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = alloc.alloc().expect("out of frames");
        InactivePageTable::new(frame, &mut active_table, &mut temp_page)
    };

    active_table.with(&mut new_table, &mut temp_page, |mapper| {
        let elf_sects_tag = boot_info.elf_sections_tag()
            .expect("memory map tag required");

        for section in elf_sects_tag.sections() {
            if !section.is_allocated() {
                continue;
            }

            // TODO: handle empty sections (?)

            assert_eq!(section.start_address() % PAGE_SIZE, 0, "sections must be page-aligned");
            println!("mapping section at addr: {:#x}, size: {:#x}", section.addr, section.size);

            let mut flags = EntryFlags::from_elf_section(section);

            let start_frame = Frame::containing_addr(section.start_address());
            let end_frame = Frame::containing_addr(section.end_address() - 1);

            Frame::range_inclusive(start_frame, end_frame)
                .for_each(|f| mapper.identity_map(f, flags, alloc));
        }

        let vga_buf_frame = Frame::containing_addr(0xb8000);
        mapper.identity_map(vga_buf_frame, WRITABLE, alloc);

        let mb_start = Frame::containing_addr(boot_info.start_address());
        let mb_end = Frame::containing_addr(boot_info.end_address());
        Frame::range_inclusive(mb_start, mb_end).for_each(|f| mapper.identity_map(f, PRESENT, alloc));
    });

    let old_table = active_table.switch(new_table);
    println!("kernel remapped.");

    let old_p4 = Page::containing_addr(
        old_table.p4_frame.start_addr()
    );

    active_table.unmap(old_p4, alloc);
    println!("guard page at {:#x}", old_p4.start_addr());

    active_table
}

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temp_page: &mut TemporaryPage) -> InactivePageTable {
        {
            // create a page for the frame, zero it, and recursive-map it
            let table = temp_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), PRESENT | WRITABLE);
        }
        temp_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }
}
