use super::{FrameSet, FrameSetMut, Frame};

use memory::frame::IFrame;

#[derive(Clone, Copy, Debug)]
pub struct EmptyFrameSet;

impl FrameSet for EmptyFrameSet {
    fn contains(&self, _: &Frame) -> bool {
        false
    }
}

impl FrameSetMut for EmptyFrameSet {
    type Err = ();

    fn add(&mut self, frame: Frame) -> Result<(), ()> {
        Ok(())
    }

    fn remove(&mut self, frame_index: usize) -> Result<Frame, ()> {
        Ok(Frame::containing_addr(0))
    }
}
