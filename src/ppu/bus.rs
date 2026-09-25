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
        let vram_index = usize::from(address) & 0x0FFF;
        let name_table = vram_index / 0x0400;
        let offset = vram_index & 0x3FF;

        use ScreenMirroring::*;
        let page = match self.cartridge.mirroring() {
            Vertical => name_table & 1,
            Horizontal => name_table >> 1,
            SingleScreenLower => 0,
            SingleScreenUpper => 1,
            FourScreen => panic!("Four screen mirroring not supported.. yet."),
        };

        page * 0x400 + offset
    }
}
