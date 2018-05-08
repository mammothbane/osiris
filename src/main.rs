#![no_std]
#![no_main]

#![feature(lang_items)]
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

extern crate fixedvec;
extern crate bit_field;
extern crate linked_list_allocator;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;
extern crate os_bootinfo as bootinfo;
extern crate lateinit;
extern crate raw_cpuid as cpuid;

use linked_list_allocator::LockedHeap;

#[macro_use]
mod vga_buffer;
mod memory;
mod interrupts;
mod io;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

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
    memory::extend_heap();

    enable_syscall();

    io::apic::setup_apic();

    use io::ScanCode;

    fn read_one() -> ScanCode {
        loop {
            let status = unsafe { io::inb(0x64) };
            if status & 1 == 0 {
                continue;
            }

            let data = unsafe { io::inb(0x60) };

            if status & 1 << 5 != 0 { // ignore mouse input
                continue;
            }

            let scancode = unsafe { core::mem::transmute(data) };

            if scancode == ScanCode::Extended {
                let _ = read_one();
                continue;
            }

            return scancode;
        }
    }

    loop {
        let x = read_one();
        x.ascii().map(|c| print!("{}", c as char));
    }
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("\n\npanic in {} at line {}:", file, line);
    println!("    {}", fmt);

    unsafe { x86_64::instructions::halt(); }
    loop {}
}

#[lang = "oom"]
#[no_mangle]
pub extern fn oom() -> ! {
    panic!("out of memory");
}
