use super::Frame;

#[derive(Debug, Fail)]
pub enum FrameSetError {
    #[fail(display = "invalid frame: {}", index)]
    InvalidFrame {
        index: usize,
    },

    #[fail(display = "not enough capacity")]
    Capacity
}

pub trait FrameSet {
    type Error;

    fn contains(&self, frame: Frame) -> bool;
    fn add(&mut self, frame: Frame) -> Result<(), Self::Error>;
    fn remove(&mut self, frame_index: usize) -> Result<Frame, Self::Error>;
}
