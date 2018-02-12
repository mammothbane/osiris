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
use super::NopAllocator;

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

pub fn remap_kernel(boot_info: &BootInformation) -> ActivePageTable {
    use super::{KERNEL_BASE, PAGE_SIZE};

    let elf_sections_tag = boot_info.elf_sections_tag().expect("elf sections required");

    let kernel_end = elf_sections_tag.sections()
        .filter(|s| s.is_allocated() && s.size > 0)
        .map(|s| s.addr)
        .max().unwrap();

    // TODO: this is super unsafe. at least check that these are in valid memory areas.
    let temp_frames = [
        Frame::containing_addr(kernel_end as usize + PAGE_SIZE),
        Frame::containing_addr(kernel_end as usize + PAGE_SIZE*2),
        Frame::containing_addr(kernel_end as usize + PAGE_SIZE*3)
    ];

    let mut temp_page = TemporaryPage::from_frames(
        Page::new_from_index(0xcafebabe),
        temp_frames
    );

    // TODO: see above
    let new_table_frame = Frame::containing_addr(kernel_end as usize + PAGE_SIZE*4);

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = InactivePageTable::new(new_table_frame, &mut active_table, &mut temp_page);

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

            let section_start = Frame::containing_addr(section.start_address());
            let section_end = Frame::containing_addr(section.end_address() - 1);

            Frame::range_inclusive(section_start, section_end)
                .for_each(|f| {
                    mapper.map_to(
                        Page::containing_addr(KERNEL_BASE + f.start_addr()), f.clone(), flags, alloc
                    );

                    mapper.identity_map(f, flags, alloc);
                });
        }

        let vga_buf_frame = Frame::containing_addr(0xb8000);
        mapper.map_to(
            Page::containing_addr(KERNEL_BASE + vga_buf_frame.start_addr()),
            vga_buf_frame,
            WRITABLE | PRESENT,
            alloc
        );

        let mb_start = Frame::containing_addr(boot_info.start_address());
        let mb_end = Frame::containing_addr(boot_info.end_address());
        Frame::range_inclusive(mb_start, mb_end)
            .for_each(|f| {
                let page = Page::containing_addr(KERNEL_BASE + f.start_addr());
                mapper.map_to(page, f.clone(), PRESENT, alloc);
            });
    });

    let old_table = active_table.switch(new_table);

    unsafe {
        use vga_buffer;
        vga_buffer::update_vga_base(super::VGA_BASE + KERNEL_BASE);
    }

    println!("kernel remapped.");

    let old_p4 = Page::containing_addr(
        old_table.p4_frame().start_addr()
    );

    active_table.unmap(old_p4, NopAllocator{});

    active_table
}
