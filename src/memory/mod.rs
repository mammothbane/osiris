pub use self::frame::Frame;
pub use self::frame_allocator::*;
pub use self::frame_set::*;
pub use self::paging::{PhysicalAddr, VirtualAddr};
pub use self::stack_allocator::Stack;

use self::frame_allocator::AreaFrameAllocator;
use lateinit::LateInit;
use bootinfo::{BootInfo, MemoryMap, MemoryRegion};
use fixedvec::FixedVec;

mod paging;
mod stack_allocator;
mod frame;
pub mod frame_set;
pub mod frame_allocator;
//pub mod bump_allocator;

pub const PAGE_SIZE: usize = 4096;
pub const VGA_BASE: usize = 0xb8000;

pub static HEAP_START: LateInit<VirtualAddr> = LateInit::new();
pub const HEAP_SIZE: usize = 100 * 1024;

const BOOT_INFO_PTR: *const BootInfo = 0xb0071f0000 as *const _;

pub const KERNEL_BASE: VirtualAddr = 0xffff_8000_0000_0000; // higher half
pub static KERNEL_MAX: LateInit<VirtualAddr> = LateInit::new();

pub static MEMORY_MAP: LateInit<MemoryMap> = LateInit::new();

pub fn init() -> MemoryController {
    use self::paging::{Page, ActivePageTable};
    use self::frame_allocator::AreaFrameAllocator;
    use super::HEAP_ALLOCATOR;

    assert_has_not_been_called!("memory::init must only be called once");

    let boot_info = unsafe { BOOT_INFO_PTR.as_ref().unwrap() };
    unsafe { MEMORY_MAP.init(boot_info.memory_map.clone()) }

    println!("physical memory regions:");
    MEMORY_MAP.iter().for_each(|region| {
        println!("    {:>12?}: {:#x}-{:#x}", region.region_type, region.range.start_addr(), region.range.end_addr())
    });
    println!();

    let mut mem: [MemoryRegion; 4] = unsafe { ::core::mem::uninitialized() };
    let mut kernel_regions = FixedVec::<MemoryRegion>::new(&mut mem);

    MEMORY_MAP.iter()
        .filter(|reg| reg.region_type == ::bootinfo::MemoryRegionType::Kernel)
        .for_each(|reg| kernel_regions.push(*reg).unwrap());

    assert_eq!(kernel_regions.len(), 1);
    let kernel_range = kernel_regions[0].range;

    let kernel_size = (kernel_range.end_addr() - kernel_range.start_addr()) as usize;
    let kernel_end = KERNEL_BASE + kernel_size;

    println!("kernel end identified at {:#x}", kernel_end);

    unsafe { KERNEL_MAX.init(kernel_end + PAGE_SIZE - (kernel_end % PAGE_SIZE)); }
    println!("KERNEL_MAX: {:#x}", *KERNEL_MAX);

    let mut active_table = unsafe { ActivePageTable::new() };

    unsafe { HEAP_START.init(*KERNEL_MAX + PAGE_SIZE); }

    let heap_start_page = Page::containing_addr(*HEAP_START);
    let heap_end_page = Page::containing_addr(*HEAP_START + HEAP_SIZE - 1);

    println!("mapping heap in range: {:#x} - {:#x}", *HEAP_START, *HEAP_START + HEAP_SIZE - 1);

    {
        let mut ary: [Frame; 2048] = unsafe { ::core::mem::uninitialized() };

        let mut tmp_alloc = AreaFrameAllocator::new(
            MEMORY_MAP.clone(),
            StackFrameSet::new(&mut ary),
        );

        Page::range_inclusive(heap_start_page, heap_end_page)
            .for_each(|p| active_table.map(p, paging::WRITABLE, &mut tmp_alloc));
    }

    unsafe { HEAP_ALLOCATOR.lock().init(*HEAP_START, HEAP_SIZE); }

    let frame_allocator = AreaFrameAllocator::new(
        MEMORY_MAP.clone(),
        VecFrameSet::new(),
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