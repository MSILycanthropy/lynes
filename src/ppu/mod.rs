use crate::{cartridge::ScreenMirroring, NES};

pub(crate) mod registers;

pub trait PPU {
    fn ppu_clock(&mut self, cycles: usize);
    fn ppu_read(&mut self) -> u8;
    fn ppu_write(&mut self, value: u8);

    fn ppu_write_address(&mut self, data: u8);
    fn ppu_write_control(&mut self, data: u8);

    fn mirror_vram_address(&self, address: u16) -> u16;
}

impl PPU for NES {
    fn ppu_clock(&mut self, cycles: usize) {
        self.ppu_cycles += cycles * 3;

        if self.ppu_cycles >= 341 {
            self.ppu_cycles = self.ppu_cycles - 341;
            self.ppu_scanline += 1;

            if self.ppu_scanline == 241 {
                if self.ppu_registers.control.generate_nmi() {
                    self.ppu_registers.status.set_vblank_started(true);
                    self.next_interrupt = Some(crate::Interrupt::NMI)
                }
            }
        }
    }

    fn ppu_read(&mut self) -> u8 {
        let address = self.ppu_registers.address.as_u16();
        self.ppu_registers.increment_vram_address();

        match address {
            0..=0x1FFF => todo!("Can't read from chr_rom yet"),
            0x2000..=0x2FFF => todo!("Can't read from vram yet"),
            0x3000..=0x3EFF => {
                unreachable!(
                    "0x3000..0x3EFF shouldnt be used, attempted to use {}",
                    address
                )
            }
            0x3F00..=0x3FFF => self.palette_table[(address - 0x3F00) as usize],
            _ => unreachable!("attempted to access mirrored address space {}", address),
        }
    }

    fn ppu_write(&mut self, value: u8) {
        let address = self.ppu_registers.address.as_u16();
        match address {
            0..=0x1fff => println!("attempt to write to chr rom space {}", address),
            0x2000..=0x2fff => {
                self.ppu_vram[self.mirror_vram_address(address) as usize] = value;
            }
            0x3000..=0x3eff => unimplemented!("address {} shouldn't be used in reallity", address),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let add_mirror = address - 0x10;
                self.palette_table[(add_mirror - 0x3f00) as usize] = value;
            }
            0x3f00..=0x3fff => {
                self.palette_table[(address - 0x3f00) as usize] = value;
            }
            _ => panic!("unexpected access to mirrored space {}", address),
        }

        self.ppu_registers.increment_vram_address();
    }

    fn ppu_write_address(&mut self, data: u8) {
        self.ppu_registers.address.update(data);
    }

    fn ppu_write_control(&mut self, data: u8) {
        self.ppu_registers.address.update(data);
    }

    fn mirror_vram_address(&self, address: u16) -> u16 {
        let mirrored_vram = address & 0b10111111111111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x0400;

        match (&self.mirroring, name_table) {
            (ScreenMirroring::Vertical, 2) | (ScreenMirroring::Vertical, 3) => vram_index - 0x0800,
            (ScreenMirroring::Horizontal, 2) | (ScreenMirroring::Horizontal, 1) => {
                vram_index - 0x0400
            }
            (ScreenMirroring::Horizontal, 3) => vram_index - 0x0800,
            _ => vram_index,
        }
    }
}
