use super::{FrameSet, Frame};

pub struct EmptyFrameSet;

impl FrameSet for EmptyFrameSet {
    fn contains(&self, frame: &Frame) -> bool {
        false
    }
}
