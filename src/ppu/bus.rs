use crate::cartridge::{Cartridge, ScreenMirroring};

pub(crate) struct PpuBus<'a> {
    pub(crate) cartridge: &'a mut Cartridge,
    pub(crate) ciram: &'a mut [u8; 2048],
}

impl PpuBus<'_> {
    pub(crate) fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.cartridge.ppu_read(address),
            0x2000..=0x3EFF => self.ciram[self.mirror_vram_address(address)],
            _ => panic!("invalid ppu bus read at {address:#06X}"),
        }
    }

    pub(crate) fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => self.cartridge.ppu_write(address, value),
            0x2000..=0x3EFF => self.ciram[self.mirror_vram_address(address)] = value,
            _ => panic!("invalid ppu bus write to {address:#06X}"),
        }
    }

    fn mirror_vram_address(&self, address: u16) -> usize {
        let mirrored_vram = address & 0b10111111111111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x0400;

        let index = match (&self.cartridge.screen_mirroring, name_table) {
            (ScreenMirroring::Vertical, 2) | (ScreenMirroring::Vertical, 3) => vram_index - 0x800,
            (ScreenMirroring::Horizontal, 2) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 1) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        };

        usize::from(index)
    }
}
