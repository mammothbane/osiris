use multiboot2::ElfSection;

use memory::Frame;
use memory::frame::IFrame;

#[derive(Debug)]
pub struct Entry(u64);

impl Entry {
    pub fn unused(&self) -> bool {
        self.0 == 0
    }

    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    pub fn pointed_frame(&self) -> Option<Frame> {
        self.pointed_addr().map(Frame::containing_addr)
    }

    pub fn set(&mut self, frame: Frame, flags: EntryFlags) {
        assert_eq!(frame.start_addr() & !0x000fffff_fffff000, 0);
        self.0 = (frame.start_addr() as u64) | flags.bits();
    }

    fn pointed_addr(&self) -> Option<usize> {
        if !self.flags().contains(PRESENT) {
            return None
        }

        Some(self.0 as usize & 0x000fffff_fffff000)
    }
}

bitflags! {
    pub struct EntryFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER_ACCESSIBLE = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
        const NX = 1 << 63;
    }
}

impl EntryFlags {
    pub fn from_elf_section(sect: &ElfSection) -> EntryFlags {
        use multiboot2::ElfSectionFlags;

        let mut flags = EntryFlags::empty();
        if sect.flags().contains(ElfSectionFlags::ALLOCATED) {
            flags |= PRESENT;
        }

        if sect.flags().contains(ElfSectionFlags::WRITABLE) {
            flags |= WRITABLE;
        }

        if !sect.flags().contains(ElfSectionFlags::EXECUTABLE) {
            flags |= NX
        }

        flags
    }
}