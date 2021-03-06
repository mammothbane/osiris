use memory::Frame;

use super::*;
use super::temporary_page::TemporaryPage;

pub struct InactivePageTable {
    p4_frame: Frame,
}

impl InactivePageTable {
    pub fn new(frame: Frame, active_table: &mut ActivePageTable, temp_page: &mut TemporaryPage) -> InactivePageTable {
        {
            // create a page for the frame, zero it, and recursive-map it
            let table = temp_page.map_table_frame(frame.clone(), active_table);
            table.zero();
            table[511].set(frame.clone(), PRESENT | WRITABLE);
        }
        temp_page.unmap(active_table);

        InactivePageTable { p4_frame: frame }
    }

    pub(crate) fn new_from_p4_frame(f: Frame) -> InactivePageTable {
        InactivePageTable {
            p4_frame: f,
        }
    }

    pub(crate) fn p4_frame(&self) -> &Frame {
        &self.p4_frame
    }
}
