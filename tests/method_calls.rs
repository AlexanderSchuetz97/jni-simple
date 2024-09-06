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

    unsafe fn get_test_class() -> jclass {
        let env = get_env();
        let class_loaded = env.FindClass_str("MethodCalls");
        if !class_loaded.is_null() {
            let class_global = env.NewGlobalRef(class_loaded);
            env.DeleteLocalRef(class_loaded);
            return class_global;
        }

        env.ExceptionClear(); //Clear ClassNotFoundException
        let class_blob = include_bytes!("../java_testcode/MethodCalls.class");
        let class_loaded = env.DefineClass_str("MethodCalls", null_mut(), class_blob);
        if class_loaded.is_null() {
            env.ExceptionDescribe();
            env.FatalError_str("failed to load class");
        }

        let class_global = env.NewGlobalRef(class_loaded);
        env.DeleteLocalRef(class_loaded);
        return class_global;
    }

    unsafe fn reset_it() {
        let env = get_env();
        let class = get_test_class();
        let reset = env.GetStaticMethodID_str(class, "reset", "()V");
        env.CallStaticVoidMethod0(class, reset);
        env.DeleteGlobalRef(class);
    }

    unsafe fn assert_fn_name(name: &str) {
        let env = get_env();
        let class = get_test_class();
        let name_field = env.GetStaticFieldID_str(class, "name", "Ljava/lang/String;");
        let name_obj = env.GetStaticObjectField(class, name_field);
        if name_obj.is_null() {
            panic!("assert_fn_name expected {} got null", name);
        }
        let got = env.GetStringUTFChars_as_string(name_obj).expect("failed to get string");
        env.DeleteLocalRef(name_obj);
        env.DeleteGlobalRef(class);
        assert_eq!(name, got.as_str());
    }

    unsafe fn assert_a(v: i16) {
        let env = get_env();
        let class = get_test_class();
        let field = env.GetStaticFieldID_str(class, "a", "S");
        let value = env.GetStaticShortField(class, field);
        env.DeleteGlobalRef(class);
        assert_eq!(v, value);
    }

    unsafe fn new_global_obj() -> jobject {
        let env = get_env();
        let class = env.FindClass_str("java/lang/Object");
        let meth = env.GetMethodID_str(class, "<init>", "()V");
        let obj = env.NewObjectA(class, meth, null_mut());
        let gref = env.NewGlobalRef(obj);
        env.DeleteLocalRef(obj);
        env.DeleteLocalRef(class);
        gref
    }
    unsafe fn assert_b(v: jobject) {
        let env = get_env();
        let class = get_test_class();
        let field = env.GetStaticFieldID_str(class, "b", "Ljava/lang/Object;");
        let value = env.GetStaticObjectField(class, field);
        env.DeleteGlobalRef(class);
        if v.is_null() {
            assert!(value.is_null());
            return;
        }
        assert!(!value.is_null());
        assert!(env.IsSameObject(v, value));
        env.DeleteLocalRef(value);
    }

    unsafe fn assert_c(v: std::ffi::c_double) {
        let env = get_env();
        let class = get_test_class();
        let field = env.GetStaticFieldID_str(class, "c", "D");
        let value = env.GetStaticDoubleField(class, field);
        env.DeleteGlobalRef(class);
        assert_eq!(v, value, "RUST={} GOT={}", v, value);
    }

    #[test]
    fn test_static_void() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let void0 = env.GetStaticMethodID_str(class, "staticVoidMethod0", "()V");
            env.CallStaticVoidMethod0(class, void0);
            assert_fn_name("staticVoidMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetStaticMethodID_str(class, "staticVoidMethod1", "(S)V");
            env.CallStaticVoidMethod1(class, void, 15i16);
            assert_fn_name("staticVoidMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetStaticMethodID_str(class, "staticVoidMethod2", "(SLjava/lang/Object;)V");
            env.CallStaticVoidMethod2(class, void, 1245i16, null_mut());
            assert_fn_name("staticVoidMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetStaticMethodID_str(class, "staticVoidMethod3", "(SLjava/lang/Object;D)V");
            let my_value = 88 as std::ffi::c_double;
            env.CallStaticVoidMethod3(class, void, 26225i16, global, my_value);
            assert_fn_name("staticVoidMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let void = env.GetStaticMethodID_str(class, "staticVoidMethod3", "(SLjava/lang/Object;D)V");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallStaticVoidMethodA(class, void, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("staticVoidMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_object() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticObjectMethod0", "()Ljava/lang/Object;");
            let result = env.CallStaticObjectMethod0(class, meth);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticObjectMethod1", "(S)Ljava/lang/Object;");
            let result =env.CallStaticObjectMethod1(class, meth, 15i16);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticObjectMethod2", "(SLjava/lang/Object;)Ljava/lang/Object;");
            let result =env.CallStaticObjectMethod2(class, meth, 1245i16, null_mut());
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let my_value = 88 as std::ffi::c_double;
            let result =env.CallStaticObjectMethod3(class, meth, 26225i16, global, my_value);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let my_value = 69.2 as std::ffi::c_double;
            let result =env.CallStaticObjectMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_boolean() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticBooleanMethod0", "()Z");
            let result = env.CallStaticBooleanMethod0(class, meth);
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticBooleanMethod1", "(S)Z");
            let result = env.CallStaticBooleanMethod1(class, meth, 15i16);
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticBooleanMethod2", "(SLjava/lang/Object;)Z");
            let result = env.CallStaticBooleanMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticBooleanMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticBooleanMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_byte() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticByteMethod0", "()B");
            let result = env.CallStaticByteMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticByteMethod1", "(S)B");
            let result = env.CallStaticByteMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticByteMethod2", "(SLjava/lang/Object;)B");
            let result = env.CallStaticByteMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticByteMethod3", "(SLjava/lang/Object;D)B");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticByteMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticByteMethod3", "(SLjava/lang/Object;D)B");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticByteMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_char() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticCharMethod0", "()C");
            let result = env.CallStaticCharMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticCharMethod1", "(S)C");
            let result = env.CallStaticCharMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticCharMethod2", "(SLjava/lang/Object;)C");
            let result = env.CallStaticCharMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticCharMethod3", "(SLjava/lang/Object;D)C");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticCharMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticCharMethod3", "(SLjava/lang/Object;D)C");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticCharMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_short() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticShortMethod0", "()S");
            let result = env.CallStaticShortMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticShortMethod1", "(S)S");
            let result = env.CallStaticShortMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticShortMethod2", "(SLjava/lang/Object;)S");
            let result = env.CallStaticShortMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticShortMethod3", "(SLjava/lang/Object;D)S");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticShortMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticShortMethod3", "(SLjava/lang/Object;D)S");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticShortMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_int() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticIntMethod0", "()I");
            let result = env.CallStaticIntMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticIntMethod1", "(S)I");
            let result = env.CallStaticIntMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticIntMethod2", "(SLjava/lang/Object;)I");
            let result = env.CallStaticIntMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticIntMethod3", "(SLjava/lang/Object;D)I");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticIntMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticIntMethod3", "(SLjava/lang/Object;D)I");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticIntMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_long() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticLongMethod0", "()J");
            let result = env.CallStaticLongMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticLongMethod1", "(S)J");
            let result = env.CallStaticLongMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticLongMethod2", "(SLjava/lang/Object;)J");
            let result = env.CallStaticLongMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticLongMethod3", "(SLjava/lang/Object;D)J");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticLongMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticLongMethod3", "(SLjava/lang/Object;D)J");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticLongMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_float() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticFloatMethod0", "()F");
            let result = env.CallStaticFloatMethod0(class, meth);
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticFloatMethod1", "(S)F");
            let result = env.CallStaticFloatMethod1(class, meth, 15i16);
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticFloatMethod2", "(SLjava/lang/Object;)F");
            let result = env.CallStaticFloatMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticFloatMethod3", "(SLjava/lang/Object;D)F");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticFloatMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticFloatMethod3", "(SLjava/lang/Object;D)F");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticFloatMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_double() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID_str(class, "staticDoubleMethod0", "()D");
            let result = env.CallStaticDoubleMethod0(class, meth);
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticDoubleMethod1", "(S)D");
            let result = env.CallStaticDoubleMethod1(class, meth, 15i16);
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticDoubleMethod2", "(SLjava/lang/Object;)D");
            let result = env.CallStaticDoubleMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID_str(class, "staticDoubleMethod3", "(SLjava/lang/Object;D)D");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticDoubleMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID_str(class, "staticDoubleMethod3", "(SLjava/lang/Object;D)D");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticDoubleMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }
}