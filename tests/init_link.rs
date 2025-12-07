use jni_simple::*;
use std::ffi::c_void;
use std::process::abort;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;

static CALLED: AtomicBool = AtomicBool::new(false);

extern "system" fn cjvm(inv_ptr: *mut *mut c_void, env_ptr: *mut *mut c_void, init_ptr: *mut JavaVMInitArgs) -> jint {
    unsafe {
        assert_eq!((*init_ptr).version(), JNI_VERSION_1_8);
        *inv_ptr = core::ptr::dangling_mut();
        *env_ptr = core::ptr::dangling_mut();
    }

    CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
    JNI_OK
}

extern "system" fn gjvm(_inv_ptr: *mut *mut c_void, _cnt: jsize, _sz: *mut jsize) -> jint {
    //We don't call this function in this test.
    abort();
}

#[test]
pub fn test_init_link() {
    assert!(!is_jvm_loaded());
    assert!(init_dynamic_link(cjvm as *const c_void, gjvm as *const c_void));
    assert!(is_jvm_loaded());
    assert!(!init_dynamic_link(cjvm as *const c_void, gjvm as *const c_void));
    assert!(is_jvm_loaded());
    assert!(!CALLED.load(SeqCst));

    unsafe {
        let (_vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &[""], true).expect("Failed to create fake jvm");
    }

    assert!(CALLED.load(SeqCst));
}
