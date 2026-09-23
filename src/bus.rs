use crate::{cartridge::Cartridge, input::Controller, ppu::Ppu};

pub struct CpuBus {
    pub(crate) ram: [u8; 2048],
    pub(crate) ppu: Ppu,
    pub(crate) ciram: [u8; 2048],
    pub(crate) controller: Controller,
    pub(crate) cartridge: Cartridge,
}

impl Default for CpuBus {
    fn default() -> Self {
        Self {
            ram: [0; 2048],
            ppu: Ppu::default(),
            ciram: [0; 2048],
            controller: Controller::new(),
            cartridge: Cartridge::default(),
        }
    }
}
