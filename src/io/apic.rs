use cpuid::CpuId;
use lateinit::LateInit;
use x86_64::registers::msr::{rdmsr, wrmsr, IA32_APIC_BASE};

use memory::{PhysicalAddr, VirtualAddr};

pub const APIC_PHYS: PhysicalAddr = 0xfee0_0000;
pub static APIC_VIRT: LateInit<VirtualAddr> = LateInit::new();  // initialized in memory::init

pub fn setup_apic() {
    let cpu_info = CpuId::new();
    let feature_info = cpu_info.get_feature_info().expect("cpu feature information not available");

    if !feature_info.has_apic() {
        panic!("APIC not supported");
    }

    unsafe {
        let apic_base = rdmsr(IA32_APIC_BASE);

        assert_eq!(apic_base & 0xffffff000, APIC_PHYS as u64);

        if apic_base & (1 << 11) == 0 {
            println!("NOTE: APIC was disabled in MSR, enabling it");
            wrmsr(IA32_APIC_BASE, apic_base | 1 << 11);
        }
    }

    let sivr = SIVR::get();
    if !sivr.apic_enabled() {
        println!("APIC was disabled by SIVR, enabling it");
        unsafe { sivr.enable_apic(); }
    }
}

pub unsafe fn enable_apic() {
    let apic_base = rdmsr(IA32_APIC_BASE);

    if apic_base & (1 << 11) == 0 {
        wrmsr(IA32_APIC_BASE, apic_base | 1 << 11);
    }

    SIVR::get().enable_apic()
}

pub unsafe fn disable_apic() {
    let apic_base = rdmsr(IA32_APIC_BASE);

    if apic_base & (1 << 11) == 1 {
        wrmsr(IA32_APIC_BASE, apic_base & !(1 << 11));
    }

    SIVR::get().disable_apic()
}

/// Spurious interrupt vector register layout
pub struct SIVR(u32);

impl SIVR {
    const OFFSET: usize = 0xf0; // offset from APIC_VIRT

    #[inline]
    pub fn get() -> &'static mut Self {
        let ptr = (*APIC_VIRT + Self::OFFSET) as *mut SIVR;
        unsafe { ptr.as_mut().unwrap() }
    }

    #[inline]
    pub fn eoi_suppress(&self) -> bool {
        ((self.0 >> 12) & 1) == 1
    }

    #[inline]
    pub fn focus_processor_checking(&self) -> bool {
        ((self.0 >> 9) & 1) == 1
    }

    #[inline]
    pub fn apic_enabled(&self) -> bool {
        ((self.0 >> 8) & 1) == 1
    }

    #[inline]
    unsafe fn disable_apic(&mut self) {
        self.0 |= 1 << 8;
    }

    #[inline]
    unsafe fn enable_apic(&mut self) {
        self.0 &= !(1 << 8);
    }
}
