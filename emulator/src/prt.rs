/**
 * Programmable reload timers
 * Known limitations:
 *  1. Interrupts are not implemented
 *  2. TIF1/0 are reset when TCR is read, not when TCR is read followed by TMDRn
 *  3. Timer 1 is not implemented
 *
 */
use std::cell::RefCell;
use std::time::Instant;

use crate::bus::Bus;
use crate::types::*;

pub struct PRT {
    timer0: RefCell<(Instant, u16)>,
    timer1: RefCell<(Instant, u16)>,
    rldr0: RefCell<u16>,
    tmdr0: RefCell<u16>,
    tmdr0t: RefCell<u8>,
    rldr1: RefCell<u16>,
    tmdr1: RefCell<u16>,
    tmdr1t: RefCell<u8>,
    tcr: RefCell<u8>,
}

impl PRT {
    pub fn new() -> PRT {
        PRT {
            timer0: RefCell::new((Instant::now(), 0xffff)),
            timer1: RefCell::new((Instant::now(), 0xffff)),
            rldr0: RefCell::new(0),
            tmdr0: RefCell::new(0xffff),
            tmdr0t: RefCell::new(0),
            rldr1: RefCell::new(0),
            tmdr1: RefCell::new(0xffff),
            tmdr1t: RefCell::new(0),
            tcr: RefCell::new(0),
        }
    }
}

impl Peripheral for PRT {
    fn cycle(&self, _bus: &Bus) -> Option<Interrupt> {
        let mut tcr = self.tcr.borrow_mut();
        if (*tcr & 0b0000_0001) != 0 {
            let (then, base) = *self.timer0.borrow();
            // get tick count since 'then' - 20e9 for divided by 20, converted to nanoseconds
            let ticks = then.elapsed().as_nanos() * 18432000 / 20000000000;

            // has the timer overflowed?
            if ticks >= base as u128 {
                let rldr = *self.rldr0.borrow();
                *tcr |= 0b0100_0000;
                // say ticks is 110, base is 100, reload is 100
                // ticks - base is 10 overflow
                // should overflow again in 90
                let now = (Instant::now(), rldr - ((ticks - base as u128) % (rldr as u128)) as u16);
                *self.tmdr0.borrow_mut() = now.1;
                *self.timer0.borrow_mut() = now;
            } else {
                *self.tmdr0.borrow_mut() = (base as u128 - ticks) as u16;
            }
        }
        None
    }

    #[rustfmt::skip]
    fn io_read(&self, address: u16) -> Option<u8> {
        match address {
            // Timer 0
            0x000c => { 
                let tmdr = *self.tmdr0.borrow();
                *self.tmdr0t.borrow_mut() = (tmdr >> 8) as u8;
                Some(tmdr as u8)
            }
            0x000d => Some(*self.tmdr0t.borrow()),
            0x000e => Some((*self.rldr0.borrow() >> 8) as u8),
            0x000f => Some(*self.rldr0.borrow() as u8),

            // Timer 1
            0x0014 => {
                let tmdr = *self.tmdr1.borrow();
                *self.tmdr1t.borrow_mut() = (tmdr >> 8) as u8;
                Some(tmdr as u8)
            }
            0x0015 => Some(*self.tmdr1t.borrow()),
            0x0016 => Some((*self.rldr0.borrow() >> 8) as u8),
            0x0017 => Some(*self.rldr0.borrow() as u8),

            0x0010 => {
                // not stricly to spec: turn off TIF1:0 when TCR is read
                let tcr = *self.tcr.borrow();
                *self.tcr.borrow_mut() &= 0b0011_1111;
                Some(tcr)
            },

            _ => None,
        }
    }

    #[rustfmt::skip]
    fn io_write(&self, address: u16, data: u8) {
        match address {
            // Timer 0
            0x000c => {
                let mut tmdr = self.tmdr0.borrow_mut();
                *tmdr = (*tmdr & 0xff00) | data as u16;
            }
            0x000d => {
                let mut tmdr = self.tmdr0.borrow_mut();
                *tmdr = (*tmdr & 0x00ff) | (data as u16) << 8;
            }
            0x000e => {
                let mut rldr = self.rldr0.borrow_mut();
                *rldr = (*rldr & 0xff00) | data as u16;
            }
            0x000f => {
                let mut rldr = self.rldr0.borrow_mut();
                *rldr = (*rldr & 0x00ff) | (data as u16) << 8;
            }

            // Timer 1
            0x0014 => {
                let mut tmdr = self.tmdr1.borrow_mut();
                *tmdr = (*tmdr & 0xff00) | data as u16;
            }
            0x0015 => {
                let mut tmdr = self.tmdr1.borrow_mut();
                *tmdr = (*tmdr & 0x00ff) | (data as u16) << 8;
            }
            0x0016 => {
                let mut rldr = self.rldr1.borrow_mut();
                *rldr = (*rldr & 0xff00) | data as u16;
            }
            0x0017 => {
                let mut rldr = self.rldr1.borrow_mut();
                *rldr = (*rldr & 0x00ff) | (data as u16) << 8;
            }

            0x0010 => {
                let mut tcr = self.tcr.borrow_mut();
                // If TDE0/TDE1 is changing, reset the state for that timer
                if data & 0b0000_0001 != *tcr & 0b0000_0001 {
                    *self.timer0.borrow_mut() = (Instant::now(), *self.tmdr0.borrow());
                }
                if data & 0b0000_0010 != *tcr & 0b0000_0010 {
                    *self.timer1.borrow_mut() = (Instant::now(), *self.tmdr1.borrow());
                }
                *tcr = (*tcr & 0b1100_0000) | (data & 0b0011_1111);
            }

            _ => (),
        }
    }
}
