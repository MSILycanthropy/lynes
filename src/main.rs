use lynes::*;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
    EventPump,
};

fn main() {
    let (creator, mut canvas, mut event_pump) = init_sdl2();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut nes = NES::default();

    nes.start("roms/pacman.nes", move |frame, controller| {
        texture.update(None, frame.data(), 256 * 3).unwrap();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Down) => controller.button_state.set_down(true),
                    Some(Keycode::Up) => controller.button_state.set_up(true),
                    Some(Keycode::Right) => controller.button_state.set_right(true),
                    Some(Keycode::Left) => controller.button_state.set_left(true),
                    Some(Keycode::Space) => controller.button_state.set_select(true),
                    Some(Keycode::Return) => controller.button_state.set_start(true),
                    Some(Keycode::A) => controller.button_state.set_a(true),
                    Some(Keycode::S) => controller.button_state.set_b(true),
                    _ => {}
                },
                Event::KeyUp { keycode, .. } => match keycode {
                    Some(Keycode::Down) => controller.button_state.set_down(false),
                    Some(Keycode::Up) => controller.button_state.set_up(false),
                    Some(Keycode::Right) => controller.button_state.set_right(false),
                    Some(Keycode::Left) => controller.button_state.set_left(false),
                    Some(Keycode::Space) => controller.button_state.set_select(false),
                    Some(Keycode::Return) => controller.button_state.set_start(false),
                    Some(Keycode::A) => controller.button_state.set_a(false),
                    Some(Keycode::S) => controller.button_state.set_b(false),
                    _ => {}
                },
                _ => {}
            }
        }
    });
}

fn init_sdl2() -> (TextureCreator<WindowContext>, Canvas<Window>, EventPump) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Gaming", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    return (canvas.texture_creator(), canvas, event_pump);
}
