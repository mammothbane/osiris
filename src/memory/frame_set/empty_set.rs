use super::{FrameSet, FrameSetMut, Frame};

#[derive(Clone, Copy, Debug)]
pub struct EmptyFrameSet;

impl FrameSet for EmptyFrameSet {
    fn contains(&self, _: &Frame) -> bool {
        false
    }
}

impl FrameSetMut for EmptyFrameSet {
    type Err = ();

    fn add(&mut self, _: Frame) -> Result<(), ()> {
        Ok(())
    }

    fn remove(&mut self, _: usize) -> Result<Frame, ()> {
        Ok(Frame::containing_addr(0))
    }
}
