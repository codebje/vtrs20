use crate::bus::Bus;
use crate::cpu::CPU;

/**
 * Arithmetic and Logical Functions (8-bit)
 *
 * Table 38 of Z8018x specification.
 */

impl CPU {
    // Compute flags for 8-bit additions:
    //  Set Sign, Zero, Half, and Carry.
    //  Set P/V for oVerflow.
    //  Reset Negative.
    //
    // Overflow is set if the twos-complement addition is out of range. This happens only when
    // adding two positive numbers or two negative numbers (ie, bit 7 is equal in src and add)
    // and the result is a different sign.
    fn add_flags(src: u16, add: u16, result: u16) -> u8 {
        let flags =
            (result & 0b1000_0000)                  // sign equals bit 7 of result
            | (if result == 0 { 0b0100_0000 } else { 0 }) // zero
            | ((src ^ add ^ result) & 0b0001_0000)  // half-carry set if bit carried into result bit 5
            | ((((src ^ add ^ 0b1000_0000) & (src ^ result)) >> 5) & 0b0000_0100) // overflow
            | (result >> 8 & 0b0000_0001)           // carry equals bit 8 of result
            ;
        flags as u8
    }

    // Execute ADD A,g or ADD A,(HL)
    pub(super) fn add_a_g(&mut self, bus: &mut Bus, g: u8) {
        match self.load_g_hl(bus, g) {
            Ok(reg) => {
                let result = self.gr.a as u16 + reg as u16;
                self.gr.f = CPU::add_flags(self.gr.a as u16, reg as u16, result);
                self.gr.a = result as u8;
            }
            _ => self.error(),
        }
    }
}

#[cfg(test)]
mod alu_test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::CPU;
    use crate::rom::ROM;

    #[test]
    fn adder_flags() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let rom = ROM::new(0x80000, vec![0x3e, 0x70, 0x06, 0x30, 0x80]);
        bus.add(Rc::new(rom));
        cpu.reset();
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        // 112+48=160, Sz-h-Vnc
        assert_eq!(cpu.gr.f & 0b11010111, 0b1000_0100);
    }
}
