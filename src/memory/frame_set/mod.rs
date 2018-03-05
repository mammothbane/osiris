use super::Frame;

mod stack_set;
mod empty_set;
mod vec_set;

pub use self::stack_set::*;
pub use self::empty_set::*;
pub use self::vec_set::*;

pub trait FrameSet<'a> {
    type Iter : Iterator<Item=&'a Frame>;

    fn contains(&self, frame: &Frame) -> bool;
    fn iter(&self) -> Self::Iter;
}

pub trait FrameSetMut<'a> : FrameSet<'a> {
    type Err;

    fn add(&mut self, frame: Frame) -> Result<(), Self::Err>;
    fn remove(&mut self, frame_index: usize) -> Result<Frame, Self::Err>;
}
