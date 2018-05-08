pub use self::frame::Frame;
pub use self::frame_allocator::*;
pub use self::frame_set::*;
pub use self::paging::{PhysicalAddr, VirtualAddr};
pub use self::stack_allocator::Stack;

use self::frame_allocator::AreaFrameAllocator;
use self::paging::{Page, ActivePageTable};
use lateinit::LateInit;
use bootinfo::{BootInfo, MemoryMap, MemoryRegion, MemoryRegionType};
use fixedvec::FixedVec;

mod paging;
mod stack_allocator;
mod frame;
pub mod frame_set;
pub mod frame_allocator;
pub mod bump_allocator;

pub const PAGE_SIZE: usize = 4096;
pub const VGA_BASE: usize = 0xb8000;

pub static HEAP_START: LateInit<VirtualAddr> = LateInit::new();
pub const HEAP_INIT_SIZE: usize = 1024 * PAGE_SIZE;
pub const HEAP_SIZE: usize = 100 * HEAP_INIT_SIZE;

const BOOT_INFO_PTR: *const BootInfo = 0xb0071f0000 as *const _;

pub const KERNEL_BASE: VirtualAddr = 0xffff_8000_0000_0000; // higher half
pub static KERNEL_MAX: LateInit<VirtualAddr> = LateInit::new();

pub static MEMORY_MAP: LateInit<MemoryMap> = LateInit::new();

pub fn init() -> MemoryController {
    use self::frame_allocator::AreaFrameAllocator;
    use super::HEAP_ALLOCATOR;

    assert_has_not_been_called!("memory::init must only be called once");

    let boot_info = unsafe { BOOT_INFO_PTR.as_ref().unwrap() };
    let mut memory_map = boot_info.memory_map.clone();

    println!("physical memory regions:");
    memory_map.iter().for_each(|region| {
        println!("    {:>12?}: {:#x}-{:#x}", region.region_type, region.range.start_addr(), region.range.end_addr())
    });
    println!();

    let mut active_table = unsafe { ActivePageTable::new() };

    unmap_bootloader(&mut active_table, &mut memory_map);

    let mut mem: [MemoryRegion; 4] = unsafe { ::core::mem::uninitialized() };
    let mut kernel_regions = FixedVec::<MemoryRegion>::new(&mut mem);

    memory_map.iter()
        .filter(|reg| reg.region_type == MemoryRegionType::Kernel)
        .for_each(|reg| kernel_regions.push(*reg).unwrap());

    assert_eq!(kernel_regions.len(), 1);
    let kernel_range = kernel_regions[0].range;

    let kernel_size = (kernel_range.end_addr() - kernel_range.start_addr()) as usize;
    let kernel_end = KERNEL_BASE + kernel_size;

    println!("kernel end identified at {:#x}", kernel_end);

    unsafe { KERNEL_MAX.init(kernel_end + PAGE_SIZE - (kernel_end % PAGE_SIZE)); }
    println!("KERNEL_MAX: {:#x}", *KERNEL_MAX);

    unsafe { HEAP_START.init(*KERNEL_MAX + PAGE_SIZE); }

    let heap_start_page = Page::containing_addr(*HEAP_START);
    let heap_end_page = Page::containing_addr(*HEAP_START + HEAP_INIT_SIZE - 1);

    println!("mapping heap in range: {:#x} - {:#x}", *HEAP_START, *HEAP_START + HEAP_INIT_SIZE - 1);

    let last_tmp_frame = {
        let mut tmp_alloc = AreaFrameAllocator::new(
            memory_map.clone(),
            EmptyFrameSet,
        );

        Page::range_inclusive(heap_start_page, heap_end_page)
            .for_each(|p| active_table.map(p, paging::WRITABLE, &mut tmp_alloc));

        tmp_alloc.next_free()
    };

    unsafe { HEAP_ALLOCATOR.lock().init(*HEAP_START, HEAP_INIT_SIZE); }

    // map APIC after heap
    map_apic(&mut active_table, heap_end_page + 1, &mut memory_map);

    let mut frame_allocator = AreaFrameAllocator::new(
        memory_map.clone(),
        VecFrameSet::new(),
    );
    frame_allocator.set_start_frame(last_tmp_frame);

    let stack_allocator = {
        let stack_start = heap_end_page + 1_000_000;
        let stack_end = stack_start + 100;
        let stack_alloc_range = Page::range_inclusive(stack_start, stack_end);

        stack_allocator::StackAllocator::new(stack_alloc_range)
    };

    unsafe { MEMORY_MAP.init(memory_map) };

    MemoryController {
        active_table,
        frame_allocator,
        stack_allocator,
    }
}

fn map_apic(active_table: &mut ActivePageTable, apic_page: Page, memory_map: &mut MemoryMap) {
    use io::apic::{APIC_PHYS, APIC_VIRT};
    use self::paging::{WRITABLE, NX, NO_CACHE, PRESENT, WRITE_THROUGH};

    const APIC_PHYS_U64: u64 = APIC_PHYS as u64;

    println!("mapping APIC to {:#x}", apic_page.start_addr());
    unsafe { APIC_VIRT.init(apic_page.start_addr()) };

    let create_region = {
        let apic_region = memory_map.iter()
            .find(|reg| {
                reg.range.start_addr() <= APIC_PHYS_U64 &&
                    reg.range.end_addr() >= APIC_PHYS_U64 + (PAGE_SIZE as u64)
            });

        if let Some(apic_region) = apic_region {
            assert_eq!(apic_region.region_type, MemoryRegionType::Reserved);
            false
        } else {
            true
        }
    };

    if create_region {
        use bootinfo::FrameRange;

        println!("creating physical region for APIC");

        memory_map.add_region(MemoryRegion {
            range: FrameRange::new(APIC_PHYS_U64, APIC_PHYS_U64 + (PAGE_SIZE as u64)),
            region_type: MemoryRegionType::Reserved,
        });
    }

    let apic_frame = Frame::containing_addr(APIC_PHYS);

    // NOTE: the page needs to be mapped strong uncacheable (UC) according to the intel system programming guide
    // by default this means we need PAT3 or PAT7 => cache disable + write-through
    // PLEASE GOD NEVER TOUCH THE PAT MSR
    // see 4-34, Vol. 3A (paging), 11-35 (programming the PAT)
    active_table.map_to(apic_page, apic_frame, WRITABLE | NX | NO_CACHE | PRESENT | WRITE_THROUGH, &mut NopAllocator)
}

fn unmap_bootloader(active_table: &mut ActivePageTable, memory_map: &mut MemoryMap) {
    // bootloader is identity mapped, unmap it
    memory_map.iter_mut()
        .filter(|reg| reg.region_type == MemoryRegionType::Bootloader)
        .for_each(|reg| {
            let start_page = Page::containing_addr(reg.range.start_addr() as usize);
            let end_page = Page::containing_addr(reg.range.end_addr() as usize - 1);

            Page::range_inclusive(start_page, end_page)
                .for_each(|p| active_table.unmap(p, &mut NopAllocator));

            reg.region_type = MemoryRegionType::Usable;
        });
}

// extend the heap to its full capacity. should only run after page fault handler is enabled.
pub fn extend_heap() {
    use super::HEAP_ALLOCATOR;
    unsafe { HEAP_ALLOCATOR.lock().extend(HEAP_SIZE - HEAP_INIT_SIZE) }
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