#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Debug)]
pub struct ColorPalette([Color; 64]);

impl Default for ColorPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPalette {
    pub fn new() -> Self {
        Self([Color::default(); 64])
    }

    pub fn load(filename: &str) -> Self {
        let palette_file = std::fs::read(filename).unwrap();
        Self::load_bytes(palette_file)
    }

    pub fn load_bytes(bytes: Vec<u8>) -> Self {
        let mut palette = Self::new();
        for (i, chunk) in bytes.chunks(3).enumerate() {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            palette.0[i] = Color::new(r, g, b);
        }

        palette
    }

    pub fn get(&self, index: u8) -> Color {
        self.0[index as usize]
    }

    pub fn set(&mut self, index: u8, color: Color) {
        self.0[index as usize] = color;
    }
}
