use crate::cpu::CPU;

/**
 * Special Control Instructions
 *
 * Table 47 of Z8018x specification.
 */

impl CPU {
    pub(super) fn nop(&mut self) {
        self.sr.pc += 1;
    }

    pub(super) fn di(&mut self) {
        self.ie = false;
    }

    pub(super) fn ei(&mut self) {
        self.ie = true;
    }
}
