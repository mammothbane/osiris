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

extern crate rlibc;
extern crate volatile;
extern crate spin;
extern crate multiboot2;
extern crate x86_64;
extern crate linked_list_allocator;
extern crate bit_field;

#[allow(unused_imports)]
#[macro_use] extern crate alloc;
#[macro_use] extern crate once;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate lazy_static;

use linked_list_allocator::LockedHeap;
use multiboot2::BootInformation;
use self::memory::KERNEL_BASE;
use spin::Once;

#[macro_use]
mod vga_buffer;
mod memory;
mod interrupts;

pub const HEAP_START: usize = KERNEL_BASE + 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

static BOOT_INFO: Once<&'static BootInformation> = Once::new();

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

//fn syscall(addr: VirtualAddr) {
//    use x86_64::registers::msr::{IA32_LSTAR, IA32_STAR};
//
//    unsafe { asm!("syscall"); }
//}



#[no_mangle]
pub extern "C" fn osiris_main(multiboot_info: usize) {
    vga_buffer::clear_screen();

    enable_nx();
    enable_write_protect();

    let mut memory_controller= memory::init(unsafe { multiboot2::load(multiboot_info) });

    BOOT_INFO.call_once(|| {
        unsafe { multiboot2::load(multiboot_info + KERNEL_BASE) }
    });

    println!();
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    interrupts::init(&mut memory_controller);

    enable_syscall();


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
