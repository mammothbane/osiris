use core::convert::{Into, From};
use core::ops::Try;
use alloc::Vec;

use super::{Frame, FrameSet, FrameSetMut};
use memory::frame::IFrame;


/// A simple FrameSet implementation backed by a vector. This SHOULD NOT be used for recording
/// actual frame allocation data except temporarily in bootstrapping situations.
#[derive(Debug)]
pub struct VecFrameSet {
    frames: Vec<Frame>,
}

#[derive(Debug, Clone, Copy, Fail)]
pub enum VecFrameSetErr {
    #[fail(display = "invalid frame: {}", index)]
    InvalidFrame { index: usize },
}

impl VecFrameSet {
    pub fn new() -> Self {
        VecFrameSet {
            frames: vec![],
        }
    }
}

impl <T: Into<Vec<Frame>>> From<T> for VecFrameSet {
    fn from(t: T) -> Self {
        VecFrameSet {
            frames: t.into(),
        }
    }
}

impl FrameSet for VecFrameSet {
    fn contains(&self, frame: &Frame) -> bool {
        self.frames.contains(frame)
    }
}

impl FrameSetMut for VecFrameSet {
    type Err = VecFrameSetErr;

    fn add(&mut self, frame: Frame) -> Result<(), VecFrameSetErr> {
        self.frames.push(frame);
        Ok(())
    }

    fn remove(&mut self, frame_index: usize) -> Result<Frame, VecFrameSetErr> {
        let result = {
            self.frames.iter()
                .enumerate()
                .find(|(_, f)| f.index() == frame_index)
                .into_result()
        };

        result
            .map(|(idx, _)| self.frames.remove(idx))
            .map_err(|_| VecFrameSetErr::InvalidFrame { index: frame_index })
    }
}
