use std::{error::Error, path::PathBuf};

use clap::Parser;
use lynes::{
    NES,
    cartridge::Cartridge,
    living_room::LivingRoom,
    tv::{ratatui::RatatuiTV, wgpu::WgpuTV},
};
use winit::event_loop::EventLoop;

#[derive(Parser)]
#[command(version, about = "An NES emulator with window and terminal frontends")]
struct Args {
    /// Path to the NES ROM
    rom: PathBuf,

    /// Use the terminal frontend
    #[arg(long)]
    terminal: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut nes = NES::default();
    nes.insert_cart(Cartridge::load(&args.rom));
    nes.reset();

    if args.terminal {
        LivingRoom::<RatatuiTV>::new(nes).run()?;
    } else {
        let event_loop = EventLoop::new()?;
        let mut living_room = LivingRoom::<WgpuTV>::new(nes);
        event_loop.run_app(&mut living_room)?;
    }

    Ok(())
}
