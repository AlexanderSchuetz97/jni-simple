#[cfg(feature = "loadjvm")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::ptr::null_mut;
    use std::sync::Mutex;

    //Cargo runs the tests on different threads.
    static MUTEX: Mutex<()> = Mutex::new(());

    unsafe fn get_env() -> JNIEnv {
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

    unsafe fn get_test_class() -> jclass {
        let env = get_env();
        let class_loaded = env.FindClass("FieldTests");
        if !class_loaded.is_null() {
            let class_global = env.NewGlobalRef(class_loaded);
            env.DeleteLocalRef(class_loaded);
            return class_global;
        }

        env.ExceptionClear(); //Clear ClassNotFoundException
        let class_blob = include_bytes!("../java_testcode/FieldTests.class");
        let class_loaded = env.DefineClass_from_slice("FieldTests", null_mut(), class_blob);
        if class_loaded.is_null() {
            env.ExceptionDescribe();
            env.FatalError("failed to load class");
        }

        let class_global = env.NewGlobalRef(class_loaded);
        env.DeleteLocalRef(class_loaded);
        class_global
    }
    unsafe fn get_test_obj() -> jobject {
        let env = get_env();
        let tc = get_test_class();
        let local_obj = env.GetStaticFieldID(tc, "staticInstance", "LFieldTests;");
        let test_obj = env.GetStaticObjectField(tc, local_obj);
        test_obj
    }
    unsafe fn reset_it() {
        let env = get_env();
        let class = get_test_class();
        let reset = env.GetStaticMethodID(class, "reset", "()V");
        env.CallStaticVoidMethod0(class, reset);
        env.DeleteGlobalRef(class);
    }
    //unsafe fn dump_it() {
    //    let env = get_env();
    //    let class = get_test_class();
    //    let reset = env.GetStaticMethodID(class, "dump", "()V");
    //    env.CallStaticVoidMethod0(class, reset);
    //    env.DeleteGlobalRef(class);
    //}

    unsafe fn add_it() {
        let env = get_env();
        let class = get_test_class();
        let reset = env.GetStaticMethodID(class, "add", "()V");
        env.CallStaticVoidMethod0(class, reset);
        env.DeleteGlobalRef(class);
    }
    unsafe fn new_global_obj() -> jobject {
        let env = get_env();
        let class = env.FindClass("java/lang/Object");
        let meth = env.GetMethodID(class, "<init>", "()V");
        let obj = env.NewObjectA(class, meth, null_mut());
        let gref = env.NewGlobalRef(obj);
        env.DeleteLocalRef(obj);
        env.DeleteLocalRef(class);
        gref
    }

    #[test]
    fn test_bool() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticBool", "Z");
            env.SetStaticBooleanField(test_class, field, true);
            assert_eq!(true, env.GetStaticBooleanField(test_class, field));
            env.SetStaticBooleanField(test_class, field, false);
            assert_eq!(false, env.GetStaticBooleanField(test_class, field));
            env.SetStaticBooleanField(test_class, field, false);
            assert_eq!(false, env.GetStaticBooleanField(test_class, field));
            env.SetStaticBooleanField(test_class, field, true);
            env.SetStaticBooleanField(test_class, field, true);
            assert_eq!(true, env.GetStaticBooleanField(test_class, field));

            add_it();
            assert_eq!(false, env.GetStaticBooleanField(test_class, field));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynBool", "Z");
            env.SetBooleanField(test_obj, field, true);
            assert_eq!(true, env.GetBooleanField(test_obj, field));
            env.SetBooleanField(test_obj, field, false);
            assert_eq!(false, env.GetBooleanField(test_obj, field));
            env.SetBooleanField(test_obj, field, false);
            assert_eq!(false, env.GetBooleanField(test_obj, field));
            env.SetBooleanField(test_obj, field, true);
            env.SetBooleanField(test_obj, field, true);
            assert_eq!(true, env.GetBooleanField(test_obj, field));

            add_it();
            assert_eq!(false, env.GetBooleanField(test_obj, field));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
        }
    }

    #[test]
    fn test_byte() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticByte", "B");
            env.SetStaticByteField(test_class, field, 1);
            assert_eq!(1, env.GetStaticByteField(test_class, field));
            env.SetStaticByteField(test_class, field, 1);
            assert_eq!(1, env.GetStaticByteField(test_class, field));
            env.SetStaticByteField(test_class, field, 2);
            assert_eq!(2, env.GetStaticByteField(test_class, field));
            env.SetStaticByteField(test_class, field, 11);
            env.SetStaticByteField(test_class, field, 11);
            assert_eq!(11, env.GetStaticByteField(test_class, field));

            add_it();
            assert_eq!(12, env.GetStaticByteField(test_class, field));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynByte", "B");
            env.SetByteField(test_obj, field, 1);
            assert_eq!(1, env.GetByteField(test_obj, field));
            env.SetByteField(test_obj, field, 1);
            assert_eq!(1, env.GetByteField(test_obj, field));
            env.SetByteField(test_obj, field, 2);
            assert_eq!(2, env.GetByteField(test_obj, field));
            env.SetByteField(test_obj, field, 11);
            env.SetByteField(test_obj, field, 11);
            assert_eq!(11, env.GetByteField(test_obj, field));

            add_it();
            assert_eq!(12, env.GetByteField(test_obj, field));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
        }
    }

    #[test]
    fn test_short() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticShort", "S");
            env.SetStaticShortField(test_class, field, 1);
            assert_eq!(1, env.GetStaticShortField(test_class, field));
            env.SetStaticShortField(test_class, field, 1);
            assert_eq!(1, env.GetStaticShortField(test_class, field));
            env.SetStaticShortField(test_class, field, 2);
            assert_eq!(2, env.GetStaticShortField(test_class, field));
            env.SetStaticShortField(test_class, field, 11);
            env.SetStaticShortField(test_class, field, 11);
            assert_eq!(11, env.GetStaticShortField(test_class, field));

            add_it();
            assert_eq!(12, env.GetStaticShortField(test_class, field));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynShort", "S");
            env.SetShortField(test_obj, field, 1);
            assert_eq!(1, env.GetShortField(test_obj, field));
            env.SetShortField(test_obj, field, 1);
            assert_eq!(1, env.GetShortField(test_obj, field));
            env.SetShortField(test_obj, field, 2);
            assert_eq!(2, env.GetShortField(test_obj, field));
            env.SetShortField(test_obj, field, 11);
            env.SetShortField(test_obj, field, 11);
            assert_eq!(11, env.GetShortField(test_obj, field));

            add_it();
            assert_eq!(12, env.GetShortField(test_obj, field));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
        }
    }

    #[test]
    fn test_char() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticChar", "C");
            env.SetStaticCharField(test_class, field, 1);
            assert_eq!(1, env.GetStaticCharField(test_class, field));
            env.SetStaticCharField(test_class, field, 1);
            assert_eq!(1, env.GetStaticCharField(test_class, field));
            env.SetStaticCharField(test_class, field, 2);
            assert_eq!(2, env.GetStaticCharField(test_class, field));
            env.SetStaticCharField(test_class, field, 11);
            env.SetStaticCharField(test_class, field, 11);
            assert_eq!(11, env.GetStaticCharField(test_class, field));

            add_it();
            assert_eq!(12, env.GetStaticCharField(test_class, field));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynChar", "C");
            env.SetCharField(test_obj, field, 1);
            assert_eq!(1, env.GetCharField(test_obj, field));
            env.SetCharField(test_obj, field, 1);
            assert_eq!(1, env.GetCharField(test_obj, field));
            env.SetCharField(test_obj, field, 2);
            assert_eq!(2, env.GetCharField(test_obj, field));
            env.SetCharField(test_obj, field, 11);
            env.SetCharField(test_obj, field, 11);
            assert_eq!(11, env.GetCharField(test_obj, field));

            add_it();
            assert_eq!(12, env.GetCharField(test_obj, field));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
        }
    }

    #[test]
    fn test_int() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticInt", "I");
            env.SetStaticIntField(test_class, field, 1);
            assert_eq!(1, env.GetStaticIntField(test_class, field));
            env.SetStaticIntField(test_class, field, 1);
            assert_eq!(1, env.GetStaticIntField(test_class, field));
            env.SetStaticIntField(test_class, field, 2);
            assert_eq!(2, env.GetStaticIntField(test_class, field));
            env.SetStaticIntField(test_class, field, 11);
            env.SetStaticIntField(test_class, field, 11);
            assert_eq!(11, env.GetStaticIntField(test_class, field));

            add_it();
            assert_eq!(12, env.GetStaticIntField(test_class, field));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynInt", "I");
            env.SetIntField(test_obj, field, 1);
            assert_eq!(1, env.GetIntField(test_obj, field));
            env.SetIntField(test_obj, field, 1);
            assert_eq!(1, env.GetIntField(test_obj, field));
            env.SetIntField(test_obj, field, 2);
            assert_eq!(2, env.GetIntField(test_obj, field));
            env.SetIntField(test_obj, field, 11);
            env.SetIntField(test_obj, field, 11);
            assert_eq!(11, env.GetIntField(test_obj, field));

            add_it();
            assert_eq!(12, env.GetIntField(test_obj, field));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
        }
    }

    #[test]
    fn test_long() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticLong", "J");
            env.SetStaticLongField(test_class, field, 1);
            assert_eq!(1, env.GetStaticLongField(test_class, field));
            env.SetStaticLongField(test_class, field, 1);
            assert_eq!(1, env.GetStaticLongField(test_class, field));
            env.SetStaticLongField(test_class, field, 2);
            assert_eq!(2, env.GetStaticLongField(test_class, field));
            env.SetStaticLongField(test_class, field, 11);
            env.SetStaticLongField(test_class, field, 11);
            assert_eq!(11, env.GetStaticLongField(test_class, field));

            add_it();
            assert_eq!(12, env.GetStaticLongField(test_class, field));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynLong", "J");
            env.SetLongField(test_obj, field, 1);
            assert_eq!(1, env.GetLongField(test_obj, field));
            env.SetLongField(test_obj, field, 1);
            assert_eq!(1, env.GetLongField(test_obj, field));
            env.SetLongField(test_obj, field, 2);
            assert_eq!(2, env.GetLongField(test_obj, field));
            env.SetLongField(test_obj, field, 11);
            env.SetLongField(test_obj, field, 11);
            assert_eq!(11, env.GetLongField(test_obj, field));

            add_it();
            assert_eq!(12, env.GetLongField(test_obj, field));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
        }
    }

    #[test]
    fn test_float() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticFloat", "F");
            env.SetStaticFloatField(test_class, field, 1f32);
            assert_eq!(1f32, env.GetStaticFloatField(test_class, field));
            env.SetStaticFloatField(test_class, field, 1f32);
            assert_eq!(1f32, env.GetStaticFloatField(test_class, field));
            env.SetStaticFloatField(test_class, field, 2f32);
            assert_eq!(2f32, env.GetStaticFloatField(test_class, field));
            env.SetStaticFloatField(test_class, field, 11f32);
            env.SetStaticFloatField(test_class, field, 11f32);
            assert_eq!(11f32, env.GetStaticFloatField(test_class, field));

            add_it();
            assert_eq!(12f32, env.GetStaticFloatField(test_class, field));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynFloat", "F");
            env.SetFloatField(test_obj, field, 1f32);
            assert_eq!(1f32, env.GetFloatField(test_obj, field));
            env.SetFloatField(test_obj, field, 1f32);
            assert_eq!(1f32, env.GetFloatField(test_obj, field));
            env.SetFloatField(test_obj, field, 2f32);
            assert_eq!(2f32, env.GetFloatField(test_obj, field));
            env.SetFloatField(test_obj, field, 11f32);
            env.SetFloatField(test_obj, field, 11f32);
            assert_eq!(11f32, env.GetFloatField(test_obj, field));

            add_it();
            assert_eq!(12f32, env.GetFloatField(test_obj, field));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
        }
    }

    #[test]
    fn test_double() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticDouble", "D");
            env.SetStaticDoubleField(test_class, field, 1f64);
            assert_eq!(1f64, env.GetStaticDoubleField(test_class, field));
            env.SetStaticDoubleField(test_class, field, 1f64);
            assert_eq!(1f64, env.GetStaticDoubleField(test_class, field));
            env.SetStaticDoubleField(test_class, field, 2f64);
            assert_eq!(2f64, env.GetStaticDoubleField(test_class, field));
            env.SetStaticDoubleField(test_class, field, 11f64);
            env.SetStaticDoubleField(test_class, field, 11f64);
            assert_eq!(11f64, env.GetStaticDoubleField(test_class, field));

            add_it();
            assert_eq!(12f64, env.GetStaticDoubleField(test_class, field));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynDouble", "D");
            env.SetDoubleField(test_obj, field, 1f64);
            assert_eq!(1f64, env.GetDoubleField(test_obj, field));
            env.SetDoubleField(test_obj, field, 1f64);
            assert_eq!(1f64, env.GetDoubleField(test_obj, field));
            env.SetDoubleField(test_obj, field, 2f64);
            assert_eq!(2f64, env.GetDoubleField(test_obj, field));
            env.SetDoubleField(test_obj, field, 11f64);
            env.SetDoubleField(test_obj, field, 11f64);
            assert_eq!(11f64, env.GetDoubleField(test_obj, field));

            add_it();
            assert_eq!(12f64, env.GetDoubleField(test_obj, field));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
        }
    }

    #[test]
    fn test_obj() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let g1 = new_global_obj();
            let g2 = new_global_obj();

            let env = get_env();
            let test_class = get_test_class();
            let field = env.GetStaticFieldID(test_class, "staticObject", "Ljava/lang/Object;");
            env.SetStaticObjectField(test_class, field, g1);
            assert!(env.IsSameObject(g1, env.GetStaticObjectField(test_class, field)));
            env.SetStaticObjectField(test_class, field, g1);
            assert!(env.IsSameObject(g1, env.GetStaticObjectField(test_class, field)));
            env.SetStaticObjectField(test_class, field, null_mut());
            assert!(env.IsSameObject(null_mut(), env.GetStaticObjectField(test_class, field)));
            env.SetStaticObjectField(test_class, field, g2);
            env.SetStaticObjectField(test_class, field, g2);
            assert!(env.IsSameObject(g2, env.GetStaticObjectField(test_class, field)));

            let test_obj = get_test_obj();
            let field = env.GetFieldID(test_class, "dynObject", "Ljava/lang/Object;");
            env.SetObjectField(test_obj, field, g1);
            assert!(env.IsSameObject(g1, env.GetObjectField(test_obj, field)));
            env.SetObjectField(test_obj, field, g1);
            assert!(env.IsSameObject(g1, env.GetObjectField(test_obj, field)));
            env.SetObjectField(test_obj, field, null_mut());
            assert!(env.IsSameObject(null_mut(), env.GetObjectField(test_obj, field)));
            env.SetObjectField(test_obj, field, g2);
            env.SetObjectField(test_obj, field, g2);
            assert!(env.IsSameObject(g2, env.GetObjectField(test_obj, field)));

            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
            env.DeleteGlobalRef(g1);
            env.DeleteGlobalRef(g2);
        }
    }

    #[test]
    fn test_reflect() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let test_class = get_test_class();

            let field_static = env.GetStaticFieldID(test_class, "staticObject", "Ljava/lang/Object;");
            assert!(!field_static.is_null());

            let field_dyn = env.GetFieldID(test_class, "dynObject", "Ljava/lang/Object;");
            assert!(!field_dyn.is_null());

            let reflect_static = env.ToReflectedField(test_class, field_static, true);
            assert!(!reflect_static.is_null());

            let reflect_dyn = env.ToReflectedField(test_class, field_dyn, false);
            assert!(!reflect_dyn.is_null());

            let de_static = env.FromReflectedField(reflect_static);
            let de_dyn = env.FromReflectedField(reflect_dyn);

            let g1 = new_global_obj();
            let g2 = new_global_obj();

            env.SetStaticObjectField(test_class, de_static, g1);
            assert!(env.IsSameObject(g1, env.GetStaticObjectField(test_class, de_static)));
            env.SetStaticObjectField(test_class, de_static, g1);
            assert!(env.IsSameObject(g1, env.GetStaticObjectField(test_class, de_static)));
            env.SetStaticObjectField(test_class, de_static, null_mut());
            assert!(env.IsSameObject(null_mut(), env.GetStaticObjectField(test_class, de_static)));
            env.SetStaticObjectField(test_class, de_static, g2);
            env.SetStaticObjectField(test_class, de_static, g2);
            assert!(env.IsSameObject(g2, env.GetStaticObjectField(test_class, de_static)));

            let test_obj = get_test_obj();
            env.SetObjectField(test_obj, de_dyn, g1);
            assert!(env.IsSameObject(g1, env.GetObjectField(test_obj, de_dyn)));
            env.SetObjectField(test_obj, de_dyn, g1);
            assert!(env.IsSameObject(g1, env.GetObjectField(test_obj, de_dyn)));
            env.SetObjectField(test_obj, de_dyn, null_mut());
            assert!(env.IsSameObject(null_mut(), env.GetObjectField(test_obj, de_dyn)));
            env.SetObjectField(test_obj, de_dyn, g2);
            env.SetObjectField(test_obj, de_dyn, g2);
            assert!(env.IsSameObject(g2, env.GetObjectField(test_obj, de_dyn)));

            env.DeleteLocalRef(reflect_static);
            env.DeleteLocalRef(reflect_dyn);
            env.DeleteLocalRef(test_obj);
            env.DeleteGlobalRef(test_class);
            env.DeleteGlobalRef(g1);
            env.DeleteGlobalRef(g2);
        }
    }
}
