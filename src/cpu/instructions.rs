use crate::Interrupt;

use super::{AddrMode, Cpu, addressing::AccessKind, bus::CpuBus};

macro_rules! instr {
    ($name: expr, $mode: expr, $fn: expr) => {
        Instruction {
            name: $name,
            mode: $mode,
            operate: $fn,
            legal: true,
        }
    };
}

macro_rules! il_instr {
    ($name: expr, $mode: expr, $fn: expr) => {
        Instruction {
            name: $name,
            mode: $mode,
            operate: $fn,
            legal: false,
        }
    };
}

pub(crate) const INSTRUCTIONS_TABLE: [Instruction; 256] = [
    instr!("BRK", AddrMode::Implied, brk),
    instr!("ORA", AddrMode::IndirectX, ora),
    il_instr!("KIL", AddrMode::Implied, kil), // This technically is not correct, KIL explodes the CPU but we'll treat it as a NOP
    il_instr!("SLO", AddrMode::IndirectX, slo),
    il_instr!("DOP", AddrMode::ZeroPage, dop),
    instr!("ORA", AddrMode::ZeroPage, ora),
    instr!("ASL", AddrMode::ZeroPage, asl),
    il_instr!("SLO", AddrMode::ZeroPage, slo),
    instr!("PHP", AddrMode::Implied, php),
    instr!("ORA", AddrMode::Immediate, ora),
    instr!("ASL", AddrMode::Accumulator, asl),
    il_instr!("ANC", AddrMode::Immediate, anc),
    il_instr!("TOP", AddrMode::Absolute, top),
    instr!("ORA", AddrMode::Absolute, ora),
    instr!("ASL", AddrMode::Absolute, asl),
    il_instr!("SLO", AddrMode::Absolute, slo),
    instr!("BPL", AddrMode::Relative, bpl),
    instr!("ORA", AddrMode::IndirectY, ora),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("SLO", AddrMode::IndirectY, slo),
    il_instr!("DOP", AddrMode::ZeroPageX, dop),
    instr!("ORA", AddrMode::ZeroPageX, ora),
    instr!("ASL", AddrMode::ZeroPageX, asl),
    il_instr!("SLO", AddrMode::ZeroPageX, slo),
    instr!("CLC", AddrMode::Implied, clc),
    instr!("ORA", AddrMode::AbsoluteY, ora),
    il_instr!("NOP", AddrMode::Implied, nop),
    il_instr!("SLO", AddrMode::AbsoluteY, slo),
    il_instr!("TOP", AddrMode::AbsoluteX, top),
    instr!("ORA", AddrMode::AbsoluteX, ora),
    instr!("ASL", AddrMode::AbsoluteX, asl),
    il_instr!("SLO", AddrMode::AbsoluteX, slo),
    instr!("JSR", AddrMode::Absolute, jsr),
    instr!("AND", AddrMode::IndirectX, and),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("RLA", AddrMode::IndirectX, rla),
    instr!("BIT", AddrMode::ZeroPage, bit),
    instr!("AND", AddrMode::ZeroPage, and),
    instr!("ROL", AddrMode::ZeroPage, rol),
    il_instr!("RLA", AddrMode::ZeroPage, rla),
    instr!("PLP", AddrMode::Implied, plp),
    instr!("AND", AddrMode::Immediate, and),
    instr!("ROL", AddrMode::Accumulator, rol),
    il_instr!("ANC", AddrMode::Immediate, anc),
    instr!("BIT", AddrMode::Absolute, bit),
    instr!("AND", AddrMode::Absolute, and),
    instr!("ROL", AddrMode::Absolute, rol),
    il_instr!("RLA", AddrMode::Absolute, rla),
    instr!("BMI", AddrMode::Relative, bmi),
    instr!("AND", AddrMode::IndirectY, and),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("RLA", AddrMode::IndirectY, rla),
    il_instr!("DOP", AddrMode::ZeroPageX, dop),
    instr!("AND", AddrMode::ZeroPageX, and),
    instr!("ROL", AddrMode::ZeroPageX, rol),
    il_instr!("RLA", AddrMode::ZeroPageX, rla),
    instr!("SEC", AddrMode::Implied, sec),
    instr!("AND", AddrMode::AbsoluteY, and),
    il_instr!("NOP", AddrMode::Implied, nop),
    il_instr!("RLA", AddrMode::AbsoluteY, rla),
    il_instr!("TOP", AddrMode::AbsoluteX, top),
    instr!("AND", AddrMode::AbsoluteX, and),
    instr!("ROL", AddrMode::AbsoluteX, rol),
    il_instr!("RLA", AddrMode::AbsoluteX, rla),
    instr!("RTI", AddrMode::Implied, rti),
    instr!("EOR", AddrMode::IndirectX, eor),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("SRE", AddrMode::IndirectX, sre),
    il_instr!("DOP", AddrMode::ZeroPage, dop),
    instr!("EOR", AddrMode::ZeroPage, eor),
    instr!("LSR", AddrMode::ZeroPage, lsr),
    il_instr!("SRE", AddrMode::ZeroPage, sre),
    instr!("PHA", AddrMode::Implied, pha),
    instr!("EOR", AddrMode::Immediate, eor),
    instr!("LSR", AddrMode::Accumulator, lsr),
    il_instr!("ASR", AddrMode::Immediate, asr),
    instr!("JMP", AddrMode::Absolute, jmp),
    instr!("EOR", AddrMode::Absolute, eor),
    instr!("LSR", AddrMode::Absolute, lsr),
    il_instr!("SRE", AddrMode::Absolute, sre),
    instr!("BVC", AddrMode::Relative, bvc),
    instr!("EOR", AddrMode::IndirectY, eor),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("SRE", AddrMode::IndirectY, sre),
    il_instr!("DOP", AddrMode::ZeroPageX, dop),
    instr!("EOR", AddrMode::ZeroPageX, eor),
    instr!("LSR", AddrMode::ZeroPageX, lsr),
    il_instr!("SRE", AddrMode::ZeroPageX, sre),
    instr!("CLI", AddrMode::Implied, cli),
    instr!("EOR", AddrMode::AbsoluteY, eor),
    il_instr!("NOP", AddrMode::Implied, nop),
    il_instr!("SRE", AddrMode::AbsoluteY, sre),
    il_instr!("TOP", AddrMode::AbsoluteX, top),
    instr!("EOR", AddrMode::AbsoluteX, eor),
    instr!("LSR", AddrMode::AbsoluteX, lsr),
    il_instr!("SRE", AddrMode::AbsoluteX, sre),
    instr!("RTS", AddrMode::Implied, rts),
    instr!("ADC", AddrMode::IndirectX, adc),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("RRA", AddrMode::IndirectX, rra),
    il_instr!("DOP", AddrMode::ZeroPage, dop),
    instr!("ADC", AddrMode::ZeroPage, adc),
    instr!("ROR", AddrMode::ZeroPage, ror),
    il_instr!("RRA", AddrMode::ZeroPage, rra),
    instr!("PLA", AddrMode::Implied, pla),
    instr!("ADC", AddrMode::Immediate, adc),
    instr!("ROR", AddrMode::Accumulator, ror),
    il_instr!("ARR", AddrMode::Immediate, arr),
    instr!("JMP", AddrMode::Indirect, jmp),
    instr!("ADC", AddrMode::Absolute, adc),
    instr!("ROR", AddrMode::Absolute, ror),
    il_instr!("RRA", AddrMode::Absolute, rra),
    instr!("BVS", AddrMode::Relative, bvs),
    instr!("ADC", AddrMode::IndirectY, adc),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("RRA", AddrMode::IndirectY, rra),
    il_instr!("DOP", AddrMode::ZeroPageX, dop),
    instr!("ADC", AddrMode::ZeroPageX, adc),
    instr!("ROR", AddrMode::ZeroPageX, ror),
    il_instr!("RRA", AddrMode::ZeroPageX, rra),
    instr!("SEI", AddrMode::Implied, sei),
    instr!("ADC", AddrMode::AbsoluteY, adc),
    il_instr!("NOP", AddrMode::Implied, nop),
    il_instr!("RRA", AddrMode::AbsoluteY, rra),
    il_instr!("TOP", AddrMode::AbsoluteX, top),
    instr!("ADC", AddrMode::AbsoluteX, adc),
    instr!("ROR", AddrMode::AbsoluteX, ror),
    il_instr!("RRA", AddrMode::AbsoluteX, rra),
    il_instr!("DOP", AddrMode::Immediate, dop),
    instr!("STA", AddrMode::IndirectX, sta),
    il_instr!("DOP", AddrMode::Immediate, dop),
    il_instr!("SAX", AddrMode::IndirectX, sax),
    instr!("STY", AddrMode::ZeroPage, sty),
    instr!("STA", AddrMode::ZeroPage, sta),
    instr!("STX", AddrMode::ZeroPage, stx),
    il_instr!("SAX", AddrMode::ZeroPage, sax),
    instr!("DEY", AddrMode::Implied, dey),
    il_instr!("DOP", AddrMode::Immediate, dop),
    instr!("TXA", AddrMode::Implied, txa),
    il_instr!("XAA", AddrMode::Immediate, xaa),
    instr!("STY", AddrMode::Absolute, sty),
    instr!("STA", AddrMode::Absolute, sta),
    instr!("STX", AddrMode::Absolute, stx),
    il_instr!("SAX", AddrMode::Absolute, sax),
    instr!("BCC", AddrMode::Relative, bcc),
    instr!("STA", AddrMode::IndirectY, sta),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("AXA", AddrMode::IndirectY, axa),
    instr!("STY", AddrMode::ZeroPageX, sty),
    instr!("STA", AddrMode::ZeroPageX, sta),
    instr!("STX", AddrMode::ZeroPageY, stx),
    il_instr!("SAX", AddrMode::ZeroPageY, sax),
    instr!("TYA", AddrMode::Implied, tya),
    instr!("STA", AddrMode::AbsoluteY, sta),
    instr!("TXS", AddrMode::Implied, txs),
    il_instr!("XAS", AddrMode::AbsoluteY, xas),
    il_instr!("SYA", AddrMode::AbsoluteX, sya),
    instr!("STA", AddrMode::AbsoluteX, sta),
    il_instr!("SXA", AddrMode::AbsoluteY, sxa),
    il_instr!("AXA", AddrMode::AbsoluteY, axa),
    instr!("LDY", AddrMode::Immediate, ldy),
    instr!("LDA", AddrMode::IndirectX, lda),
    instr!("LDX", AddrMode::Immediate, ldx),
    il_instr!("LAX", AddrMode::IndirectX, lax),
    instr!("LDY", AddrMode::ZeroPage, ldy),
    instr!("LDA", AddrMode::ZeroPage, lda),
    instr!("LDX", AddrMode::ZeroPage, ldx),
    il_instr!("LAX", AddrMode::ZeroPage, lax),
    instr!("TAY", AddrMode::Implied, tay),
    instr!("LDA", AddrMode::Immediate, lda),
    instr!("TAX", AddrMode::Implied, tax),
    il_instr!("LXA", AddrMode::Immediate, lxa),
    instr!("LDY", AddrMode::Absolute, ldy),
    instr!("LDA", AddrMode::Absolute, lda),
    instr!("LDX", AddrMode::Absolute, ldx),
    il_instr!("LAX", AddrMode::Absolute, lax),
    instr!("BCS", AddrMode::Relative, bcs),
    instr!("LDA", AddrMode::IndirectY, lda),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("LAX", AddrMode::IndirectY, lax),
    instr!("LDY", AddrMode::ZeroPageX, ldy),
    instr!("LDA", AddrMode::ZeroPageX, lda),
    instr!("LDX", AddrMode::ZeroPageY, ldx),
    il_instr!("LAX", AddrMode::ZeroPageY, lax),
    instr!("CLV", AddrMode::Implied, clv),
    instr!("LDA", AddrMode::AbsoluteY, lda),
    instr!("TSX", AddrMode::Implied, tsx),
    il_instr!("LAS", AddrMode::AbsoluteY, las),
    instr!("LDY", AddrMode::AbsoluteX, ldy),
    instr!("LDA", AddrMode::AbsoluteX, lda),
    instr!("LDX", AddrMode::AbsoluteY, ldx),
    il_instr!("LAX", AddrMode::AbsoluteY, lax),
    instr!("CPY", AddrMode::Immediate, cpy),
    instr!("CMP", AddrMode::IndirectX, cmp),
    il_instr!("DOP", AddrMode::Immediate, dop),
    il_instr!("DCP", AddrMode::IndirectX, dcp),
    instr!("CPY", AddrMode::ZeroPage, cpy),
    instr!("CMP", AddrMode::ZeroPage, cmp),
    instr!("DEC", AddrMode::ZeroPage, dec),
    il_instr!("DCP", AddrMode::ZeroPage, dcp),
    instr!("INY", AddrMode::Implied, iny),
    instr!("CMP", AddrMode::Immediate, cmp),
    instr!("DEX", AddrMode::Implied, dex),
    il_instr!("AXS", AddrMode::Immediate, axs),
    instr!("CPY", AddrMode::Absolute, cpy),
    instr!("CMP", AddrMode::Absolute, cmp),
    instr!("DEC", AddrMode::Absolute, dec),
    il_instr!("DCP", AddrMode::Absolute, dcp),
    instr!("BNE", AddrMode::Relative, bne),
    instr!("CMP", AddrMode::IndirectY, cmp),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("DCP", AddrMode::IndirectY, dcp),
    il_instr!("DOP", AddrMode::ZeroPageX, dop),
    instr!("CMP", AddrMode::ZeroPageX, cmp),
    instr!("DEC", AddrMode::ZeroPageX, dec),
    il_instr!("DCP", AddrMode::ZeroPageX, dcp),
    instr!("CLD", AddrMode::Implied, cld),
    instr!("CMP", AddrMode::AbsoluteY, cmp),
    il_instr!("NOP", AddrMode::Implied, nop),
    il_instr!("DCP", AddrMode::AbsoluteY, dcp),
    il_instr!("TOP", AddrMode::AbsoluteX, top),
    instr!("CMP", AddrMode::AbsoluteX, cmp),
    instr!("DEC", AddrMode::AbsoluteX, dec),
    il_instr!("DCP", AddrMode::AbsoluteX, dcp),
    instr!("CPX", AddrMode::Immediate, cpx),
    instr!("SBC", AddrMode::IndirectX, sbc),
    il_instr!("DOP", AddrMode::Immediate, dop),
    il_instr!("ISB", AddrMode::IndirectX, isb),
    instr!("CPX", AddrMode::ZeroPage, cpx),
    instr!("SBC", AddrMode::ZeroPage, sbc),
    instr!("INC", AddrMode::ZeroPage, inc),
    il_instr!("ISB", AddrMode::ZeroPage, isb),
    instr!("INX", AddrMode::Implied, inx),
    instr!("SBC", AddrMode::Immediate, sbc),
    instr!("NOP", AddrMode::Implied, nop),
    il_instr!("SBC", AddrMode::Immediate, sbc),
    instr!("CPX", AddrMode::Absolute, cpx),
    instr!("SBC", AddrMode::Absolute, sbc),
    instr!("INC", AddrMode::Absolute, inc),
    il_instr!("ISB", AddrMode::Absolute, isb),
    instr!("BEQ", AddrMode::Relative, beq),
    instr!("SBC", AddrMode::IndirectY, sbc),
    il_instr!("KIL", AddrMode::Implied, kil),
    il_instr!("ISB", AddrMode::IndirectY, isb),
    il_instr!("DOP", AddrMode::ZeroPageX, dop),
    instr!("SBC", AddrMode::ZeroPageX, sbc),
    instr!("INC", AddrMode::ZeroPageX, inc),
    il_instr!("ISB", AddrMode::ZeroPageX, isb),
    instr!("SED", AddrMode::Implied, sed),
    instr!("SBC", AddrMode::AbsoluteY, sbc),
    il_instr!("NOP", AddrMode::Implied, nop),
    il_instr!("ISB", AddrMode::AbsoluteY, isb),
    il_instr!("TOP", AddrMode::AbsoluteX, top),
    instr!("SBC", AddrMode::AbsoluteX, sbc),
    instr!("INC", AddrMode::AbsoluteX, inc),
    il_instr!("ISB", AddrMode::AbsoluteX, isb),
];

pub struct Instruction {
    pub name: &'static str,
    pub mode: AddrMode,
    pub operate: fn(&mut Cpu, &mut CpuBus, &AddrMode),
    pub legal: bool,
}

impl Instruction {
    pub fn execute(&self, cpu: &mut Cpu, bus: &mut CpuBus) {
        (self.operate)(cpu, bus, &self.mode);
    }
}

fn adc(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    add_to_accumulator(cpu, value);
}

fn and(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    set_accumulator(cpu, cpu.registers.accumulator & value);
}

fn asl(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let old_value = if let AddrMode::Accumulator = mode {
        cpu.poll_interrupts();
        cpu.read(bus, cpu.registers.program_counter);
        let old_value = cpu.registers.accumulator;

        set_accumulator(cpu, old_value << 1);

        old_value
    } else {
        let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
        let old_value = cpu.read(bus, addr);
        let result = old_value << 1;

        cpu.write(bus, addr, old_value);
        cpu.poll_interrupts();
        cpu.write(bus, addr, result);
        update_zero_and_negative_flags(cpu, result);

        old_value
    };

    cpu.registers.status.set_carry(old_value >> 7 == 1);
}

fn bcc(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    branch(cpu, bus, !cpu.registers.status.carry());
}

fn bcs(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    branch(cpu, bus, cpu.registers.status.carry());
}

fn beq(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    branch(cpu, bus, cpu.registers.status.zero());
}

fn bit(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);
    let result = value & cpu.registers.accumulator;

    cpu.registers.status.set_zero(result == 0);
    cpu.registers.status.set_overflow(value & 0x40 > 0);
    cpu.registers.status.set_negative(value >> 7 == 1);
}

fn bmi(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    branch(cpu, bus, cpu.registers.status.negative());
}

fn bne(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    branch(cpu, bus, !cpu.registers.status.zero());
}

fn bpl(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    branch(cpu, bus, !cpu.registers.status.negative());
}

fn brk(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.fetch_instruction_byte(bus);

    let mut status = cpu.registers.status.clone();
    status.set_b(0b11);

    cpu.finish_interrupt_entry(bus, &Interrupt::IRQ, status.bits());
}

fn bvc(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    branch(cpu, bus, !cpu.registers.status.overflow());
}

fn bvs(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    branch(cpu, bus, cpu.registers.status.overflow());
}

fn clc(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.status.set_carry(false);
}

fn cld(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.status.set_decimal(false);
}

fn cli(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.status.set_interrupt_disable(false);
}

fn clv(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.status.set_overflow(false);
}

fn cmp(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    compare(cpu, cpu.registers.accumulator, value);
}

fn cpx(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    compare(cpu, cpu.registers.x, value);
}

fn cpy(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    compare(cpu, cpu.registers.y, value);
}

fn dec(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
    let old_value = cpu.read(bus, addr);
    let result = old_value.wrapping_sub(1);

    cpu.write(bus, addr, old_value);
    cpu.poll_interrupts();
    cpu.write(bus, addr, result);

    update_zero_and_negative_flags(cpu, result);
}

fn dex(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    let result = cpu.registers.x.wrapping_sub(1);

    cpu.registers.x = result;
    update_zero_and_negative_flags(cpu, result);
}

fn dey(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    let result = cpu.registers.y.wrapping_sub(1);

    cpu.registers.y = result;
    update_zero_and_negative_flags(cpu, result);
}

fn eor(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);
    let result = cpu.registers.accumulator ^ value;

    set_accumulator(cpu, result);
}

fn inc(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);

    increment_memory(cpu, bus, addr);
}

fn inx(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    let result = cpu.registers.x.wrapping_add(1);

    cpu.registers.x = result;
    update_zero_and_negative_flags(cpu, result);
}

fn iny(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    let result = cpu.registers.y.wrapping_add(1);

    cpu.registers.y = result;
    update_zero_and_negative_flags(cpu, result);
}

fn jmp(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let (low, high) = match mode {
        AddrMode::Absolute => {
            let low = cpu.fetch_instruction_byte(bus);
            cpu.poll_interrupts();
            let high = cpu.fetch_instruction_byte(bus);

            (low, high)
        }
        AddrMode::Indirect => {
            let pointer_low = cpu.fetch_instruction_byte(bus);
            let pointer_high = cpu.fetch_instruction_byte(bus);
            let pointer = u16::from_le_bytes([pointer_low, pointer_high]);

            // The 6502 wraps within the pointer's page for the high byte.
            let high_address = (pointer & 0xFF00) | (pointer.wrapping_add(1) & 0x00FF);

            let low = cpu.read(bus, pointer);
            cpu.poll_interrupts();
            let high = cpu.read(bus, high_address);

            (low, high)
        }
        _ => panic!("Invalid JMP addressing mode: {mode:?}"),
    };

    cpu.registers.program_counter = u16::from_le_bytes([low, high]);
}

fn jsr(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    let low = cpu.fetch_instruction_byte(bus);
    let return_address = cpu.registers.program_counter;

    dummy_stack_read(cpu, bus);

    cpu.stack_push_u16(bus, return_address);
    cpu.poll_interrupts();

    let high = cpu.fetch_instruction_byte(bus);
    cpu.registers.program_counter = u16::from_le_bytes([low, high]);
}

fn lda(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let set = cpu.read_operand(bus, mode);

    set_accumulator(cpu, set);
}

fn ldx(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    cpu.registers.x = cpu.read_operand(bus, mode);
    update_zero_and_negative_flags(cpu, cpu.registers.x);
}

fn ldy(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    cpu.registers.y = cpu.read_operand(bus, mode);
    update_zero_and_negative_flags(cpu, cpu.registers.y);
}

fn lsr(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let old_value = match mode {
        AddrMode::Accumulator => {
            cpu.poll_interrupts();
            cpu.read(bus, cpu.registers.program_counter);
            let old_value = cpu.registers.accumulator;

            set_accumulator(cpu, old_value >> 1);

            old_value
        }
        _ => {
            let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
            let old_value = cpu.read(bus, addr);
            let result = old_value >> 1;

            cpu.write(bus, addr, old_value);
            cpu.poll_interrupts();
            cpu.write(bus, addr, result);
            update_zero_and_negative_flags(cpu, result);

            old_value
        }
    };

    cpu.registers.status.set_carry(old_value & 1 == 1);
}

fn nop(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
}

fn ora(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);
    let result = cpu.registers.accumulator | value;

    set_accumulator(cpu, result);
}

fn pha(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.read(bus, cpu.registers.program_counter);
    cpu.poll_interrupts();
    cpu.stack_push(bus, cpu.registers.accumulator);
}

fn php(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    let mut status = cpu.registers.status.clone();
    status.set_b(0b11);

    cpu.read(bus, cpu.registers.program_counter);
    cpu.poll_interrupts();
    cpu.stack_push(bus, status.bits());
}

fn pla(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.read(bus, cpu.registers.program_counter);
    dummy_stack_read(cpu, bus);
    cpu.poll_interrupts();

    let result = cpu.stack_pop(bus);

    set_accumulator(cpu, result);
}

fn plp(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.read(bus, cpu.registers.program_counter);
    dummy_stack_read(cpu, bus);
    cpu.poll_interrupts();

    let result = cpu.stack_pop(bus);

    cpu.registers.status.set_bits(result);
    cpu.registers.status.set_b(0b10);
}

fn rol(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let old_value = match mode {
        AddrMode::Accumulator => {
            cpu.poll_interrupts();
            cpu.read(bus, cpu.registers.program_counter);
            let old_value = cpu.registers.accumulator;
            let result = (old_value << 1) | (cpu.registers.status.carry() as u8);

            set_accumulator(cpu, result);

            old_value
        }
        _ => {
            let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
            let old_value = cpu.read(bus, addr);
            let result = (old_value << 1) | (cpu.registers.status.carry() as u8);

            cpu.write(bus, addr, old_value);
            cpu.poll_interrupts();
            cpu.write(bus, addr, result);

            update_zero_and_negative_flags(cpu, result);

            old_value
        }
    };

    cpu.registers.status.set_carry(old_value >> 7 == 1);
}

fn ror(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let old_value = match mode {
        AddrMode::Accumulator => {
            cpu.poll_interrupts();
            cpu.read(bus, cpu.registers.program_counter);
            let old_value = cpu.registers.accumulator;
            let result = (old_value >> 1) | ((cpu.registers.status.carry() as u8) << 7);

            set_accumulator(cpu, result);

            old_value
        }
        _ => {
            let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
            let old_value = cpu.read(bus, addr);
            let result = (old_value >> 1) | ((cpu.registers.status.carry() as u8) << 7);

            cpu.write(bus, addr, old_value);
            cpu.poll_interrupts();
            cpu.write(bus, addr, result);
            update_zero_and_negative_flags(cpu, result);

            old_value
        }
    };

    cpu.registers.status.set_carry(old_value & 1 == 1);
}

fn rti(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.read(bus, cpu.registers.program_counter);
    dummy_stack_read(cpu, bus);

    let status = cpu.stack_pop(bus);
    cpu.registers.status.set_bits(status);
    cpu.registers.status.set_b(0b10);

    let low = cpu.stack_pop(bus);
    cpu.poll_interrupts();
    let high = cpu.stack_pop(bus);

    cpu.registers.program_counter = u16::from_le_bytes([low, high]);
}

fn rts(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.read(bus, cpu.registers.program_counter);
    dummy_stack_read(cpu, bus);

    let address = cpu.stack_pop_u16(bus);

    cpu.poll_interrupts();
    cpu.read(bus, address);

    cpu.registers.program_counter = address.wrapping_add(1);
}

fn sbc(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let temp = cpu.read_operand(bus, mode);
    let value = temp.wrapping_neg().wrapping_sub(1);

    add_to_accumulator(cpu, value);
}

fn sec(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.status.set_carry(true);
}

fn sed(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.status.set_decimal(true);
}

fn sei(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.status.set_interrupt_disable(true);
}

fn sta(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::Write);
    let value = cpu.registers.accumulator;

    cpu.poll_interrupts();
    cpu.write(bus, addr, value);
}

fn stx(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::Write);
    let value = cpu.registers.x;

    cpu.poll_interrupts();
    cpu.write(bus, addr, value);
}

fn sty(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::Write);
    let value = cpu.registers.y;

    cpu.poll_interrupts();
    cpu.write(bus, addr, value);
}

fn tax(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.x = cpu.registers.accumulator;

    update_zero_and_negative_flags(cpu, cpu.registers.x);
}

fn tay(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.y = cpu.registers.accumulator;

    update_zero_and_negative_flags(cpu, cpu.registers.y);
}

fn tsx(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.x = cpu.registers.stack_pointer;

    update_zero_and_negative_flags(cpu, cpu.registers.x);
}

fn txa(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    set_accumulator(cpu, cpu.registers.x);
}

fn txs(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    cpu.registers.stack_pointer = cpu.registers.x;
}

fn tya(cpu: &mut Cpu, bus: &mut CpuBus, _mode: &AddrMode) {
    cpu.poll_interrupts();
    cpu.read(bus, cpu.registers.program_counter);
    set_accumulator(cpu, cpu.registers.y);
}

// ILLEGAL INSTRUCTIONS

fn anc(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    let result = cpu.registers.accumulator & value;

    set_accumulator(cpu, result);

    cpu.registers
        .status
        .set_carry(cpu.registers.status.negative());
}

fn arr(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    let result =
        ((cpu.registers.accumulator & value) >> 1) | ((cpu.registers.status.carry() as u8) << 7);

    cpu.registers.status.set_carry(value & 1 == 1);

    set_accumulator(cpu, result);

    let accumulator = cpu.registers.accumulator;
    let fifth_bit = (accumulator >> 5) & 1;
    let sixth_bit = (accumulator >> 6) & 1;

    cpu.registers.status.set_carry(sixth_bit == 1);
    cpu.registers
        .status
        .set_overflow(fifth_bit ^ sixth_bit == 1);
    update_zero_and_negative_flags(cpu, accumulator);
}

fn asr(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);
    let result = cpu.registers.accumulator & value;

    cpu.registers.status.set_carry(result & 1 != 0);

    set_accumulator(cpu, result >> 1);
}

fn axa(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::Write);
    let value = cpu.registers.x & cpu.registers.accumulator & (addr >> 8) as u8;

    cpu.poll_interrupts();
    cpu.write(bus, addr, value);
}

fn axs(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);
    let x_and_a = cpu.registers.x & cpu.registers.accumulator;
    let result = x_and_a.wrapping_sub(value);

    cpu.registers.status.set_carry(x_and_a >= value);
    update_zero_and_negative_flags(cpu, result);

    cpu.registers.x = result;
}

fn dcp(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
    let value = cpu.read(bus, addr);
    let result = value.wrapping_sub(1);

    cpu.write(bus, addr, value);
    cpu.poll_interrupts();
    cpu.write(bus, addr, result);

    compare(cpu, cpu.registers.accumulator, result);
}

fn dop(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    cpu.read_operand(bus, mode);
}

fn isb(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
    let value = increment_memory(cpu, bus, addr);
    let result = (value as i8).wrapping_neg().wrapping_sub(1) as u8;

    add_to_accumulator(cpu, result);
}

fn kil(_cpu: &mut Cpu, _bus: &mut CpuBus, _mode: &AddrMode) {}

fn las(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);
    let result = cpu.registers.stack_pointer & value;

    set_accumulator(cpu, result);
    cpu.registers.x = result;
    cpu.registers.stack_pointer = result;
}

fn lax(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    cpu.registers.x = value;
    set_accumulator(cpu, value);
}

fn lxa(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    cpu.registers.x = value;
    set_accumulator(cpu, value);
}

fn rla(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
    let value = cpu.read(bus, addr);
    let result = (value << 1) | (cpu.registers.status.carry() as u8);

    cpu.write(bus, addr, value);
    cpu.poll_interrupts();
    cpu.write(bus, addr, result);

    cpu.registers.status.set_carry(value >> 7 == 1);

    set_accumulator(cpu, cpu.registers.accumulator & result);
}

fn rra(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
    let value = cpu.read(bus, addr);
    let result = (value >> 1) | (cpu.registers.status.carry() as u8) << 7;

    cpu.write(bus, addr, value);
    cpu.poll_interrupts();
    cpu.write(bus, addr, result);

    cpu.registers.status.set_carry(value & 1 == 1);

    add_to_accumulator(cpu, result);
}

fn sax(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::Write);

    let result = cpu.registers.accumulator & cpu.registers.x;

    cpu.poll_interrupts();
    cpu.write(bus, addr, result);
}

fn slo(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
    let value = cpu.read(bus, addr);
    let result = value << 1;

    cpu.write(bus, addr, value);
    cpu.poll_interrupts();
    cpu.write(bus, addr, result);

    cpu.registers.status.set_carry(value >> 7 == 1);

    set_accumulator(cpu, result | cpu.registers.accumulator);
}

fn sre(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::ReadModifyWrite);
    let value = cpu.read(bus, addr);
    let result = value >> 1;

    cpu.write(bus, addr, value);
    cpu.poll_interrupts();
    cpu.write(bus, addr, result);

    cpu.registers.status.set_carry(value & 1 == 1);

    set_accumulator(cpu, result ^ cpu.registers.accumulator);
}

fn sxa(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::Write);
    let value = cpu.registers.x & ((addr >> 8) as u8 + 1);

    cpu.poll_interrupts();
    cpu.write(bus, addr, value);
}

fn sya(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::Write);
    let value = cpu.registers.y & ((addr >> 8) as u8 + 1);

    cpu.poll_interrupts();
    cpu.write(bus, addr, value);
}

fn top(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    cpu.read_operand(bus, mode);
}

// This guy isn't super well documented, this seems like what it does?
fn xaa(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let value = cpu.read_operand(bus, mode);

    set_accumulator(cpu, cpu.registers.x);
    set_accumulator(cpu, cpu.registers.accumulator & value);
}

fn xas(cpu: &mut Cpu, bus: &mut CpuBus, mode: &AddrMode) {
    let result = cpu.registers.x & cpu.registers.accumulator;
    cpu.registers.stack_pointer = result;

    let addr = cpu.fetch_operand_address(bus, mode, AccessKind::Write);
    let value = result & ((addr >> 8) as u8 + 1);

    cpu.poll_interrupts();
    cpu.write(bus, addr, value);
}

fn add_to_accumulator(cpu: &mut Cpu, value: u8) {
    let result: u16 = cpu.registers.accumulator as u16
        + value as u16
        + Into::<u16>::into(cpu.registers.status.carry());
    cpu.registers.status.set_carry(result > 0xFF);

    let result = result as u8;

    cpu.registers
        .status
        .set_overflow((value ^ result) & (result ^ cpu.registers.accumulator) & 0x80 != 0);

    set_accumulator(cpu, result);
}

fn increment_memory(cpu: &mut Cpu, bus: &mut CpuBus, addr: u16) -> u8 {
    let old_value = cpu.read(bus, addr);
    let result = old_value.wrapping_add(1);

    cpu.write(bus, addr, old_value);
    cpu.poll_interrupts();
    cpu.write(bus, addr, result);
    update_zero_and_negative_flags(cpu, result);

    result
}

fn set_accumulator(cpu: &mut Cpu, value: u8) {
    cpu.registers.accumulator = value;

    update_zero_and_negative_flags(cpu, value);
}

fn update_zero_and_negative_flags(cpu: &mut Cpu, value: u8) {
    cpu.registers.status.set_zero(value == 0);
    cpu.registers.status.set_negative(value >> 7 == 1);
}

fn branch(cpu: &mut Cpu, bus: &mut CpuBus, condition: bool) {
    cpu.poll_interrupts();
    let offset = cpu.fetch_instruction_byte(bus) as i8;
    let next_address = cpu.registers.program_counter;

    if !condition {
        return;
    }

    cpu.read(bus, next_address);

    let target = next_address.wrapping_add_signed(i16::from(offset));

    if next_address & 0xFF00 != target & 0xFF00 {
        let dummy_address = (next_address & 0xFF00) | (target & 0x00FF);

        cpu.poll_interrupts();
        cpu.read(bus, dummy_address);
    }

    cpu.registers.program_counter = target;
}

fn compare(cpu: &mut Cpu, register: u8, value: u8) {
    let result = register.wrapping_sub(value);

    cpu.registers.status.set_carry(register >= value);
    update_zero_and_negative_flags(cpu, result);
}

fn dummy_stack_read(cpu: &mut Cpu, bus: &mut CpuBus) {
    let stack_address: u16 = 0x0100 | u16::from(cpu.registers.stack_pointer);

    cpu.read(bus, stack_address);
}

#[cfg(test)]
mod test {
    use crate::{NES, StepKind};

    use super::*;

    #[test]
    fn brk_saves_pc_plus_two_and_old_status_and_rti_skips_padding() {
        for status in [0xEB, 0xEF] {
            let mut nes = NES::default();
            nes.bus.cartridge.prg_rom = vec![0; 0x4000];
            nes.bus.cartridge.prg_rom[0x3FFA..0x3FFC].copy_from_slice(&[0x00, 0x07]);
            nes.bus.cartridge.prg_rom[0x3FFE..0x4000].copy_from_slice(&[0x00, 0x06]);
            nes.bus.write(0x0200, 0x00); // BRK
            nes.bus.write(0x0201, 0x02); // Padding is discarded, not executed.
            nes.bus.write(0x0600, 0x40); // RTI
            nes.cpu.registers.program_counter = 0x0200;
            nes.cpu.registers.stack_pointer = 0xFD;
            nes.cpu.registers.status.set_bits(status);

            let entry = nes.step();

            assert!(matches!(
                entry.kind,
                StepKind::Instructrion {
                    pc: 0x0200,
                    opcode: 0x00
                }
            ));
            assert_eq!(entry.cpu_cycles, 7);
            assert_eq!(nes.bus.ppu.dot, 21);
            assert_eq!(nes.cpu.registers.program_counter, 0x0600);
            assert_eq!(nes.cpu.registers.stack_pointer, 0xFA);
            assert_eq!(nes.bus.peek(0x01FD), 0x02);
            assert_eq!(nes.bus.peek(0x01FC), 0x02);
            assert_eq!(nes.bus.peek(0x01FB), status | 0x10); // B set; original I.
            assert_eq!(nes.cpu.registers.status.bits(), status | 0x04);

            let resumed = nes.step();

            assert_eq!(resumed.cpu_cycles, 6);
            assert_eq!(nes.cpu.registers.program_counter, 0x0202);
            assert_eq!(nes.cpu.registers.stack_pointer, 0xFD);
            assert_eq!(nes.cpu.registers.status.bits(), status);
        }
    }

    #[test]
    fn brk_padding_fetch_wraps_pc() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        // $FFFF is both the BRK opcode and the IRQ vector's high byte.
        nes.bus.cartridge.prg_rom[0x3FFE] = 0x80;
        nes.bus.write(0x0000, 0xEA);
        nes.cpu.registers.program_counter = 0xFFFF;
        nes.cpu.registers.stack_pointer = 0xFD;

        let entry = nes.step();

        assert_eq!(entry.cpu_cycles, 7);
        assert_eq!(nes.cpu.registers.program_counter, 0x0080);
        assert_eq!(nes.bus.peek(0x01FD), 0x00);
        assert_eq!(nes.bus.peek(0x01FC), 0x01);
    }

    #[test]
    fn brk_padding_fetch_applies_bus_side_effects_once() {
        let mut nes = NES::default();
        nes.bus.cartridge.prg_rom = vec![0; 0x4000];
        nes.bus.cartridge.prg_rom[0x3FFE..0x4000].copy_from_slice(&[0x00, 0x06]);
        nes.bus.controller.button_state.set_a(true);
        nes.bus.controller.button_state.set_select(true);
        nes.bus.write(0x0000, 0); // APU status stub at $4015 will read $00 (BRK).
        nes.cpu.registers.program_counter = 0x4015;
        nes.cpu.registers.stack_pointer = 0xFD;

        let entry = nes.step();

        assert!(matches!(
            entry.kind,
            StepKind::Instructrion {
                pc: 0x4015,
                opcode: 0x00
            }
        ));
        assert_eq!(entry.cpu_cycles, 7);
        assert_eq!(nes.cpu.registers.program_counter, 0x0600);
        assert_eq!(nes.bus.peek(0x01FD), 0x40);
        assert_eq!(nes.bus.peek(0x01FC), 0x17);
        assert_eq!(nes.bus.controller.peek(), 0); // One read consumed A; B is next.
    }

    #[test]
    fn lsr_accumulator_uses_two_cycles_without_writing_memory() {
        let mut nes = NES::default();
        nes.bus.write(0x0000, 0x4A); // LSR A
        nes.bus.write(0x0001, 0xEA);
        nes.cpu.registers.accumulator = 0x01;
        nes.cpu.registers.status.set_negative(true);

        let result = nes.step();

        assert_eq!(result.cpu_cycles, 2);
        assert_eq!(nes.cpu.registers.program_counter, 1);
        assert_eq!(nes.cpu.registers.accumulator, 0);
        assert!(nes.cpu.registers.status.carry());
        assert!(nes.cpu.registers.status.zero());
        assert!(!nes.cpu.registers.status.negative());
        assert_eq!(nes.bus.peek(0x0000), 0x4A);
        assert_eq!(nes.bus.peek(0x0001), 0xEA);
    }

    #[test]
    fn lsr_memory_writes_original_then_shifted_value_to_io() {
        let mut nes = NES::default();
        nes.bus.write(0x0000, 0x4E); // LSR $2004 (OAMDATA)
        nes.bus.write(0x0001, 0x04);
        nes.bus.write(0x0002, 0x20);
        nes.bus.write(0x2003, 0xFF);
        nes.bus.ppu.oam_data[0xFF] = 0x03;
        nes.cpu.registers.accumulator = 0xA5;

        let result = nes.step();

        assert_eq!(result.cpu_cycles, 6);
        assert_eq!(nes.cpu.registers.program_counter, 3);
        assert_eq!(nes.cpu.registers.accumulator, 0xA5);
        // Each write to OAMDATA increments OAMADDR, exposing both writes.
        assert_eq!(nes.bus.ppu.oam_data[0xFF], 0x03);
        assert_eq!(nes.bus.ppu.oam_data[0x00], 0x01);
        assert!(nes.cpu.registers.status.carry());
        assert!(!nes.cpu.registers.status.zero());
        assert!(!nes.cpu.registers.status.negative());
    }

    #[test]
    fn lda_sets_accumulator() {
        let mut nes = NES::default();

        nes.cpu.registers.accumulator = 0x00;
        nes.cpu.registers.program_counter += 1;
        nes.bus.write(0x0001, 0x01);

        lda(&mut nes.cpu, &mut nes.bus, &AddrMode::Immediate);

        assert_eq!(nes.cpu.registers.accumulator, 0x01);
    }

    #[test]
    fn lda_sets_zero_flag() {
        let mut nes = NES::default();

        nes.cpu.registers.accumulator = 0x00;
        nes.cpu.registers.program_counter += 1;
        nes.bus.write(0x0001, 0x00);

        lda(&mut nes.cpu, &mut nes.bus, &AddrMode::Immediate);

        assert_eq!(nes.cpu.registers.accumulator, 0x00);
        assert_eq!(nes.cpu.registers.status.zero(), true);
    }

    #[test]
    fn lda_sets_negative_flag() {
        let mut nes = NES::default();

        nes.cpu.registers.accumulator = 0x00;
        nes.cpu.registers.program_counter += 1;
        nes.bus.write(0x0001, 0xFF);

        lda(&mut nes.cpu, &mut nes.bus, &AddrMode::Immediate);

        assert_eq!(nes.cpu.registers.accumulator, 0xFF);
        assert_eq!(nes.cpu.registers.status.negative(), true);
    }
}
