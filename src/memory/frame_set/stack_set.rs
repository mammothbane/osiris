use core::ops::Try;

use itertools::Itertools;

use super::{Frame, FrameSet, FrameSetMut};
use memory::frame::IFrame;

/// A simple FrameSet implementation backed by stack-allocated storage.
#[derive(Debug)]
pub struct StackFrameSet<'a> {
    ary: &'a mut [Frame],
    current_index: usize,
}

#[derive(Debug, Fail)]
pub enum StackFrameSetError {
    #[fail(display = "invalid frame: {}", index)]
    InvalidFrame {
        index: usize,
    },

    #[fail(display = "not enough capacity")]
    Capacity
}

impl <'a> StackFrameSet<'a> {
    pub fn new(ary: &'a mut [Frame]) -> Self {
        StackFrameSet {
            ary,
            current_index: 0,
        }
    }
}

impl <'a> FrameSet<'a> for StackFrameSet<'a> {
    type Iter = ::core::slice::Iter<'a, Frame>;

    fn contains(&self, frame: &Frame) -> bool {
        (&self.ary[0..self.current_index]).contains(frame)
    }

    fn iter(&self) -> Self::Iter {
        self.ary.iter()
    }
}

impl <'a> FrameSetMut<'a> for StackFrameSet<'a> {
    type Err = StackFrameSetError;

    fn add(&mut self, frame: Frame) -> Result<(), StackFrameSetError> {
        if self.current_index >= self.ary.len() {
            return Err(StackFrameSetError::Capacity);
        }

        self.ary[self.current_index] = frame;
        self.current_index += 1;

        Ok(())
    }

    fn remove(&mut self, frame_index: usize) -> Result<Frame, StackFrameSetError> {

        let index = (&self.ary[0..self.current_index]).iter()
            .find_position(|f| f.index() == frame_index)
            .map(|(i, _)| i);

        index
            .map(|i| {
                use core::mem;

                let f: Frame = unsafe { mem::replace(&mut self.ary[i], mem::uninitialized()) };

                // just bubble up--this doesn't have to be particularly performant
                for j in i..self.current_index {
                    self.ary.swap(j, j+1)
                }

                self.current_index -= 1;

                f
            })
            .into_result()
            .map_err(|_| StackFrameSetError::InvalidFrame { index: frame_index })
    }
}
