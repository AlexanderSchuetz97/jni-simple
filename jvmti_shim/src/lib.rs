#![allow(non_snake_case)]
use std::ffi::{c_char, c_void, CStr};
use std::mem;

#[unsafe(no_mangle)]
extern "system" fn Agent_OnLoad(vm: *mut c_void, options: *const c_char, reserved: *mut c_void) -> i32 {

    eprintln!("Agent_OnLoad");
    unsafe {
        let n = CStr::from_ptr(options).to_string_lossy().to_string();
        eprintln!("{}", n);
        let Ok(ptr) = n.parse::<usize>() else {
          return -1;
        };
        eprintln!("{}", ptr);
        let r = mem::transmute::<usize, extern "C" fn(*mut c_void, *const c_char, *mut c_void) -> i32>(ptr);
        let n = r(vm, options, reserved);
        eprintln!("Agent_OnLoad_exit");
        n
    }
}