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
        }
    };
}

pub(crate) const INSTRUCTIONS_TABLE: [Option<Instruction>; 256] = [
    Some(instr!("BRK", AddrMode::Implied, 7, 1, brk)),
    Some(instr!("ORA", AddrMode::IndirectX, 6, 2, ora)),
    None,
    None,
    None,
    Some(instr!("ORA", AddrMode::ZeroPage, 3, 2, ora)),
    Some(instr!("ASL", AddrMode::ZeroPage, 5, 2, asl)),
    None,
    Some(instr!("PHP", AddrMode::Implied, 3, 1, php)),
    Some(instr!("ORA", AddrMode::Immediate, 2, 2, ora)),
    Some(instr!("ASL", AddrMode::Accumulator, 2, 1, asl)),
    None,
    None,
    Some(instr!("ORA", AddrMode::Absolute, 4, 3, ora)),
    Some(instr!("ASL", AddrMode::Absolute, 6, 3, asl)),
    None,
    Some(instr!("BPL", AddrMode::Relative, 2, 2, bpl)),
    Some(instr!("ORA", AddrMode::IndirectY, 5, 2, ora)),
    None,
    None,
    None,
    Some(instr!("ORA", AddrMode::ZeroPageX, 4, 2, ora)),
    Some(instr!("ASL", AddrMode::ZeroPageX, 6, 2, asl)),
    None,
    Some(instr!("CLC", AddrMode::Implied, 2, 1, clc)),
    Some(instr!("ORA", AddrMode::AbsoluteY, 4, 3, ora)),
    None,
    None,
    None,
    Some(instr!("ORA", AddrMode::AbsoluteX, 4, 3, ora)),
    Some(instr!("ASL", AddrMode::AbsoluteX, 7, 3, asl)),
    None,
    Some(instr!("JSR", AddrMode::Absolute, 6, 3, jsr)),
    Some(instr!("AND", AddrMode::IndirectX, 6, 2, and)),
    None,
    None,
    Some(instr!("BIT", AddrMode::ZeroPage, 3, 2, bit)),
    Some(instr!("AND", AddrMode::ZeroPage, 3, 2, and)),
    Some(instr!("ROL", AddrMode::ZeroPage, 5, 2, rol)),
    None,
    Some(instr!("PLP", AddrMode::Implied, 4, 1, plp)),
    Some(instr!("AND", AddrMode::Immediate, 2, 2, and)),
    Some(instr!("ROL", AddrMode::Accumulator, 2, 1, rol)),
    None,
    Some(instr!("BIT", AddrMode::Absolute, 4, 3, bit)),
    Some(instr!("AND", AddrMode::Absolute, 4, 3, and)),
    Some(instr!("ROL", AddrMode::Absolute, 6, 3, rol)),
    None,
    Some(instr!("BMI", AddrMode::Relative, 2, 2, bmi)),
    Some(instr!("AND", AddrMode::IndirectY, 5, 2, and)),
    None,
    None,
    None,
    Some(instr!("AND", AddrMode::ZeroPageX, 4, 2, and)),
    Some(instr!("ROL", AddrMode::ZeroPageX, 6, 2, rol)),
    None,
    Some(instr!("SEC", AddrMode::Implied, 2, 1, sec)),
    Some(instr!("AND", AddrMode::AbsoluteY, 4, 3, and)),
    None,
    None,
    None,
    Some(instr!("AND", AddrMode::AbsoluteX, 4, 3, and)),
    Some(instr!("ROL", AddrMode::AbsoluteX, 7, 3, rol)),
    None,
    Some(instr!("RTI", AddrMode::Implied, 6, 1, rti)),
    Some(instr!("EOR", AddrMode::IndirectX, 6, 2, eor)),
    None,
    None,
    None,
    Some(instr!("EOR", AddrMode::ZeroPage, 3, 2, eor)),
    Some(instr!("LSR", AddrMode::ZeroPage, 5, 2, lsr)),
    None,
    Some(instr!("PHA", AddrMode::Implied, 3, 1, pha)),
    Some(instr!("EOR", AddrMode::Immediate, 2, 2, eor)),
    Some(instr!("LSR", AddrMode::Accumulator, 2, 1, lsr)),
    None,
    Some(instr!("JMP", AddrMode::Absolute, 3, 3, jmp)),
    Some(instr!("EOR", AddrMode::Absolute, 4, 3, eor)),
    Some(instr!("LSR", AddrMode::Absolute, 6, 3, lsr)),
    None,
    Some(instr!("BVC", AddrMode::Relative, 2, 2, bvc)),
    Some(instr!("EOR", AddrMode::IndirectY, 5, 2, eor)),
    None,
    None,
    None,
    Some(instr!("EOR", AddrMode::ZeroPageX, 4, 2, eor)),
    Some(instr!("LSR", AddrMode::ZeroPageX, 6, 2, lsr)),
    None,
    Some(instr!("CLI", AddrMode::Implied, 2, 1, cli)),
    Some(instr!("EOR", AddrMode::AbsoluteY, 4, 3, eor)),
    None,
    None,
    None,
    Some(instr!("EOR", AddrMode::AbsoluteX, 4, 3, eor)),
    Some(instr!("LSR", AddrMode::AbsoluteX, 7, 3, lsr)),
    None,
    Some(instr!("RTS", AddrMode::Implied, 6, 1, rts)),
    Some(instr!("ADC", AddrMode::IndirectX, 6, 2, adc)),
    None,
    None,
    None,
    Some(instr!("ADC", AddrMode::ZeroPage, 3, 2, adc)),
    Some(instr!("ROR", AddrMode::ZeroPage, 5, 2, ror)),
    None,
    Some(instr!("PLA", AddrMode::Implied, 4, 1, pla)),
    Some(instr!("ADC", AddrMode::Immediate, 2, 2, adc)),
    Some(instr!("ROR", AddrMode::Accumulator, 2, 1, ror)),
    None,
    Some(instr!("JMP", AddrMode::Indirect, 5, 3, jmp)),
    Some(instr!("ADC", AddrMode::Absolute, 4, 3, adc)),
    Some(instr!("ROR", AddrMode::Absolute, 6, 3, ror)),
    None,
    Some(instr!("BVS", AddrMode::Relative, 2, 2, bvs)),
    Some(instr!("ADC", AddrMode::IndirectY, 5, 2, adc)),
    None,
    None,
    None,
    Some(instr!("ADC", AddrMode::ZeroPageX, 4, 2, adc)),
    Some(instr!("ROR", AddrMode::ZeroPageX, 6, 2, ror)),
    None,
    Some(instr!("SEI", AddrMode::Implied, 2, 1, sei)),
    Some(instr!("ADC", AddrMode::AbsoluteY, 4, 3, adc)),
    None,
    None,
    None,
    Some(instr!("ADC", AddrMode::AbsoluteX, 4, 3, adc)),
    Some(instr!("ROR", AddrMode::AbsoluteX, 7, 3, ror)),
    None,
    None,
    Some(instr!("STA", AddrMode::IndirectX, 6, 2, sta)),
    None,
    None,
    Some(instr!("STY", AddrMode::ZeroPage, 3, 2, sty)),
    Some(instr!("STA", AddrMode::ZeroPage, 3, 2, sta)),
    Some(instr!("STX", AddrMode::ZeroPage, 3, 2, stx)),
    None,
    Some(instr!("DEY", AddrMode::Implied, 2, 1, dey)),
    None,
    Some(instr!("TXA", AddrMode::Implied, 2, 1, txa)),
    None,
    Some(instr!("STY", AddrMode::Absolute, 4, 3, sty)),
    Some(instr!("STA", AddrMode::Absolute, 4, 3, sta)),
    Some(instr!("STX", AddrMode::Absolute, 4, 3, stx)),
    None,
    Some(instr!("BCC", AddrMode::Relative, 2, 2, bcc)),
    Some(instr!("STA", AddrMode::IndirectY, 6, 2, sta)),
    None,
    None,
    Some(instr!("STY", AddrMode::ZeroPageX, 4, 2, sty)),
    Some(instr!("STA", AddrMode::ZeroPageX, 4, 2, sta)),
    Some(instr!("STX", AddrMode::ZeroPageY, 4, 2, stx)),
    None,
    Some(instr!("TYA", AddrMode::Implied, 2, 1, tya)),
    Some(instr!("STA", AddrMode::AbsoluteY, 5, 3, sta)),
    Some(instr!("TXS", AddrMode::Implied, 2, 1, txs)),
    None,
    None,
    Some(instr!("STA", AddrMode::AbsoluteX, 5, 3, sta)),
    None,
    None,
    Some(instr!("LDY", AddrMode::Immediate, 2, 2, ldy)),
    Some(instr!("LDA", AddrMode::IndirectX, 6, 2, lda)),
    Some(instr!("LDX", AddrMode::Immediate, 2, 2, ldx)),
    None,
    Some(instr!("LDY", AddrMode::ZeroPage, 3, 2, ldy)),
    Some(instr!("LDA", AddrMode::ZeroPage, 3, 2, lda)),
    Some(instr!("LDX", AddrMode::ZeroPage, 3, 2, ldx)),
    None,
    Some(instr!("TAY", AddrMode::Implied, 2, 1, tay)),
    Some(instr!("LDA", AddrMode::Immediate, 2, 2, lda)),
    Some(instr!("TAX", AddrMode::Implied, 2, 1, tax)),
    None,
    Some(instr!("LDY", AddrMode::Absolute, 4, 3, ldy)),
    Some(instr!("LDA", AddrMode::Absolute, 4, 3, lda)),
    Some(instr!("LDX", AddrMode::Absolute, 4, 3, ldx)),
    None,
    Some(instr!("BCS", AddrMode::Relative, 2, 2, bcs)),
    Some(instr!("LDA", AddrMode::IndirectY, 5, 2, lda)),
    None,
    None,
    Some(instr!("LDY", AddrMode::ZeroPageX, 4, 2, ldy)),
    Some(instr!("LDA", AddrMode::ZeroPageX, 4, 2, lda)),
    Some(instr!("LDX", AddrMode::ZeroPageY, 4, 2, ldx)),
    None,
    Some(instr!("CLV", AddrMode::Implied, 2, 1, clv)),
    Some(instr!("LDA", AddrMode::AbsoluteY, 4, 3, lda)),
    Some(instr!("TSX", AddrMode::Implied, 2, 1, tsx)),
    None,
    Some(instr!("LDY", AddrMode::AbsoluteX, 4, 3, ldy)),
    Some(instr!("LDA", AddrMode::AbsoluteX, 4, 3, lda)),
    Some(instr!("LDX", AddrMode::AbsoluteY, 4, 3, ldx)),
    None,
    Some(instr!("CPY", AddrMode::Immediate, 2, 2, cpy)),
    Some(instr!("CMP", AddrMode::IndirectX, 6, 2, cmp)),
    None,
    None,
    Some(instr!("CPY", AddrMode::ZeroPage, 3, 2, cpy)),
    Some(instr!("CMP", AddrMode::ZeroPage, 3, 2, cmp)),
    Some(instr!("DEC", AddrMode::ZeroPage, 5, 2, dec)),
    None,
    Some(instr!("INY", AddrMode::Implied, 2, 1, iny)),
    Some(instr!("CMP", AddrMode::Immediate, 2, 2, cmp)),
    Some(instr!("DEX", AddrMode::Implied, 2, 1, dex)),
    None,
    Some(instr!("CPY", AddrMode::Absolute, 4, 3, cpy)),
    Some(instr!("CMP", AddrMode::Absolute, 4, 3, cmp)),
    Some(instr!("DEC", AddrMode::Absolute, 6, 3, dec)),
    None,
    Some(instr!("BNE", AddrMode::Relative, 2, 2, bne)),
    Some(instr!("CMP", AddrMode::IndirectY, 5, 2, cmp)),
    None,
    None,
    None,
    Some(instr!("CMP", AddrMode::ZeroPageX, 4, 2, cmp)),
    Some(instr!("DEC", AddrMode::ZeroPageX, 6, 2, dec)),
    None,
    Some(instr!("CLD", AddrMode::Implied, 2, 1, cld)),
    Some(instr!("CMP", AddrMode::AbsoluteY, 4, 3, cmp)),
    None,
    None,
    None,
    Some(instr!("CMP", AddrMode::AbsoluteX, 4, 3, cmp)),
    Some(instr!("DEC", AddrMode::AbsoluteX, 7, 3, dec)),
    None,
    Some(instr!("CPX", AddrMode::Immediate, 2, 2, cpx)),
    Some(instr!("SBC", AddrMode::IndirectX, 6, 2, sbc)),
    None,
    None,
    Some(instr!("CPX", AddrMode::ZeroPage, 3, 2, cpx)),
    Some(instr!("SBC", AddrMode::ZeroPage, 3, 2, sbc)),
    Some(instr!("INC", AddrMode::ZeroPage, 5, 2, inc)),
    None,
    Some(instr!("INX", AddrMode::Implied, 2, 1, inx)),
    Some(instr!("SBC", AddrMode::Immediate, 2, 2, sbc)),
    Some(instr!("NOP", AddrMode::Implied, 2, 1, nop)),
    None,
    Some(instr!("CPX", AddrMode::Absolute, 4, 3, cpx)),
    Some(instr!("SBC", AddrMode::Absolute, 4, 3, sbc)),
    Some(instr!("INC", AddrMode::Absolute, 6, 3, inc)),
    None,
    Some(instr!("BEQ", AddrMode::Relative, 2, 2, beq)),
    Some(instr!("SBC", AddrMode::IndirectY, 5, 2, sbc)),
    None,
    None,
    None,
    Some(instr!("SBC", AddrMode::ZeroPageX, 4, 2, sbc)),
    Some(instr!("INC", AddrMode::ZeroPageX, 6, 2, inc)),
    None,
    Some(instr!("SED", AddrMode::Implied, 2, 1, sed)),
    Some(instr!("SBC", AddrMode::AbsoluteY, 4, 3, sbc)),
    None,
    None,
    None,
    Some(instr!("SBC", AddrMode::AbsoluteX, 4, 3, sbc)),
    Some(instr!("INC", AddrMode::AbsoluteX, 7, 3, inc)),
    None,
];

pub struct Instruction {
    pub name: &'static str,
    pub mode: AddrMode,
    pub cycles: usize,
    pub len: u8,
    pub operate: fn(&mut NES, mode: &AddrMode) -> usize,
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

    let result: u16 = nes.cpu_registers.accumulator as u16
        + value as u16
        + Into::<u16>::into(nes.cpu_registers.status.carry());
    nes.cpu_registers.status.set_carry(result > 0xFF);

    let result = result as u8;

    nes.cpu_registers
        .status
        .set_overflow((value ^ result) & (result ^ nes.cpu_registers.accumulator) & 0x80 != 0);

    nes.set_accumulator(result);

    page_crossed.into()
}

fn and(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    nes.set_accumulator(nes.cpu_registers.accumulator & value);

    page_crossed.into()
}

fn asl(nes: &mut NES, mode: &AddrMode) -> usize {
    let old_value = if let AddrMode::Accumulator = mode {
        let old_value = nes.cpu_registers.accumulator;

        nes.set_accumulator(old_value << 1);

        old_value
    } else {
        let (addr, _) = nes.get_operating_address(mode);
        let old_value = nes.cpu_read(addr);
        let result = old_value << 1;

        nes.cpu_write(addr, result);
        nes.update_zero_and_negative_flags(result);

        old_value
    };

    nes.cpu_registers.status.set_carry(old_value >> 7 == 1);

    0
}

fn bcc(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.branch(!nes.cpu_registers.status.carry())
}

fn bcs(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.branch(nes.cpu_registers.status.carry())
}

fn beq(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.branch(nes.cpu_registers.status.zero())
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
    nes.branch(nes.cpu_registers.status.negative())
}

fn bne(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.branch(!nes.cpu_registers.status.zero())
}

fn bpl(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.branch(!nes.cpu_registers.status.negative())
}

fn brk(nes: &mut NES, _mode: &AddrMode) -> usize {
    // TODO: interrupts
    // std::process::exit(0);
    0
}

fn bvc(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.branch(!nes.cpu_registers.status.overflow())
}

fn bvs(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.branch(nes.cpu_registers.status.overflow())
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

    nes.compare(nes.cpu_registers.accumulator, value);

    page_crossed.into()
}

fn cpx(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    nes.compare(nes.cpu_registers.x, value);

    0
}

fn cpy(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);

    nes.compare(nes.cpu_registers.y, value);

    0
}

fn dec(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let old_value = nes.cpu_read(addr);
    let result = old_value.wrapping_sub(1);

    nes.cpu_write(addr, result);
    nes.update_zero_and_negative_flags(result);

    0
}

fn dex(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.x.wrapping_sub(1);

    nes.cpu_registers.x = result;
    nes.update_zero_and_negative_flags(result);

    0
}

fn dey(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.y.wrapping_sub(1);

    nes.cpu_registers.y = result;
    nes.update_zero_and_negative_flags(result);

    0
}

fn eor(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);
    let value = nes.cpu_read(addr);
    let result = nes.cpu_registers.accumulator ^ value;

    nes.set_accumulator(result);

    page_crossed.into()
}

fn inc(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, _) = nes.get_operating_address(mode);
    let old_value = nes.cpu_read(addr);
    let result = old_value.wrapping_add(1);

    nes.cpu_write(addr, result);
    nes.update_zero_and_negative_flags(result);

    0
}

fn inx(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.x.wrapping_add(1);

    nes.cpu_registers.x = result;
    nes.update_zero_and_negative_flags(result);

    0
}

fn iny(nes: &mut NES, _mode: &AddrMode) -> usize {
    let result = nes.cpu_registers.y.wrapping_add(1);

    nes.cpu_registers.y = result;
    nes.update_zero_and_negative_flags(result);

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

    nes.set_accumulator(nes.cpu_read(addr));

    page_crossed.into()
}

fn ldx(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);

    nes.cpu_registers.x = nes.cpu_read(addr);
    nes.update_zero_and_negative_flags(nes.cpu_registers.x);

    page_crossed.into()
}

fn ldy(nes: &mut NES, mode: &AddrMode) -> usize {
    let (addr, page_crossed) = nes.get_operating_address(mode);

    nes.cpu_registers.y = nes.cpu_read(addr);
    nes.update_zero_and_negative_flags(nes.cpu_registers.y);

    page_crossed.into()
}

fn lsr(nes: &mut NES, mode: &AddrMode) -> usize {
    let old_value = match mode {
        AddrMode::Accumulator => {
            let old_value = nes.cpu_registers.accumulator;

            nes.set_accumulator(old_value >> 1);

            old_value
        }
        _ => {
            let (addr, _) = nes.get_operating_address(mode);
            let old_value = nes.cpu_read(addr);
            let result = old_value >> 1;

            nes.cpu_write(addr, result);
            nes.update_zero_and_negative_flags(result);

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

    nes.set_accumulator(result);

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

    nes.set_accumulator(result);

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

            nes.set_accumulator(result);

            old_value
        }
        _ => {
            let (addr, _) = nes.get_operating_address(mode);
            let old_value = nes.cpu_read(addr);
            let result = (old_value << 1) | (nes.cpu_registers.status.carry() as u8);

            nes.cpu_write(addr, result);
            nes.update_zero_and_negative_flags(result);

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

            nes.set_accumulator(result);

            old_value
        }
        _ => {
            let (addr, _) = nes.get_operating_address(mode);
            let old_value = nes.cpu_read(addr);
            let result = (old_value >> 1) | ((nes.cpu_registers.status.carry() as u8) << 7);

            nes.cpu_write(addr, result);
            nes.update_zero_and_negative_flags(result);

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

    let result: u16 = nes.cpu_registers.accumulator as u16
        + value as u16
        + Into::<u16>::into(nes.cpu_registers.status.carry());
    nes.cpu_registers.status.set_carry(result > 0xFF);

    let result = result as u8;

    nes.cpu_registers
        .status
        .set_overflow((value ^ result) & (result ^ nes.cpu_registers.accumulator) & 0x80 != 0);

    nes.set_accumulator(result);

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

    nes.update_zero_and_negative_flags(nes.cpu_registers.x);

    0
}

fn tay(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.y = nes.cpu_registers.accumulator;

    nes.update_zero_and_negative_flags(nes.cpu_registers.y);

    0
}

fn tsx(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.x = nes.cpu_registers.stack_pointer;

    nes.update_zero_and_negative_flags(nes.cpu_registers.x);

    0
}

fn txa(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.set_accumulator(nes.cpu_registers.x);

    0
}

fn txs(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.cpu_registers.stack_pointer = nes.cpu_registers.x;

    0
}

fn tya(nes: &mut NES, _mode: &AddrMode) -> usize {
    nes.set_accumulator(nes.cpu_registers.y);

    0
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
