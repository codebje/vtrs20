use crate::cpu::CPU;

/**
 * Special Control Instructions
 *
 * Table 47 of Z8018x specification.
 */

impl CPU {
    pub(super) fn nop(&mut self) {}

    pub(super) fn di(&mut self) {
        self.ie = false;
    }

    pub(super) fn ei(&mut self) {
        self.ie = true;
    }
}

#[cfg(test)]
mod alu_test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::{Register, CPU};
    use crate::ram::RAM;

    #[test]
    fn nop_incrs_pc() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(0x0000, &[0x00]);
        bus.add(ram);
        cpu.reset();

        let pc_prev = cpu.reg(Register::PC);
        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::PC), pc_prev + 1, "PC increments");
    }
}
