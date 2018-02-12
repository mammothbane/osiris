use multiboot2::BootInformation;
pub use self::area_frame_allocator::AreaFrameAllocator;
pub use self::frame::Frame;
pub use self::nop_allocator::NopAllocator;
pub use self::paging::{PhysicalAddr, remap_kernel, VirtualAddr};
use self::paging::ActivePageTable;
pub use self::stack_allocator::Stack;

mod area_frame_allocator;
mod paging;
mod stack_allocator;
mod frame;
mod nop_allocator;
pub mod heap_allocator;

pub const PAGE_SIZE: usize = 4096;
pub const KERNEL_BASE: VirtualAddr = 0xffff_8000_0000_0000; // higher half
pub const VGA_BASE: usize = 0xb8000;

pub fn init(boot_info: &BootInformation) -> ActivePageTable {
    assert_has_not_been_called!("memory::init must only be called once");
    let mmap_tag = boot_info.memory_map_tag().expect("memory map tag required");

    println!("memory areas:");
    for area in mmap_tag.memory_areas() {
        println!("    start: {:#x}, length: {:#x}", area.base_addr, area.length);
    }

    let elf_sections_tag = boot_info.elf_sections_tag().expect("elf sections required");

    println!("\nkernel sections:");
    for section in elf_sections_tag.sections().filter(|s| s.is_allocated() && s.size > 0) {
        println!("    addr: {:#x}, size: {:#x}, flags: {:#b}", section.addr, section.size, section.flags);
    }

    let kernel_start = elf_sections_tag.sections()
        .filter(|s| s.is_allocated() && s.size > 0)
        .map(|s| s.addr)
        .min().unwrap();

    let kernel_end = elf_sections_tag.sections()
        .filter(|s| s.is_allocated() && s.size > 0)
        .map(|s| s.addr)
        .max().unwrap();

    println!("\nkernel start: {:#x}, end: {:#x}", kernel_start, kernel_end);
    println!("multiboot start: {:#x}, end: {:#x}", boot_info.start_address(), boot_info.end_address());

    let mut active_table = remap_kernel(boot_info);
}

pub fn finish_init(mut active_table: ActivePageTable, boot_info: &BootInformation) {
    use self::paging::Page;
    use {HEAP_START, HEAP_SIZE, BOOT_INFO};

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize,
        boot_info.start_address(), boot_info.end_address(), mmap_tag.memory_areas()
    );

    let heap_start_page = Page::containing_addr(HEAP_START);
    let heap_end_page = Page::containing_addr(HEAP_START + HEAP_SIZE - 1);

    Page::range_inclusive(heap_start_page, heap_end_page)
        .for_each(|p| active_table.map(p, paging::WRITABLE, &mut frame_allocator));

    let stack_allocator = {
        let stack_start = heap_end_page + 1;
        let stack_end = stack_start + 100;
        let stack_alloc_range = Page::range_inclusive(stack_start, stack_end);

        stack_allocator::StackAllocator::new(stack_alloc_range)
    };

    MemoryController {
        active_table,
        frame_allocator,
        stack_allocator,
    }
}

//#[no_mangle]
//#[link(name = "mem_init_cleanup")]
//extern "C" fn cleanup(active_table: ActivePageTable) -> ! {
//    use self::paging::Page;
//    use {HEAP_START, HEAP_SIZE};
//
//    let heap_start_page = Page::containing_addr(HEAP_START);
//    let heap_end_page = Page::containing_addr(HEAP_START + HEAP_SIZE - 1);
//
//    Page::range_inclusive(heap_start_page, heap_end_page)
//        .for_each(|p| active_table.map(p, paging::WRITABLE, &mut frame_allocator));
//
//    let stack_allocator = {
//        let stack_start = heap_end_page + 1;
//        let stack_end = stack_start + 100;
//        let stack_alloc_range = Page::range_inclusive(stack_start, stack_end);
//
//        stack_allocator::StackAllocator::new(stack_alloc_range)
//    };
//
//    MemoryController {
//        active_table,
//        frame_allocator,
//        stack_allocator,
//    }
//}

pub trait FrameAllocator {
    fn alloc(&mut self) -> Option<Frame>;
    fn release(&mut self, frame: Frame);
}

pub struct MemoryController {
    active_table: paging::ActivePageTable,
    frame_allocator: AreaFrameAllocator,
    stack_allocator: stack_allocator::StackAllocator,
}

impl MemoryController {
    pub fn alloc_stack(&mut self, size_in_pages: usize) -> Option<Stack> {
        let &mut MemoryController {
            ref mut active_table,
            ref mut frame_allocator,
            ref mut stack_allocator
        } = self;

        stack_allocator.alloc(active_table, frame_allocator, size_in_pages)
    }
}