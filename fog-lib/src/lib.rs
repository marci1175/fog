#![no_std]
use core::panic::PanicInfo;

#[panic_handler]
fn panic(panic: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn test() -> i32 {
    return 0;
}