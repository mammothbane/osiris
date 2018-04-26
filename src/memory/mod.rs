use multiboot2::BootInformation;

pub use self::frame::Frame;
pub use self::frame_allocator::*;
pub use self::frame_set::*;
pub use self::paging::{PhysicalAddr, VirtualAddr};
pub use self::stack_allocator::Stack;

use self::frame_allocator::AreaFrameAllocator;
use self::mem_info::MemoryInfo;
use lateinit::LateInit;
use bootinfo::BootInfo;
use fixedvec::FixedVec;

mod paging;
mod stack_allocator;
mod frame;
mod mem_info;
pub mod frame_set;
pub mod frame_allocator;
pub mod bump_allocator;

pub const PAGE_SIZE: usize = 4096;
pub const KERNEL_BASE: VirtualAddr = 0xffff_8000_0000_0000; // higher half
pub const VGA_BASE: usize = 0xb8000;

pub const HEAP_OFFSET: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

const BOOT_INFO_PTR: *const BootInfo = 0xb0071f0000 as *const _;

pub static HEAP_START: LateInit<VirtualAddr> = LateInit::new();
pub static BOOT_MEM_INFO: LateInit<MemoryInfo> = LateInit::new();

pub fn init() -> MemoryController {
    use self::paging::{Page, ActivePageTable};
    use self::frame_allocator::AreaFrameAllocator;
    use super::HEAP_ALLOCATOR;

    assert_has_not_been_called!("memory::init must only be called once");

    let boot_info = unsafe { BOOT_INFO_PTR.as_ref().unwrap() };

    println!("physical memory regions:");
    boot_info.memory_map.iter().for_each(|region| {
        println!("    {:>12?}: {:#x}-{:#x}", region.region_type, region.range.start_addr(), region.range.end_addr())
    });

    let mem = alloc_stack!([MemoryRegion; 4]);
    let mut kernel_regions = FixedVec::new(&mut mem);

    boot_info.memory_map.iter()
        .filter(|reg| reg.region_type == ::bootinfo::MemoryRegionType::Kernel)
        .clone_into(&mut vec);

    assert_eq!(kernel_regions.len(), 1);



    let mut active_table = unsafe { ActivePageTable::new() };

    let heap_start = (kernel_end % 4096 + 8192) as usize;

    let heap_start_page = Page::containing_addr(heap_start);
    let heap_end_page = Page::containing_addr(heap_start + HEAP_SIZE - 1);

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

    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_OFFSET, HEAP_OFFSET + HEAP_SIZE);
        BOOT_MEM_INFO.init(boot_info.into());
        HEAP_START.init(heap_start as VirtualAddr);
    }

    let frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
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