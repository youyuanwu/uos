use core::fmt;
use core::fmt::Write;
use spin::Mutex;
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

static SERIAL1: Mutex<Option<Uart16550Tty<PioBackend>>> = Mutex::new(None);

pub fn init() {
    let tty = unsafe { Uart16550Tty::new_port(0x3F8, Config::default()) }
        .expect("failed to init serial port");
    *SERIAL1.lock() = Some(tty);
    let _ = crate::logger::init();
}

pub fn _print(args: fmt::Arguments) {
    if let Some(ref mut serial) = *SERIAL1.lock() {
        serial.write_fmt(args).expect("serial write failed");
    }
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}
