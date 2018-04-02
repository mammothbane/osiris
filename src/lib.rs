#![feature(lang_items)]
#![no_std]

#![feature(asm)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
#![feature(unique)]
#![feature(ptr_internals)]
#![feature(const_fn)]
#![feature(abi_x86_interrupt)]
#![feature(conservative_impl_trait)]
#![feature(universal_impl_trait)]
#![feature(try_trait)]
#![feature(match_default_bindings)]

#[allow(unused_imports)]
#[macro_use] extern crate alloc;
extern crate bit_field;
#[macro_use] extern crate bitflags;
#[allow(unused_imports)]
#[macro_use] extern crate failure;
#[macro_use] extern crate failure_derive;
#[allow(unused_imports)]
#[macro_use] extern crate itertools;
#[macro_use] extern crate lazy_static;
extern crate linked_list_allocator;
extern crate multiboot2;
#[macro_use] extern crate once;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

use linked_list_allocator::LockedHeap;
use multiboot2::BootInformation;
use self::memory::KERNEL_BASE;
use spin::Once;

#[macro_use]
mod vga_buffer;
mod memory;
mod interrupts;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

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

fn enable_syscall() {
    use x86_64::registers::msr::{IA32_EFER, rdmsr, wrmsr};

    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, 1 << 0);
    }
}

#[no_mangle]
pub extern "C" fn osiris_main(multiboot_info: usize) -> ! {
    vga_buffer::clear_screen();

    enable_nx();
    enable_write_protect();

    let boot_info = unsafe { multiboot2::load(multiboot_info) };
    let mut memory_controller = memory::init(&boot_info);

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
