use crate::color::Color;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 240;

#[derive(Clone, Debug)]
pub struct Frame(pub Vec<u8>);

impl Frame {
    pub fn new() -> Self {
        Self(vec![0; SCREEN_WIDTH * SCREEN_HEIGHT * 3])
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let index = (y * SCREEN_WIDTH + x) * 3;

        if index + 2 >= self.0.len() {
            return;
        }

        self.0[index] = color.0;
        self.0[index + 1] = color.1;
        self.0[index + 2] = color.2;
    }
}

pub trait Renderer {
    fn render(&mut self, frame: &Frame);
}
pub struct SDLRenderer<'a> {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture: sdl2::render::Texture<'a>
}

impl<'a> SDLRenderer<'a> {
    pub fn new(canvas: sdl2::render::Canvas<sdl2::video::Window>, texture: sdl2::render::Texture<'a>) -> Self {
        Self {
            canvas,
            texture
        }
    }
}

impl<'a> Renderer for SDLRenderer<'a> {
    fn render(&mut self, frame: &Frame) {
        self.texture
            .update(None, &frame.0, SCREEN_WIDTH * 3)
            .unwrap();

        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }
}
