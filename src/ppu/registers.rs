use modular_bitfield::{
    bitfield,
    specifiers::{B2, B5},
};

#[derive(Clone)]
pub struct PpuRegisters {
    pub control: Control,
    pub mask: Mask,
    pub status: Status,
    pub oam_address: u8,
    // pub oam_data: u8,
    pub scroll: Scroll,
    pub address: Address,
    // pub data: u8,
    // pub oam_dma: u8,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self {
            control: Control::default(),
            mask: Mask::default(),
            status: Status::default(),
            oam_address: 0,
            scroll: Scroll::default(),
            address: Address::default(),
            // data: 0,
            // oam_dma: 0,
        }
    }
}

#[bitfield]
#[derive(Clone)]
pub struct Control {
    pub nametable_address: B2,
    pub vram_address_increment: bool,
    pub sprite_pattern_table_address: bool,
    pub background_pattern_table_address: bool,
    pub sprite_size: bool,
    pub ppu_master_slave_select: bool,
    pub generate_nmi: bool,
}

impl Default for Control {
    fn default() -> Self {
        Self::new()
    }
}

impl Control {
    pub fn write(&mut self, data: u8) {
        self.set_nametable_address(data >> 0 & 0b11);
        self.set_vram_address_increment(data >> 2 & 1 == 1);
        self.set_sprite_pattern_table_address(data >> 3 & 1 == 1);
        self.set_background_pattern_table_address(data >> 4 & 1 == 1);
        self.set_sprite_size(data >> 5 & 1 == 1);
        self.set_ppu_master_slave_select(data >> 6 & 1 == 1);
        self.set_generate_nmi(data >> 7 & 1 == 1);
    }

    pub fn get(&self) -> u8 {
        self.clone().into_bytes()[0]
    }
}

#[bitfield]
#[derive(Clone)]
pub struct Mask {
    pub greyscale: bool,
    pub show_left_background: bool,
    pub show_left_sprites: bool,
    pub show_background: bool,
    pub show_sprites: bool,
    pub emphasize_red: bool,
    pub emphasize_green: bool,
    pub emphasize_blue: bool,
}

impl Default for Mask {
    fn default() -> Self {
        Self::new()
    }
}

impl Mask {
    pub fn write(&mut self, data: u8) {
        self.set_greyscale(data >> 0 & 1 == 1);
        self.set_show_left_background(data >> 1 & 1 == 1);
        self.set_show_left_sprites(data >> 2 & 1 == 1);
        self.set_show_background(data >> 3 & 1 == 1);
        self.set_show_sprites(data >> 4 & 1 == 1);
        self.set_emphasize_red(data >> 5 & 1 == 1);
        self.set_emphasize_green(data >> 6 & 1 == 1);
        self.set_emphasize_blue(data >> 7 & 1 == 1);
    }

    pub fn get(&self) -> u8 {
        self.clone().into_bytes()[0]
    }

    // TODO: Maybe just put this in a B3?
    pub fn emphasize(&self) -> (bool, bool, bool) {
        (
            self.emphasize_red(),
            self.emphasize_green(),
            self.emphasize_blue(),
        )
    }
}

#[bitfield]
#[derive(Clone)]
pub struct Status {
    pub unused: B5,
    pub sprite_overflow: bool,
    pub sprite_zero_hit: bool,
    pub vertical_blank: bool,
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

impl Status {
    pub fn write(&mut self, data: u8) {
        self.set_sprite_overflow(data >> 5 & 1 == 1);
        self.set_sprite_zero_hit(data >> 6 & 1 == 1);
        self.set_vertical_blank(data >> 7 & 1 == 1);
    }

    pub fn get(&self) -> u8 {
        self.clone().into_bytes()[0]
    }
}

#[derive(Clone)]
pub struct Scroll {
    pub x: u8,
    pub y: u8,
    pub write_x: bool,
}

impl Default for Scroll {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            write_x: true,
        }
    }
}

impl Scroll {
    pub fn get(&self) -> u16 {
        u16::from_le_bytes([self.x, self.y])
    }

    pub fn peek(&self) -> u8 {
        if self.write_x {
            self.x
        } else {
            self.y
        }
    }

    pub fn set(&mut self, addr: u16) {
        let [x, y] = addr.to_le_bytes();

        self.x = x;
        self.y = y;
    }

    pub fn write(&mut self, data: u8) {
        if self.write_x {
            self.x = data;
        } else {
            self.y = data;
        }

        self.write_x = !self.write_x;
    }

    pub fn reset_latch(&mut self) {
        self.write_x = true;
    }
}

#[derive(Clone)]
pub struct Address {
    pub low: u8,
    pub high: u8,
    pub write_high: bool,
}

impl Default for Address {
    fn default() -> Self {
        Self {
            low: 0,
            high: 0,
            write_high: true,
        }
    }
}

impl Address {
    pub fn get(&self) -> u16 {
        u16::from_le_bytes([self.low, self.high])
    }

    pub fn peek(&self) -> u8 {
        if self.write_high {
            self.high
        } else {
            self.low
        }
    }

    pub fn set(&mut self, addr: u16) {
        let [low, high] = addr.to_le_bytes();

        self.low = low;
        self.high = high;
    }

    pub fn increment(&mut self, increment: u16) {
        let addr = self.get().wrapping_add(increment);
        let mirrored_addr = if addr > 0x3FFF { addr & 0x3FFF } else { addr };

        self.set(mirrored_addr);
    }

    pub fn write(&mut self, data: u8) {
        if self.write_high {
            self.high = data;
        } else {
            self.low = data;
        }

        if self.get() > 0x3fff {
            self.set(self.get() & 0x3fff);
        }

        self.write_high = !self.write_high;
    }

    pub fn reset_latch(&mut self) {
        self.write_high = true;
    }
}
