use core::ops::{Deref, DerefMut};
use memory::{Frame, PAGE_SIZE};
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
use super::AreaFrameAllocator;
use super::StackFrameSet;

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

pub fn remap_kernel(boot_info: &BootInformation) {
    use super::{KERNEL_BASE, PAGE_SIZE};

    let (kernel_start, kernel_end) = {
        let elf_sections_tag = boot_info.elf_sections_tag().expect("elf sections required");

        let kernel_start = elf_sections_tag.sections()
            .filter(|s| s.is_allocated() && s.size > 0)
            .map(|s| s.addr)
            .min().unwrap();

        let kernel_end = elf_sections_tag.sections()
            .filter(|s| s.is_allocated() && s.size > 0)
            .map(|s| s.addr)
            .max().unwrap();

        (kernel_start as usize, kernel_end as usize)
    };

    use core::mem;
    let mut frame_ary: [Frame; 32] = unsafe { mem::uninitialized() };
    let frame_set = StackFrameSet::new(&mut frame_ary);

    let mut alloc = AreaFrameAllocator::new(
        kernel_start, kernel_end,
        boot_info.start_address(), boot_info.end_address(),
        boot_info.memory_map_tag().unwrap().memory_areas(),
        frame_set,
    );

    let mut temp_page = TemporaryPage::new(
        Page::new_from_index(0xcafebabe),
        &mut alloc
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

            // NOTE: old ELF frames are still mapped at this point
            Frame::range_inclusive(section_start, section_end)
                .for_each(|f| {
                    mapper.map_to(
                        Page::containing_addr(KERNEL_BASE + f.start_addr()), f.clone(), flags, &mut alloc
                    );

                    mapper.identity_map(f, flags, &mut alloc);
                });
        }

        let vga_buf_frame = Frame::containing_addr(0xb8000);
        mapper.map_to(
            Page::containing_addr(KERNEL_BASE + vga_buf_frame.start_addr()),
            vga_buf_frame,
            WRITABLE | PRESENT,
            &mut alloc
        );

        let mb_start = Frame::containing_addr(boot_info.start_address());
        let mb_end = Frame::containing_addr(boot_info.end_address());
        Frame::range_inclusive(mb_start, mb_end)
            .for_each(|f| {
                let page = Page::containing_addr(KERNEL_BASE + f.start_addr());
                mapper.map_to(page, f.clone(), PRESENT, &mut alloc);
            });
    });

    use x86_64::registers::control_regs;
    let old_table = InactivePageTable::new_from_p4_frame(Frame::containing_addr(control_regs::cr3().0 as usize));

    extern "C" {
        /// swaps active tables and bumps stack
        fn relocate_kernel(new_ptl4: u64, kern_base: u64);
    }

    unsafe {
        relocate_kernel(new_table.p4_frame().start_addr() as u64, KERNEL_BASE as u64);

        use vga_buffer;
        vga_buffer::update_vga_base(super::VGA_BASE + KERNEL_BASE);
    }

    println!("kernel remapped.");

    let old_p4 = Page::containing_addr(
        old_table.p4_frame().start_addr()
    );

    active_table.unmap(old_p4, &mut alloc);

    let kstart_frame = Page::containing_addr(kernel_start);
    let kend_frame = Page::containing_addr(kernel_end);

    Page::range_inclusive(kstart_frame, kend_frame)
        .for_each(|p| {
            active_table.unmap(p, &mut alloc);
        });
}
