use multiboot2::BootInformation;
pub use self::frame::Frame;
pub use self::paging::{PhysicalAddr, remap_kernel, VirtualAddr};
pub use self::stack_allocator::Stack;
pub use self::frame_allocator::*;
pub use self::frame_set::*;

mod paging;
mod stack_allocator;
mod frame;
pub mod frame_set;
pub mod frame_allocator;
pub mod bump_allocator;

pub const PAGE_SIZE: usize = 4096;
pub const KERNEL_BASE: VirtualAddr = 0xffff_8000_0000_0000; // higher half
pub const VGA_BASE: usize = 0xb8000;

pub const HEAP_START: usize = KERNEL_BASE + 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

fn kernel_bounds(boot_info: &BootInformation) -> (u64, u64) {
    let elf_sections_tag = boot_info.elf_sections_tag().expect("elf sections required");

    let kernel_start = elf_sections_tag.sections()
        .filter(|s| s.is_allocated() && s.size > 0)
        .map(|s| s.addr)
        .min().unwrap();

    let kernel_end = elf_sections_tag.sections()
        .filter(|s| s.is_allocated() && s.size > 0)
        .map(|s| s.addr)
        .max().unwrap();

    (kernel_start, kernel_end)
}

pub fn preinit(boot_info: &BootInformation) {
    assert_has_not_been_called!("memory::preinit must only be called once");

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

    let (kernel_start, kernel_end) = kernel_bounds(&boot_info);

    println!("\nkernel start: {:#x}, end: {:#x}", kernel_start, kernel_end);
    println!("multiboot start: {:#x}, end: {:#x}", boot_info.start_address(), boot_info.end_address());

    remap_kernel(boot_info);
}

pub fn init(boot_info: &BootInformation) -> MemoryController {
    use self::paging::{Page, ActivePageTable};
    use self::frame_allocator::AreaFrameAllocator;
    use super::HEAP_ALLOCATOR;

    assert_has_not_been_called!("memory::init must only be called once");

    // TODO: unmap unused (low) pages

    println!("in memory::init");

    let mut active_table = unsafe { ActivePageTable::new() };

    let (kernel_start, kernel_end) = kernel_bounds(&boot_info);
    let mmap_tag = boot_info.memory_map_tag().expect("memory map tag required");

//    println!("got kernel start, mmap tag");
//    unsafe { ::x86_64::instructions::halt() };

    let heap_start_page = Page::containing_addr(HEAP_START);
    let heap_end_page = Page::containing_addr(HEAP_START + HEAP_SIZE - 1);

    {
        let mut ary: [Frame; 2048] = unsafe { ::core::mem::uninitialized() };
        println!("need {} frames, have {}", HEAP_SIZE/PAGE_SIZE, ary.len());

        let mut tmp_alloc = AreaFrameAllocator::new(
            kernel_start as usize + KERNEL_BASE,
            kernel_end as usize + KERNEL_BASE,
            boot_info.start_address(), // offsets are already accounted for here
            boot_info.end_address(),
            mmap_tag.memory_areas(),
            StackFrameSet::new(&mut ary),
        );

        Page::range_inclusive(heap_start_page, heap_end_page)
            .for_each(|p| active_table.map(p, paging::WRITABLE, &mut tmp_alloc));
    }

    println!("heap pages mapped");
    unsafe { ::x86_64::instructions::halt() };

    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    println!("heap created");
    unsafe { ::x86_64::instructions::halt() };

    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize + KERNEL_BASE,
        kernel_end as usize + KERNEL_BASE,
        boot_info.start_address() + KERNEL_BASE,
        boot_info.end_address() + KERNEL_BASE,
        mmap_tag.memory_areas(),
        VecFrameSet::new(), // TODO: extract from active table
    );

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

use self::frame_allocator::AreaFrameAllocator;

pub struct MemoryController {
    active_table: paging::ActivePageTable,
    frame_allocator: AreaFrameAllocator<VecFrameSet>, // TODO: replace
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