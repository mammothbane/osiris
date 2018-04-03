use super::{FrameSet, Frame};

#[derive(Clone, Copy, Debug)]
pub struct EmptyFrameSet;

impl FrameSet for EmptyFrameSet {
    fn contains(&self, _: &Frame) -> bool {
        false
    }
}
