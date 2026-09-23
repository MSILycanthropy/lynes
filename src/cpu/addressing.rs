use super::{AddrMode, Cpu, bus::CpuBus};

impl Cpu {
    pub(crate) fn get_operating_address(
        &mut self,
        bus: &mut CpuBus,
        mode: &AddrMode,
    ) -> (u16, bool) {
        match mode {
            AddrMode::Implied => {
                panic!("Implied addressing mode has no operating address as it is implied")
            }
            AddrMode::Accumulator => panic!(
                "Accumulator addressing mode has no operating address as it operates on the accumulator"
            ),
            AddrMode::Immediate => {
                let address = self.registers.program_counter;
                (address, false)
            }
            _ => self.get_absolute_address(bus, self.registers.program_counter, mode),
        }
    }

    pub(crate) fn get_absolute_address(
        &mut self,
        bus: &mut CpuBus,
        address: u16,
        mode: &AddrMode,
    ) -> (u16, bool) {
        match mode {
            AddrMode::ZeroPage => {
                let address = self.read(bus, address) as u16;
                (address, false)
            }
            AddrMode::ZeroPageX => {
                let address = self.read(bus, address).wrapping_add(self.registers.x) as u16;
                (address, false)
            }
            AddrMode::ZeroPageY => {
                let address = self.read(bus, address).wrapping_add(self.registers.y) as u16;
                (address, false)
            }
            AddrMode::Relative => {
                let offset = self.read(bus, address) as u16;
                let old_address = address;
                let address = old_address.wrapping_add(1).wrapping_add(offset);

                (address, page_crossed(old_address, address))
            }
            AddrMode::Absolute => {
                let address = self.read_u16(bus, address);
                (address, false)
            }
            AddrMode::AbsoluteX => {
                let old_address = self.read_u16(bus, address);
                let address = old_address.wrapping_add(self.registers.x as u16);

                (address, page_crossed(old_address, address))
            }
            AddrMode::AbsoluteY => {
                let old_address = self.read_u16(bus, address);
                let address = old_address.wrapping_add(self.registers.y as u16);

                (address, page_crossed(old_address, address))
            }
            AddrMode::Indirect => {
                let old_address = self.read_u16(bus, address);

                let address = if old_address & 0x00FF == 0x00FF {
                    let low = self.read(bus, old_address);
                    let high = self.read(bus, old_address & 0xFF00);

                    u16::from_le_bytes([low, high])
                } else {
                    self.read_u16(bus, old_address)
                };

                (address, false)
            }
            AddrMode::IndirectX => {
                let zero_page_address = self.read(bus, address);
                let pointer = zero_page_address.wrapping_add(self.registers.x);
                let low = self.read(bus, pointer as u16);
                let high = self.read(bus, pointer.wrapping_add(1) as u16);

                let address = u16::from_le_bytes([low, high]);

                (address, false)
            }
            AddrMode::IndirectY => {
                let zero_page_address = self.read(bus, address);
                let low = self.read(bus, zero_page_address as u16);
                let high = self.read(bus, zero_page_address.wrapping_add(1) as u16);

                let old_address = u16::from_le_bytes([low, high]);
                let address = old_address.wrapping_add(self.registers.y as u16);

                (address, page_crossed(old_address, address))
            }
            _ => panic!("Invalid absolute addressing mode"),
        }
    }
}

fn page_crossed(old_address: u16, new_address: u16) -> bool {
    old_address & 0xFF00 != new_address & 0xFF00
}
