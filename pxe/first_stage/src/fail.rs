use core::arch::asm;

use crate::println;

fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {
        hlt()
    }
}
