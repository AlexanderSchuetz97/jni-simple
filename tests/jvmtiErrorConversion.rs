use jni_simple::{JVMTI_ERROR_NONE, JvmtiError, jvmtiError};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::ffi::c_int;

#[test]
fn test_conversion() {
    let mut hs1 = HashSet::new();
    let mut hs2 = HashSet::new();

    let mut bs1 = BTreeSet::new();
    let mut bs2 = BTreeSet::new();

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
            assert!(m.into_result().is_ok());
        } else {
            assert!(m.is_err());
            assert!(n.is_err());
            assert!(n.into_result().is_err());
            assert!(m.into_result().is_err());
        }

        let f1 = m.to_string();
        let f2 = n.to_string();
        assert_eq!(f1, f2);

        assert!(!hs1.contains(&m));
        assert!(!hs2.contains(&n));
        assert!(hs1.insert(m));
        assert!(hs2.insert(n));
        assert!(hs1.contains(&m));
        assert!(hs2.contains(&n));

        assert!(!bs1.contains(&m));
        assert!(!bs2.contains(&n));
        assert!(bs1.insert(m));
        assert!(bs2.insert(n));
        assert!(bs1.contains(&m));
        assert!(bs2.contains(&n));

        assert_eq!(m.partial_cmp(&m), Some(Ordering::Equal));
        assert_eq!(n.partial_cmp(&n), Some(Ordering::Equal));
        let xg = c_int::from(x + 1);
        let mg = JvmtiError::from(xg);
        let ng = jvmtiError::from(xg);
        assert_eq!(m.partial_cmp(&mg), Some(Ordering::Less));
        assert_eq!(n.partial_cmp(&ng), Some(Ordering::Less));

        if x > 0 {
            let xl = c_int::from(x - 1);
            let ml = JvmtiError::from(xl);
            let nl = jvmtiError::from(xl);
            assert_eq!(m.partial_cmp(&ml), Some(Ordering::Greater));
            assert_eq!(n.partial_cmp(&nl), Some(Ordering::Greater));
        }
    }

    for (idx, n) in bs1.iter().enumerate() {
        let n: c_int = n.into();
        assert_eq!(idx as c_int, n);
    }

    for (idx, n) in bs2.iter().enumerate() {
        let n: c_int = n.into();
        assert_eq!(idx as c_int, n);
    }
}
