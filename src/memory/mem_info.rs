use core::convert::From;
use core::ops::Range;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use multiboot2::BootInformation;

use super::{PhysicalAddr, VirtualAddr};

#[derive(Clone, Debug, Default)]
pub struct MemoryInfo {
    areas: Vec<PhysicalArea>,
    mappings: Vec<Mapping>,
}

impl MemoryInfo {
    fn mappings(&self) -> impl Iterator<Item=&Mapping> {
        self.mappings.iter()
    }

    fn physical_areas(&self) -> impl Iterator<Item=&PhysicalArea> {
        self.areas.iter()
    }
}

impl <'a> From<&'a BootInformation> for MemoryInfo {
    fn from(boot_info: &'a BootInformation) -> Self {
        let mut ret = MemoryInfo::default();

        ret.mappings = boot_info.elf_sections_tag().unwrap().sections()
            .filter(|s| s.name() != "boot" && s.is_allocated() && s.size() > 0)
            .map(|s| Mapping {
                virt_addr: s.start_address() as usize,
                phys_addr: s.offset() as usize,
                size: s.size() as usize,
                name: s.name().to_owned(),
            }).collect();

        ret.areas = boot_info.memory_map_tag().unwrap()
            .memory_areas()
            .map(|area| PhysicalArea {
                addr: area.start_address(),
                size: area.size(),
            }).collect();

        ret
    }
}

#[derive(Clone, Debug)]
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn identity_mapped(&self) -> bool {
        self.virt_addr == self.phys_addr
    }
}

#[derive(Clone, Copy, Debug)]
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