#![cfg_attr(not(feature = "std"), no_std)]
#![no_main]


#[cfg(not(any(test, feature = "std")))]
#[panic_handler]
fn panic(_panic: &core::panic::PanicInfo) -> ! {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn return_1() -> i32 {
    return 1;
}
