use super::{Frame, FrameSet, FrameSetError};

#[derive(Debug, Clone)]
pub struct StackFrameSet<'a> {
    ary: &'a mut [Frame],
    current_index: usize,
}

impl <'a> StackFrameSet<'a> {
    fn new(ary: &'a mut [Frame]) -> Self {
        StackFrameSet {
            ary,
            current_index: 0,
        }
    }
}

impl <'a> FrameSet for StackFrameSet<'a> {
    type Error = FrameSetError;

    fn contains(&self, frame: Frame) -> bool {
        (&self.ary[0..self.current_index]).slice_contains(frame)
    }

    fn add(&mut self, frame: Frame) -> Result<(), FrameSetError> {
        if self.current_index >= self.ary.len() {
            return Err(FrameSetError::Capacity);
        }

        self.ary[self.current_index] = frame;
        self.current_index += 1;

        Ok(())
    }

    fn remove(&mut self, frame_index: usize) -> Result<Frame, FrameSetError> {
        self.ary.iter()
            .find(|f| f.index() == frame_index)
            .into_result()
            .map_err(|_| FrameSetError::InvalidFrame { index: frame_index })
    }
}
