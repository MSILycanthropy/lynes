use bitfield_struct::bitfield;

#[bitfield(u8)]
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
        self.into_bits()
    }

    pub fn clear(&mut self) {
        *self = Self::new();
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
            button_state: ButtonState::new(),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;

        if self.strobe {
            self.button_index = 0
        }
    }

    pub fn peek(&self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }

        (self.button_state.bits() & (1 << self.button_index)) >> self.button_index
    }

    pub fn read(&mut self) -> u8 {
        let response = self.peek();

        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::Controller;

    #[test]
    fn peeking_does_not_consume_controller_bits() {
        let mut controller = Controller::new();
        controller.button_state.set_a(true);
        controller.write(1);
        assert_eq!(controller.peek(), 1);
        assert_eq!(controller.read(), 1);
        controller.write(0);

        for expected in [1, 0, 0, 0, 0, 0, 0, 0, 1, 1] {
            assert_eq!(controller.peek(), expected);
            assert_eq!(controller.peek(), expected);
            assert_eq!(controller.read(), expected);
        }
    }
}
