#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::fmt::Debug;
    use std::ptr::null_mut;
    use std::slice;
    use std::sync::Mutex;

    //Cargo runs the tests on different threads.
    static MUTEX: Mutex<()> = Mutex::new(());

    unsafe fn get_env() -> JNIEnv {
        unsafe {
            if !is_jvm_loaded() {
                load_jvm_from_java_home().expect("failed to load jvm");
            }

            let thr = JNI_GetCreatedJavaVMs_first().expect("failed to get jvm");
            if thr.is_none() {
                //let args: Vec<String> = vec!["-Xcheck:jni".to_string()];
                //let args: Vec<String> = vec!["-Xint".to_string()];
                let args: Vec<String> = vec![];

                let (_, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create jvm");
                return env;
            }

            let jvm = thr.unwrap().clone();
            let env = jvm.GetEnv(JNI_VERSION_1_8);
            let env = env.unwrap_or_else(|c| {
                if c != JNI_EDETACHED {
                    panic!("JVM ERROR {}", c);
                }

                jvm.AttachCurrentThread_str(JNI_VERSION_1_8, (), null_mut()).expect("failed to attach thread")
            });

            env
        }
    }

    #[test]
    fn test_crit_normal() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let array = env.NewByteArray(512);
            assert!(!array.is_null());
            let ptr = env.GetPrimitiveArrayCritical(array, null_mut());
            assert!(!ptr.is_null());
            {
                let critical_slice: &mut [i8] = std::slice::from_raw_parts_mut(ptr.cast(), 512);

                for i in 0usize..512 {
                    assert_eq!(critical_slice[i], 0); //JVM guarantees zeroed mem.
                    critical_slice[i] = i as i8;
                }
            }

            env.ReleasePrimitiveArrayCritical(array, ptr, JNI_OK);

            let mut rust_buf = [0i8; 512];
            env.GetByteArrayRegion(array, 0, 512, rust_buf.as_mut_ptr());
            for i in 0usize..512 {
                assert_eq!(rust_buf[i], i as i8);
            }
        }
    }

    #[test]
    #[cfg(feature = "asserts")]
    fn test_crit_assert() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let array = env.NewByteArray(512);
            assert!(!array.is_null());
            let array2 = env.NewByteArray(512);
            assert!(!array2.is_null());
            let ptr = env.GetPrimitiveArrayCritical(array, null_mut());
            assert!(!ptr.is_null());
            let ptr2 = env.GetPrimitiveArrayCritical(array2, null_mut());
            assert!(!ptr2.is_null());
            let ptr3 = env.GetPrimitiveArrayCritical(array2, null_mut());
            assert!(!ptr3.is_null());
            let result = std::panic::catch_unwind(|| {
                //Should panic because not allowed when 3 crit ptr is not released
                let _ = env.ExceptionCheck();
            });

            assert!(result.is_err(), "No panic occurred");
            env.ReleasePrimitiveArrayCritical(array, ptr, JNI_OK);
            let result = std::panic::catch_unwind(|| {
                //Should panic because not allowed when 2 crit ptr is not released
                env.ExceptionClear();
            });
            assert!(result.is_err(), "No panic occurred");
            env.ReleasePrimitiveArrayCritical(array, ptr2, JNI_OK);

            let result = std::panic::catch_unwind(|| {
                //Should panic because not allowed when 1 crit ptr is not released
                env.ExceptionDescribe();
            });
            assert!(result.is_err(), "No panic occurred");
            env.ReleasePrimitiveArrayCritical(array, ptr3, JNI_OK);

            //Now we shouldnt panic
            assert_eq!(false, env.ExceptionCheck());

            let result = std::panic::catch_unwind(|| {
                //Should panic because double free
                env.ReleasePrimitiveArrayCritical(array, ptr3, JNI_OK);
            });
            assert!(result.is_err(), "No panic occurred");
        }
    }

    fn run_array_test<
        T: Default + Copy + PartialEq + Debug,
        Conv: Fn(usize) -> T,
        NewFunc: Fn(&JNIEnv, jsize) -> jarray,
        RegionFunc: Fn(&JNIEnv, jarray, jsize, jsize, *mut T),
        RegionFunc2: Fn(&JNIEnv, jarray, jsize, jsize, *mut T),
        GetEleFunc: Fn(&JNIEnv, jarray, *mut jboolean) -> *mut T,
        ReleaseEleFunc: Fn(&JNIEnv, jarray, *mut T, jint),
    >(
        conv: Conv,
        new_func: NewFunc,
        get_region_func: RegionFunc,
        set_region_func: RegionFunc2,
        get_ele_func: GetEleFunc,
        release_ele_func: ReleaseEleFunc,
    ) {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let array = new_func(&env, 512);
            let arr = env.GetArrayLength(array);
            assert_eq!(arr, 512);

            let mut g = [T::default(); 512];
            get_region_func(&env, array, 0, 512, g.as_mut_ptr());

            for x in 0usize..512 {
                assert_eq!(g[x], conv(0));
                g[x] = conv(x);
            }

            set_region_func(&env, array, 0, 512, g.as_mut_ptr());

            for x in 0usize..512 {
                assert_eq!(g[x], conv(x));
            }

            let mut g = [T::default(); 512];

            get_region_func(&env, array, 0, 256, g.as_mut_ptr());

            for x in 0usize..256 {
                assert_eq!(g[x], conv(x));
                g[x] = conv(x + 1);
            }

            for x in 256usize..512 {
                assert_eq!(g[x], conv(0));
            }

            get_region_func(&env, array, 256, 256, g.as_mut_ptr().add(256));

            for x in 0usize..256 {
                assert_eq!(g[x], conv(x + 1));
            }

            for x in 256usize..512 {
                assert_eq!(g[x], conv(x));
            }

            let ele = get_ele_func(&env, array, null_mut());
            let g = slice::from_raw_parts_mut(ele, 512);
            for x in 0usize..512 {
                assert_eq!(g[x], conv(x));
                g[x] = conv(x + 1)
            }

            release_ele_func(&env, array, ele, JNI_ABORT);
            let ele = get_ele_func(&env, array, null_mut());
            let g = slice::from_raw_parts_mut(ele, 512);
            for x in 0usize..512 {
                assert_eq!(g[x], conv(x));
                g[x] = conv(x + 2);
            }
            release_ele_func(&env, array, ele, JNI_OK);
            let mut g = [T::default(); 512];

            get_region_func(&env, array, 0, 512, g.as_mut_ptr());
            for x in 0usize..512 {
                assert_eq!(g[x], conv(x + 2));
            }
        }
    }
    #[test]
    fn test_short_array() {
        run_array_test(
            |x: usize| x as i16,
            |env, size| unsafe {
                return env.NewShortArray(size);
            },
            |env, array, from, to, copy| unsafe {
                env.GetShortArrayRegion(array, from, to, copy);
            },
            |env, array, from, to, copy| unsafe {
                env.SetShortArrayRegion(array, from, to, copy);
            },
            |env, array, copy| unsafe {
                return env.GetShortArrayElements(array, copy);
            },
            |env, array, elements, mode| unsafe {
                return env.ReleaseShortArrayElements(array, elements, mode);
            },
        );
    }

    #[test]
    fn test_byte_array() {
        run_array_test(
            |x: usize| x as i8,
            |env, size| unsafe {
                return env.NewByteArray(size);
            },
            |env, array, from, to, copy| unsafe {
                env.GetByteArrayRegion(array, from, to, copy);
            },
            |env, array, from, to, copy| unsafe {
                env.SetByteArrayRegion(array, from, to, copy);
            },
            |env, array, copy| unsafe {
                return env.GetByteArrayElements(array, copy);
            },
            |env, array, elements, mode| unsafe {
                return env.ReleaseByteArrayElements(array, elements, mode);
            },
        );
    }

    #[test]
    fn test_char_array() {
        run_array_test(
            |x: usize| x as u16,
            |env, size| unsafe {
                return env.NewCharArray(size);
            },
            |env, array, from, to, copy| unsafe {
                env.GetCharArrayRegion(array, from, to, copy);
            },
            |env, array, from, to, copy| unsafe {
                env.SetCharArrayRegion(array, from, to, copy);
            },
            |env, array, copy| unsafe {
                return env.GetCharArrayElements(array, copy);
            },
            |env, array, elements, mode| unsafe {
                return env.ReleaseCharArrayElements(array, elements, mode);
            },
        );
    }

    #[test]
    fn test_int_array() {
        run_array_test(
            |x: usize| x as i32,
            |env, size| unsafe {
                return env.NewIntArray(size);
            },
            |env, array, from, to, copy| unsafe {
                env.GetIntArrayRegion(array, from, to, copy);
            },
            |env, array, from, to, copy| unsafe {
                env.SetIntArrayRegion(array, from, to, copy);
            },
            |env, array, copy| unsafe {
                return env.GetIntArrayElements(array, copy);
            },
            |env, array, elements, mode| unsafe {
                return env.ReleaseIntArrayElements(array, elements, mode);
            },
        );
    }

    #[test]
    fn test_long_array() {
        run_array_test(
            |x: usize| x as i64,
            |env, size| unsafe {
                return env.NewLongArray(size);
            },
            |env, array, from, to, copy| unsafe {
                env.GetLongArrayRegion(array, from, to, copy);
            },
            |env, array, from, to, copy| unsafe {
                env.SetLongArrayRegion(array, from, to, copy);
            },
            |env, array, copy| unsafe {
                return env.GetLongArrayElements(array, copy);
            },
            |env, array, elements, mode| unsafe {
                return env.ReleaseLongArrayElements(array, elements, mode);
            },
        );
    }

    #[test]
    fn test_float_array() {
        run_array_test(
            |x: usize| x as f32,
            |env, size| unsafe {
                return env.NewFloatArray(size);
            },
            |env, array, from, to, copy| unsafe {
                env.GetFloatArrayRegion(array, from, to, copy);
            },
            |env, array, from, to, copy| unsafe {
                env.SetFloatArrayRegion(array, from, to, copy);
            },
            |env, array, copy| unsafe {
                return env.GetFloatArrayElements(array, copy);
            },
            |env, array, elements, mode| unsafe {
                return env.ReleaseFloatArrayElements(array, elements, mode);
            },
        );
    }

    #[test]
    fn test_double_array() {
        run_array_test(
            |x: usize| x as f64,
            |env, size| unsafe {
                return env.NewDoubleArray(size);
            },
            |env, array, from, to, copy| unsafe {
                env.GetDoubleArrayRegion(array, from, to, copy);
            },
            |env, array, from, to, copy| unsafe {
                env.SetDoubleArrayRegion(array, from, to, copy);
            },
            |env, array, copy| unsafe {
                return env.GetDoubleArrayElements(array, copy);
            },
            |env, array, elements, mode| unsafe {
                return env.ReleaseDoubleArrayElements(array, elements, mode);
            },
        );
    }

    #[test]
    fn test_boolean_array() {
        run_array_test(
            |x: usize| x % 2 == 1,
            |env, size| unsafe {
                return env.NewBooleanArray(size);
            },
            |env, array, from, to, copy| unsafe {
                env.GetBooleanArrayRegion(array, from, to, copy);
            },
            |env, array, from, to, copy| unsafe {
                env.SetBooleanArrayRegion(array, from, to, copy);
            },
            |env, array, copy| unsafe {
                return env.GetBooleanArrayElements(array, copy);
            },
            |env, array, elements, mode| unsafe {
                return env.ReleaseBooleanArrayElements(array, elements, mode);
            },
        );
    }

    #[test]
    fn test_object_array() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let str_class = env.FindClass("java/lang/String");
            let array = env.NewObjectArray(16, str_class, null_mut());
            for x in 0..16 {
                let element = env.GetObjectArrayElement(array, x);
                assert!(element.is_null());
                let nstr = env.NewStringUTF(format!("{}", x).as_str());
                env.SetObjectArrayElement(array, x, nstr);
                env.DeleteLocalRef(nstr);
            }
            for x in 0..16 {
                let element = env.GetObjectArrayElement(array, x);
                assert!(!element.is_null());
                let rstring = env.GetStringUTFChars_as_string(element).unwrap();
                env.DeleteLocalRef(element);
                assert_eq!(format!("{}", x), rstring);
            }

            env.DeleteLocalRef(array);
        }
    }
}
