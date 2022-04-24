use core::arch::asm;

#[no_mangle]
pub extern "C" fn print_char(c: u8) {
    let ax = u16::from(c) | 0x0e00;
    unsafe {
        asm!("int 0x10", in("ax") ax, in("bx") 0);
    }
}

#[macro_export]
macro_rules! print {
    ($($tt:tt)*) => {{
        use core::fmt::Write;
        write!($crate::print::Printer, $($tt)*).unwrap();
    }};
}

#[macro_export]
macro_rules! println {
    ($($tt:tt)*) => {{
        use core::fmt::Write;
        writeln!($crate::print::Printer, $($tt)*).unwrap();
    }};
}

pub struct Printer;

impl core::fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                print_char(b'\r');
            }
            print_char(c as u8);
        }
        Ok(())
    }
}
