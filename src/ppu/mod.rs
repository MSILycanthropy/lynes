use crate::{cartridge::ScreenMirroring, NES};

pub(crate) mod registers;

pub trait PPU {
    fn ppu_clock(&mut self, cycles: usize) -> bool;
    fn ppu_read(&mut self) -> u8;
    fn ppu_write(&mut self, value: u8);

    fn ppu_write_address(&mut self, data: u8);
    fn ppu_write_control(&mut self, data: u8);
    fn ppu_write_oam_address(&mut self, data: u8);
    fn ppu_write_oam_data(&mut self, data: u8);
    fn ppu_write_oam_dma(&mut self, buffer: &[u8; 256]);
    fn ppu_write_mask(&mut self, data: u8);
    fn ppu_write_scroll(&mut self, data: u8);

    fn ppu_read_status(&mut self) -> u8;
    fn ppu_read_oam_data(&mut self) -> u8;

    fn background_palette(&self, tile_x: usize, tile_y: usize) -> [u8; 4];
    fn sprite_palette(&self, index: usize) -> [u8; 4];
    fn mirror_vram_address(&self, address: u16) -> u16;
}

impl PPU for NES {
    fn ppu_clock(&mut self, cycles: usize) -> bool {
        self.ppu_cycles += cycles * 3;

        if self.ppu_cycles >= 341 {
            self.ppu_cycles = self.ppu_cycles - 341;
            self.ppu_scanline += 1;

            if self.ppu_scanline == 241 {
                self.ppu_registers.status.set_vblank_started(true);
                self.ppu_registers.status.set_sprite_zero_hit(true);

                if self.ppu_registers.control.generate_nmi() {
                    self.next_interrupt = Some(crate::Interrupt::NMI)
                }
            }

            if self.ppu_scanline >= 262 {
                self.ppu_scanline = 0;
                self.next_interrupt = None;
                self.ppu_registers.status.set_sprite_zero_hit(false);
                self.ppu_registers.status.set_vblank_started(false);

                return true;
            }
        }

        return false;
    }

    fn ppu_read(&mut self) -> u8 {
        let address = self.ppu_registers.address.as_u16();

        self.ppu_registers.increment_vram_address();

        match address {
            0..=0x1FFF => {
                let result = self.ppu_read_buffer;
                self.ppu_read_buffer = self.chr_rom[address as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.ppu_read_buffer;
                self.ppu_read_buffer = self.ppu_vram[self.mirror_vram_address(address) as usize];
                result
            }
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
        let nmi_status_before = self.ppu_registers.control.generate_nmi();

        self.ppu_registers.control.update(data);

        let nmi_status_after = self.ppu_registers.control.generate_nmi();

        if !nmi_status_before && nmi_status_after && self.ppu_registers.status.vblank_started() {
            self.next_interrupt = Some(crate::Interrupt::NMI)
        }
    }

    fn ppu_write_mask(&mut self, data: u8) {
        self.ppu_registers.mask.update(data);
    }

    fn ppu_write_scroll(&mut self, data: u8) {
        self.ppu_registers.scroll.update(data)
    }

    fn ppu_write_oam_address(&mut self, data: u8) {
        self.ppu_registers.oam_addr = data;
    }

    fn ppu_write_oam_data(&mut self, data: u8) {
        self.oam_data[self.ppu_registers.oam_addr as usize] = data;
        self.ppu_registers.oam_addr = self.ppu_registers.oam_addr.wrapping_add(1);
    }

    fn ppu_write_oam_dma(&mut self, buffer: &[u8; 256]) {
        for data in buffer.iter() {
            self.ppu_write_oam_data(*data);
        }
    }

    fn ppu_read_status(&mut self) -> u8 {
        let status = self.ppu_registers.status.clone();
        let data = *status.into_bytes().first().unwrap();

        self.ppu_registers.status.set_vblank_started(false);
        self.ppu_registers.address.reset_latch();
        self.ppu_registers.scroll.reset_latch();

        data
    }

    fn ppu_read_oam_data(&mut self) -> u8 {
        self.oam_data[self.ppu_registers.oam_addr as usize]
    }

    fn background_palette(&self, tile_x: usize, tile_y: usize) -> [u8; 4] {
        let attribute_table_index = tile_y / 4 * 8 + tile_x / 4;
        let attibute_table_value = self.ppu_vram[0x3c0 + attribute_table_index];

        let palette_table_index = match (tile_x % 4 / 2, tile_y % 4 / 2) {
            (0, 0) => (attibute_table_value >> 0) & 0b11,
            (1, 0) => (attibute_table_value >> 2) & 0b11,
            (0, 1) => (attibute_table_value >> 4) & 0b11,
            (1, 1) => (attibute_table_value >> 6) & 0b11,
            _ => unreachable!(),
        } as usize;
        let palette_start = palette_table_index * 4 + 1;

        [
            self.palette_table[0],
            self.palette_table[palette_start],
            self.palette_table[palette_start + 1],
            self.palette_table[palette_start + 2],
        ]
    }

    fn sprite_palette(&self, index: usize) -> [u8; 4] {
        let palette_index = self.oam_data[index + 2] & 0b11;
        let palette_start = 0x11 + (palette_index * 4) as usize;

        [
            0,
            self.palette_table[palette_start],
            self.palette_table[palette_start + 1],
            self.palette_table[palette_start + 2],
        ]
    }

    fn mirror_vram_address(&self, address: u16) -> u16 {
        let mirrored_vram = address & 0b10111111111111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x0400;

        match (&self.mirroring, name_table) {
            (ScreenMirroring::Vertical, 2) | (ScreenMirroring::Vertical, 3) => vram_index - 0x800,
            (ScreenMirroring::Horizontal, 2) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 1) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }
}
