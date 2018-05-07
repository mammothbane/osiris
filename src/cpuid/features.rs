// See Intel x86 manual CPUID reference p. 3-192.

use core::mem;

bitflags! {
    pub struct FeatureInfo: u64 {
        // ECX
        const SSE3              = 1 << 0;
        const PCLMULQDQ         = 1 << 1;
        const DTES64            = 1 << 2;
        const MONITOR           = 1 << 3;
        const DS_CPL            = 1 << 4;
        const VMX               = 1 << 5;
        const SMX               = 1 << 6;
        const EIST              = 1 << 7;
        const TM2               = 1 << 8;
        const SSSE3             = 1 << 9;
        const CNXT_ID           = 1 << 10;
        const SDBG              = 1 << 11;
        const FMA               = 1 << 12;
        const PUBXCHG16B        = 1 << 13;
        const XTPR_UPDATE_CTL   = 1 << 14;
        const PDCM              = 1 << 15;
        // RESERVED
        const PCID              = 1 << 17;
        const DCA               = 1 << 18;
        const SSE4_1            = 1 << 19;
        const SSE4_2            = 1 << 20;
        const X2APIC            = 1 << 21;
        const MOVBE             = 1 << 22;
        const POPCNT            = 1 << 23;
        const TSC_DEADLINE      = 1 << 24;
        const AESNI             = 1 << 25;
        const XSAVE             = 1 << 26;
        const OSXSAVE           = 1 << 27;
        const AVX               = 1 << 28;
        const F16C              = 1 << 29;
        const RDRAND            = 1 << 30;
        // RESERVED

        // EDX
        const FPU               = 1 << 32;
        const VME               = 1 << 33;
        const DE                = 1 << 34;
        const PSE               = 1 << 35;
        const TSC               = 1 << 36;
        const MSR               = 1 << 37;
        const PAE               = 1 << 38;
        const MCE               = 1 << 39;
        const CX8               = 1 << 40;
        const APIC              = 1 << 41;
        const SEP               = 1 << 43;
        const MTRR              = 1 << 44;
        const PGE               = 1 << 45;
        const MCA               = 1 << 46;
        const CMOV              = 1 << 47;
        const PAT               = 1 << 48;
        const PSE_36            = 1 << 49;
        const PSN               = 1 << 50;
        const CLFSH             = 1 << 51;
        // RESERVED
        const DS                = 1 << 53;
        const ACPI              = 1 << 54;
        const MMX               = 1 << 55;
        const FXSR              = 1 << 56;
        const SSE               = 1 << 57;
        const SSE2              = 1 << 58;
        const SS                = 1 << 59;
        const HTT               = 1 << 60;
        const TM                = 1 << 61;
        // RESERVED
        const PBE               = 1 << 63;
    }
}


#[repr(u8)]
pub enum ProcessorType {
    OriginalOEM = 0,
    Overdrive,
    Dual,
    IntelReserved,
}

pub struct ModelInfo(u32);

impl ModelInfo {
    #[inline]
    pub fn stepping_id(&self) -> u8 {
        (self.0 & 0xf) as u8
    }

    #[inline]
    pub fn model(&self) -> u8 {
        ((self.0 >> 4) & 0xf) as u8
    }

    #[inline]
    pub fn family_id(&self) -> u8 {
        ((self.0 >> 8) & 0xf) as u8
    }

    #[inline]
    pub fn processor_type(&self) -> ProcessorType {
        unsafe { mem::transmute(((self.0 >> 12) & 0x3) as u8) }
    }

    #[inline]
    pub fn model_id(&self) -> u8 {
        ((self.0 >> 16) & 0xf) as u8
    }

    #[inline]
    pub fn extended_family_id(&self) -> u8 {
        ((self.0 >> 20) & 0xff) as u8
    }
}

pub struct MiscInfo(u32);

impl MiscInfo {
    #[inline]
    pub fn brand_index(&self) -> u8 {
        (self.0 & 0xff) as u8
    }

    #[inline]
    pub fn clflush_size(&self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    #[inline]
    pub fn max_addressable_processor_ids(&self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    #[inline]
    pub fn initial_apic_id(&self) -> u8 {
        ((self.0 >> 24) & 0xff) as u8
    }
}


pub struct CpuFeatures {
    pub misc_info: MiscInfo,
    pub model_info: ModelInfo,
    pub feature_info: FeatureInfo,
}

pub fn features() -> CpuFeatures {
    let result_low: u32;
    let result_high: u32;
    let misc_info: u32;
    let model_info: u32;

    unsafe {
        asm!("
                mov $$0x1, %eax
                cpuid
            "
            : "={eax}"(model_info), "={ebx}"(misc_info), "={ecx}"(result_low), "={edx}"(result_high)
        );

        CpuFeatures {
            misc_info: mem::transmute(misc_info),
            model_info: mem::transmute(model_info),
            feature_info: mem::transmute(((result_high as u64) << 32) | (result_low as u64))
        }
    }
}
