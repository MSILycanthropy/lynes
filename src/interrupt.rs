use crate::{cpu::CPU, NES};

pub trait Interrupt {
    fn irq(&mut self);
    fn nmi(&mut self);
    fn brk(&mut self);
}

impl Interrupt for NES {
    fn irq(&mut self) {
        if !self.cpu_registers.status.interrupt_disable() {
            interrupt(self, 0b10, 0xfffe);
        }
    }

    fn nmi(&mut self) {
        interrupt(self, 0b10, 0xfffa);
    }

    fn brk(&mut self) {
        if !self.cpu_registers.status.interrupt_disable() {
            interrupt(self, 0b11, 0xfffe);
        }
    }
}

fn interrupt(nes: &mut NES, new_b: u8, addr: u16) {
    nes.stack_push_u16(nes.cpu_registers.program_counter);

    let mut status = nes.cpu_registers.status.clone();
    status.set_b(new_b);

    nes.stack_push(status.bits());

    nes.cpu_cycle += 7;
    nes.cpu_total_cycles += 7;
    nes.cpu_registers.program_counter = nes.cpu_read_u16(addr);
}
