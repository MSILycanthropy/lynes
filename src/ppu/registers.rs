use modular_bitfield::{
    bitfield,
    prelude::{B2, B5},
};

pub struct PpuRegisters {
    pub address: Address,
    pub control: Control,
    pub status: Status,
    pub scroll: Scroll,
    pub mask: Mask,

    pub oam_addr: u8,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self {
            address: Address::default(),
            control: Control::new(),
            status: Status::new(),
            scroll: Scroll::default(),
            mask: Mask::new(),

            oam_addr: 0,
        }
    }
}

impl PpuRegisters {
    pub fn increment_vram_address(&mut self) {
        self.address
            .increment(self.control.vram_address_increment_amount());
    }
}

pub struct Address {
    high: u8,
    low: u8,
    latch: bool,
}

impl Default for Address {
    fn default() -> Self {
        Self {
            high: 0,
            low: 0,
            latch: true,
        }
    }
}

impl Address {
    fn set(&mut self, data: u16) {
        self.high = (data >> 8) as u8;
        self.low = (data & 0xFF) as u8;
    }

    fn mirror_down(&mut self) {
        if self.as_u16() <= 0x3FFF {
            return;
        }

        self.set(self.as_u16() & 0b01111111_11111111)
    }

    fn flip_latch(&mut self) {
        self.latch = !self.latch
    }

    pub fn update(&mut self, data: u8) {
        if self.latch {
            self.high = data;
        } else {
            self.low = data;
        }

        self.mirror_down();
        self.flip_latch();
    }

    pub fn increment(&mut self, amt: u8) {
        let original_low = self.low;
        self.low = self.low.wrapping_add(amt);

        if original_low > self.low {
            self.high = self.high.wrapping_add(1);
        }

        self.mirror_down();
    }

    pub fn reset_latch(&mut self) {
        self.latch = true;
    }

    pub fn as_u16(&self) -> u16 {
        ((self.high as u16) << 8) | (self.low as u16)
    }
}

// 7  bit  0
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Generate an NMI at the start of the
//            vertical blanking interval (0: off; 1: on)
#[bitfield]
#[derive(Clone)]
pub struct Control {
    pub nametable: B2,
    pub vram_address_increment: bool,
    pub sprite_pattern_address: bool,
    pub background_pattern_address: bool,
    pub sprite_size: bool,
    pub master_slave_select: bool,
    pub generate_nmi: bool,
}

impl Control {
    pub fn bits(&self) -> u8 {
        self.clone().into_bytes()[0]
    }

    pub fn name_table_address(&self) -> u16 {
        match self.nametable() {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => unreachable!(),
        }
    }

    pub fn vram_address_increment_amount(&self) -> u8 {
        if self.vram_address_increment() {
            32
        } else {
            1
        }
    }

    pub fn background_pattern_address_value(&self) -> u16 {
        if self.background_pattern_address() {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn sprite_pattern_address_value(&self) -> u16 {
        if self.sprite_pattern_address() {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn update(&mut self, bits: u8) {
        self.bytes = [bits];
    }
}

// 7  bit  0
// ---- ----
// VSOx xxxx
// |||| ||||
// |||+-++++- (PPU open bus or 2C05 PPU identifier)
// ||+------- Sprite overflow flag
// |+-------- Sprite 0 hit flag
// +--------- Vblank flag, cleared on read.
#[bitfield]
#[derive(Clone)]
pub struct Status {
    #[allow(dead_code)]
    unused: B5,
    pub sprite_overflow: bool,
    pub sprite_zero_hit: bool,
    pub vblank_started: bool,
}

#[derive(Default)]
pub struct Scroll {
    pub scroll_x: u8,
    pub scroll_y: u8,

    pub latch: bool,
}

impl Scroll {
    pub fn update(&mut self, data: u8) {
        if self.latch {
            self.scroll_y = data
        } else {
            self.scroll_x = data
        }

        self.latch = !self.latch
    }

    pub fn reset_latch(&mut self) {
        self.latch = false
    }
}

// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: greyscale)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
// |||| +---- 1: Enable background rendering
// |||+------ 1: Enable sprite rendering
// ||+------- Emphasize red (green on PAL/Dendy)
// |+-------- Emphasize green (red on PAL/Dendy)
// +--------- Emphasize blue
#[bitfield]
pub struct Mask {
    pub greyscale: bool,
    pub leftmost_8px_background: bool,
    pub leftmost_8px_sprite: bool,
    pub show_background: bool,
    pub show_sprite: bool,
    pub emphasize_red: bool,
    pub emphasize_green: bool,
    pub emphasize_blue: bool,
}

impl Mask {
    pub fn update(&mut self, bits: u8) {
        self.bytes = [bits];
    }
}
