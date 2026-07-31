#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::ffi::c_char;
    use std::ptr::null_mut;
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
                let (_, env) = JNI_CreateJavaVM_with_string_args::<&str>(JNI_VERSION_1_8, &[], false).expect("failed to create jvm");
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
                [
                    'T' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16
                ]
                .as_slice(),
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
                [
                    'T' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16, 0
                ]
                .as_slice(),
                sl
            );
            env.ReleaseStringChars(str, cstr);
            env.DeleteLocalRef(str);
        }
    }

    #[test]
    fn test_get_utf_len_j24() {
        let _lock = MUTEX.lock().unwrap();

        unsafe {
            let env = get_env();
            if env.GetVersion() < JNI_VERSION_24 {
                //CANT TEST THIS WITH THIS JDK
                return;
            }
            let str = env.NewStringUTF("Test String");
            let utf_len = env.GetStringUTFLengthAsLong(str);
            env.DeleteLocalRef(str);
            assert_eq!(utf_len, 11);
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
                [
                    'T' as u16, 'e' as u16, 's' as u16, 't' as u16, ' ' as u16, 'S' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'n' as u16, 'g' as u16
                ]
                .as_slice(),
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
                let _ = env.ExceptionCheck();
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

    #[test]
    fn test_supplementary_character() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let my_string = "abc𝕊abc"; //U+1D54A MATHEMATICAL DOUBLE-STRUCK CAPITAL S
            let java_string: jstring = env.NewString_from_str(my_string);
            assert!(!java_string.is_null());
            let and_back_again: String = env.GetStringChars_as_string(java_string).unwrap();
            assert_eq!(my_string, and_back_again.as_str());
            assert_eq!(None, env.GetStringUTFChars_as_string(java_string));

            let unhappy_string = env.NewStringUTF(my_string);
            assert!(!unhappy_string.is_null());
            let and_back_again: String = env.GetStringChars_as_string(unhappy_string).unwrap();
            assert_eq!("abcð\u{9d}\u{95}\u{8a}", and_back_again.as_str()); //This is different because 𝕊 is a supplementary character.
        }
    }

    #[test]
    #[cfg(not(feature = "asserts"))] //The assertions would panic on this test.
    fn test_illegal() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let invalid_string = [b'a', 0b1100_1111, b'a', 0b1001_1111, b'a', 0u8];
            let java_string: jstring = env.NewStringUTF(invalid_string.as_ptr());
            assert!(!java_string.is_null());
            let and_back_again: String = env.GetStringChars_as_string(java_string).unwrap();
            assert_eq!("aÏa\u{9f}", and_back_again.as_str());
        }
    }

    #[test]
    fn test_into_slice_jchar() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let exc_cl = env.FindClass("java/lang/StringIndexOutOfBoundsException");
            assert!(!exc_cl.is_null());

            let java_string: jstring = env.NewStringUTF("ÄÖÜ");
            assert!(!java_string.is_null());
            let mut utf_16_slice = [0u16; 3];
            env.GetStringRegion_into_slice(java_string, 0, &mut utf_16_slice);
            assert!(!env.ExceptionCheck());

            let rust_string_from_slice = String::from_utf16_lossy(&utf_16_slice);
            assert_eq!("ÄÖÜ", rust_string_from_slice.as_str());

            let mut utf_16_slice = [0u16; 2];
            env.GetStringRegion_into_slice(java_string, 1, &mut utf_16_slice);
            assert!(!env.ExceptionCheck());

            let rust_string_from_slice = String::from_utf16_lossy(&utf_16_slice);
            assert_eq!("ÖÜ", rust_string_from_slice.as_str());

            let mut utf_16_slice = [0u16; 2];
            env.GetStringRegion_into_slice(java_string, 0, &mut utf_16_slice);
            assert!(!env.ExceptionCheck());

            let rust_string_from_slice = String::from_utf16_lossy(&utf_16_slice);
            assert_eq!("ÄÖ", rust_string_from_slice.as_str());

            let mut utf_16_slice = [0u16; 2];
            env.GetStringRegion_into_slice(java_string, 2, &mut utf_16_slice);
            assert!(env.ExceptionCheck());
            let thr = env.ExceptionOccurred();
            assert!(!thr.is_null());
            env.ExceptionClear();
            assert!(env.IsInstanceOf(thr, exc_cl));
            env.DeleteLocalRef(thr);

            let mut utf_16_slice = [0u16; 2];
            env.GetStringRegion_into_slice(java_string, 3, &mut utf_16_slice);
            assert!(env.ExceptionCheck());
            let thr = env.ExceptionOccurred();
            assert!(!thr.is_null());
            env.ExceptionClear();
            assert!(env.IsInstanceOf(thr, exc_cl));
            env.DeleteLocalRef(thr);

            let mut utf_16_slice = [0u16; 0];
            env.GetStringRegion_into_slice(java_string, 4, &mut utf_16_slice);
            assert!(env.ExceptionCheck());
            let thr = env.ExceptionOccurred();
            assert!(!thr.is_null());
            env.ExceptionClear();
            assert!(env.IsInstanceOf(thr, exc_cl));
            env.DeleteLocalRef(thr);

            let mut utf_16_slice = [0u16; 0];
            env.GetStringRegion_into_slice(java_string, 3, &mut utf_16_slice);
            assert!(!env.ExceptionCheck());

            //MUST REMAIN HERE, CLEANUP
            env.DeleteLocalRef(exc_cl);
        }
    }
}
