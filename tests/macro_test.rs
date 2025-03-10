use jni_simple::{jobject, jtype, jtypes};

fn fake_jni_call(typ: *const jtype) {
    assert_ne!(0usize, std::hint::black_box(typ) as usize)
}

#[test]
fn test() {
    unsafe {
        let mut v = vec![64; 0];
        let m: jobject = v.as_mut_ptr().cast();
        let n = jtypes!(1i32, 2i32, 3i32, 4f64, m);
        assert_eq!(n[0].int(), 1);
        assert_eq!(n[1].int(), 2);
        assert_eq!(n[2].int(), 3);
        assert_eq!(n[3].double(), 4f64);
        assert_eq!(n[4].object(), m);

        fake_jni_call(jtypes!(1i32, 2i32, 3i32, 4f64, m).as_ptr())
    }
}
