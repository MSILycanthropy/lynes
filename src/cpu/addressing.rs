use super::{AddrMode, Cpu, bus::CpuBus};

#[derive(Copy, Clone)]
pub(crate) enum AccessKind {
    Read,
    Write,
    ReadModifyWrite,
}

impl Cpu {
    fn indexed_address(
        &mut self,
        bus: &mut CpuBus,
        base: u16,
        index: u8,
        access: AccessKind,
    ) -> u16 {
        let address = base.wrapping_add(u16::from(index));
        let crossed_page = page_crossed(base, address);

        if crossed_page || !matches!(access, AccessKind::Read) {
            let dummy_address = (base & 0xFF00) | (address & 0x00FF);
            self.read(bus, dummy_address);
        }

        address
    }

    fn fetch_instruction_address(&mut self, bus: &mut CpuBus) -> u16 {
        let low = self.fetch_instruction_byte(bus);
        let high = self.fetch_instruction_byte(bus);
        u16::from_le_bytes([low, high])
    }

    pub(crate) fn read_operand(&mut self, bus: &mut CpuBus, mode: &AddrMode) -> u8 {
        match mode {
            AddrMode::Immediate => {
                self.poll_interrupts();
                self.fetch_instruction_byte(bus)
            }
            _ => {
                let address = self.fetch_operand_address(bus, mode, AccessKind::Read);
                self.poll_interrupts();
                self.read(bus, address)
            }
        }
    }

    pub(crate) fn fetch_operand_address(
        &mut self,
        bus: &mut CpuBus,
        mode: &AddrMode,
        access: AccessKind,
    ) -> u16 {
        match mode {
            AddrMode::ZeroPage => u16::from(self.fetch_instruction_byte(bus)),
            AddrMode::ZeroPageX => {
                let base = self.fetch_instruction_byte(bus);
                self.read(bus, u16::from(base));
                u16::from(base.wrapping_add(self.registers.x))
            }
            AddrMode::ZeroPageY => {
                let base = self.fetch_instruction_byte(bus);
                self.read(bus, u16::from(base));
                u16::from(base.wrapping_add(self.registers.y))
            }
            AddrMode::Absolute => self.fetch_instruction_address(bus),
            AddrMode::AbsoluteX => {
                let base = self.fetch_instruction_address(bus);
                self.indexed_address(bus, base, self.registers.x, access)
            }
            AddrMode::AbsoluteY => {
                let base = self.fetch_instruction_address(bus);
                self.indexed_address(bus, base, self.registers.y, access)
            }
            AddrMode::IndirectX => {
                let zero_page_address = self.fetch_instruction_byte(bus);
                self.read(bus, u16::from(zero_page_address));
                let pointer = zero_page_address.wrapping_add(self.registers.x);
                let low = self.read(bus, pointer as u16);
                let high = self.read(bus, pointer.wrapping_add(1) as u16);

                u16::from_le_bytes([low, high])
            }
            AddrMode::IndirectY => {
                let zero_page_address = self.fetch_instruction_byte(bus);
                let low = self.read(bus, zero_page_address as u16);
                let high = self.read(bus, zero_page_address.wrapping_add(1) as u16);

                let base = u16::from_le_bytes([low, high]);
                self.indexed_address(bus, base, self.registers.y, access)
            }
            AddrMode::Immediate
            | AddrMode::Indirect
            | AddrMode::Relative
            | AddrMode::Implied
            | AddrMode::Accumulator => {
                panic!("Addressing mode {mode:?} has no memory operand address")
            }
        }
    }
}

fn page_crossed(old_address: u16, new_address: u16) -> bool {
    old_address & 0xFF00 != new_address & 0xFF00
}
