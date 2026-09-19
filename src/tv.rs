pub mod chafa;

#[cfg(feature = "wgpu")]
pub mod wgpu;

use crate::frame::Frame;

pub trait TV {
    type Error;

    fn present(&mut self, frame: &Frame) -> Result<(), Self::Error>;
}
