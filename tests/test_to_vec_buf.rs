#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
mod test {
    use jni_simple::{
        JNI_CreateJavaVM_with_string_args, JNI_VERSION_1_8, JNIEnv, jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort, jsize, load_jvm_from_java_home,
    };
    use std::fmt::Debug;

    fn assert_aiob(env: JNIEnv) {
        unsafe {
            assert!(env.ExceptionCheck());
            let occ = env.ExceptionOccurred();
            assert!(!occ.is_null());
            env.ExceptionClear();
            let exc_class = env.GetObjectClass(occ);
            assert!(!exc_class.is_null());
            env.DeleteLocalRef(occ);
            let ar_cl = env.FindClass("java/lang/ArrayIndexOutOfBoundsException");
            assert!(!ar_cl.is_null());
            assert!(env.IsSameObject(ar_cl, exc_class));
            env.DeleteLocalRef(ar_cl);
            env.DeleteLocalRef(exc_class);
        }
    }

    trait FromNum {
        fn from(by: jbyte) -> Self;
    }

    impl FromNum for jshort {
        fn from(by: jbyte) -> Self {
            by as _
        }
    }

    impl FromNum for jbyte {
        fn from(by: jbyte) -> Self {
            by as _
        }
    }

    impl FromNum for jchar {
        fn from(by: jbyte) -> Self {
            by as _
        }
    }

    impl FromNum for jint {
        fn from(by: jbyte) -> Self {
            by as _
        }
    }

    impl FromNum for jlong {
        fn from(by: jbyte) -> Self {
            by as _
        }
    }

    impl FromNum for jfloat {
        fn from(by: jbyte) -> Self {
            by as _
        }
    }

    impl FromNum for jdouble {
        fn from(by: jbyte) -> Self {
            by as _
        }
    }

    impl FromNum for jboolean {
        fn from(by: jbyte) -> Self {
            by & 1 == 0
        }
    }

    fn run_test<T: FromNum + PartialEq + Debug>(env: JNIEnv, constructor: impl FnOnce(JNIEnv, &[T]) -> jobject, getter: impl Fn(JNIEnv, jobject, jsize, Option<jsize>) -> Vec<T>) {
        unsafe {
            let data = &[
                T::from(0),
                T::from(1),
                T::from(2),
                T::from(3),
                T::from(4),
                T::from(5),
                T::from(6),
                T::from(7),
                T::from(8),
                T::from(9),
                T::from(10),
                T::from(11),
                T::from(12),
                T::from(13),
                T::from(14),
                T::from(15),
            ];
            let under_test = constructor(env, data);
            assert!(!env.ExceptionCheck());

            let copy = getter(env, under_test, 0, None);
            assert_eq!(data, copy.as_slice());
            assert!(!env.ExceptionCheck());

            let copy = getter(env, under_test, 0, Some(1));
            assert_eq!(&[T::from(0)], copy.as_slice());
            assert!(!env.ExceptionCheck());

            let copy = getter(env, under_test, 4, Some(4));
            assert_eq!(&[T::from(4), T::from(5), T::from(6), T::from(7)], copy.as_slice());
            assert!(!env.ExceptionCheck());

            let copy = getter(env, under_test, 16, None);
            assert!(copy.is_empty());
            assert!(!env.ExceptionCheck());

            let copy = getter(env, under_test, 13, Some(0));
            assert!(copy.is_empty());
            assert!(!env.ExceptionCheck());

            let copy = getter(env, under_test, 16, Some(-5));
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, jsize::MIN, None);
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, jsize::MAX, None);
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, -1, Some(4));
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, -1, None);
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, 69, None);
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, 15, Some(5));
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, 16, Some(1));
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, 13, Some(-1));
            assert!(copy.is_empty());
            assert_aiob(env);

            let copy = getter(env, under_test, 69, Some(-1));
            assert!(copy.is_empty());
            assert_aiob(env);

            env.DeleteLocalRef(under_test);
        }
    }

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("FAILED TO LOAD JVM");
            let (_, env) = JNI_CreateJavaVM_with_string_args::<&str>(JNI_VERSION_1_8, &[], true).expect("Failed to start JVM");
            run_test::<jboolean>(
                env,
                |env, data| {
                    let array = env.NewBooleanArray(data.len() as jsize);
                    assert!(!array.is_null());
                    env.SetBooleanArrayRegion_from_slice(array, 0, data);
                    array
                },
                |env, array, start, len| env.GetBooleanArrayRegion_as_vec(array, start, len),
            );
            run_test::<jshort>(
                env,
                |env, data| {
                    let array = env.NewShortArray(data.len() as jsize);
                    assert!(!array.is_null());
                    env.SetShortArrayRegion_from_slice(array, 0, data);
                    array
                },
                |env, array, start, len| env.GetShortArrayRegion_as_vec(array, start, len),
            );
            run_test::<jint>(
                env,
                |env, data| {
                    let array = env.NewIntArray(data.len() as jsize);
                    assert!(!array.is_null());
                    env.SetIntArrayRegion_from_slice(array, 0, data);
                    array
                },
                |env, array, start, len| env.GetIntArrayRegion_as_vec(array, start, len),
            );
            run_test::<jlong>(
                env,
                |env, data| {
                    let array = env.NewLongArray(data.len() as jsize);
                    assert!(!array.is_null());
                    env.SetLongArrayRegion_from_slice(array, 0, data);
                    array
                },
                |env, array, start, len| env.GetLongArrayRegion_as_vec(array, start, len),
            );

            run_test::<jchar>(
                env,
                |env, data| {
                    let array = env.NewCharArray(data.len() as jsize);
                    assert!(!array.is_null());
                    env.SetCharArrayRegion_from_slice(array, 0, data);
                    array
                },
                |env, array, start, len| env.GetCharArrayRegion_as_vec(array, start, len),
            );

            run_test::<jbyte>(
                env,
                |env, data| {
                    let array = env.NewByteArray(data.len() as jsize);
                    assert!(!array.is_null());
                    env.SetByteArrayRegion_from_slice(array, 0, data);
                    array
                },
                |env, array, start, len| env.GetByteArrayRegion_as_vec(array, start, len),
            );

            run_test::<jfloat>(
                env,
                |env, data| {
                    let array = env.NewFloatArray(data.len() as jsize);
                    assert!(!array.is_null());
                    env.SetFloatArrayRegion_from_slice(array, 0, data);
                    array
                },
                |env, array, start, len| env.GetFloatArrayRegion_as_vec(array, start, len),
            );
            run_test::<jdouble>(
                env,
                |env, data| {
                    let array = env.NewDoubleArray(data.len() as jsize);
                    assert!(!array.is_null());
                    env.SetDoubleArrayRegion_from_slice(array, 0, data);
                    array
                },
                |env, array, start, len| env.GetDoubleArrayRegion_as_vec(array, start, len),
            );
        }
    }
}
