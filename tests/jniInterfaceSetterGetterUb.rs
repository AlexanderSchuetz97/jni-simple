use jni_simple::*;
use std::ffi::c_void;
use std::ptr::null_mut;

extern "system" fn beep() -> *mut *mut c_void {
    null_mut()
}
#[test]
pub fn test() {
    unsafe {
        let mut data: Vec<*mut c_void> = Vec::new();
        data.resize(100, null_mut::<c_void>());
        let mut_ptr = data.as_mut_ptr();

        let raw = jniNativeInterface::from_raw_ptr(mut_ptr.cast());
        raw.set(1, beep as _);
        let o = raw.get::<extern "system" fn() -> JNIEnv>(1)();
        assert_eq!(o.vtable(), null_mut())
    }
}
