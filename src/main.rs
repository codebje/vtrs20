use std::rc::Rc;
use std::{thread, time};

use clap::{App, Arg};

use emulator::asci::*;
use emulator::bus::Bus;
use emulator::cpu::{Mode, Register, CPU};
use emulator::dma::*;
use emulator::prt::*;
use emulator::ram::*;
use emulator::rom::*;
use emulator::sdcard::*;

fn print_cpu(cpu: &mut CPU, bus: &mut Bus) {
    let mut opcodes: [u8; 4] = [0, 0, 0, 0];
    cpu.get_current_opcodes(bus, &mut opcodes);
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

fn print_bios_call(cpu: &mut CPU, _bus: &mut Bus, pc: u16) {
    match pc {
        0xf600 => println!("BIOS_REBOOT"),
        0xf603 => println!("BIOS_WBOOT"),
        0xf606 => {} // console status, who cares
        0xf609 => {} // console in
        0xf60c => {} // console out
        0xf60f => {} // list out
        0xf612 => {} // punch out
        0xf615 => {} // reader out
        0xf618 => println!("BIOS_HOME"),
        0xf61b => println!("BIOS_SELDSK({})", cpu.reg(Register::C)),
        0xf61e => println!("BIOS_SETTRK({})", cpu.reg(Register::BC)),
        0xf621 => println!("BIOS_SETSEC({})", cpu.reg(Register::BC)),
        0xf624 => println!("BIOS_SETDMA({:04x})", cpu.reg(Register::BC)),
        0xf627 => println!("BIOS_READ"),
        0xf62a => println!("BIOS_WRITE"),
        0xf62d => {} // list status
        0xf630 => println!("BIOS_SECTRN({}, {})", cpu.reg(Register::BC), cpu.reg(Register::DE)),
        _ => {}
    }
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
    bus.add(ram.clone());

    let prt = Rc::new(PRT::new());
    bus.add(prt);

    let dma = Rc::new(DMA::new());
    bus.add(dma);

    let sdcard = Rc::new(SDCard::new());
    bus.add(sdcard);

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
    //
    // consider reading .lst file with symtab

    let mut tracing = false;
    let mut booted = false;
    loop {
        let pc = cpu.reg(Register::PC);
        if pc == 0x101 && booted {
            tracing = true;

            let delay = time::Duration::from_millis(100);
            thread::sleep(delay);
        }
        if tracing {
            print_cpu(&mut cpu, &mut bus);
            let one_ms = time::Duration::from_millis(1);
            thread::sleep(one_ms);
        }
        if pc == 0xf600 {
            println!("BIOS has booted");
            booted = true;
            tracing = false;
        }
        if pc == 0xf603 {
            println!("BIOS has warm booted");
        }
        if pc >= 0xf979 && pc < 0xf9f5 {
            //print_cpu(&mut cpu, &mut bus);
        }
        // print BIOS calls
        if pc >= 0xF600 && pc < 0xF633 && false {
            print_bios_call(&mut cpu, &mut bus, pc);
        }
        if cpu.mode != Mode::OpCodeFetch {
            break;
        }
        cpu.cycle(&mut bus);
    }

    Ok(())
}
