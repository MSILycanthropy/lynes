use crate::frame::Frame;

pub trait TV {
    type Error;

    fn present(&mut self, frame: &Frame) -> Result<(), Self::Error>;
}
