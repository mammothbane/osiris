use super::Frame;

mod stack_set;
mod empty_set;
mod vec_set;

pub use self::stack_set::*;
pub use self::empty_set::*;
pub use self::vec_set::*;

pub trait FrameSet {
    fn contains(&self, frame: &Frame) -> bool;
}

pub trait FrameSetMut : FrameSet {
    type Err;

    fn add(&mut self, frame: Frame) -> Result<(), Self::Err>;
    fn remove(&mut self, frame_index: usize) -> Result<Frame, Self::Err>;
}
