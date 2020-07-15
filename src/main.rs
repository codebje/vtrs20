use std::rc::Rc;

use clap::{App, Arg};

use emulator::asci::*;
use emulator::bus::Bus;
use emulator::cpu::{Mode, Register, CPU};
use emulator::dma::*;
use emulator::prt::*;
use emulator::ram::*;
use emulator::rom::*;

fn print_cpu(cpu: &CPU, bus: &mut Bus) {
    let opcodes = [
        bus.mem_read(cpu.reg(Register::PC) as u32), // assume identity MMU
        bus.mem_read(cpu.reg(Register::PC) as u32 + 1),
        bus.mem_read(cpu.reg(Register::PC) as u32 + 2),
        bus.mem_read(cpu.reg(Register::PC) as u32 + 3),
    ];
    let flags = cpu.reg(Register::F);
    println!(
        "PC=${:04x}, SP=${:04x} \
                A=${:02x} BC=${:04x} DE=${:04x} HL=${:04x} IX=${:04x} IY=${:04x} \
                {}{}-{}-{}{}{}    {}",
        cpu.reg(Register::PC),
        cpu.reg(Register::SP),
        cpu.reg(Register::A),
        cpu.reg(Register::BC),
        cpu.reg(Register::DE),
        cpu.reg(Register::HL),
        cpu.reg(Register::IX),
        cpu.reg(Register::IY),
        if flags & 0b1000_0000 != 0 { 'S' } else { 's' },
        if flags & 0b0100_0000 != 0 { 'Z' } else { 'z' },
        if flags & 0b0001_0000 != 0 { 'H' } else { 'h' },
        if flags & 0b0000_0100 != 0 { 'P' } else { 'p' },
        if flags & 0b0000_0010 != 0 { 'N' } else { 'n' },
        if flags & 0b0000_0001 != 0 { 'C' } else { 'c' },
        emulator::disasm::disasm(&opcodes),
    );
}

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("Virtual TRS-20")
        .version("1.0")
        .about("Emulate the TRS-20 SBC")
        .arg(Arg::with_name("ROM").required(true).index(1))
        .arg(
            Arg::with_name("tty")
                .short("t")
                .long("tty")
                .value_name("DEVICE")
                .help("Tie ASCI0 to a TTY device")
                .takes_value(true),
        )
        .get_matches();

    let mut bus = Bus::new();
    let mut cpu = CPU::new(&mut bus);
    let ram = Rc::new(RAM::new(0x00000, 0x80000));
    let rom_data = std::fs::read(matches.value_of("ROM").unwrap())?;
    let rom = Rc::new(ROM::new(0x80000, rom_data));

    bus.add(rom); // ROM is first to allow address masking to work
    bus.add(ram);

    let prt = Rc::new(PRT::new());
    bus.add(prt);

    let dma = Rc::new(DMA::new());
    bus.add(dma);

    match matches.value_of("tty") {
        Some(tty) => {
            let uart = ASCI::new(Channel::CH0, tty);
            bus.add(Rc::new(uart));
        }
        None => (),
    }

    cpu.reset();

    // to implement a simple debugger:
    // https://docs.rs/rustyline/6.2.0/rustyline/
    // https://docs.rs/ctrlc/3.1.5/ctrlc/
    //
    // or
    // https://microsoft.github.io/debug-adapter-protocol/specification#Types_Capabilities

    loop {
        let mut instr = [0u8; 4];
        cpu.get_current_opcodes(&mut bus, &mut instr);
        let pc = cpu.reg(Register::PC);
        if pc >= 0x32a && pc < 0x470 {
            print_cpu(&cpu, &mut bus);
        }
        if pc == 0x0413 && cpu.reg(Register::A) != 0x04 {
            let base = cpu.reg(Register::HL) as usize;
            let frame = cpu.reg(Register::A);
            let range = if frame == 0x01 { 0..8 } else { 0..64 };
            for i in range {
                print!("{:04x}   ", base + i * 16);
                let mut m = [0u8; 16];
                for b in 0..16 {
                    m[b] = bus.mem_read((base + i * 16 + b) as u32);
                    print!("{:02x} ", m[b]);
                    if b == 7 {
                        print!(" ");
                    }
                }
                print!("     ");
                for b in 0..16 {
                    print!("{}", if m[b] >= 32 { m[b] as char } else { '.' });
                }
                println!("");
            }
        }
        if cpu.mode != Mode::OpCodeFetch {
            break;
        }
        cpu.cycle(&mut bus);
    }

    Ok(())
}
