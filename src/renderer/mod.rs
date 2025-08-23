pub mod palette;

const FRAME_WIDTH: usize = 256;
const FRAME_HEIGHT: usize = 240;

type FrameData = [u8; FRAME_HEIGHT * FRAME_WIDTH * 3];

pub struct Frame {
    data: FrameData,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            data: [0; FRAME_HEIGHT * FRAME_WIDTH * 3],
        }
    }

    pub fn data(&self) -> &FrameData {
        return &self.data;
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let base = y * 3 * FRAME_WIDTH + x * 3;

        if base + 2 < self.data.len() {
            self.data[base] = color.0;
            self.data[base + 1] = color.1;
            self.data[base + 2] = color.2;
        }
    }
}
