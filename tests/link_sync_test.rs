#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
extern crate alloc;

/// Polyfill to make linking.rs compile.
mod polyfill {
    use jni_simple::jniNativeInterface;
    pub use jni_simple::{JNI_OK, jint, jsize};
    use std::ffi::{c_char, c_void};
    use sync_ptr::SyncMutPtr;

    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct JavaVM {
        pub vtable: JNIInvPtr,
    }
    /// Vtable of `JNIEnv` is passed like this.
    type JNIEnvVTable = *mut jniNativeInterface;
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct JNIEnv {
        pub vtable: JNIEnvVTable,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct JavaVMOption {
        pub optionString: *mut c_char,
        pub extraInfo: *mut c_void,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct JavaVMInitArgs {
        pub version: i32,
        pub nOptions: i32,
        pub options: *mut JavaVMOption,
        pub ignoreUnrecognized: u8,
    }

    pub type JNIInvPtr = SyncMutPtr<*mut *mut c_void>;
}

pub use polyfill::*;

mod linking {
    include!("../src/linking.rs");
}

#[cfg(not(feature = "dynlink"))]
mod test {
    use crate::linking::JNIDynamicLink;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering::SeqCst;
    use std::thread;
    use std::time::Duration;

    #[test]
    pub fn test_it() {
        static STATE: AtomicUsize = AtomicUsize::new(0);

        let guard = crate::linking::link_read();
        assert!(guard.is_none());
        drop(guard);

        let jh = thread::spawn(move || {
            let mut k = crate::linking::link_write();
            assert!(k.is_none());
            STATE.store(1, SeqCst);
            thread::sleep(Duration::from_secs(1));
            *k = Some(JNIDynamicLink::new(std::ptr::dangling(), std::ptr::dangling()));
            STATE.store(2, SeqCst);
            drop(k);
        });

        loop {
            if STATE.load(SeqCst) != 0 {
                break;
            }

            if jh.is_finished() {
                if STATE.load(SeqCst) != 0 {
                    break;
                }

                jh.join().unwrap();
                panic!("Other thread finished without error");
            }

            std::hint::spin_loop();
        }

        let guard = crate::linking::link_read();
        assert_eq!(STATE.load(SeqCst), 2);
        assert!(guard.is_some());
        jh.join().unwrap();
        drop(guard);

        let jh = thread::spawn(move || {
            let guard = crate::linking::link_read();
            assert!(guard.is_some());
            STATE.store(3, SeqCst);
            thread::sleep(Duration::from_secs(1));
            drop(guard);
            STATE.store(4, SeqCst);
        });

        loop {
            if STATE.load(SeqCst) > 2 {
                break;
            }

            if jh.is_finished() {
                if STATE.load(SeqCst) > 2 {
                    break;
                }

                jh.join().unwrap();
                panic!("Other thread finished without error");
            }

            std::hint::spin_loop();
        }

        let guard = crate::linking::link_read();
        assert!(guard.is_some());
        assert_eq!(STATE.load(SeqCst), 3);
        jh.join().unwrap();
        assert_eq!(STATE.load(SeqCst), 4);
        drop(guard);
    }
}
