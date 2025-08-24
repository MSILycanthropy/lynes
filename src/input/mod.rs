use modular_bitfield::bitfield;

#[bitfield]
#[derive(Clone)]
pub struct ButtonState {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl ButtonState {
    fn bits(&self) -> u8 {
        *self.clone().into_bytes().first().unwrap()
    }
}

pub struct Controller {
    strobe: bool,
    button_index: u8,
    pub button_state: ButtonState,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            strobe: false,
            button_index: 0,
            button_state: ButtonState::from_bytes([0]),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;

        if self.strobe {
            self.button_index = 0
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }

        let response = (self.button_state.bits() & (1 << self.button_index)) >> self.button_index;

        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }

        response
    }
}
