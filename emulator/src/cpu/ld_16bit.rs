use crate::bus::Bus;
use crate::cpu::*;

/**
 * 16-bit Load
 *
 * Table 42 of Z8018x specification.
 */

impl CPU {
    // There's really nothing to this after the decode...
    pub(super) fn ld_16(&mut self, bus: &mut Bus, src: Operand, dst: Operand) {
        let val = self.load_operand(bus, src);
        self.store_operand(bus, dst, val);
    }
}
