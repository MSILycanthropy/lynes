use modular_bitfield::{bitfield, specifiers::B2};

pub struct CpuRegisters {
    pub accumulator: u8,
    pub x: u8,
    pub y: u8,
    pub stack_pointer: u8,
    pub status: Status,
    pub program_counter: u16,
}

impl Default for CpuRegisters {
    fn default() -> Self {
        Self {
            accumulator: 0,
            x: 0,
            y: 0,
            stack_pointer: 0xFD,
            status: Status::new().with_b(0b10).with_interrupt_disable(true),
            program_counter: 0,
        }
    }
}

//
// 7      bit     0
// ----------------
// N V s s  D I Z C
// | | | |  | | | |
// | | | |  | | | +- Carry
// | | | |  | | +--- Zero
// | | | |  | +----- Interrupt Disable
// | | | |  +------- Decimal
// | | + +---------- No CPU effect, see: the B flag
// | +-------------- Overflow
// +---------------- Negative
//
#[bitfield]
#[derive(Clone)]
pub struct Status {
    pub carry: bool,
    pub zero: bool,
    pub interrupt_disable: bool,
    pub decimal: bool,
    pub b: B2,
    pub overflow: bool,
    pub negative: bool,
}

impl Status {
    pub fn bits(&self) -> u8 {
        self.clone().into_bytes()[0]
    }

    pub fn set_bits(&mut self, bits: u8) {
        self.bytes = [bits];
    }
}
