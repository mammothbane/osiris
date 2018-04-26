pub use self::active_page_table::ActivePageTable;
pub use self::entry::*;
pub use self::inactive_page_table::InactivePageTable;
pub use self::page::{Page, PageIter};

mod page;
mod entry;
mod table;
mod mapper;
mod temporary_page;
mod inactive_page_table;
mod active_page_table;

const ENTRY_COUNT: usize = 512;

pub type PhysicalAddr = usize;
pub type VirtualAddr = usize;
