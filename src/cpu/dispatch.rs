use crate::bus::Bus;
use crate::cpu::*;

impl CPU {
    // Fetch, decode, dispatch.
    pub(super) fn dispatch(&mut self, bus: &mut Bus) {
        let opcode = bus.mem_read(self.mmu.to_physical(self.sr.pc));
        self.sr.pc += 1;

        //println!("Opcode is ${:02x} (0b{:08b})", opcode, opcode);

        // The full 256 opcode values are listed explicitly to allow a jump table to be
        // generated. It would be possible to use bitmasks to reduce the size of this list,
        // but doing so would significantly reduce the efficiency of the dispatcher.
        match opcode {
            0b00_000_000 => self.nop(),
            0b00_000_001 => self.ld_16(bus, Operand::Immediate16(), Operand::Direct(Register::BC)),
            0b00_000_010 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Indirect(RegIndirect::BC)),
            0b00_000_011 => self.inc(bus, Operand::Direct(Register::BC)),
            0b00_000_100 => self.inc(bus, Operand::Direct(Register::B)),
            0b00_000_101 => self.dec(bus, Operand::Direct(Register::B)),
            0b00_000_110 => self.ld_8(bus, Operand::Immediate(), Operand::Direct(Register::B)),
            0b00_000_111 => self.rot_left(bus, Operand::Direct(Register::A), true),

            0b00_001_000 => self.exchange(bus, Exchange::AF_AFs),
            0b00_001_001 => self.add_hl_ww(RegW::BC),
            0b00_001_010 => self.error("dispatch"),
            0b00_001_011 => self.dec(bus, Operand::Direct(Register::BC)),
            0b00_001_100 => self.inc(bus, Operand::Direct(Register::C)),
            0b00_001_101 => self.dec(bus, Operand::Direct(Register::C)),
            0b00_001_110 => self.ld_8(bus, Operand::Immediate(), Operand::Direct(Register::C)),
            0b00_001_111 => self.rot_right(bus, Operand::Direct(Register::A), true),

            0b00_010_000 => self.error("dispatch"),
            0b00_010_001 => self.ld_16(bus, Operand::Immediate16(), Operand::Direct(Register::DE)),
            0b00_010_010 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Indirect(RegIndirect::DE)),
            0b00_010_011 => self.inc(bus, Operand::Direct(Register::DE)),
            0b00_010_100 => self.inc(bus, Operand::Direct(Register::D)),
            0b00_010_101 => self.dec(bus, Operand::Direct(Register::D)),
            0b00_010_110 => self.ld_8(bus, Operand::Immediate(), Operand::Direct(Register::D)),
            0b00_010_111 => self.rot_left(bus, Operand::Direct(Register::A), false),

            0b00_011_000 => self.error("dispatch"),
            0b00_011_001 => self.add_hl_ww(RegW::DE),
            0b00_011_010 => self.ld_8(bus, Operand::Indirect(RegIndirect::DE), Operand::Direct(Register::A)),
            0b00_011_011 => self.dec(bus, Operand::Direct(Register::DE)),
            0b00_011_100 => self.inc(bus, Operand::Direct(Register::E)),
            0b00_011_101 => self.dec(bus, Operand::Direct(Register::E)),
            0b00_011_110 => self.ld_8(bus, Operand::Immediate(), Operand::Direct(Register::E)),
            0b00_011_111 => self.error("dispatch"),

            0b00_100_000 => self.error("dispatch"),
            0b00_100_001 => self.ld_16(bus, Operand::Immediate16(), Operand::Direct(Register::HL)),
            0b00_100_010 => self.ld_16(bus, Operand::Direct(Register::HL), Operand::Extended16()),
            0b00_100_011 => self.inc(bus, Operand::Direct(Register::HL)),
            0b00_100_100 => self.inc(bus, Operand::Direct(Register::H)),
            0b00_100_101 => self.dec(bus, Operand::Direct(Register::H)),
            0b00_100_110 => self.ld_8(bus, Operand::Immediate(), Operand::Direct(Register::H)),
            0b00_100_111 => self.error("dispatch"),

            0b00_101_000 => self.error("dispatch"),
            0b00_101_001 => self.add_hl_ww(RegW::HL),
            0b00_101_010 => self.ld_16(bus, Operand::Extended16(), Operand::Direct(Register::HL)),
            0b00_101_011 => self.dec(bus, Operand::Direct(Register::HL)),
            0b00_101_100 => self.inc(bus, Operand::Direct(Register::L)),
            0b00_101_101 => self.dec(bus, Operand::Direct(Register::L)),
            0b00_101_110 => self.ld_8(bus, Operand::Immediate(), Operand::Direct(Register::L)),
            0b00_101_111 => self.error("dispatch"),

            0b00_110_000 => self.error("dispatch"),
            0b00_110_001 => self.ld_16(bus, Operand::Immediate16(), Operand::Direct(Register::SP)),
            0b00_110_010 => self.ld_16(bus, Operand::Direct(Register::A), Operand::Extended()),
            0b00_110_011 => self.inc(bus, Operand::Direct(Register::SP)),
            0b00_110_100 => self.inc(bus, Operand::Indirect(RegIndirect::HL)),
            0b00_110_101 => self.dec(bus, Operand::Indirect(RegIndirect::HL)),
            0b00_110_110 => self.ld_8(bus, Operand::Immediate(), Operand::Indirect(RegIndirect::HL)),
            0b00_110_111 => self.error("dispatch"),

            0b00_111_000 => self.error("dispatch"),
            0b00_111_001 => self.add_hl_ww(RegW::SP),
            0b00_111_010 => self.ld_8(bus, Operand::Extended(), Operand::Direct(Register::A)),
            0b00_111_011 => self.dec(bus, Operand::Direct(Register::SP)),
            0b00_111_100 => self.inc(bus, Operand::Direct(Register::A)),
            0b00_111_101 => self.dec(bus, Operand::Direct(Register::A)),
            0b00_111_110 => self.ld_8(bus, Operand::Immediate(), Operand::Direct(Register::A)),
            0b00_111_111 => self.error("dispatch"),

            0b01_000_000 => self.ld_8(bus, Operand::Direct(Register::B), Operand::Direct(Register::B)),
            0b01_000_001 => self.ld_8(bus, Operand::Direct(Register::C), Operand::Direct(Register::B)),
            0b01_000_010 => self.ld_8(bus, Operand::Direct(Register::D), Operand::Direct(Register::B)),
            0b01_000_011 => self.ld_8(bus, Operand::Direct(Register::E), Operand::Direct(Register::B)),
            0b01_000_100 => self.ld_8(bus, Operand::Direct(Register::H), Operand::Direct(Register::B)),
            0b01_000_101 => self.ld_8(bus, Operand::Direct(Register::L), Operand::Direct(Register::B)),
            0b01_000_110 => self.ld_8(bus, Operand::Indirect(RegIndirect::HL), Operand::Direct(Register::B)),
            0b01_000_111 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Direct(Register::B)),

            0b01_001_000 => self.ld_8(bus, Operand::Direct(Register::B), Operand::Direct(Register::C)),
            0b01_001_001 => self.ld_8(bus, Operand::Direct(Register::C), Operand::Direct(Register::C)),
            0b01_001_010 => self.ld_8(bus, Operand::Direct(Register::D), Operand::Direct(Register::C)),
            0b01_001_011 => self.ld_8(bus, Operand::Direct(Register::E), Operand::Direct(Register::C)),
            0b01_001_100 => self.ld_8(bus, Operand::Direct(Register::H), Operand::Direct(Register::C)),
            0b01_001_101 => self.ld_8(bus, Operand::Direct(Register::L), Operand::Direct(Register::C)),
            0b01_001_110 => self.ld_8(bus, Operand::Indirect(RegIndirect::HL), Operand::Direct(Register::C)),
            0b01_001_111 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Direct(Register::C)),

            0b01_010_000 => self.ld_8(bus, Operand::Direct(Register::B), Operand::Direct(Register::D)),
            0b01_010_001 => self.ld_8(bus, Operand::Direct(Register::C), Operand::Direct(Register::D)),
            0b01_010_010 => self.ld_8(bus, Operand::Direct(Register::D), Operand::Direct(Register::D)),
            0b01_010_011 => self.ld_8(bus, Operand::Direct(Register::E), Operand::Direct(Register::D)),
            0b01_010_100 => self.ld_8(bus, Operand::Direct(Register::H), Operand::Direct(Register::D)),
            0b01_010_101 => self.ld_8(bus, Operand::Direct(Register::L), Operand::Direct(Register::D)),
            0b01_010_110 => self.ld_8(bus, Operand::Indirect(RegIndirect::HL), Operand::Direct(Register::D)),
            0b01_010_111 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Direct(Register::D)),

            0b01_011_000 => self.ld_8(bus, Operand::Direct(Register::B), Operand::Direct(Register::E)),
            0b01_011_001 => self.ld_8(bus, Operand::Direct(Register::C), Operand::Direct(Register::E)),
            0b01_011_010 => self.ld_8(bus, Operand::Direct(Register::D), Operand::Direct(Register::E)),
            0b01_011_011 => self.ld_8(bus, Operand::Direct(Register::E), Operand::Direct(Register::E)),
            0b01_011_100 => self.ld_8(bus, Operand::Direct(Register::H), Operand::Direct(Register::E)),
            0b01_011_101 => self.ld_8(bus, Operand::Direct(Register::L), Operand::Direct(Register::E)),
            0b01_011_110 => self.ld_8(bus, Operand::Indirect(RegIndirect::HL), Operand::Direct(Register::E)),
            0b01_011_111 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Direct(Register::E)),

            0b01_100_000 => self.ld_8(bus, Operand::Direct(Register::B), Operand::Direct(Register::H)),
            0b01_100_001 => self.ld_8(bus, Operand::Direct(Register::C), Operand::Direct(Register::H)),
            0b01_100_010 => self.ld_8(bus, Operand::Direct(Register::D), Operand::Direct(Register::H)),
            0b01_100_011 => self.ld_8(bus, Operand::Direct(Register::E), Operand::Direct(Register::H)),
            0b01_100_100 => self.ld_8(bus, Operand::Direct(Register::H), Operand::Direct(Register::H)),
            0b01_100_101 => self.ld_8(bus, Operand::Direct(Register::L), Operand::Direct(Register::H)),
            0b01_100_110 => self.ld_8(bus, Operand::Indirect(RegIndirect::HL), Operand::Direct(Register::H)),
            0b01_100_111 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Direct(Register::H)),

            0b01_101_000 => self.ld_8(bus, Operand::Direct(Register::B), Operand::Direct(Register::L)),
            0b01_101_001 => self.ld_8(bus, Operand::Direct(Register::C), Operand::Direct(Register::L)),
            0b01_101_010 => self.ld_8(bus, Operand::Direct(Register::D), Operand::Direct(Register::L)),
            0b01_101_011 => self.ld_8(bus, Operand::Direct(Register::E), Operand::Direct(Register::L)),
            0b01_101_100 => self.ld_8(bus, Operand::Direct(Register::H), Operand::Direct(Register::L)),
            0b01_101_101 => self.ld_8(bus, Operand::Direct(Register::L), Operand::Direct(Register::L)),
            0b01_101_110 => self.ld_8(bus, Operand::Indirect(RegIndirect::HL), Operand::Direct(Register::L)),
            0b01_101_111 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Direct(Register::L)),

            0b01_110_000 => self.ld_8(bus, Operand::Direct(Register::B), Operand::Indirect(RegIndirect::HL)),
            0b01_110_001 => self.ld_8(bus, Operand::Direct(Register::C), Operand::Indirect(RegIndirect::HL)),
            0b01_110_010 => self.ld_8(bus, Operand::Direct(Register::D), Operand::Indirect(RegIndirect::HL)),
            0b01_110_011 => self.ld_8(bus, Operand::Direct(Register::E), Operand::Indirect(RegIndirect::HL)),
            0b01_110_100 => self.ld_8(bus, Operand::Direct(Register::H), Operand::Indirect(RegIndirect::HL)),
            0b01_110_101 => self.ld_8(bus, Operand::Direct(Register::L), Operand::Indirect(RegIndirect::HL)),
            0b01_110_110 => self.halt(),
            0b01_110_111 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Indirect(RegIndirect::HL)),

            0b01_111_000 => self.ld_8(bus, Operand::Direct(Register::B), Operand::Direct(Register::A)),
            0b01_111_001 => self.ld_8(bus, Operand::Direct(Register::C), Operand::Direct(Register::A)),
            0b01_111_010 => self.ld_8(bus, Operand::Direct(Register::D), Operand::Direct(Register::A)),
            0b01_111_011 => self.ld_8(bus, Operand::Direct(Register::E), Operand::Direct(Register::A)),
            0b01_111_100 => self.ld_8(bus, Operand::Direct(Register::H), Operand::Direct(Register::A)),
            0b01_111_101 => self.ld_8(bus, Operand::Direct(Register::L), Operand::Direct(Register::A)),
            0b01_111_110 => self.ld_8(bus, Operand::Indirect(RegIndirect::HL), Operand::Direct(Register::A)),
            0b01_111_111 => self.ld_8(bus, Operand::Direct(Register::A), Operand::Direct(Register::A)),

            0b10_000_000 => self.add_a(bus, Operand::Direct(Register::B), false),
            0b10_000_001 => self.add_a(bus, Operand::Direct(Register::C), false),
            0b10_000_010 => self.add_a(bus, Operand::Direct(Register::D), false),
            0b10_000_011 => self.add_a(bus, Operand::Direct(Register::E), false),
            0b10_000_100 => self.add_a(bus, Operand::Direct(Register::H), false),
            0b10_000_101 => self.add_a(bus, Operand::Direct(Register::L), false),
            0b10_000_110 => self.add_a(bus, Operand::Indirect(RegIndirect::HL), false),
            0b10_000_111 => self.add_a(bus, Operand::Direct(Register::A), false),

            0b10_001_000 => self.add_a(bus, Operand::Direct(Register::B), true),
            0b10_001_001 => self.add_a(bus, Operand::Direct(Register::C), true),
            0b10_001_010 => self.add_a(bus, Operand::Direct(Register::D), true),
            0b10_001_011 => self.add_a(bus, Operand::Direct(Register::E), true),
            0b10_001_100 => self.add_a(bus, Operand::Direct(Register::H), true),
            0b10_001_101 => self.add_a(bus, Operand::Direct(Register::L), true),
            0b10_001_110 => self.add_a(bus, Operand::Indirect(RegIndirect::HL), true),
            0b10_001_111 => self.add_a(bus, Operand::Direct(Register::A), true),

            0b10_010_000 => self.sub_a(bus, Operand::Direct(Register::B), false, true),
            0b10_010_001 => self.sub_a(bus, Operand::Direct(Register::C), false, true),
            0b10_010_010 => self.sub_a(bus, Operand::Direct(Register::D), false, true),
            0b10_010_011 => self.sub_a(bus, Operand::Direct(Register::E), false, true),
            0b10_010_100 => self.sub_a(bus, Operand::Direct(Register::H), false, true),
            0b10_010_101 => self.sub_a(bus, Operand::Direct(Register::L), false, true),
            0b10_010_110 => self.sub_a(bus, Operand::Indirect(RegIndirect::HL), false, true),
            0b10_010_111 => self.sub_a(bus, Operand::Direct(Register::A), false, true),

            0b10_011_000 => self.sub_a(bus, Operand::Direct(Register::B), true, true),
            0b10_011_001 => self.sub_a(bus, Operand::Direct(Register::C), true, true),
            0b10_011_010 => self.sub_a(bus, Operand::Direct(Register::D), true, true),
            0b10_011_011 => self.sub_a(bus, Operand::Direct(Register::E), true, true),
            0b10_011_100 => self.sub_a(bus, Operand::Direct(Register::H), true, true),
            0b10_011_101 => self.sub_a(bus, Operand::Direct(Register::L), true, true),
            0b10_011_110 => self.sub_a(bus, Operand::Indirect(RegIndirect::HL), true, true),
            0b10_011_111 => self.sub_a(bus, Operand::Direct(Register::A), true, true),

            0b10_100_000 => self.and_a(bus, Operand::Direct(Register::B)),
            0b10_100_001 => self.and_a(bus, Operand::Direct(Register::C)),
            0b10_100_010 => self.and_a(bus, Operand::Direct(Register::D)),
            0b10_100_011 => self.and_a(bus, Operand::Direct(Register::E)),
            0b10_100_100 => self.and_a(bus, Operand::Direct(Register::H)),
            0b10_100_101 => self.and_a(bus, Operand::Direct(Register::L)),
            0b10_100_110 => self.and_a(bus, Operand::Indirect(RegIndirect::HL)),
            0b10_100_111 => self.and_a(bus, Operand::Direct(Register::A)),

            0b10_101_000 => self.error("dispatch"),
            0b10_101_001 => self.error("dispatch"),
            0b10_101_010 => self.error("dispatch"),
            0b10_101_011 => self.error("dispatch"),
            0b10_101_100 => self.error("dispatch"),
            0b10_101_101 => self.error("dispatch"),
            0b10_101_110 => self.error("dispatch"),
            0b10_101_111 => self.error("dispatch"),

            0b10_110_000 => self.or_a(bus, Operand::Direct(Register::B)),
            0b10_110_001 => self.or_a(bus, Operand::Direct(Register::C)),
            0b10_110_010 => self.or_a(bus, Operand::Direct(Register::D)),
            0b10_110_011 => self.or_a(bus, Operand::Direct(Register::E)),
            0b10_110_100 => self.or_a(bus, Operand::Direct(Register::H)),
            0b10_110_101 => self.or_a(bus, Operand::Direct(Register::L)),
            0b10_110_110 => self.or_a(bus, Operand::Indirect(RegIndirect::HL)),
            0b10_110_111 => self.or_a(bus, Operand::Direct(Register::A)),

            // CP A, g
            0b10_111_000 => self.sub_a(bus, Operand::Direct(Register::B), false, false),
            0b10_111_001 => self.sub_a(bus, Operand::Direct(Register::C), false, false),
            0b10_111_010 => self.sub_a(bus, Operand::Direct(Register::D), false, false),
            0b10_111_011 => self.sub_a(bus, Operand::Direct(Register::E), false, false),
            0b10_111_100 => self.sub_a(bus, Operand::Direct(Register::H), false, false),
            0b10_111_101 => self.sub_a(bus, Operand::Direct(Register::L), false, false),
            0b10_111_110 => self.sub_a(bus, Operand::Indirect(RegIndirect::HL), false, false),
            0b10_111_111 => self.sub_a(bus, Operand::Direct(Register::A), false, false),

            0b11_000_000 => self.error("dispatch"),
            0b11_000_001 => self.pop(bus, Operand::Direct(Register::BC)),
            0b11_000_010 => self.jp(bus, Operand::Immediate16(), Some(Condition::NonZero)),
            0b11_000_011 => self.jp(bus, Operand::Immediate16(), None),
            0b11_000_100 => self.call(bus, Operand::Immediate16(), Some(Condition::NonZero)),
            0b11_000_101 => self.push(bus, Operand::Direct(Register::BC)),
            0b11_000_110 => self.add_a(bus, Operand::Immediate(), false),
            0b11_000_111 => self.error("dispatch"),

            0b11_001_000 => self.error("dispatch"),
            0b11_001_001 => self.ret(bus, None),
            0b11_001_010 => self.jp(bus, Operand::Immediate16(), Some(Condition::Zero)),
            0b11_001_011 => self.bits(bus),
            0b11_001_100 => self.call(bus, Operand::Immediate16(), Some(Condition::Zero)),
            0b11_001_101 => self.call(bus, Operand::Immediate16(), None),
            0b11_001_110 => self.add_a(bus, Operand::Immediate(), true),
            0b11_001_111 => self.error("dispatch"),

            0b11_010_000 => self.error("dispatch"),
            0b11_010_001 => self.pop(bus, Operand::Direct(Register::DE)),
            0b11_010_010 => self.jp(bus, Operand::Immediate16(), Some(Condition::NonCarry)),
            0b11_010_011 => self.error("dispatch"),
            0b11_010_100 => self.call(bus, Operand::Immediate16(), Some(Condition::NonCarry)),
            0b11_010_101 => self.push(bus, Operand::Direct(Register::DE)),
            0b11_010_110 => self.sub_a(bus, Operand::Immediate(), false, true),
            0b11_010_111 => self.error("dispatch"),

            0b11_011_000 => self.error("dispatch"),
            0b11_011_001 => self.exchange(bus, Exchange::X),
            0b11_011_010 => self.jp(bus, Operand::Immediate16(), Some(Condition::Carry)),
            0b11_011_011 => self.error("dispatch"),
            0b11_011_100 => self.call(bus, Operand::Immediate16(), Some(Condition::Carry)),
            0b11_011_101 => self.index(bus, RegIndex::IX),
            0b11_011_110 => self.sub_a(bus, Operand::Immediate(), true, true),
            0b11_011_111 => self.error("dispatch"),

            0b11_100_000 => self.error("dispatch"),
            0b11_100_001 => self.pop(bus, Operand::Direct(Register::HL)),
            0b11_100_010 => self.jp(bus, Operand::Immediate16(), Some(Condition::ParityOdd)),
            0b11_100_011 => self.exchange(bus, Exchange::SP_HL),
            0b11_100_100 => self.call(bus, Operand::Immediate16(), Some(Condition::ParityOdd)),
            0b11_100_101 => self.push(bus, Operand::Direct(Register::HL)),
            0b11_100_110 => self.and_a(bus, Operand::Immediate()),
            0b11_100_111 => self.error("dispatch"),

            0b11_101_000 => self.error("dispatch"),
            0b11_101_001 => self.error("dispatch"),
            0b11_101_010 => self.jp(bus, Operand::Immediate16(), Some(Condition::ParityEven)),
            0b11_101_011 => self.exchange(bus, Exchange::DE_HL),
            0b11_101_100 => self.call(bus, Operand::Immediate16(), Some(Condition::ParityEven)),
            0b11_101_101 => self.extended(bus),
            0b11_101_110 => self.error("dispatch"),
            0b11_101_111 => self.error("dispatch"),

            0b11_110_000 => self.error("dispatch"),
            0b11_110_001 => self.pop(bus, Operand::Direct(Register::AF)),
            0b11_110_010 => self.jp(bus, Operand::Immediate16(), Some(Condition::SignPlus)),
            0b11_110_011 => self.di(),
            0b11_110_100 => self.call(bus, Operand::Immediate16(), Some(Condition::SignPlus)),
            0b11_110_101 => self.push(bus, Operand::Direct(Register::AF)),
            0b11_110_110 => self.error("dispatch"),
            0b11_110_111 => self.error("dispatch"),

            0b11_111_000 => self.error("dispatch"),
            0b11_111_001 => self.ld_16(bus, Operand::Direct(Register::HL), Operand::Direct(Register::SP)),
            0b11_111_010 => self.jp(bus, Operand::Immediate16(), Some(Condition::SignMinus)),
            0b11_111_011 => self.ei(),
            0b11_111_100 => self.call(bus, Operand::Immediate16(), Some(Condition::SignMinus)),
            0b11_111_101 => self.index(bus, RegIndex::IY),
            0b11_111_110 => self.sub_a(bus, Operand::Immediate(), false, false),
            0b11_111_111 => self.error("dispatch"),
        }
    }

    // Extended instructions. Again, only constant expressions are used to allow a lookup table.
    // Unlike the basic opcode set the $ED group isn't complete, so there is a default entry.
    fn extended(&mut self, bus: &mut Bus) {
        let opcode = bus.mem_read(self.mmu.to_physical(self.sr.pc));
        self.sr.pc += 1;

        match opcode {
            0b00_000_001 => self.out0(bus, Operand::Direct(Register::B), Operand::Immediate()),
            0b00_001_001 => self.out0(bus, Operand::Direct(Register::C), Operand::Immediate()),
            0b00_010_001 => self.out0(bus, Operand::Direct(Register::D), Operand::Immediate()),
            0b00_011_001 => self.out0(bus, Operand::Direct(Register::E), Operand::Immediate()),
            0b00_100_001 => self.out0(bus, Operand::Direct(Register::H), Operand::Immediate()),
            0b00_101_001 => self.out0(bus, Operand::Direct(Register::L), Operand::Immediate()),
            0b00_111_001 => self.out0(bus, Operand::Direct(Register::A), Operand::Immediate()),

            0b01_000_010 => self.sub_hl_ww(RegW::BC, true),
            0b01_010_010 => self.sub_hl_ww(RegW::DE, true),
            0b01_100_010 => self.sub_hl_ww(RegW::HL, true),
            0b01_110_010 => self.sub_hl_ww(RegW::SP, true),

            0b01_110_011 => self.ld_16(bus, Operand::Direct(Register::SP), Operand::Extended16()),
            0b01_111_011 => self.ld_16(bus, Operand::Extended16(), Operand::Direct(Register::SP)),

            0b10_110_000 => self.ldir(bus),

            _ => self.error("extended"),
        }
    }

    // Bit instructions: rot, shift, test.
    fn bits(&mut self, bus: &mut Bus) {
        let opcode = bus.mem_read(self.mmu.to_physical(self.sr.pc));
        self.sr.pc += 1;

        match opcode {
            // RLC g/(HL)
            0b00_000_000 => self.rot_left(bus, Operand::Direct(Register::B), true),
            0b00_000_001 => self.rot_left(bus, Operand::Direct(Register::C), true),
            0b00_000_010 => self.rot_left(bus, Operand::Direct(Register::D), true),
            0b00_000_011 => self.rot_left(bus, Operand::Direct(Register::E), true),
            0b00_000_100 => self.rot_left(bus, Operand::Direct(Register::H), true),
            0b00_000_101 => self.rot_left(bus, Operand::Direct(Register::L), true),
            0b00_000_110 => self.rot_left(bus, Operand::Indirect(RegIndirect::HL), true),
            0b00_000_111 => self.rot_left(bus, Operand::Direct(Register::A), true),

            // RRC g/(HL)
            0b00_001_000 => self.rot_right(bus, Operand::Direct(Register::B), true),
            0b00_001_001 => self.rot_right(bus, Operand::Direct(Register::C), true),
            0b00_001_010 => self.rot_right(bus, Operand::Direct(Register::D), true),
            0b00_001_011 => self.rot_right(bus, Operand::Direct(Register::E), true),
            0b00_001_100 => self.rot_right(bus, Operand::Direct(Register::H), true),
            0b00_001_101 => self.rot_right(bus, Operand::Direct(Register::L), true),
            0b00_001_110 => self.rot_right(bus, Operand::Indirect(RegIndirect::HL), true),
            0b00_001_111 => self.rot_right(bus, Operand::Direct(Register::A), true),

            // RL g/(HL)
            0b00_010_000 => self.rot_left(bus, Operand::Direct(Register::B), false),
            0b00_010_001 => self.rot_left(bus, Operand::Direct(Register::C), false),
            0b00_010_010 => self.rot_left(bus, Operand::Direct(Register::D), false),
            0b00_010_011 => self.rot_left(bus, Operand::Direct(Register::E), false),
            0b00_010_100 => self.rot_left(bus, Operand::Direct(Register::H), false),
            0b00_010_101 => self.rot_left(bus, Operand::Direct(Register::L), false),
            0b00_010_110 => self.rot_left(bus, Operand::Indirect(RegIndirect::HL), false),
            0b00_010_111 => self.rot_left(bus, Operand::Direct(Register::A), false),

            // RR g/(HL)
            0b00_011_000 => self.rot_right(bus, Operand::Direct(Register::B), false),
            0b00_011_001 => self.rot_right(bus, Operand::Direct(Register::C), false),
            0b00_011_010 => self.rot_right(bus, Operand::Direct(Register::D), false),
            0b00_011_011 => self.rot_right(bus, Operand::Direct(Register::E), false),
            0b00_011_100 => self.rot_right(bus, Operand::Direct(Register::H), false),
            0b00_011_101 => self.rot_right(bus, Operand::Direct(Register::L), false),
            0b00_011_110 => self.rot_right(bus, Operand::Indirect(RegIndirect::HL), false),
            0b00_011_111 => self.rot_right(bus, Operand::Direct(Register::A), false),

            _ => self.error("bits"),
        }
    }

    // Index register opcodes. The opcode sets are identical for IX and IY.
    fn index(&mut self, bus: &mut Bus, index: RegIndex) {
        let opcode = bus.mem_read(self.mmu.to_physical(self.sr.pc));
        self.sr.pc += 1;

        match opcode {
            0b11_100_001 => self.pop(bus, Operand::Direct(index.into())),
            0b11_100_101 => self.push(bus, Operand::Direct(index.into())),

            _ => self.error("index"),
        }
    }
}
