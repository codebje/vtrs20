# Virtual TRS-20

This is a companion emulator for my TRS-20 electronics project, which is scattered throughout repositories here.

There are many, many Z80 emulators around. Sadly, there don't seem to be many Z180 emulators, so the CPU core is original code.

I am a novice at Rust, so this code will likely be tortuous and inefficiently designed. Here we go.

There is a [board](src/board.rs) that controls the whole show. It has a [CPU](src/cpu/) and a [bus](src/bus.rs), and the bus has a set of peripherals that can handle memory or I/O requests.

The CPU executes one machine cycle at a time - there's no clock cycle emulation. During a machine cycle the CPU may read or write on the bus as much as it needs, such as reading an opcode (up to three bytes) and performing any requested memory or I/O operations.

The Z8S180 has a whole stack of built-in peripherals, such as an MMU unit that translates the CPU core's 16-bit logical address space to the die's 20-bit physical address space. I use `Rc` reference counting to allow both the bus and the CPU to hold a stake in ownership over CPU peripherals, and I use `RefCell` to allow the MMU to be shared, but to only be mutable via its `io_write` implementation that only the bus should ever call.
