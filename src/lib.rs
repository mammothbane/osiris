#![feature(lang_items)]
#![no_std]

#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(unique)]
#![feature(ptr_internals)]
#![feature(const_fn)]
#![feature(abi_x86_interrupt)]

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
extern crate x86_64;
extern crate linked_list_allocator;

#[macro_use] extern crate once;
#[macro_use] extern crate alloc;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate lazy_static;

#[macro_use]
mod vga_buffer;
mod memory;
mod interrupts;

use linked_list_allocator::LockedHeap;

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

fn enable_nx() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | 1 << 11);
    }
}

fn enable_write_protect() {
    use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}

#[no_mangle]
pub extern "C" fn rust_main(multiboot_info: usize) {
    vga_buffer::clear_screen();

    let boot_info = unsafe { multiboot2::load(multiboot_info) };

    enable_nx();
    enable_write_protect();
    memory::init(boot_info);

    println!();

    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    interrupts::init();
    x86_64::instructions::interrupts::int3();

    println!("\n\nHalting normally.");
    unsafe { x86_64::instructions::halt(); }
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\n\npanic in {} at line {}:", file, line);
    println!("    {}", fmt);

    unsafe { x86_64::instructions::halt(); }
    loop {}
}
