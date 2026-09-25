use bitfield_struct::bitfield;

pub struct PpuRegisters {
    pub scroll: ScrollState,
    pub control: Control,
    pub status: Status,
    pub mask: Mask,

    pub oam_addr: u8,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self {
            scroll: ScrollState::default(),
            control: Control::new(),
            status: Status::new(),
            mask: Mask::new(),

            oam_addr: 0,
        }
    }
}

impl PpuRegisters {
    pub fn increment_vram_address(&mut self) {
        let amount = self.control.vram_address_increment_amount();

        self.scroll.increment_address(amount);
    }
}

#[derive(Default)]
pub struct ScrollState {
    current_address: u16,   // v
    temporary_address: u16, // t
    fine_x: u8,
    write_toggle: bool,
}

impl ScrollState {
    pub fn write_control(&mut self, data: u8) {
        const NAMETABLE_MASK: u16 = 0x03 << 10;

        let preserved = self.temporary_address & !NAMETABLE_MASK;
        let nametable_bits = u16::from(data & 0x03) << 10;

        self.temporary_address = preserved | nametable_bits;
    }

    pub fn write_scroll(&mut self, data: u8) {
        const COARSE_X_MASK: u16 = 0x1F;
        const COARSE_Y_MASK: u16 = 0x1F << 5;
        const FINE_Y_MASK: u16 = 0x07 << 12;

        let coarse = u16::from(data >> 3);
        let fine = data & 0x07;

        if self.write_toggle {
            let coarse_y = coarse << 5;
            let fine_y = u16::from(fine) << 12;

            let preserved = self.temporary_address & !(COARSE_Y_MASK | FINE_Y_MASK);
            self.temporary_address = preserved | coarse_y | fine_y;
        } else {
            let coarse_x = coarse;
            let preserved = self.temporary_address & !COARSE_X_MASK;

            self.temporary_address = preserved | coarse_x;
            self.fine_x = fine;
        }

        self.write_toggle = !self.write_toggle;
    }

    pub fn write_address(&mut self, data: u8) {
        if self.write_toggle {
            let preserved = self.temporary_address & 0x7F00;
            let overwrite = u16::from(data);

            self.temporary_address = preserved | overwrite;
            self.current_address = self.temporary_address;
        } else {
            let preserved = self.temporary_address & 0xFF;
            let overwrite = u16::from(data & 0x3F) << 8;

            self.temporary_address = preserved | overwrite;
        }

        self.write_toggle = !self.write_toggle;
    }

    pub fn render_address(&self) -> u16 {
        self.current_address
    }

    pub fn fine_x(&self) -> u8 {
        self.fine_x
    }

    pub fn memory_address(&self) -> u16 {
        self.current_address & 0x3FFF
    }

    pub fn increment_address(&mut self, amount: u8) {
        self.current_address = self.current_address.wrapping_add(u16::from(amount)) & 0x7FFF;
    }

    pub fn reset_write_toggle(&mut self) {
        self.write_toggle = false;
    }

    pub fn increment_render_x(&mut self) {
        if self.current_address & 0x001F == 31 {
            self.current_address &= !0x001F;
            self.current_address ^= 0x0400;
        } else {
            self.current_address += 1;
        }
    }

    pub fn increment_render_y(&mut self) {
        if self.current_address & 0x7000 != 0x7000 {
            self.current_address += 0x1000;
            return;
        }

        self.current_address &= !0x7000;

        let mut coarse_y = (self.current_address >> 5) & 0x001F;

        match coarse_y {
            29 => {
                coarse_y = 0;
                self.current_address ^= 0x0800;
            }
            31 => coarse_y = 0,
            _ => coarse_y += 1,
        }

        self.current_address = (self.current_address & !0x3E0) | (coarse_y << 5);
    }

    pub fn copy_render_x(&mut self) {
        const MASK: u16 = 0x041F;

        self.current_address = (self.current_address & !MASK) | (self.temporary_address & MASK);
    }

    pub fn copy_render_y(&mut self) {
        const MASK: u16 = 0x7BE0;

        self.current_address = (self.current_address & !MASK) | (self.temporary_address & MASK);
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
#[bitfield(u8)]
pub struct Control {
    #[bits(2)]
    pub nametable: u8,
    pub vram_address_increment: bool,
    pub sprite_pattern_address: bool,
    pub background_pattern_address: bool,
    pub sprite_size: bool,
    pub master_slave_select: bool,
    pub generate_nmi: bool,
}

impl Control {
    pub fn bits(&self) -> u8 {
        self.into_bits()
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
        if self.vram_address_increment() { 32 } else { 1 }
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
        *self = Self::from_bits(bits);
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
#[bitfield(u8)]
pub struct Status {
    #[allow(dead_code)]
    #[bits(5)]
    unused: u8,
    pub sprite_overflow: bool,
    pub sprite_zero_hit: bool,
    pub vblank_started: bool,
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
#[bitfield(u8)]
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
        *self = Self::from_bits(bits);
    }
}

#[cfg(test)]
mod tests {
    use super::ScrollState;

    #[test]
    fn control_and_scroll_writes_preserve_each_others_fields() {
        let mut scroll = ScrollState::default();

        scroll.write_control(3);
        scroll.write_scroll(19);
        scroll.write_scroll(29);

        assert_eq!(scroll.temporary_address, 0x5C62);
        assert_eq!(scroll.current_address, 0);
        assert_eq!(scroll.fine_x, 3);
        assert!(!scroll.write_toggle);

        scroll.write_control(1);
        assert_eq!(scroll.temporary_address, 0x5462);
        assert_eq!(scroll.current_address, 0);
        assert_eq!(scroll.fine_x, 3);
        assert!(!scroll.write_toggle);
    }

    #[test]
    fn scroll_and_address_writes_share_the_toggle() {
        let mut scroll = ScrollState::default();

        scroll.write_control(3);
        scroll.write_scroll(19);
        assert!(scroll.write_toggle);

        // This completes the pair started by the scroll write.
        scroll.write_address(0x80);

        assert_eq!(scroll.temporary_address, 0x0C80);
        assert_eq!(scroll.current_address, 0x0C80);
        assert_eq!(scroll.fine_x, 3);
        assert!(!scroll.write_toggle);
    }

    #[test]
    fn resetting_toggle_restarts_address_write_and_masks_high_byte() {
        let mut scroll = ScrollState::default();

        scroll.write_scroll(19);
        scroll.reset_write_toggle();
        assert!(!scroll.write_toggle);

        scroll.write_address(0xFF);
        assert_eq!(scroll.temporary_address, 0x3F02);
        assert_eq!(scroll.current_address, 0);
        assert!(scroll.write_toggle);

        scroll.write_address(0x80);
        assert_eq!(scroll.temporary_address, 0x3F80);
        assert_eq!(scroll.current_address, 0x3F80);
        assert_eq!(scroll.fine_x, 3);
        assert!(!scroll.write_toggle);
    }
}
