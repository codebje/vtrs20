use crate::bus::Bus;
use crate::cpu::*;

/**
 * I/O Instructions
 *
 * Table 38 of Z8018x specification.
 */

impl CPU {
    // OUT0 (m), g
    pub(super) fn out0(&mut self, bus: &mut Bus, src: Operand, dst: Operand) {
        let addr = self.load_operand(bus, dst);
        let data = self.load_operand(bus, src);
        bus.io_write(addr as u16, data as u8);
    }
}
