#![no_std]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn putchar(char: i32) -> i32 {
    unsafe {
        libc::putchar(char)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getchar() -> i32 {
    unsafe {
        libc::getchar()
    }
}