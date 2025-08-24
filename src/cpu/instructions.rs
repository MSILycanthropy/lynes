use crate::NES;

use super::{AddrMode, CPU};

macro_rules! instr {
    ($name: expr, $mode: expr, $cycles: expr, $len: expr, $fn: expr) => {
        Instruction {
            name: $name,
            mode: $mode,
            cycles: $cycles,
            len: $len,
            operate: $fn,
            legal: true,
        }
    };
}

macro_rules! il_instr {
    ($name: expr, $mode: expr, $cycles: expr, $len: expr, $fn: expr) => {
        Instruction {
            name: $name,
            mode: $mode,
            cycles: $cycles,
            len: $len,
            operate: $fn,
            legal: false,
        }
    };
}

pub(crate) const INSTRUCTIONS_TABLE: [Instruction; 256] = [
    instr!("BRK", AddrMode::Implied, 7, 1, brk),
    instr!("ORA", AddrMode::IndirectX, 6, 2, ora),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil), // This technically is not correct, KIL explodes the CPU but we'll treat it as a NOP
    il_instr!("SLO", AddrMode::IndirectX, 8, 2, slo),
    il_instr!("DOP", AddrMode::ZeroPage, 3, 2, dop),
    instr!("ORA", AddrMode::ZeroPage, 3, 2, ora),
    instr!("ASL", AddrMode::ZeroPage, 5, 2, asl),
    il_instr!("SLO", AddrMode::ZeroPage, 5, 2, slo),
    instr!("PHP", AddrMode::Implied, 3, 1, php),
    instr!("ORA", AddrMode::Immediate, 2, 2, ora),
    instr!("ASL", AddrMode::Accumulator, 2, 1, asl),
    il_instr!("ANC", AddrMode::Immediate, 2, 2, anc),
    il_instr!("TOP", AddrMode::Absolute, 4, 3, top),
    instr!("ORA", AddrMode::Absolute, 4, 3, ora),
    instr!("ASL", AddrMode::Absolute, 6, 3, asl),
    il_instr!("SLO", AddrMode::Absolute, 6, 3, slo),
    instr!("BPL", AddrMode::Relative, 2, 2, bpl),
    instr!("ORA", AddrMode::IndirectY, 5, 2, ora),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("SLO", AddrMode::IndirectY, 8, 2, slo),
    il_instr!("DOP", AddrMode::ZeroPageX, 4, 2, dop),
    instr!("ORA", AddrMode::ZeroPageX, 4, 2, ora),
    instr!("ASL", AddrMode::ZeroPageX, 6, 2, asl),
    il_instr!("SLO", AddrMode::ZeroPageX, 6, 2, slo),
    instr!("CLC", AddrMode::Implied, 2, 1, clc),
    instr!("ORA", AddrMode::AbsoluteY, 4, 3, ora),
    il_instr!("NOP", AddrMode::Implied, 2, 1, nop),
    il_instr!("SLO", AddrMode::AbsoluteY, 7, 3, slo),
    il_instr!("TOP", AddrMode::AbsoluteX, 4, 3, top),
    instr!("ORA", AddrMode::AbsoluteX, 4, 3, ora),
    instr!("ASL", AddrMode::AbsoluteX, 7, 3, asl),
    il_instr!("SLO", AddrMode::AbsoluteX, 7, 3, slo),
    instr!("JSR", AddrMode::Absolute, 6, 3, jsr),
    instr!("AND", AddrMode::IndirectX, 6, 2, and),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("RLA", AddrMode::IndirectX, 8, 2, rla),
    instr!("BIT", AddrMode::ZeroPage, 3, 2, bit),
    instr!("AND", AddrMode::ZeroPage, 3, 2, and),
    instr!("ROL", AddrMode::ZeroPage, 5, 2, rol),
    il_instr!("RLA", AddrMode::ZeroPage, 5, 2, rla),
    instr!("PLP", AddrMode::Implied, 4, 1, plp),
    instr!("AND", AddrMode::Immediate, 2, 2, and),
    instr!("ROL", AddrMode::Accumulator, 2, 1, rol),
    il_instr!("ANC", AddrMode::Immediate, 2, 2, anc),
    instr!("BIT", AddrMode::Absolute, 4, 3, bit),
    instr!("AND", AddrMode::Absolute, 4, 3, and),
    instr!("ROL", AddrMode::Absolute, 6, 3, rol),
    il_instr!("RLA", AddrMode::Absolute, 6, 3, rla),
    instr!("BMI", AddrMode::Relative, 2, 2, bmi),
    instr!("AND", AddrMode::IndirectY, 5, 2, and),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("RLA", AddrMode::IndirectY, 8, 2, rla),
    il_instr!("DOP", AddrMode::ZeroPageX, 4, 2, dop),
    instr!("AND", AddrMode::ZeroPageX, 4, 2, and),
    instr!("ROL", AddrMode::ZeroPageX, 6, 2, rol),
    il_instr!("RLA", AddrMode::ZeroPageX, 6, 2, rla),
    instr!("SEC", AddrMode::Implied, 2, 1, sec),
    instr!("AND", AddrMode::AbsoluteY, 4, 3, and),
    il_instr!("NOP", AddrMode::Implied, 2, 1, nop),
    il_instr!("RLA", AddrMode::AbsoluteY, 7, 3, rla),
    il_instr!("TOP", AddrMode::AbsoluteX, 4, 3, top),
    instr!("AND", AddrMode::AbsoluteX, 4, 3, and),
    instr!("ROL", AddrMode::AbsoluteX, 7, 3, rol),
    il_instr!("RLA", AddrMode::AbsoluteX, 7, 3, rla),
    instr!("RTI", AddrMode::Implied, 6, 1, rti),
    instr!("EOR", AddrMode::IndirectX, 6, 2, eor),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("SRE", AddrMode::IndirectX, 8, 2, sre),
    il_instr!("DOP", AddrMode::ZeroPage, 3, 2, dop),
    instr!("EOR", AddrMode::ZeroPage, 3, 2, eor),
    instr!("LSR", AddrMode::ZeroPage, 5, 2, lsr),
    il_instr!("SRE", AddrMode::ZeroPage, 5, 2, sre),
    instr!("PHA", AddrMode::Implied, 3, 1, pha),
    instr!("EOR", AddrMode::Immediate, 2, 2, eor),
    instr!("LSR", AddrMode::Accumulator, 2, 1, lsr),
    il_instr!("ASR", AddrMode::Immediate, 2, 2, asr),
    instr!("JMP", AddrMode::Absolute, 3, 3, jmp),
    instr!("EOR", AddrMode::Absolute, 4, 3, eor),
    instr!("LSR", AddrMode::Absolute, 6, 3, lsr),
    il_instr!("SRE", AddrMode::Absolute, 6, 3, sre),
    instr!("BVC", AddrMode::Relative, 2, 2, bvc),
    instr!("EOR", AddrMode::IndirectY, 5, 2, eor),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("SRE", AddrMode::IndirectY, 8, 2, sre),
    il_instr!("DOP", AddrMode::ZeroPageX, 4, 2, dop),
    instr!("EOR", AddrMode::ZeroPageX, 4, 2, eor),
    instr!("LSR", AddrMode::ZeroPageX, 6, 2, lsr),
    il_instr!("SRE", AddrMode::ZeroPageX, 6, 2, sre),
    instr!("CLI", AddrMode::Implied, 2, 1, cli),
    instr!("EOR", AddrMode::AbsoluteY, 4, 3, eor),
    il_instr!("NOP", AddrMode::Implied, 2, 1, nop),
    il_instr!("SRE", AddrMode::AbsoluteY, 7, 3, sre),
    il_instr!("TOP", AddrMode::AbsoluteX, 4, 3, top),
    instr!("EOR", AddrMode::AbsoluteX, 4, 3, eor),
    instr!("LSR", AddrMode::AbsoluteX, 7, 3, lsr),
    il_instr!("SRE", AddrMode::AbsoluteX, 7, 3, sre),
    instr!("RTS", AddrMode::Implied, 6, 1, rts),
    instr!("ADC", AddrMode::IndirectX, 6, 2, adc),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("RRA", AddrMode::IndirectX, 8, 2, rra),
    il_instr!("DOP", AddrMode::ZeroPage, 3, 2, dop),
    instr!("ADC", AddrMode::ZeroPage, 3, 2, adc),
    instr!("ROR", AddrMode::ZeroPage, 5, 2, ror),
    il_instr!("RRA", AddrMode::ZeroPage, 5, 2, rra),
    instr!("PLA", AddrMode::Implied, 4, 1, pla),
    instr!("ADC", AddrMode::Immediate, 2, 2, adc),
    instr!("ROR", AddrMode::Accumulator, 2, 1, ror),
    il_instr!("ARR", AddrMode::Immediate, 2, 2, arr),
    instr!("JMP", AddrMode::Indirect, 5, 3, jmp),
    instr!("ADC", AddrMode::Absolute, 4, 3, adc),
    instr!("ROR", AddrMode::Absolute, 6, 3, ror),
    il_instr!("RRA", AddrMode::Absolute, 6, 3, rra),
    instr!("BVS", AddrMode::Relative, 2, 2, bvs),
    instr!("ADC", AddrMode::IndirectY, 5, 2, adc),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("RRA", AddrMode::IndirectY, 8, 2, rra),
    il_instr!("DOP", AddrMode::ZeroPageX, 4, 2, dop),
    instr!("ADC", AddrMode::ZeroPageX, 4, 2, adc),
    instr!("ROR", AddrMode::ZeroPageX, 6, 2, ror),
    il_instr!("RRA", AddrMode::ZeroPageX, 6, 2, rra),
    instr!("SEI", AddrMode::Implied, 2, 1, sei),
    instr!("ADC", AddrMode::AbsoluteY, 4, 3, adc),
    il_instr!("NOP", AddrMode::Implied, 2, 1, nop),
    il_instr!("RRA", AddrMode::AbsoluteY, 7, 3, rra),
    il_instr!("TOP", AddrMode::AbsoluteX, 4, 3, top),
    instr!("ADC", AddrMode::AbsoluteX, 4, 3, adc),
    instr!("ROR", AddrMode::AbsoluteX, 7, 3, ror),
    il_instr!("RRA", AddrMode::AbsoluteX, 7, 3, rra),
    il_instr!("DOP", AddrMode::Immediate, 2, 2, dop),
    instr!("STA", AddrMode::IndirectX, 6, 2, sta),
    il_instr!("DOP", AddrMode::Immediate, 2, 2, dop),
    il_instr!("SAX", AddrMode::IndirectX, 6, 2, sax),
    instr!("STY", AddrMode::ZeroPage, 3, 2, sty),
    instr!("STA", AddrMode::ZeroPage, 3, 2, sta),
    instr!("STX", AddrMode::ZeroPage, 3, 2, stx),
    il_instr!("SAX", AddrMode::ZeroPage, 3, 2, sax),
    instr!("DEY", AddrMode::Implied, 2, 1, dey),
    il_instr!("DOP", AddrMode::Immediate, 2, 2, dop),
    instr!("TXA", AddrMode::Implied, 2, 1, txa),
    il_instr!("XAA", AddrMode::Immediate, 2, 2, xaa),
    instr!("STY", AddrMode::Absolute, 4, 3, sty),
    instr!("STA", AddrMode::Absolute, 4, 3, sta),
    instr!("STX", AddrMode::Absolute, 4, 3, stx),
    il_instr!("SAX", AddrMode::Absolute, 4, 3, sax),
    instr!("BCC", AddrMode::Relative, 2, 2, bcc),
    instr!("STA", AddrMode::IndirectY, 6, 2, sta),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("AXA", AddrMode::IndirectY, 6, 2, axa),
    instr!("STY", AddrMode::ZeroPageX, 4, 2, sty),
    instr!("STA", AddrMode::ZeroPageX, 4, 2, sta),
    instr!("STX", AddrMode::ZeroPageY, 4, 2, stx),
    il_instr!("SAX", AddrMode::ZeroPageY, 4, 2, sax),
    instr!("TYA", AddrMode::Implied, 2, 1, tya),
    instr!("STA", AddrMode::AbsoluteY, 5, 3, sta),
    instr!("TXS", AddrMode::Implied, 2, 1, txs),
    il_instr!("XAS", AddrMode::AbsoluteY, 5, 3, xas),
    il_instr!("SYA", AddrMode::AbsoluteX, 5, 3, sya),
    instr!("STA", AddrMode::AbsoluteX, 5, 3, sta),
    il_instr!("SXA", AddrMode::AbsoluteY, 5, 3, sxa),
    il_instr!("AXA", AddrMode::AbsoluteY, 5, 3, axa),
    instr!("LDY", AddrMode::Immediate, 2, 2, ldy),
    instr!("LDA", AddrMode::IndirectX, 6, 2, lda),
    instr!("LDX", AddrMode::Immediate, 2, 2, ldx),
    il_instr!("LAX", AddrMode::IndirectX, 6, 2, lax),
    instr!("LDY", AddrMode::ZeroPage, 3, 2, ldy),
    instr!("LDA", AddrMode::ZeroPage, 3, 2, lda),
    instr!("LDX", AddrMode::ZeroPage, 3, 2, ldx),
    il_instr!("LAX", AddrMode::ZeroPage, 3, 2, lax),
    instr!("TAY", AddrMode::Implied, 2, 1, tay),
    instr!("LDA", AddrMode::Immediate, 2, 2, lda),
    instr!("TAX", AddrMode::Implied, 2, 1, tax),
    il_instr!("LXA", AddrMode::Immediate, 2, 2, lxa),
    instr!("LDY", AddrMode::Absolute, 4, 3, ldy),
    instr!("LDA", AddrMode::Absolute, 4, 3, lda),
    instr!("LDX", AddrMode::Absolute, 4, 3, ldx),
    il_instr!("LAX", AddrMode::Absolute, 4, 3, lax),
    instr!("BCS", AddrMode::Relative, 2, 2, bcs),
    instr!("LDA", AddrMode::IndirectY, 5, 2, lda),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("LAX", AddrMode::IndirectY, 5, 2, lax),
    instr!("LDY", AddrMode::ZeroPageX, 4, 2, ldy),
    instr!("LDA", AddrMode::ZeroPageX, 4, 2, lda),
    instr!("LDX", AddrMode::ZeroPageY, 4, 2, ldx),
    il_instr!("LAX", AddrMode::ZeroPageY, 4, 2, lax),
    instr!("CLV", AddrMode::Implied, 2, 1, clv),
    instr!("LDA", AddrMode::AbsoluteY, 4, 3, lda),
    instr!("TSX", AddrMode::Implied, 2, 1, tsx),
    il_instr!("LAS", AddrMode::AbsoluteY, 4, 3, las),
    instr!("LDY", AddrMode::AbsoluteX, 4, 3, ldy),
    instr!("LDA", AddrMode::AbsoluteX, 4, 3, lda),
    instr!("LDX", AddrMode::AbsoluteY, 4, 3, ldx),
    il_instr!("LAX", AddrMode::AbsoluteY, 4, 3, lax),
    instr!("CPY", AddrMode::Immediate, 2, 2, cpy),
    instr!("CMP", AddrMode::IndirectX, 6, 2, cmp),
    il_instr!("DOP", AddrMode::Immediate, 2, 2, dop),
    il_instr!("DCP", AddrMode::IndirectX, 8, 2, dcp),
    instr!("CPY", AddrMode::ZeroPage, 3, 2, cpy),
    instr!("CMP", AddrMode::ZeroPage, 3, 2, cmp),
    instr!("DEC", AddrMode::ZeroPage, 5, 2, dec),
    il_instr!("DCP", AddrMode::ZeroPage, 5, 2, dcp),
    instr!("INY", AddrMode::Implied, 2, 1, iny),
    instr!("CMP", AddrMode::Immediate, 2, 2, cmp),
    instr!("DEX", AddrMode::Implied, 2, 1, dex),
    il_instr!("AXS", AddrMode::Immediate, 2, 2, axs),
    instr!("CPY", AddrMode::Absolute, 4, 3, cpy),
    instr!("CMP", AddrMode::Absolute, 4, 3, cmp),
    instr!("DEC", AddrMode::Absolute, 6, 3, dec),
    il_instr!("DCP", AddrMode::Absolute, 6, 3, dcp),
    instr!("BNE", AddrMode::Relative, 2, 2, bne),
    instr!("CMP", AddrMode::IndirectY, 5, 2, cmp),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("DCP", AddrMode::IndirectY, 8, 2, dcp),
    il_instr!("DOP", AddrMode::ZeroPageX, 4, 2, dop),
    instr!("CMP", AddrMode::ZeroPageX, 4, 2, cmp),
    instr!("DEC", AddrMode::ZeroPageX, 6, 2, dec),
    il_instr!("DCP", AddrMode::ZeroPageX, 6, 2, dcp),
    instr!("CLD", AddrMode::Implied, 2, 1, cld),
    instr!("CMP", AddrMode::AbsoluteY, 4, 3, cmp),
    il_instr!("NOP", AddrMode::Implied, 2, 1, nop),
    il_instr!("DCP", AddrMode::AbsoluteY, 7, 3, dcp),
    il_instr!("TOP", AddrMode::AbsoluteX, 4, 3, top),
    instr!("CMP", AddrMode::AbsoluteX, 4, 3, cmp),
    instr!("DEC", AddrMode::AbsoluteX, 7, 3, dec),
    il_instr!("DCP", AddrMode::AbsoluteX, 7, 3, dcp),
    instr!("CPX", AddrMode::Immediate, 2, 2, cpx),
    instr!("SBC", AddrMode::IndirectX, 6, 2, sbc),
    il_instr!("DOP", AddrMode::Immediate, 2, 2, dop),
    il_instr!("ISB", AddrMode::IndirectX, 8, 2, isb),
    instr!("CPX", AddrMode::ZeroPage, 3, 2, cpx),
    instr!("SBC", AddrMode::ZeroPage, 3, 2, sbc),
    instr!("INC", AddrMode::ZeroPage, 5, 2, inc),
    il_instr!("ISB", AddrMode::ZeroPage, 5, 2, isb),
    instr!("INX", AddrMode::Implied, 2, 1, inx),
    instr!("SBC", AddrMode::Immediate, 2, 2, sbc),
    instr!("NOP", AddrMode::Implied, 2, 1, nop),
    il_instr!("SBC", AddrMode::Immediate, 2, 2, sbc),
    instr!("CPX", AddrMode::Absolute, 4, 3, cpx),
    instr!("SBC", AddrMode::Absolute, 4, 3, sbc),
    instr!("INC", AddrMode::Absolute, 6, 3, inc),
    il_instr!("ISB", AddrMode::Absolute, 6, 3, isb),
    instr!("BEQ", AddrMode::Relative, 2, 2, beq),
    instr!("SBC", AddrMode::IndirectY, 5, 2, sbc),
    il_instr!("KIL", AddrMode::Implied, 2, 1, kil),
    il_instr!("ISB", AddrMode::IndirectY, 8, 2, isb),
    il_instr!("DOP", AddrMode::ZeroPageX, 4, 2, dop),
    instr!("SBC", AddrMode::ZeroPageX, 4, 2, sbc),
    instr!("INC", AddrMode::ZeroPageX, 6, 2, inc),
    il_instr!("ISB", AddrMode::ZeroPageX, 6, 2, isb),
    instr!("SED", AddrMode::Implied, 2, 1, sed),
    instr!("SBC", AddrMode::AbsoluteY, 4, 3, sbc),
    il_instr!("NOP", AddrMode::Implied, 2, 1, nop),
    il_instr!("ISB", AddrMode::AbsoluteY, 7, 3, isb),
    il_instr!("TOP", AddrMode::AbsoluteX, 4, 3, top),
    instr!("SBC", AddrMode::AbsoluteX, 4, 3, sbc),
    instr!("INC", AddrMode::AbsoluteX, 7, 3, inc),
    il_instr!("ISB", AddrMode::AbsoluteX, 7, 3, isb),
];

pub struct Instruction {
    pub name: &'static str,
    pub mode: AddrMode,
    pub cycles: usize,
    pub len: u8,
    pub operate: fn(&mut NES, mode: &AddrMode) -> usize,
    pub legal: bool,
}

impl Instruction {
    pub fn execute(&self, nes: &mut NES) -> usize {
        (self.operate)(nes, &self.mode) + self.cycles
    }

    pub fn size(&self) -> u16 {
        (self.len - 1).into()
    }
}

fn adc(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    add_to_accumulator(nes, value);

    page_crossed.into()
}

fn and(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    set_accumulator(nes, nes.cpu_registers.accumulator & value);

    page_crossed.into()
}

fn asl(nes: &mut NES, mode: &AddrMode) -> usize {
    let old_value = if let AddrMode::Accumulator = mode {
        let old_value = nes.cpu_registers.accumulator;

        set_accumulator(nes, old_value << 1);

        old_value
    } else {
        let (addr, _) = nes.get_operating_address(mode);
        let old_value = nes.cpu_read(addr);
        let result = old_value << 1;

        nes.cpu_write(addr, result);
        update_zero_and_negative_flags(nes, result);

        old_value
    };

    nes.cpu_registers.status.set_carry(old_value >> 7 == 1);

    0
}

fn bcc(nes: &mut NES, _mode: &AddrMode) -> usize {
    branch(nes, !nes.cpu_registers.status.carry())
}

fn bcs(nes: &mut NES, _mode: &AddrMode) -> usize {
    branch(nes, nes.cpu_registers.status.carry())
}

fn beq(nes: &mut NES, _mode: &AddrMode) -> usize {
    branch(nes, nes.cpu_registers.status.zero())
}

fn bit(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = value & nes.cpu_registers.accumulator;

    nes.cpu_registers.status.set_zero(result == 0);
    nes.cpu_registers.status.set_overflow(value & 0x40 > 0);
    nes.cpu_registers.status.set_negative(value >> 7 == 1);

    0
}

fn bmi(nes: &mut NES, _mode: &AddrMode) -> usize {
    branch(nes, nes.cpu_registers.status.negative())
}

fn bne(nes: &mut NES, _mode: &AddrMode) -> usize {
    branch(nes, !nes.cpu_registers.status.zero())
}

fn bpl(nes: &mut NES, _mode: &AddrMode) -> usize {
    branch(nes, !nes.cpu_registers.status.negative())
}

fn brk(_nes: &mut NES, _mode: &AddrMode) -> usize {
    // TODO: interrupts
    // std::process::exit(0);
    0
}

fn bvc(nes: &mut NES, _mode: &AddrMode) -> usize {
    branch(nes, !nes.cpu_registers.status.overflow())
}

fn bvs(nes: &mut NES, _mode: &AddrMode) -> usize {
    branch(nes, nes.cpu_registers.status.overflow())
}

fn clc(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.status.set_carry(false);

    0
}

fn cld(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.status.set_decimal(false);

    0
}

fn cli(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.status.set_interrupt_disable(false);

    0
}

fn clv(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.status.set_overflow(false);

    0
}

fn cmp(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    compare(nes, nes.cpu_registers.accumulator, value);

    page_crossed.into()
}

fn cpx(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    compare(nes, nes.cpu_registers.x, value);

    0
}

fn cpy(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    compare(nes, nes.cpu_registers.y, value);

    0
}

fn dec(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let old_value = nes.cpu_read(addr);
    let result = old_value.wrapping_sub(1);

    nes.cpu_write(addr, result);
    update_zero_and_negative_flags(nes, result);

    0
}

fn dex(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.x.wrapping_sub(1);

    nes.cpu_registers.x = result;
    update_zero_and_negative_flags(nes, result);

    0
}

fn dey(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.y.wrapping_sub(1);

    nes.cpu_registers.y = result;
    update_zero_and_negative_flags(nes, result);

    0
}

fn eor(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = nes.cpu_registers.accumulator ^ value;

    set_accumulator(nes, result);

    page_crossed.into()
}

fn inc(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);

    increment_memory(nes, addr);

    0
}

fn inx(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.x.wrapping_add(1);

    nes.cpu_registers.x = result;
    update_zero_and_negative_flags(nes, result);

    0
}

fn iny(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.y.wrapping_add(1);

    nes.cpu_registers.y = result;
    update_zero_and_negative_flags(nes, result);

    0
}

fn jmp(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);

    nes.cpu_registers.program_counter = addr;

    0
}

fn jsr(nes: &mut NES, mode: &AddrMode) -> usize {
    nes.stack_push_u16(nes.cpu_registers.program_counter + 2 - 1);

    let (addr, _) = nes.get_operating_address(mode);

    nes.cpu_registers.program_counter = addr;

    0
}

fn lda(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let set = nes.cpu_read(addr);

    set_accumulator(nes, set);

    page_crossed.into()
}

fn ldx(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);

    nes.cpu_registers.x = nes.cpu_read(addr);
    update_zero_and_negative_flags(nes, nes.cpu_registers.x);

    page_crossed.into()
}

fn ldy(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);

    nes.cpu_registers.y = nes.cpu_read(addr);
    update_zero_and_negative_flags(nes, nes.cpu_registers.y);

    page_crossed.into()
}

fn lsr(nes: &mut NES, mode: &AddrMode) -> usize {
    let old_value = match mode {
        AddrMode::Accumulator => {
            let old_value = nes.cpu_registers.accumulator;

            set_accumulator(nes, old_value >> 1);

            old_value
        }
        _ => {
            let (addr, _) = nes.get_operating_address(mode);
            let old_value = nes.cpu_read(addr);
            let result = old_value >> 1;

            nes.cpu_write(addr, result);
            update_zero_and_negative_flags(nes, result);

            old_value
        }
    };

    nes.cpu_registers.status.set_carry(old_value & 1 == 1);

    0
}

fn nop(_nes: &mut NES, _mode: &AddrMode) -> usize {
    0
}

fn ora(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = nes.cpu_registers.accumulator | value;

    set_accumulator(nes, result);

    page_crossed.into()
}

fn pha(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.stack_push(nes.cpu_registers.accumulator);

    0
}

fn php(nes: &mut NES, _mode: &AddrMode) -> usize {
    let mut status = nes.cpu_registers.status.clone();
    status.set_b(0b11);

    nes.stack_push(status.bits());

    0
}

fn pla(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.stack_pop();

    set_accumulator(nes, result);

    0
}

fn plp(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.stack_pop();

    nes.cpu_registers.status.set_bits(result);
    nes.cpu_registers.status.set_b(0b10);

    0
}

fn rol(nes: &mut NES, mode: &AddrMode) -> usize {
    let old_value = match mode {
        AddrMode::Accumulator => {
            let old_value = nes.cpu_registers.accumulator;
            let result = (old_value << 1) | (nes.cpu_registers.status.carry() as u8);

            set_accumulator(nes, result);

            old_value
        }
        _ => {
            let (addr, _) = nes.get_operating_address(mode);
            let old_value = nes.cpu_read(addr);
            let result = (old_value << 1) | (nes.cpu_registers.status.carry() as u8);

            nes.cpu_write(addr, result);
            update_zero_and_negative_flags(nes, result);

            old_value
        }
    };

    nes.cpu_registers.status.set_carry(old_value >> 7 == 1);

    0
}

fn ror(nes: &mut NES, mode: &AddrMode) -> usize {
    let old_value = match mode {
        AddrMode::Accumulator => {
            let old_value = nes.cpu_registers.accumulator;
            let result = (old_value >> 1) | ((nes.cpu_registers.status.carry() as u8) << 7);

            set_accumulator(nes, result);

            old_value
        }
        _ => {
            let (addr, _) = nes.get_operating_address(mode);
            let old_value = nes.cpu_read(addr);
            let result = (old_value >> 1) | ((nes.cpu_registers.status.carry() as u8) << 7);

            nes.cpu_write(addr, result);
            update_zero_and_negative_flags(nes, result);

            old_value
        }
    };

    nes.cpu_registers.status.set_carry(old_value & 1 == 1);

    0
}

fn rti(nes: &mut NES, _mode: &AddrMode) -> usize {
    let status = nes.stack_pop();
    let program_counter = nes.stack_pop_u16();

    nes.cpu_registers.status.set_bits(status);
    nes.cpu_registers.status.set_b(0b10);

    nes.cpu_registers.program_counter = program_counter;

    0
}

fn rts(nes: &mut NES, _mode: &AddrMode) -> usize {
    let program_counter = nes.stack_pop_u16() + 1;

    nes.cpu_registers.program_counter = program_counter;

    0
}

fn sbc(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let temp = nes.cpu_read(addr);
    let value = temp.wrapping_neg().wrapping_sub(1);

    add_to_accumulator(nes, value);

    page_crossed.into()
}

fn sec(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.status.set_carry(true);

    0
}

fn sed(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.status.set_decimal(true);

    0
}

fn sei(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.status.set_interrupt_disable(true);

    0
}

fn sta(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_registers.accumulator;

    nes.cpu_write(addr, value);

    0
}

fn stx(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_registers.x;

    nes.cpu_write(addr, value);

    0
}

fn sty(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_registers.y;

    nes.cpu_write(addr, value);

    0
}

fn tax(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.x = nes.cpu_registers.accumulator;

    update_zero_and_negative_flags(nes, nes.cpu_registers.x);

    0
}

fn tay(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.y = nes.cpu_registers.accumulator;

    update_zero_and_negative_flags(nes, nes.cpu_registers.y);

    0
}

fn tsx(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.x = nes.cpu_registers.stack_pointer;

    update_zero_and_negative_flags(nes, nes.cpu_registers.x);

    0
}

fn txa(nes: &mut NES, _mode: &AddrMode) -> usize {
    set_accumulator(nes, nes.cpu_registers.x);

    0
}

fn txs(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.stack_pointer = nes.cpu_registers.x;

    0
}

fn tya(nes: &mut NES, _mode: &AddrMode) -> usize {
    set_accumulator(nes, nes.cpu_registers.y);

    0
}

// ILLEGAL INSTRUCTIONS

fn anc(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    let result = nes.cpu_registers.accumulator & value;

    set_accumulator(nes, result);

    nes.cpu_registers
        .status
        .set_carry(nes.cpu_registers.status.negative());

    0
}

fn arr(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    let result = ((nes.cpu_registers.accumulator & value) >> 1)
        | ((nes.cpu_registers.status.carry() as u8) << 7);

    nes.cpu_registers.status.set_carry(value & 1 == 1);

    set_accumulator(nes, result);

    let accumulator = nes.cpu_registers.accumulator;
    let fifth_bit = (accumulator >> 5) & 1;
    let sixth_bit = (accumulator >> 6) & 1;

    nes.cpu_registers.status.set_carry(sixth_bit == 1);
    nes.cpu_registers
        .status
        .set_overflow(fifth_bit ^ sixth_bit == 1);
    update_zero_and_negative_flags(nes, accumulator);

    0
}

fn asr(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    let result = value >> 1;

    nes.cpu_write(addr, result);

    nes.cpu_registers.status.set_carry(value & 1 == 1);

    set_accumulator(nes, result & nes.cpu_registers.accumulator);

    0
}

fn axa(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_registers.x & nes.cpu_registers.accumulator & (addr >> 8) as u8;

    nes.cpu_write(addr, value);

    0
}

fn axs(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let x_and_a = nes.cpu_registers.x & nes.cpu_registers.accumulator;
    let result = x_and_a.wrapping_sub(value);

    if value <= x_and_a {
        nes.cpu_registers.status.set_carry(true);
    }
    update_zero_and_negative_flags(nes, result);

    nes.cpu_registers.x = result;

    0
}

fn dcp(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = value.wrapping_sub(1);

    nes.cpu_write(addr, result);

    if result < nes.cpu_registers.accumulator {
        nes.cpu_registers.status.set_carry(true);
    }

    update_zero_and_negative_flags(nes, nes.cpu_registers.accumulator.wrapping_sub(result));

    0
}

fn dop(_nes: &mut NES, _mode: &AddrMode) -> usize {
    0
}

fn isb(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = increment_memory(nes, addr);
    let result = (value as i8).wrapping_neg().wrapping_sub(1) as u8;

    add_to_accumulator(nes, result);

    0
}

fn kil(_nes: &mut NES, _mode: &AddrMode) -> usize {
    0
}

fn las(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = nes.cpu_registers.stack_pointer & value;

    set_accumulator(nes, result);
    nes.cpu_registers.x = result;
    nes.cpu_registers.stack_pointer = result;

    page_crossed.into()
}

fn lax(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    nes.cpu_registers.x = value;
    set_accumulator(nes, value);

    page_crossed.into()
}

fn lxa(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = nes.cpu_registers.accumulator & value;

    nes.cpu_registers.x = result;
    set_accumulator(nes, result);

    0
}

fn rla(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = (value << 1) | (nes.cpu_registers.status.carry() as u8);

    nes.cpu_write(addr, result);

    nes.cpu_registers.status.set_carry(value >> 7 == 1);

    set_accumulator(nes, nes.cpu_registers.accumulator & result);

    0
}

fn rra(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = (value >> 1) | (nes.cpu_registers.status.carry() as u8) << 7;

    nes.cpu_write(addr, result);

    nes.cpu_registers.status.set_carry(value & 1 == 1);

    add_to_accumulator(nes, result);

    0
}

fn sax(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);

    let result = nes.cpu_registers.accumulator & nes.cpu_registers.x;

    nes.cpu_write(addr, result);

    0
}

fn slo(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = value << 1;

    nes.cpu_write(addr, result);

    nes.cpu_registers.status.set_carry(value >> 7 == 1);

    set_accumulator(nes, result | nes.cpu_registers.accumulator);

    0
}

fn sre(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = value >> 1;

    nes.cpu_write(addr, result);

    nes.cpu_registers.status.set_carry(value & 1 == 1);

    set_accumulator(nes, result ^ nes.cpu_registers.accumulator);

    0
}

fn sxa(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_registers.x & ((addr >> 8) as u8 + 1);

    nes.cpu_write(addr, value);

    0
}

fn sya(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_registers.y & ((addr >> 8) as u8 + 1);

    nes.cpu_write(addr, value);

    0
}

fn top(nes: &mut NES, mode: &AddrMode) -> usize {
    let (_, page_crossed) = nes.get_operating_address(mode);

    page_crossed.into()
}

// This guy isn't super well documented, this seems like what it does?
fn xaa(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    set_accumulator(nes, nes.cpu_registers.x);
    set_accumulator(nes, nes.cpu_registers.accumulator & value);

    0
}

fn xas(nes: &mut NES, mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.x & nes.cpu_registers.accumulator;
    nes.cpu_registers.stack_pointer = result;

    let (addr, _) = nes.get_operating_address(mode);
    let value = result & ((addr >> 8) as u8 + 1);

    nes.cpu_write(addr, value);

    0
}

fn add_to_accumulator(nes: &mut NES, value: u8) {
    let result: u16 = nes.cpu_registers.accumulator as u16
        + value as u16
        + Into::<u16>::into(nes.cpu_registers.status.carry());
    nes.cpu_registers.status.set_carry(result > 0xFF);

    let result = result as u8;

    nes.cpu_registers
        .status
        .set_overflow((value ^ result) & (result ^ nes.cpu_registers.accumulator) & 0x80 != 0);

    set_accumulator(nes, result);
}

fn increment_memory(nes: &mut NES, addr: u16) -> u8 {
    let old_value = nes.cpu_read(addr);
    let result = old_value.wrapping_add(1);

    nes.cpu_write(addr, result);
    update_zero_and_negative_flags(nes, result);

    result
}

fn set_accumulator(nes: &mut NES, value: u8) {
    nes.cpu_registers.accumulator = value;

    update_zero_and_negative_flags(nes, value);
}

fn update_zero_and_negative_flags(nes: &mut NES, value: u8) {
    nes.cpu_registers.status.set_zero(value == 0);
    nes.cpu_registers.status.set_negative(value >> 7 == 1);
}

fn branch(nes: &mut NES, condition: bool) -> usize {
    if !condition {
        return 0;
    }

    let old_addr = nes.cpu_registers.program_counter.wrapping_add(1);

    let offset = nes.cpu_read(nes.cpu_registers.program_counter) as i8;
    let new_addr = old_addr.wrapping_add(offset as u16);

    // TODO: For some rason this fucks stuff up? Docs say this is how it should work tho
    let cycles = if old_addr & 0xFF00 != new_addr & 0xFF00 {
        2
    } else {
        1
    };

    nes.cpu_registers.program_counter = new_addr;

    cycles
}

fn compare(nes: &mut NES, register: u8, value: u8) {
    let result = register.wrapping_sub(value);

    nes.cpu_registers.status.set_carry(register >= value);
    update_zero_and_negative_flags(nes, result);
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn lda_sets_accumulator() {
        let mut nes = NES::default();

        nes.cpu_registers.accumulator = 0x00;
        nes.cpu_registers.program_counter += 1;
        nes.cpu_write(0x0001, 0x01);

        lda(&mut nes, &AddrMode::Immediate);

        assert_eq!(nes.cpu_registers.accumulator, 0x01);
    }

    #[test]
    fn lda_sets_zero_flag() {
        let mut nes = NES::default();

        nes.cpu_registers.accumulator = 0x00;
        nes.cpu_registers.program_counter += 1;
        nes.cpu_write(0x0001, 0x00);

        lda(&mut nes, &AddrMode::Immediate);

        assert_eq!(nes.cpu_registers.accumulator, 0x00);
        assert_eq!(nes.cpu_registers.status.zero(), true);
    }

    #[test]
    fn lda_sets_negative_flag() {
        let mut nes = NES::default();

        nes.cpu_registers.accumulator = 0x00;
        nes.cpu_registers.program_counter += 1;
        nes.cpu_write(0x0001, 0xFF);

        lda(&mut nes, &AddrMode::Immediate);

        assert_eq!(nes.cpu_registers.accumulator, 0xFF);
        assert_eq!(nes.cpu_registers.status.negative(), true);
    }
}
