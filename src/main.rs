#![feature(lang_items)]
#![no_std]
#![no_main]

#![feature(asm)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(unique)]
#![feature(ptr_internals)]
#![feature(const_fn)]
#![feature(abi_x86_interrupt)]
#![feature(try_trait)]

// TODO: remove
#![allow(dead_code)]

#[allow(unused_imports)] #[macro_use] extern crate alloc;
#[allow(unused_imports)] #[macro_use] extern crate itertools;
#[allow(unused_imports)] #[macro_use] extern crate failure;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate once;
#[macro_use] extern crate fixedvec;

extern crate bit_field;
extern crate linked_list_allocator;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;
extern crate os_bootinfo as bootinfo;

use linked_list_allocator::LockedHeap;

#[macro_use]
mod vga_buffer;
mod memory;
mod interrupts;
mod lateinit;

static ALLOC: LockedHeap = LockedHeap::empty();

#[global_allocator]
pub static HEAP_ALLOCATOR: &'static LockedHeap = &ALLOC;

fn enable_syscall() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | 1);
    }
}

#[no_mangle]
pub extern "C" fn osiris_main() -> ! {
    vga_buffer::clear_screen();

    let mut memory_controller = memory::init();

    interrupts::init(&mut memory_controller);
    enable_syscall();

    println!("\n\nHalting normally.");
    unsafe { x86_64::instructions::halt() };

    unreachable!();
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\n\npanic in {} at line {}:", file, line);
    println!("    {}", fmt);

    unsafe { x86_64::instructions::halt(); }
    loop {}
}
