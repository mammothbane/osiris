use cpuid::CpuId;

use memory::PhysicalAddr;

pub const APIC_PHYS: PhysicalAddr = 0xfee0_0000;

pub fn setup_apic() {
    let cpu_info = CpuId::new();
    let feature_info = cpu_info.get_feature_info().expect("cpu feature information not available");

    if !feature_info.has_apic() {
        panic!("APIC not supported");
    }

    unsafe {
        use x86_64::registers::msr::{rdmsr, wrmsr, IA32_APIC_BASE};
        let apic_base = rdmsr(IA32_APIC_BASE);

        assert_eq!((apic_base >> 12) & 0xffffff000, APIC_PHYS);

        if apic_base & (1 << 11) == 0 {
            println!("NOTE: APIC was disabled in MSR, enabling it");
            wrmsr(IA32_APIC_BASE, apic_base & 1 << 11);
        }
    }

    

}

pub unsafe fn enable_apic() {

}

pub unsafe fn disable_apic() {

}
