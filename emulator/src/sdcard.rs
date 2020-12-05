use std::cell::RefCell;
use std::collections::VecDeque;

use crate::types::*;

#[derive(Clone, Copy, PartialEq, Debug)]
enum CardState {
    Command,
    ACommand,
    TokenWait,
    Writing,
}

pub struct SDCard {
    spi_ctrl: RefCell<u8>,
    spi_data: RefCell<u8>,
    spi_command: RefCell<Vec<u8>>,
    spi_response: RefCell<VecDeque<u8>>,
    idle: RefCell<bool>,
    state: RefCell<CardState>,
    write: RefCell<(usize, usize)>,
    sectors: RefCell<Vec<u8>>,
}

impl SDCard {
    pub fn new() -> SDCard {
        let v = vec![0xe5; 16 * 1024 * 1024 * 4];
        SDCard {
            spi_ctrl: RefCell::new(0),
            spi_data: RefCell::new(0xff),
            spi_command: RefCell::new(Vec::new()),
            spi_response: RefCell::new(VecDeque::new()),
            idle: RefCell::new(true),
            state: RefCell::new(CardState::Command),
            write: RefCell::new((0, 0)),
            sectors: RefCell::new(v),
        }
    }

    fn do_cmd(&self, cmd: &Vec<u8>) {
        let mut response = self.spi_response.borrow_mut();
        match cmd[0] - 0x40 {
            0 => {
                response.push_back(0x01);
                *self.idle.borrow_mut() = true;
            }
            8 => {
                response.push_back(if *self.idle.borrow() { 0x01 } else { 0x00 });
                response.push_back(0x00);
                response.push_back(0x00);
                response.push_back(cmd[3]);
                response.push_back(cmd[4]);
            }
            17 => {
                if *self.idle.borrow() {
                    response.push_back(0x05);
                } else {
                    response.push_back(0xff);
                    response.push_back(0x00);
                    response.push_back(0xff);
                    response.push_back(0xff);
                    response.push_back(0xfe);
                    let addr =
                        ((cmd[1] as usize) << 24) | ((cmd[2] as usize) << 16) | ((cmd[3] as usize) << 8) | (cmd[4] as usize);
                    let start = (addr - 8192) * 512;
                    let sector = &(*self.sectors.borrow())[start..start + 512];
                    for x in sector {
                        response.push_back(*x);
                    }
                    // no-one checks the CRC anyway, right?
                    response.push_back(0x00);
                    response.push_back(0x00);
                }
            }
            24 => {
                if *self.idle.borrow() {
                    response.push_back(0x05);
                } else {
                    response.push_back(0x00);
                    let addr =
                        ((cmd[1] as usize) << 24) | ((cmd[2] as usize) << 16) | ((cmd[3] as usize) << 8) | (cmd[4] as usize);
                    *self.state.borrow_mut() = CardState::TokenWait;
                    *self.write.borrow_mut() = (512, (addr - 8192) * 512);
                }
            }
            55 => {
                *self.state.borrow_mut() = CardState::ACommand;
                response.push_back(if *self.idle.borrow() { 0x01 } else { 0x00 });
            }
            58 => {
                response.push_back(if *self.idle.borrow() { 0x01 } else { 0x00 });
                if *self.idle.borrow() {
                    response.push_back(0x00);
                } else {
                    response.push_back(0xc0);
                }
                response.push_back(0xff);
                response.push_back(0x80);
                response.push_back(0x00);
            }
            _ => {
                response.push_back(if *self.idle.borrow() { 0x05 } else { 0x04 });
            }
        }
    }

    fn do_acmd(&self, cmd: &Vec<u8>) {
        let mut response = self.spi_response.borrow_mut();
        *self.state.borrow_mut() = CardState::Command;
        match cmd[0] - 0x40 {
            41 => {
                response.push_back(0x00);
                *self.idle.borrow_mut() = false;
            }
            _ => response.push_back(if *self.idle.borrow() { 0x05 } else { 0x04 }),
        }
    }

    fn do_write(&self, data: u8) {
        let mut write = self.write.borrow_mut();
        let mut response = self.spi_response.borrow_mut();
        (*self.sectors.borrow_mut())[write.1] = data;
        *write = (write.0 - 1, write.1 + 1);
        if write.0 == 0 {
            *self.state.borrow_mut() = CardState::Command;
            response.push_back(0xff); // CRC 1
            response.push_back(0xff); // CRC 2
            response.push_back(0xff); // thinking
            response.push_back(0xff); // thinking
            response.push_back(0x05); // data accepted
            response.push_back(0x00); // writing
            response.push_back(0x00); // writing
            response.push_back(0x00); // writing
            response.push_back(0x00); // writing
            response.push_back(0x00); // writing
            response.push_back(0xff); // done
        }
    }

    fn spi_write(&self, data: u8) {
        let ctrl = *self.spi_ctrl.borrow();
        if ctrl & 0x3 == 0x3 {
            let mut cmd = self.spi_command.borrow_mut();
            let state = *self.state.borrow();
            match state {
                CardState::Command | CardState::ACommand => match cmd.len() {
                    0 => {
                        if data & 0xc0 == 0x40 {
                            cmd.push(data);
                        }
                    }
                    1 | 2 | 3 | 4 => cmd.push(data),
                    5 => {
                        cmd.push(data);
                        if state == CardState::Command {
                            self.do_cmd(&cmd);
                        } else {
                            self.do_acmd(&cmd);
                        }
                    }
                    _ => match self.spi_response.borrow_mut().pop_front() {
                        Some(x) => *self.spi_data.borrow_mut() = x,
                        None => {
                            *self.spi_data.borrow_mut() = 0xff;
                            cmd.clear();
                        }
                    },
                },
                CardState::TokenWait => match self.spi_response.borrow_mut().pop_front() {
                    Some(x) => *self.spi_data.borrow_mut() = x,
                    None => {
                        *self.spi_data.borrow_mut() = 0xff;
                        if data == 0xfe {
                            *self.state.borrow_mut() = CardState::Writing;
                        }
                    }
                },
                CardState::Writing => {
                    self.do_write(data);
                }
            }
        }
    }
}

impl Peripheral for SDCard {
    fn io_read(&self, address: u16) -> Option<u8> {
        match address {
            0xf1 => Some(*self.spi_ctrl.borrow()),
            0xf2 => Some(*self.spi_data.borrow()),
            _ => None,
        }
    }
    fn io_write(&self, address: u16, data: u8) {
        match address {
            0xf1 => {
                // don't set bit 7
                *self.spi_ctrl.borrow_mut() = data & 0x7f;
            }
            0xf2 => self.spi_write(data),
            _ => {}
        }
    }
}
