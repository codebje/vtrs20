use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};

use crate::bus::Bus;
use crate::types::*;

#[derive(PartialEq, Copy, Clone)]
enum Mode {
    READ,
    ID,
    PROGRAM,
    WRITING,
    COMMAND1,
    COMMAND2,
    ERASE1,
    ERASE2,
    ERASE3,
}

pub struct ROM {
    start: u32,
    size: u32,
    bytes: RefCell<Vec<u8>>,
    is_masking: RefCell<bool>,
    mode: RefCell<Mode>,
    begun: RefCell<Instant>,
}

impl ROM {
    pub fn new(base: u32, contents: Vec<u8>) -> ROM {
        ROM {
            start: base,
            size: contents.len() as u32,
            bytes: RefCell::new(contents),
            is_masking: RefCell::new(true),
            mode: RefCell::new(Mode::READ),
            begun: RefCell::new(Instant::now()),
        }
    }
}

impl Peripheral for ROM {
    fn reset(&self) {
        *self.is_masking.borrow_mut() = true;
    }

    fn cycle(&self, _bus: &Bus) -> Option<Interrupt> {
        // programming modes are exited after 10ms
        if *self.mode.borrow() == Mode::WRITING {
            if self.begun.borrow().elapsed() > Duration::from_millis(10) {
                *self.mode.borrow_mut() = Mode::READ;
            }
        }

        None
    }

    fn mem_read(&self, address: u32, m1: bool) -> Option<u8> {
        let mut addr = address;

        // After reset the ROM forces A19 high until the processor
        // has driven it high on its own.
        if m1 && (addr & 0b1000_0000_0000_0000_0000) != 0 {
            *self.is_masking.borrow_mut() = false;
        }

        if *self.is_masking.borrow() {
            addr |= 0b1000_0000_0000_0000_0000;
        }

        if addr >= self.start && addr <= self.start + self.size {
            let mut val = self.bytes.borrow()[(addr - self.start) as usize];

            match *self.mode.borrow() {
                Mode::READ => (),
                Mode::ID => val = if addr & 1 == 0 { 0xbf } else { 0xd6 },
                Mode::PROGRAM => (),
                Mode::WRITING => val = (val ^ 0x80) & 0x80,
                Mode::COMMAND1 => (),
                Mode::COMMAND2 => (),
                Mode::ERASE1 => (),
                Mode::ERASE2 => (),
                Mode::ERASE3 => (),
            }

            return Some(val);
        }
        None
    }

    fn mem_write(&self, address: u32, data: u8) {
        let mode = *self.mode.borrow();
        match mode {
            Mode::READ => {
                if address == self.start + 0x5555 && data == 0xaa {
                    *self.mode.borrow_mut() = Mode::COMMAND1;
                }
            }
            Mode::ID => {
                if address >= self.start && address <= self.start + self.size && data == 0xf0 {
                    *self.mode.borrow_mut() = Mode::READ;
                }
            }
            Mode::PROGRAM => {
                if address >= self.start && address <= self.start + self.size {
                    //println!("Write byte {:04x} to {:05x}", data, address);
                    self.bytes.borrow_mut()[(address - self.start) as usize] = data;
                    *self.mode.borrow_mut() = Mode::WRITING;
                    *self.begun.borrow_mut() = Instant::now();
                }
            }
            Mode::WRITING =>
            //(), // nothing is accepted during programming
            {
                if address >= self.start && address <= self.start + self.size {
                    println!("Attempt to write byte {:04x} to {:05x} during WRITING", data, address);
                    let one_ms = Duration::from_millis(100000);
                    thread::sleep(one_ms);
                }
            }
            Mode::COMMAND1 => {
                if address == self.start + 0x2aaa && data == 0x55 {
                    *self.mode.borrow_mut() = Mode::COMMAND2;
                } else if address >= self.start && address <= self.start + self.size {
                    println!("COMMAND1 exited with byte {:04x} to {:05x}", data, address);
                    *self.mode.borrow_mut() = Mode::READ;
                }
            }
            Mode::COMMAND2 => {
                if address == self.start + 0x5555 && data == 0xA0 {
                    *self.mode.borrow_mut() = Mode::PROGRAM;
                } else if address == self.start + 0x5555 && data == 0x80 {
                    *self.mode.borrow_mut() = Mode::ERASE1;
                } else if address == self.start + 0x5555 && data == 0x90 {
                    *self.mode.borrow_mut() = Mode::ID;
                } else if address >= self.start && address <= self.start + self.size {
                    println!("COMMAND2 exited with byte {:04x} to {:05x}", data, address);
                    *self.mode.borrow_mut() = Mode::READ;
                }
            }
            Mode::ERASE1 => {
                if address == self.start + 0x5555 && data == 0xaa {
                    *self.mode.borrow_mut() = Mode::ERASE2;
                } else if address >= self.start && address <= self.start + self.size {
                    *self.mode.borrow_mut() = Mode::READ;
                }
            }
            Mode::ERASE2 => {
                if address == self.start + 0x2aaa && data == 0x55 {
                    *self.mode.borrow_mut() = Mode::ERASE3;
                } else if address >= self.start && address <= self.start + self.size {
                    *self.mode.borrow_mut() = Mode::READ;
                }
            }
            Mode::ERASE3 => {
                if address == self.start + 0x5555 && data == 0x10 {
                    // erase the lot, pow!
                    self.bytes.borrow_mut().iter_mut().map(|x| *x = 0xff).count();
                    *self.mode.borrow_mut() = Mode::WRITING;
                } else if address >= self.start && address <= self.start + self.size && data == 0x30 {
                    let mut bytes = self.bytes.borrow_mut();
                    let x = (address - self.start) as usize;
                    for addr in (x & !0xfff)..(x & !0xfff) + 0x1000 {
                        bytes[addr] = 0xff;
                    }
                    *self.mode.borrow_mut() = Mode::WRITING;
                } else if address >= self.start && address <= self.start + self.size {
                    *self.mode.borrow_mut() = Mode::READ;
                }
            }
        }
    }
}
