pub const FRAME_WIDTH: usize = 256;
pub const FRAME_HEIGHT: usize = 240;
pub const FRAME_STRIDE: usize = FRAME_WIDTH * 3;

pub type FrameData = [u8; FRAME_HEIGHT * FRAME_STRIDE];

#[derive(Copy, Clone)]
pub struct Frame {
    data: FrameData,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            data: [0; FRAME_HEIGHT * FRAME_STRIDE],
        }
    }

    pub fn data(&self) -> &FrameData {
        &self.data
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        if x >= FRAME_WIDTH || y >= FRAME_HEIGHT {
            return;
        }

        let base = y * FRAME_STRIDE + x * 3;
        self.data[base..base + 3].copy_from_slice(&[color.0, color.1, color.2]);
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_are_packed_rgb_in_row_order() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 0, (1, 2, 3));
        frame.set_pixel(0, 1, (4, 5, 6));
        frame.set_pixel(FRAME_WIDTH - 1, FRAME_HEIGHT - 1, (7, 8, 9));

        assert_eq!(frame.data().len(), 256 * 240 * 3);
        assert_eq!(&frame.data()[..3], &[1, 2, 3]);
        assert_eq!(&frame.data()[FRAME_STRIDE..FRAME_STRIDE + 3], &[4, 5, 6]);
        assert_eq!(&frame.data()[frame.data().len() - 3..], &[7, 8, 9]);
    }

    #[test]
    fn out_of_bounds_pixels_do_not_overwrite_other_rows() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 1, (1, 2, 3));
        let before = frame.data().to_vec();

        for (x, y) in [
            (FRAME_WIDTH, 0),
            (0, FRAME_HEIGHT),
            (usize::MAX, 0),
            (0, usize::MAX),
        ] {
            frame.set_pixel(x, y, (255, 255, 255));
        }

        assert_eq!(frame.data().as_slice(), before);
    }
}
