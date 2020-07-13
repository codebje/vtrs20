use crate::bus::Bus;
use crate::cpu::*;

/**
 * I/O Instructions
 *
 * Table 38 of Z8018x specification.
 */

impl CPU {
    // IN0 (m), g
    pub(super) fn in0(&mut self, bus: &mut Bus, src: Operand, dst: Operand) {
        let addr = self.load_operand(bus, dst);
        let data = bus.io_read(addr as u16);
        self.store_operand(bus, src, data as u16);
    }

    // OUT0 (m), g
    pub(super) fn out0(&mut self, bus: &mut Bus, src: Operand, dst: Operand) {
        let addr = self.load_operand(bus, dst);
        let data = self.load_operand(bus, src);
        bus.io_write(addr as u16, data as u8);
    }

    // OUT (m), A
    pub(super) fn out_m(&mut self, bus: &mut Bus, port: Operand) {
        let addr = self.load_operand(bus, port) | (self.reg(Register::A) << 8);
        bus.io_write(addr, self.gr.a);
    }

    // OUT (C), g
    /*pub(super) fn out_c(&mut self, bus: &mut Bus, src: Operand) {
        let data = self.load_operand(bus, src) as u8;
        bus.io_write(self.gr.bc, data);
    }*/

    // IN (m), A
    pub(super) fn in_m(&mut self, bus: &mut Bus, port: Operand) {
        let addr = self.load_operand(bus, port) | (self.reg(Register::A) << 8);
        self.gr.a = bus.io_read(addr);
    }
}
