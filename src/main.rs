use std::{env, error::Error, num::NonZeroU32};

use lynes::{NES, cartridge::Cartridge, living_room::LivingRoom, tv::wgpu::WgpuTV};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let rom_path = args.next().ok_or("Usage: lynes <rom.nes>")?;

    if args.next().is_some() {
        return Err("Usage: lynes <rom.nes>".into());
    }

    if rom_path == "--help" || rom_path == "-h" {
        println!("Usage: lynes <rom.nes>");
        return Ok(());
    }

    let mut nes = NES::default();
    nes.insert_cart(Cartridge::load(&rom_path));
    nes.reset();

    let mut living_room = LivingRoom::<WgpuTV>::new(nes);
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut living_room)?;

    Ok(())
}
