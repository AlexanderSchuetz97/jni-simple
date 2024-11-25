#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::*;
    use std::ffi::c_char;
    use std::ptr::null_mut;
    use std::sync::Mutex;

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
    fn test_new_from_chars() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let some_chars = [
                'T' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16,
            ];
            let env = get_env();
            let str = env.NewString(some_chars.as_ptr(), some_chars.len() as jsize);
            let uw = env.GetStringUTFChars_as_string(str).unwrap();
            env.DeleteLocalRef(str);
            assert_eq!("Test String", uw.as_str());
        }
    }

    #[test]
    fn test_region() {
        let _lock = MUTEX.lock().unwrap();

        unsafe {
            let env = get_env();
            let str = env.NewStringUTF("Test String");
            let mut data = vec![0u16; env.GetStringLength(str) as usize];
            env.GetStringRegion(str, 0, data.len() as jsize, data.as_mut_ptr());
            env.DeleteLocalRef(str);
            assert_eq!(
                ['T' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16].as_slice(),
                data.as_slice()
            );
        }
    }

    #[test]
    fn test_region_utf() {
        let _lock = MUTEX.lock().unwrap();

        unsafe {
            let env = get_env();
            let str = env.NewStringUTF("Test String");
            let mut data = vec![1 as c_char; (env.GetStringUTFLength(str) as usize) + 2];
            env.GetStringUTFRegion(str, 0, (data.len() - 2) as jsize, data.as_mut_ptr());
            env.DeleteLocalRef(str);
            assert_eq!(
                [
                    'T' as c_char,
                    'e' as c_char,
                    's' as c_char,
                    't' as c_char,
                    ' ' as c_char,
                    'S' as c_char,
                    't' as c_char,
                    'r' as c_char,
                    'i' as c_char,
                    'n' as c_char,
                    'g' as c_char,
                    0 as c_char,
                    1 as c_char,
                ]
                .as_slice(),
                data.as_slice()
            );
        }
    }

    #[test]
    fn test_get_utf() {
        let _lock = MUTEX.lock().unwrap();

        unsafe {
            let env = get_env();
            let str = env.NewStringUTF("Test String");
            let cstr = env.GetStringUTFChars(str, null_mut());
            let sl = std::slice::from_raw_parts(cstr, (env.GetStringUTFLength(str) + 1) as usize);
            assert_eq!(
                [
                    'T' as c_char,
                    'e' as c_char,
                    's' as c_char,
                    't' as c_char,
                    ' ' as c_char,
                    'S' as c_char,
                    't' as c_char,
                    'r' as c_char,
                    'i' as c_char,
                    'n' as c_char,
                    'g' as c_char,
                    0
                ]
                .as_slice(),
                sl
            );
            env.ReleaseStringUTFChars(str, cstr);
            env.DeleteLocalRef(str);
        }
    }

    #[test]
    fn test_get() {
        let _lock = MUTEX.lock().unwrap();

        unsafe {
            let env = get_env();
            let str = env.NewStringUTF("Test String");
            let cstr = env.GetStringChars(str, null_mut());
            let sl = std::slice::from_raw_parts(cstr, (env.GetStringLength(str) + 1) as usize);
            assert_eq!(
                ['T' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 0].as_slice(),
                sl
            );
            env.ReleaseStringChars(str, cstr);
            env.DeleteLocalRef(str);
        }
    }

    #[test]
    fn test_crit_normal() {
        let _lock = MUTEX.lock().unwrap();

        unsafe {
            let env = get_env();
            let m = env.NewByteArray(16);
            let str = env.NewStringUTF("Test String");
            let l = env.GetStringLength(str);
            let n = env.GetStringCritical(str, null_mut());
            assert!(!n.is_null());
            let arr = env.GetPrimitiveArrayCritical(m, null_mut());
            assert!(!arr.is_null());
            let slice = std::slice::from_raw_parts(n, l as usize);
            assert_eq!(
                ['T' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16].as_slice(),
                slice
            );
            env.ReleaseStringCritical(str, n);
            env.ReleasePrimitiveArrayCritical(m, arr, JNI_OK);
            assert!(!env.ExceptionCheck());
            env.DeleteLocalRef(str);
            env.DeleteLocalRef(m);
        }
    }

    #[test]
    #[cfg(feature = "asserts")]
    fn test_crit_assert() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let array = env.NewStringUTF("Test String 1");
            assert!(!array.is_null());
            let array2 = env.NewStringUTF("Test String 2");
            assert!(!array2.is_null());
            let ptr = env.GetStringCritical(array, null_mut());
            assert!(!ptr.is_null());
            let ptr2 = env.GetStringCritical(array2, null_mut());
            assert!(!ptr2.is_null());
            let ptr3 = env.GetStringCritical(array2, null_mut());
            assert!(!ptr3.is_null());
            let result = std::panic::catch_unwind(|| {
                //Should panic because not allowed when 3 crit ptr is not released
                env.ExceptionCheck();
            });

            assert!(result.is_err(), "No panic occurred");
            env.ReleaseStringCritical(array, ptr);
            let result = std::panic::catch_unwind(|| {
                //Should panic because not allowed when 2 crit ptr is not released
                env.ExceptionClear();
            });
            assert!(result.is_err(), "No panic occurred");
            env.ReleaseStringCritical(array, ptr2);

            let result = std::panic::catch_unwind(|| {
                //Should panic because not allowed when 1 crit ptr is not released
                env.ExceptionDescribe();
            });
            assert!(result.is_err(), "No panic occurred");
            env.ReleaseStringCritical(array, ptr3);

            //Now we shouldnt panic
            assert_eq!(false, env.ExceptionCheck());

            let result = std::panic::catch_unwind(|| {
                //Should panic because double free
                env.ReleaseStringCritical(array, ptr3);
            });
            assert!(result.is_err(), "No panic occurred");
            env.DeleteLocalRef(array);
            env.DeleteLocalRef(array2);
        }
    }
}
