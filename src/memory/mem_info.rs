use core::convert::From;
use core::ops::Range;
use multiboot2::BootInformation;
use super::{PhysicalAddr, VirtualAddr};

pub struct MemoryInfo {
    areas: Vec<PhysicalArea>,
    sections: Vec<Mapping>,
}

impl From<&BootInformation> for MemoryInfo {
    fn from(boot_info: &BootInformation) -> Self {
        let sections = boot_info.elf_sections_tag().unwrap().sections()
            .filter(|s| s.name() != "boot" && s.is_allocated() && s.size() > 0)
            .map(|s| Mapping {
                virt_addr: s.addr(),
                phys_addr: s.(),
                size: s.size(),
                name: s.name().clone(),
            });





    }
}

pub struct Mapping {
    virt_addr: VirtualAddr,
    phys_addr: PhysicalAddr,
    size: usize,
    name: String,
}

impl Mapping {
    pub fn virt_start(&self) -> VirtualAddr {
        self.virt_addr
    }

    pub fn virt_end(&self) -> VirtualAddr {
        self.virt_addr + self.size
    }

    pub fn phys_start(&self) -> PhysicalAddr {
        self.phys_addr
    }

    pub fn phys_end(&self) -> PhysicalAddr {
        self.phys_addr + self.size - 1
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn name(&self) -> String {
        self.name
    }

    pub fn identity_mapped(&self) -> bool {
        self.virt_addr == self.phys_addr
    }
}

pub struct PhysicalArea {
    addr: PhysicalAddr,
    size: usize,
}

impl PhysicalArea {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn start(&self) -> PhysicalAddr {
        self.addr
    }

    pub fn end(&self) -> PhysicalAddr {
        self.addr + self.size - 1
    }
}