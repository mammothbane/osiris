use core::ops::{Deref, DerefMut};

use memory::frame::Frame;

use super::mapper::Mapper;
use super::inactive_page_table::InactivePageTable;
use super::temporary_page::TemporaryPage;
use super::entry::*;

pub struct ActivePageTable {
    mapper: Mapper,
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Mapper {
        &mut self.mapper
    }
}

impl ActivePageTable {
    pub unsafe fn new() -> ActivePageTable {
        ActivePageTable {
            mapper: Mapper::new(),
        }
    }

    pub fn with<F>(&mut self,
                   table: &mut InactivePageTable,
                   temp_page: &mut TemporaryPage,
                   f: F)
        where F: FnOnce(&mut Mapper)
    {
        use x86_64::registers::control_regs;
        use x86_64::instructions::tlb;

        {
            let backup = Frame::containing_addr(control_regs::cr3().0 as usize);

            let p4_table = temp_page.map_table_frame(backup.clone(), self);

            self.p4_mut()[511].set(table.p4_frame().clone(), PRESENT | WRITABLE);
            tlb::flush_all();

            f(self);

            p4_table[511].set(backup, PRESENT | WRITABLE);
            tlb::flush_all();
        }

        temp_page.unmap(self);
    }

    pub fn switch(&mut self, new_table: InactivePageTable) -> InactivePageTable {
        use x86_64::PhysicalAddress;
        use x86_64::registers::control_regs;

        let old_table = InactivePageTable::new_from_p4_frame(Frame::containing_addr(control_regs::cr3().0 as usize));

        unsafe {
            control_regs::cr3_write(PhysicalAddress(
                new_table.p4_frame().start_addr() as u64
            ))
        }

        old_table
    }
}
