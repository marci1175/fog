#[unsafe(no_mangle)]
pub extern "C" fn print(msg: *const i8) {
    let c_str = unsafe { std::ffi::CStr::from_ptr(msg) };
    println!("{}", c_str.to_string_lossy());
}