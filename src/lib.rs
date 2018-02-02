#![feature(lang_items)]
#![no_std]

#![feature(unique)]
#![feature(ptr_internals)]
#![feature(const_fn)]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;

#[macro_use]
mod vga_buffer;
mod memory;

#[no_mangle]
pub extern fn rust_main(multiboot_info: usize) {
    vga_buffer::clear_screen();

    let boot_info = unsafe { multiboot2::load(multiboot_info) };
    let mmap_tag = boot_info.memory_map_tag().expect("memory map tag required");

    println!("memory areas:");
    for area in mmap_tag.memory_areas() {
        println!("    start: 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
    }

    let elf_sections_tag = boot_info.elf_sections_tag().expect("elf sections required");

    println!("\nkernel sections:");
    for section in elf_sections_tag.sections() {
        println!("    addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}", section.addr, section.size, section.flags);
    }

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr).max().unwrap();

    let mb_start = multiboot_info;
    let mb_end = mb_start + (boot_info.total_size as usize);

    println!();
    println!("kernel start: {}, end: {}", kernel_start, kernel_end);
    println!("multiboot start: {}, end: {}", mb_start, mb_end);

    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize,
        mb_start, mb_end, mmap_tag.memory_areas()
    );

    use memory::FrameAllocator;
    println!("\n{:?}", frame_allocator.alloc());


    for i in 0.. {
        if let None = frame_allocator.alloc() {
            println!("allocated {} frames", i);
            break;
        }
    }

    loop{}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\n\npanic in {} at line {}:", file, line);
    println!("    {}", fmt);
    loop {}
}
