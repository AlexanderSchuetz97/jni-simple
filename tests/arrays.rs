#[cfg(feature = "loadjvm")]
pub mod test {
    use std::ptr::{null_mut};
    use std::sync::Mutex;
    use jni_simple::*;


    //Cargo runs the tests on different threads.
    static MUTEX: Mutex<()> = Mutex::new(());

    unsafe fn get_env() -> JNIEnv {
        if !is_jvm_loaded() {
            load_jvm_from_java_home().expect("failed to load jvm");
        }

        let thr = JNI_GetCreatedJavaVMs().expect("failed to get jvm");
        if thr.is_empty() {
            //let args: Vec<String> = vec!["-Xcheck:jni".to_string()];
            //let args: Vec<String> = vec!["-Xint".to_string()];
            let args: Vec<String> = vec![];

            let (_, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create jvm");
            return env;
        }

        let jvm = thr.first().unwrap().clone();
        let env = jvm.GetEnv(JNI_VERSION_1_8);
        let env = env.unwrap_or_else(|c| {
            if c != JNI_EDETACHED {
                panic!("JVM ERROR {}", c);
            }

            jvm.AttachCurrentThread_str(JNI_VERSION_1_8, None, null_mut()).expect("failed to attach thread")
        });

        env
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
                let critical_slice : &mut [i8] = std::slice::from_raw_parts_mut(ptr.cast(), 512);

                for i in 0usize .. 512 {
                    assert_eq!(critical_slice[i], 0); //JVM guarantees zeroed mem.
                    critical_slice[i] = i as i8;
                }
            }

            env.ReleasePrimitiveArrayCritical(array, ptr, JNI_OK);

            let mut rust_buf = [0i8; 512];
            env.GetByteArrayRegion(array, 0, 512, rust_buf.as_mut_ptr());
            for i in 0usize .. 512 {
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
            let array2 =  env.NewByteArray(512);
            assert!(!array2.is_null());
            let ptr = env.GetPrimitiveArrayCritical(array, null_mut());
            assert!(!ptr.is_null());
            let ptr2 = env.GetPrimitiveArrayCritical(array2, null_mut());
            assert!(!ptr2.is_null());
            let ptr3 = env.GetPrimitiveArrayCritical(array2, null_mut());
            assert!(!ptr3.is_null());
            let result = std::panic::catch_unwind(|| {
                //Should panic because not allowed when 3 crit ptr is not released
                env.ExceptionCheck();
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
}