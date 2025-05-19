#![cfg_attr(not(feature = "std"), no_std)]
#![no_main]

use libc::putchar;

#[cfg(not(any(test, feature = "std")))]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn return_1() -> i32 {
    return 1;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn printchar(c: i32) -> i32 {
    unsafe {
        putchar(c);
    };

    c
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn greet() -> i32 {
    unsafe {
        let char_array: &[u8] = b"Hello world!\n\0";
        
        libc::puts(char_array.as_ptr() as *const i8)
    }
}