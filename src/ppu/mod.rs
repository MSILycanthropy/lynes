use crate::{NmiStatus, ScreenMirroring, NES};

pub mod registers;

pub trait PPU {
    fn ppu_clock(&mut self) -> bool;
    fn ppu_read(&mut self) -> u8;
    fn ppu_write(&mut self, data: u8);
}

impl PPU for NES {
    fn ppu_clock(&mut self) -> bool {
        self.ppu_cycle += 1;

        if self.ppu_cycle >= 341 {
            self.ppu_cycle -= 341;
            self.ppu_scanline += 1;

            if self.ppu_scanline == 241 {
                self.ppu_registers.status.set_vertical_blank(true);
                self.ppu_registers.status.set_sprite_zero_hit(false);

                if self.ppu_registers.control.generate_nmi() {
                    self.nmi_status = NmiStatus::Triggered;
                }
            }

            if self.ppu_scanline >= 262 {
                self.ppu_scanline = 0;
                self.nmi_status = NmiStatus::None;

                self.ppu_registers.status.set_vertical_blank(false);
                self.ppu_registers.status.set_sprite_zero_hit(false);

                return true;
            }
        }

        false
    }

    fn ppu_read(&mut self) -> u8 {
        let addr = self.ppu_registers.address.get();
        let vram_incr_amt = if self.ppu_registers.control.vram_address_increment() {
            32
        } else {
            1
        };

        self.ppu_registers.address.increment(vram_incr_amt);

        match addr {
            0x0000..=0x1FFF => {
                let result = self.ppu_buffer_data;
                self.ppu_buffer_data = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.ppu_buffer_data;
                let name_table_mirror = name_table_addr_mirror(addr, &self.screen_mirroring);

                self.ppu_buffer_data = self.ppu_vram[name_table_mirror as usize];
                result
            }
            0x3000..=0x3EFF => unimplemented!("Should not use these in reality!"),
            0x3f00..=0x3fff => {
                self.palette_table[(addr - 0x3f00) as usize]
            },
            _ => {
                panic!("Invalid PPU read address: {:X}", addr);
            }
        }
    }

    fn ppu_write(&mut self, data: u8) {
        let addr = self.ppu_registers.address.get();

        match addr {
            0..=0x1FFF => {
                self.chr_rom[addr as usize] = data;
            }
            0x2000..=0x2FFF => {
                let name_table_mirror = name_table_addr_mirror(addr, &self.screen_mirroring);
                self.ppu_vram[name_table_mirror as usize] = data;
            }
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                self.palette_table[(addr - 0x3f10) as usize] = data;
            }
            0x3f00..=0x3fff => {
                self.palette_table[(addr - 0x3f00) as usize] = data;
            }
            _ => {
                panic!("Invalid PPU write address: {:X}", addr);
            }
        }

        let vram_incr_amt = if self.ppu_registers.control.vram_address_increment() {
            32
        } else {
            1
        };
        self.ppu_registers.address.increment(vram_incr_amt);
    }
}

fn name_table_addr_mirror(addr: u16, mirroring: &ScreenMirroring) -> u16 {
    let mirrored_vram = addr & 0b10111111111111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
    let vram_index = mirrored_vram - 0x2000; // to vram vector
    let name_table = vram_index / 0x400;
    match (&mirroring, name_table) {
        (ScreenMirroring::Vertical, 2) | (ScreenMirroring::Vertical, 3) => vram_index - 0x800,
        (ScreenMirroring::Horizontal, 2) => vram_index - 0x400,
        (ScreenMirroring::Horizontal, 1) => vram_index - 0x400,
        (ScreenMirroring::Horizontal, 3) => vram_index - 0x800,
        _ => vram_index,
    }
}
