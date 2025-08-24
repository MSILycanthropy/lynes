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

pub struct ViewPortRect {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl ViewPortRect {
    pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Self {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        }
    }

    pub fn point_is_bounded(&self, x: usize, y: usize) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }
}
