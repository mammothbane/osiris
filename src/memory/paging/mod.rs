use core::ops::{Deref, DerefMut};
use multiboot2::BootInformation;

pub use self::active_page_table::ActivePageTable;
pub use self::entry::*;
pub use self::inactive_page_table::InactivePageTable;
pub use self::page::{Page, PageIter};

use super::{Frame, PAGE_SIZE};
use super::frame::IFrame;
use super::BootstrapFrameAllocator;
use self::mapper::Mapper;
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

pub fn cleanup(boot_info: &BootInformation) -> ActivePageTable {
    use super::PAGE_SIZE;

    let elf_sections_tag = boot_info.elf_sections_tag().expect("unable to find elf sections");
    let _max_offset = elf_sections_tag
        .sections()
        .filter(|s| s.is_allocated() && s.size() > 0)
        .map(|s| s.offset() + s.size())
        .max().expect("unable to find maximum physical offset") as usize;

    let frame_base: usize = 0x500000;
    let start_frame = Frame::containing_addr(frame_base);
    let mut alloc =  BootstrapFrameAllocator::new(start_frame);

    let mut temp_page = TemporaryPage::new(
        Page::new_from_index(0xcafebabe),
        &mut alloc
    );

    use super::FrameAllocator;

    // TODO: see above
    let new_table_frame = alloc.alloc()
        .expect("bootstrap allocator couldn't provide frame");

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = InactivePageTable::new(new_table_frame, &mut active_table, &mut temp_page);

    active_table.with(&mut new_table, &mut temp_page, |mapper| {
        mapper.identity_map(Frame::containing_addr(super::VGA_BASE), WRITABLE | PRESENT, &mut alloc);

        // TODO: unmap this as soon as we have a heap
        let mb_start = Frame::containing_addr(boot_info.start_address());
        let mb_end = Frame::containing_addr(boot_info.end_address());
        Frame::range_inclusive(mb_start, mb_end)
            .for_each(|f| mapper.identity_map(f, PRESENT, &mut alloc));

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() || section.size() == 0 || section.name() == ".boot" {
                continue;
            }

            let start_addr = section.start_address();
            let _start_addr2 = section.start_address() as usize;

            let _page = Page::containing_addr(_start_addr2);

            assert_eq!(start_addr % PAGE_SIZE as u64, 0, "sections must be page-aligned");
            println!("mapping section {} at addr: {:#x}, size: {:#x}", section.name(), start_addr, section.size());

            let mut flags = EntryFlags::from_elf_section(&section);

            let _start_addr3 = section.start_address() as usize;

            let page_start = Page::containing_addr(section.start_address() as usize);
            let page_end = Page::containing_addr(section.end_address() as usize);

            let frame_start = Frame::containing_addr(section.offset() as usize);
            let frame_end = Frame::containing_addr((section.offset() + section.size() - 1) as usize);

            Frame::range_inclusive(frame_start, frame_end)
                .zip(Page::range_inclusive(page_start, page_end))
                .for_each(|(f, p)| {
                    mapper.map_to(p, f, flags, &mut alloc);
                });
        }
    });

    let _ = active_table.switch(new_table);
    println!("kernel remapped.");

    active_table
}
