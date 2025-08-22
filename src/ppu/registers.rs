use std::ops::{Add, Deref};

use modular_bitfield::{bitfield, prelude::B5};

pub struct PpuRegisters {
    pub address: Address,
    pub control: Control,
    pub status: Status,
    pub scroll: Scroll,
    pub mask: Mask,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self {
            address: Address::default(),
            control: Control::new(),
            status: Status::new(),
            scroll: Scroll::default(),
            mask: Mask::new(),
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
    pub nametable1: bool,
    pub nametable2: bool,
    pub vram_address_increment: bool,
    pub sprite_pattern_address: bool,
    pub background_attern_address: bool,
    pub sprite_size: bool,
    pub master_slave_select: bool,
    pub generate_nmi: bool,
}

impl Control {
    pub fn bits(&self) -> u8 {
        self.clone().into_bytes()[0]
    }

    pub fn vram_address_increment_amount(&self) -> u8 {
        if self.vram_address_increment() {
            32
        } else {
            1
        }
    }

    pub fn set_bits(&mut self, bits: u8) {
        self.set_nametable1(bits >> 0 & 1 == 1);
        self.set_nametable2(bits >> 1 & 1 == 1);
        self.set_vram_address_increment(bits >> 2 & 1 == 1);
        self.set_sprite_pattern_address(bits >> 3 & 1 == 1);
        self.set_background_attern_address(bits >> 4 & 1 == 1);
        self.set_sprite_pattern_address(bits >> 5 & 1 == 1);
        self.set_master_slave_select(bits >> 6 & 1 == 1);
        self.set_generate_nmi(bits >> 7 & 1 == 1);
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
pub struct Status {
    unused: B5,
    sprite_overflow: bool,
    sprite_zero_hit: bool,
    vblank_started: bool,
}

#[derive(Default)]
pub struct Scroll {
    pub scroll_x: u8,
    pub scroll_y: u8,

    pub latch: bool,
}

impl Scroll {
    pub fn write(&mut self, data: u8) {
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
    greyscale: bool,
    leftmost_8px_background: bool,
    leftmost_8px_sprite: bool,
    show_background: bool,
    show_sprite: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}
