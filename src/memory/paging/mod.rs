use core::ops::{Deref, DerefMut};
use memory::{Frame, FrameAllocator, PAGE_SIZE};
use memory::frame::IFrame;
use multiboot2::BootInformation;
pub use self::active_page_table::ActivePageTable;
pub use self::entry::*;
use self::inactive_page_table::IInactivePageTable;
pub use self::inactive_page_table::InactivePageTable;
use self::mapper::Mapper;
pub use self::page::{Page, PageIter};
use self::page::IPage;
use self::temporary_page::TemporaryPage;

mod page;
mod entry;
mod table;
mod mapper;
mod temporary_page;
mod inactive_page_table;
mod active_page_table;


const ENTRY_COUNT: usize = 512;

pub type PhysicalAddr = usize;
pub type VirtualAddr = usize;

pub fn remap_kernel<A: FrameAllocator>(alloc: &mut A, boot_info: &BootInformation) -> ActivePageTable {
    let mut temp_page = TemporaryPage::new(Page::new_from_index(0xcafebabe ), alloc);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = alloc.alloc().expect("out of frames");
        InactivePageTable::new(frame, &mut active_table, &mut temp_page)
    };

    active_table.with(&mut new_table, &mut temp_page, |mapper| {
        let elf_sects_tag = boot_info.elf_sections_tag()
            .expect("memory map tag required");

        for section in elf_sects_tag.sections() {
            if !section.is_allocated() || section.size == 0 {
                continue;
            }

            // TODO: skip empty sections (?)

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
         old_table.p4_frame().start_addr()
    );

    active_table.unmap(old_p4, alloc);
    println!("guard page at {:#x}", old_p4.start_addr());

    active_table
}


