use super::{FrameSet, Frame};

#[derive(Clone, Copy, Debug)]
pub struct EmptyFrameSet;

impl FrameSet<'static> for EmptyFrameSet {
    type Iter = EmptyIterator;

    fn contains(&self, frame: &Frame) -> bool {
        false
    }

    fn iter(&self) -> Self::Iter {
        EmptyIterator
    }
}

pub struct EmptyIterator;

impl Iterator for EmptyIterator {
    type Item = &'static Frame;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
