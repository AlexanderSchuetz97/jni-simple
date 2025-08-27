use jni_simple::{JvmtiError, jvmtiError};
use std::ffi::c_int;

#[test]
fn test_conversion() {
    for n in 0..1000 {
        let x = c_int::from(n);
        let m = JvmtiError::from(x);
        let n = jvmtiError::from(x);
        assert_eq!(x, m.into());
        assert_eq!(x, n.into());
        assert_eq!(m, n.into());
        assert_eq!(n, m.into());
    }
}
