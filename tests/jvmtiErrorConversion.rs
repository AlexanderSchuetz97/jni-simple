use jni_simple::{JVMTI_ERROR_NONE, JvmtiError, jvmtiError};
use std::ffi::c_int;

#[test]
fn test_conversion() {
    for x in 0..1000 {
        let x = c_int::from(x);
        let m = JvmtiError::from(x);
        let n = jvmtiError::from(x);
        assert_eq!(x, m.into());
        assert_eq!(x, n.into());
        assert_eq!(m, n.into());
        assert_eq!(n, m.into());
        assert_eq!(m, n.into_enum());
        assert_eq!(x, n.into_raw());

        if n == JVMTI_ERROR_NONE {
            assert!(m.is_ok());
            assert!(n.is_ok());
            assert!(n.into_result().is_ok());
        } else {
            assert!(m.is_err());
            assert!(n.is_err());
            assert!(n.into_result().is_err());
        }
    }
}
