use jni_simple::jboolean;

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
