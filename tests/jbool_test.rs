use jni_simple::{JBooleanMutPtr, jboolean};
use std::ffi::c_uchar;
use std::ptr::null_mut;

#[test]
fn test_bool_slice_transmute() {
    let k = &[true, false];
    let jbsl = jboolean::transmute_bool_slice(k);
    assert_eq!(jbsl.len(), k.len());
    assert_eq!(jbsl[0].as_wide(), 1);
    assert_eq!(jbsl[1].as_wide(), 0);
    assert_eq!(jbsl[0].as_bool(), true);
    assert_eq!(jbsl[1].as_bool(), false);
}

#[test]
fn test_wide_slice_transmute() {
    let k = &[123, 0, 222];
    let jbsl = jboolean::transmute_wide_slice(k);
    assert_eq!(jbsl.len(), k.len());
    assert_eq!(jbsl[0].as_wide(), 123);
    assert_eq!(jbsl[1].as_wide(), 0);
    assert_eq!(jbsl[2].as_wide(), 222);
    assert_eq!(jbsl[0].as_bool(), true);
    assert_eq!(jbsl[1].as_bool(), false);
    assert_eq!(jbsl[2].as_bool(), true);
}

#[test]
fn test_wide_slice_mut_transmute() {
    let k = &mut [123, 0, 222];
    let kl = k.len();
    let jbsl = jboolean::transmute_wide_slice_mut(k);
    assert_eq!(jbsl.len(), kl);
    assert_eq!(jbsl[0].as_wide(), 123);
    assert_eq!(jbsl[1].as_wide(), 0);
    assert_eq!(jbsl[2].as_wide(), 222);
    assert_eq!(jbsl[0].as_bool(), true);
    assert_eq!(jbsl[1].as_bool(), false);
    assert_eq!(jbsl[2].as_bool(), true);
    jbsl[0].narrow();
    assert_eq!(jbsl[0].as_bool(), true);
    assert_eq!(jbsl[0].as_wide(), 1);
    assert_eq!(k[0], 1);
}

#[test]
fn test_narrow_vec() {
    let k = vec![123, 0, 222];
    let ptr = k.as_ptr();
    let tm = jboolean::transmute_wide_vec(k);
    assert_eq!(tm[0].as_wide(), 123);
    assert_eq!(tm[1].as_wide(), 0);
    assert_eq!(tm[2].as_wide(), 222);
    let narrowed = jboolean::narrow_vec(tm);
    assert_eq!(narrowed[0], true);
    assert_eq!(narrowed[1], false);
    assert_eq!(narrowed[2], true);
    assert!(core::ptr::addr_eq(ptr, narrowed.as_ptr()));
}

#[test]
fn test_from_wide() {
    let mut bool = jboolean::from_wide(55);
    assert_eq!(bool.as_wide(), 55);
    assert_eq!(bool.as_bool(), true);
    assert_eq!(bool.is_wide(), true);
    bool.narrow();
    assert_eq!(bool.as_wide(), 1);
    assert_eq!(bool.as_bool(), true);
    assert_eq!(bool.is_wide(), false);

    let mut bool = jboolean::from_wide(0);
    assert_eq!(bool.as_wide(), 0);
    assert_eq!(bool.as_bool(), false);
    assert_eq!(bool.is_wide(), false);

    bool.narrow();
    assert_eq!(bool.as_wide(), 0);
    assert_eq!(bool.as_bool(), false);
    assert_eq!(bool.is_wide(), false);
}

#[test]
fn call_bool_fn() {
    fn test_fn(b: &bool) -> bool {
        *b
    }

    let mut bool = jboolean::from_wide(55);
    assert!(test_fn(bool.as_ref()));
    bool.narrow();
    assert!(test_fn(bool.as_ref()));
    bool = jboolean::from_wide(0);
    assert!(!test_fn(bool.as_ref()));
    bool.narrow();
    assert!(!test_fn(bool.as_ref()));
}
#[test]
fn call_int_fn() {
    fn test_fn(b: c_uchar) -> c_uchar {
        b
    }

    let mut bool = jboolean::from_wide(55);
    assert_eq!(test_fn(*bool), 55);
    bool.narrow();
    assert_eq!(test_fn(*bool), 1);
    bool = jboolean::from_wide(0);
    assert_eq!(test_fn(*bool), 0);
    bool.narrow();
    assert_eq!(test_fn(*bool), 0);
}

#[test]
fn use_uninit_bool() {
    //This test is mainly for miri
    fn dummy_fn(bla: impl JBooleanMutPtr) -> i32 {
        bla.use_jboolean_mut(|bla| unsafe {
            if bla.is_null() {
                return 1;
            }
            *bla = jboolean::from_wide(55);
            return 2;
        })
    }

    fn dummy_fn2(bla: impl JBooleanMutPtr) {
        bla.use_jboolean_mut(|bla| unsafe {
            if bla.is_null() {
                return;
            }
            *bla = false.into();
        })
    }

    let mut unit_bool = Box::<bool>::new_uninit();
    let mut_ptr = unit_bool.as_mut_ptr();
    dummy_fn(mut_ptr);
    let init = unsafe { unit_bool.assume_init() };
    assert_eq!(*init, true);

    let mut unit_bool = Box::<bool>::new_uninit();
    let mut_ptr = unit_bool.as_mut_ptr();
    dummy_fn2(mut_ptr);
    let init = unsafe { unit_bool.assume_init() };
    assert_eq!(*init, false);
    assert_eq!(dummy_fn(null_mut::<bool>()), 1);
}
