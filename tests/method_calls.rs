#[cfg(feature = "loadjvm")]
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
        let class_loaded = env.FindClass("MethodCalls");
        if !class_loaded.is_null() {
            let class_global = env.NewGlobalRef(class_loaded);
            env.DeleteLocalRef(class_loaded);
            return class_global;
        }

        env.ExceptionClear(); //Clear ClassNotFoundException
        let class_blob = include_bytes!("../java_testcode/MethodCalls.class");
        let class_loaded = env.DefineClass("MethodCalls", null_mut(), class_blob);
        if class_loaded.is_null() {
            env.ExceptionDescribe();
            env.FatalError("failed to load class");
        }

        let class_global = env.NewGlobalRef(class_loaded);
        env.DeleteLocalRef(class_loaded);
        return class_global;
    }

    unsafe fn get_nv_test_class() -> jclass {
        let env = get_env();
        let tc = get_test_class();

        let class_loaded = env.FindClass("MethodCalls$NvChild");
        if !class_loaded.is_null() {
            let class_global = env.NewGlobalRef(class_loaded);
            env.DeleteLocalRef(class_loaded);
            return class_global;
        }

        env.ExceptionClear(); //Clear ClassNotFoundException
        let class_blob = include_bytes!("../java_testcode/MethodCalls$NvChild.class");
        let class_loaded = env.DefineClass("MethodCalls$NvChild", null_mut(), class_blob);
        if class_loaded.is_null() {
            env.ExceptionDescribe();
            env.FatalError("failed to load class");
        }

        let class_global = env.NewGlobalRef(class_loaded);
        env.DeleteLocalRef(class_loaded);
        env.DeleteGlobalRef(tc);
        return class_global;
    }

    unsafe fn get_test_obj() -> jobject {
        let env = get_env();
        let tc = get_test_class();
        let local_obj = env.AllocObject(tc);
        env.DeleteGlobalRef(tc);
        local_obj
    }

    unsafe fn get_nv_test_obj() -> jobject {
        let env = get_env();
        let tc = get_nv_test_class();
        let local_obj = env.AllocObject(tc);
        env.DeleteGlobalRef(tc);
        local_obj
    }

    unsafe fn reset_it() {
        let env = get_env();
        let class = get_test_class();
        let reset = env.GetStaticMethodID(class, "reset", "()V");
        env.CallStaticVoidMethod0(class, reset);
        env.DeleteGlobalRef(class);
    }

    unsafe fn assert_fn_name(name: &str) {
        let env = get_env();
        let class = get_test_class();
        let name_field = env.GetStaticFieldID(class, "name", "Ljava/lang/String;");
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
        let field = env.GetStaticFieldID(class, "a", "S");
        let value = env.GetStaticShortField(class, field);
        env.DeleteGlobalRef(class);
        assert_eq!(v, value);
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
    unsafe fn assert_b(v: jobject) {
        let env = get_env();
        let class = get_test_class();
        let field = env.GetStaticFieldID(class, "b", "Ljava/lang/Object;");
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
        let field = env.GetStaticFieldID(class, "c", "D");
        let value = env.GetStaticDoubleField(class, field);
        env.DeleteGlobalRef(class);
        assert_eq!(v, value, "RUST={} GOT={}", v, value);
    }

    #[test]
    fn test_nv_void() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynVoidMethod0", "()V");
            let meth_c = env.GetMethodID(child, "dynVoidMethod0", "()V");
            env.CallVoidMethod0(inst, meth);
            assert_fn_name("nvVoidMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualVoidMethod0(inst, base, meth);
            assert_fn_name("dynVoidMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualVoidMethod0(inst, child, meth_c);
            assert_fn_name("nvVoidMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynVoidMethod1", "(S)V");
            let meth_c = env.GetMethodID(child, "dynVoidMethod1", "(S)V");
            env.CallVoidMethod1(inst, meth, 15i16);
            assert_fn_name("nvVoidMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualVoidMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynVoidMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualVoidMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvVoidMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynVoidMethod2", "(SLjava/lang/Object;)V");
            let meth_c = env.GetMethodID(child, "dynVoidMethod2", "(SLjava/lang/Object;)V");
            env.CallVoidMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvVoidMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualVoidMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynVoidMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualVoidMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvVoidMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynVoidMethod3", "(SLjava/lang/Object;D)V");
            let meth_c = env.GetMethodID(child, "dynVoidMethod3", "(SLjava/lang/Object;D)V");
            let my_value = 88 as std::ffi::c_double;
            env.CallVoidMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvVoidMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualVoidMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynVoidMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualVoidMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvVoidMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynVoidMethod3", "(SLjava/lang/Object;D)V");
            let meth_c = env.GetMethodID(child, "dynVoidMethod3", "(SLjava/lang/Object;D)V");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallVoidMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvVoidMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualVoidMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynVoidMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualVoidMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvVoidMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_object() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynObjectMethod0", "()Ljava/lang/Object;");
            let meth_c = env.GetMethodID(child, "dynObjectMethod0", "()Ljava/lang/Object;");
            env.CallObjectMethod0(inst, meth);
            assert_fn_name("nvObjectMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualObjectMethod0(inst, base, meth);
            assert_fn_name("dynObjectMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualObjectMethod0(inst, child, meth_c);
            assert_fn_name("nvObjectMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynObjectMethod1", "(S)Ljava/lang/Object;");
            let meth_c = env.GetMethodID(child, "dynObjectMethod1", "(S)Ljava/lang/Object;");
            env.CallObjectMethod1(inst, meth, 15i16);
            assert_fn_name("nvObjectMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualObjectMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynObjectMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualObjectMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvObjectMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynObjectMethod2", "(SLjava/lang/Object;)Ljava/lang/Object;");
            let meth_c = env.GetMethodID(child, "dynObjectMethod2", "(SLjava/lang/Object;)Ljava/lang/Object;");
            env.CallObjectMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvObjectMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualObjectMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynObjectMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualObjectMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvObjectMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let meth_c = env.GetMethodID(child, "dynObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let my_value = 88 as std::ffi::c_double;
            env.CallObjectMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvObjectMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualObjectMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynObjectMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualObjectMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvObjectMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let meth_c = env.GetMethodID(child, "dynObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallObjectMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvObjectMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualObjectMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynObjectMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualObjectMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvObjectMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_boolean() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynBooleanMethod0", "()Z");
            let meth_c = env.GetMethodID(child, "dynBooleanMethod0", "()Z");
            env.CallBooleanMethod0(inst, meth);
            assert_fn_name("nvBooleanMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualBooleanMethod0(inst, base, meth);
            assert_fn_name("dynBooleanMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualBooleanMethod0(inst, child, meth_c);
            assert_fn_name("nvBooleanMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynBooleanMethod1", "(S)Z");
            let meth_c = env.GetMethodID(child, "dynBooleanMethod1", "(S)Z");
            env.CallBooleanMethod1(inst, meth, 15i16);
            assert_fn_name("nvBooleanMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualBooleanMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynBooleanMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualBooleanMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvBooleanMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynBooleanMethod2", "(SLjava/lang/Object;)Z");
            let meth_c = env.GetMethodID(child, "dynBooleanMethod2", "(SLjava/lang/Object;)Z");
            env.CallBooleanMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvBooleanMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualBooleanMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynBooleanMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualBooleanMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvBooleanMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let meth_c = env.GetMethodID(child, "dynBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let my_value = 88 as std::ffi::c_double;
            env.CallBooleanMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvBooleanMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualBooleanMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynBooleanMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualBooleanMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvBooleanMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let meth_c = env.GetMethodID(child, "dynBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallBooleanMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvBooleanMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualBooleanMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynBooleanMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualBooleanMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvBooleanMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_byte() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynByteMethod0", "()B");
            let meth_c = env.GetMethodID(child, "dynByteMethod0", "()B");
            env.CallByteMethod0(inst, meth);
            assert_fn_name("nvByteMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualByteMethod0(inst, base, meth);
            assert_fn_name("dynByteMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualByteMethod0(inst, child, meth_c);
            assert_fn_name("nvByteMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynByteMethod1", "(S)B");
            let meth_c = env.GetMethodID(child, "dynByteMethod1", "(S)B");
            env.CallByteMethod1(inst, meth, 15i16);
            assert_fn_name("nvByteMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualByteMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynByteMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualByteMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvByteMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynByteMethod2", "(SLjava/lang/Object;)B");
            let meth_c = env.GetMethodID(child, "dynByteMethod2", "(SLjava/lang/Object;)B");
            env.CallByteMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvByteMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualByteMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynByteMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualByteMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvByteMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynByteMethod3", "(SLjava/lang/Object;D)B");
            let meth_c = env.GetMethodID(child, "dynByteMethod3", "(SLjava/lang/Object;D)B");
            let my_value = 88 as std::ffi::c_double;
            env.CallByteMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvByteMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualByteMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynByteMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualByteMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvByteMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynByteMethod3", "(SLjava/lang/Object;D)B");
            let meth_c = env.GetMethodID(child, "dynByteMethod3", "(SLjava/lang/Object;D)B");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallByteMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvByteMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualByteMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynByteMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualByteMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvByteMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_char() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynCharMethod0", "()C");
            let meth_c = env.GetMethodID(child, "dynCharMethod0", "()C");
            env.CallCharMethod0(inst, meth);
            assert_fn_name("nvCharMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualCharMethod0(inst, base, meth);
            assert_fn_name("dynCharMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualCharMethod0(inst, child, meth_c);
            assert_fn_name("nvCharMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynCharMethod1", "(S)C");
            let meth_c = env.GetMethodID(child, "dynCharMethod1", "(S)C");
            env.CallCharMethod1(inst, meth, 15i16);
            assert_fn_name("nvCharMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualCharMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynCharMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualCharMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvCharMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynCharMethod2", "(SLjava/lang/Object;)C");
            let meth_c = env.GetMethodID(child, "dynCharMethod2", "(SLjava/lang/Object;)C");
            env.CallCharMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvCharMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualCharMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynCharMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualCharMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvCharMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynCharMethod3", "(SLjava/lang/Object;D)C");
            let meth_c = env.GetMethodID(child, "dynCharMethod3", "(SLjava/lang/Object;D)C");
            let my_value = 88 as std::ffi::c_double;
            env.CallCharMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvCharMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualCharMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynCharMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualCharMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvCharMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynCharMethod3", "(SLjava/lang/Object;D)C");
            let meth_c = env.GetMethodID(child, "dynCharMethod3", "(SLjava/lang/Object;D)C");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallCharMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvCharMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualCharMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynCharMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualCharMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvCharMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_short() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynShortMethod0", "()S");
            let meth_c = env.GetMethodID(child, "dynShortMethod0", "()S");
            env.CallShortMethod0(inst, meth);
            assert_fn_name("nvShortMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualShortMethod0(inst, base, meth);
            assert_fn_name("dynShortMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualShortMethod0(inst, child, meth_c);
            assert_fn_name("nvShortMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynShortMethod1", "(S)S");
            let meth_c = env.GetMethodID(child, "dynShortMethod1", "(S)S");
            env.CallShortMethod1(inst, meth, 15i16);
            assert_fn_name("nvShortMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualShortMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynShortMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualShortMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvShortMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynShortMethod2", "(SLjava/lang/Object;)S");
            let meth_c = env.GetMethodID(child, "dynShortMethod2", "(SLjava/lang/Object;)S");
            env.CallShortMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvShortMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualShortMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynShortMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualShortMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvShortMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynShortMethod3", "(SLjava/lang/Object;D)S");
            let meth_c = env.GetMethodID(child, "dynShortMethod3", "(SLjava/lang/Object;D)S");
            let my_value = 88 as std::ffi::c_double;
            env.CallShortMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvShortMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualShortMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynShortMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualShortMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvShortMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynShortMethod3", "(SLjava/lang/Object;D)S");
            let meth_c = env.GetMethodID(child, "dynShortMethod3", "(SLjava/lang/Object;D)S");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallShortMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvShortMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualShortMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynShortMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualShortMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvShortMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_int() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynIntMethod0", "()I");
            let meth_c = env.GetMethodID(child, "dynIntMethod0", "()I");
            env.CallIntMethod0(inst, meth);
            assert_fn_name("nvIntMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualIntMethod0(inst, base, meth);
            assert_fn_name("dynIntMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualIntMethod0(inst, child, meth_c);
            assert_fn_name("nvIntMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynIntMethod1", "(S)I");
            let meth_c = env.GetMethodID(child, "dynIntMethod1", "(S)I");
            env.CallIntMethod1(inst, meth, 15i16);
            assert_fn_name("nvIntMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualIntMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynIntMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualIntMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvIntMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynIntMethod2", "(SLjava/lang/Object;)I");
            let meth_c = env.GetMethodID(child, "dynIntMethod2", "(SLjava/lang/Object;)I");
            env.CallIntMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvIntMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualIntMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynIntMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualIntMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvIntMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynIntMethod3", "(SLjava/lang/Object;D)I");
            let meth_c = env.GetMethodID(child, "dynIntMethod3", "(SLjava/lang/Object;D)I");
            let my_value = 88 as std::ffi::c_double;
            env.CallIntMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvIntMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualIntMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynIntMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualIntMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvIntMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynIntMethod3", "(SLjava/lang/Object;D)I");
            let meth_c = env.GetMethodID(child, "dynIntMethod3", "(SLjava/lang/Object;D)I");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallIntMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvIntMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualIntMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynIntMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualIntMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvIntMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_long() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynLongMethod0", "()J");
            let meth_c = env.GetMethodID(child, "dynLongMethod0", "()J");
            env.CallLongMethod0(inst, meth);
            assert_fn_name("nvLongMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualLongMethod0(inst, base, meth);
            assert_fn_name("dynLongMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualLongMethod0(inst, child, meth_c);
            assert_fn_name("nvLongMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynLongMethod1", "(S)J");
            let meth_c = env.GetMethodID(child, "dynLongMethod1", "(S)J");
            env.CallLongMethod1(inst, meth, 15i16);
            assert_fn_name("nvLongMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualLongMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynLongMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualLongMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvLongMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynLongMethod2", "(SLjava/lang/Object;)J");
            let meth_c = env.GetMethodID(child, "dynLongMethod2", "(SLjava/lang/Object;)J");
            env.CallLongMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvLongMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualLongMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynLongMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualLongMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvLongMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynLongMethod3", "(SLjava/lang/Object;D)J");
            let meth_c = env.GetMethodID(child, "dynLongMethod3", "(SLjava/lang/Object;D)J");
            let my_value = 88 as std::ffi::c_double;
            env.CallLongMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvLongMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualLongMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynLongMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualLongMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvLongMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynLongMethod3", "(SLjava/lang/Object;D)J");
            let meth_c = env.GetMethodID(child, "dynLongMethod3", "(SLjava/lang/Object;D)J");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallLongMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvLongMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualLongMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynLongMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualLongMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvLongMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_float() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynFloatMethod0", "()F");
            let meth_c = env.GetMethodID(child, "dynFloatMethod0", "()F");
            env.CallFloatMethod0(inst, meth);
            assert_fn_name("nvFloatMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualFloatMethod0(inst, base, meth);
            assert_fn_name("dynFloatMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualFloatMethod0(inst, child, meth_c);
            assert_fn_name("nvFloatMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynFloatMethod1", "(S)F");
            let meth_c = env.GetMethodID(child, "dynFloatMethod1", "(S)F");
            env.CallFloatMethod1(inst, meth, 15i16);
            assert_fn_name("nvFloatMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualFloatMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynFloatMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualFloatMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvFloatMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynFloatMethod2", "(SLjava/lang/Object;)F");
            let meth_c = env.GetMethodID(child, "dynFloatMethod2", "(SLjava/lang/Object;)F");
            env.CallFloatMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvFloatMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualFloatMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynFloatMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualFloatMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvFloatMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynFloatMethod3", "(SLjava/lang/Object;D)F");
            let meth_c = env.GetMethodID(child, "dynFloatMethod3", "(SLjava/lang/Object;D)F");
            let my_value = 88 as std::ffi::c_double;
            env.CallFloatMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvFloatMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualFloatMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynFloatMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualFloatMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvFloatMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynFloatMethod3", "(SLjava/lang/Object;D)F");
            let meth_c = env.GetMethodID(child, "dynFloatMethod3", "(SLjava/lang/Object;D)F");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallFloatMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvFloatMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualFloatMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynFloatMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualFloatMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvFloatMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_nv_double() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_nv_test_obj();
            let base = get_test_class();
            let child = get_nv_test_class();

            let env = get_env();
            let meth = env.GetMethodID(base, "dynDoubleMethod0", "()D");
            let meth_c = env.GetMethodID(child, "dynDoubleMethod0", "()D");
            env.CallDoubleMethod0(inst, meth);
            assert_fn_name("nvDoubleMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualDoubleMethod0(inst, base, meth);
            assert_fn_name("dynDoubleMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualDoubleMethod0(inst, child, meth_c);
            assert_fn_name("nvDoubleMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynDoubleMethod1", "(S)D");
            let meth_c = env.GetMethodID(child, "dynDoubleMethod1", "(S)D");
            env.CallDoubleMethod1(inst, meth, 15i16);
            assert_fn_name("nvDoubleMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualDoubleMethod1(inst, base, meth, 15i16);
            assert_fn_name("dynDoubleMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualDoubleMethod1(inst, child, meth_c, 15i16);
            assert_fn_name("nvDoubleMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynDoubleMethod2", "(SLjava/lang/Object;)D");
            let meth_c = env.GetMethodID(child, "dynDoubleMethod2", "(SLjava/lang/Object;)D");
            env.CallDoubleMethod2(inst, meth, 1245i16, null_mut());
            assert_fn_name("nvDoubleMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualDoubleMethod2(inst, base, meth, 15i16, null_mut());
            assert_fn_name("dynDoubleMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            env.CallNonvirtualDoubleMethod2(inst, child, meth_c, 15i16, null_mut());
            assert_fn_name("nvDoubleMethod2");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(base, "dynDoubleMethod3", "(SLjava/lang/Object;D)D");
            let meth_c = env.GetMethodID(child, "dynDoubleMethod3", "(SLjava/lang/Object;D)D");
            let my_value = 88 as std::ffi::c_double;
            env.CallDoubleMethod3(inst, meth, 26225i16, global, my_value);
            assert_fn_name("nvDoubleMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualDoubleMethod3(inst, base, meth, 26225i16, global, my_value);
            assert_fn_name("dynDoubleMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            env.CallNonvirtualDoubleMethod3(inst, child, meth_c, 26225i16, global, my_value);
            assert_fn_name("nvDoubleMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(base, "dynDoubleMethod3", "(SLjava/lang/Object;D)D");
            let meth_c = env.GetMethodID(child, "dynDoubleMethod3", "(SLjava/lang/Object;D)D");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallDoubleMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvDoubleMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualDoubleMethodA(inst, base, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynDoubleMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.CallNonvirtualDoubleMethodA(inst, child, meth_c, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("nvDoubleMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(base);
            env.DeleteGlobalRef(child);
        }
    }

    #[test]
    fn test_init() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let env = get_env();
            let class = get_test_class();

            let void0 = env.GetMethodID(class, "<init>", "()V");
            let g = env.NewObject0(class, void0);
            assert!(!g.is_null());
            env.DeleteLocalRef(g);
            assert_fn_name("init0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetMethodID(class, "<init>", "(S)V");
            let g = env.NewObject1(class, void, 15i16);
            assert!(!g.is_null());
            env.DeleteLocalRef(g);
            assert_fn_name("init1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetMethodID(class, "<init>", "(SLjava/lang/Object;)V");
            let g = env.NewObject2(class, void, 1245i16, null_mut());
            assert!(!g.is_null());
            env.DeleteLocalRef(g);
            assert_fn_name("init2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetMethodID(class, "<init>", "(SLjava/lang/Object;D)V");
            let my_value = 694.20 as std::ffi::c_double;
            let g = env.NewObject3(class, void, 26225i16, global, my_value);
            assert!(!g.is_null());
            env.DeleteLocalRef(g);
            assert_fn_name("init3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let void = env.GetMethodID(class, "<init>", "(SLjava/lang/Object;D)V");
            let my_value = 69.2 as std::ffi::c_double;
            let g = env.NewObjectA(class, void, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert!(!g.is_null());
            env.DeleteLocalRef(g);
            assert_fn_name("init3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }
    #[test]
    fn test_dyn_void() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let void0 = env.GetMethodID(class, "dynVoidMethod0", "()V");
            env.CallVoidMethod0(inst, void0);
            assert_fn_name("dynVoidMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetMethodID(class, "dynVoidMethod1", "(S)V");
            env.CallVoidMethod1(inst, void, 15i16);
            assert_fn_name("dynVoidMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetMethodID(class, "dynVoidMethod2", "(SLjava/lang/Object;)V");
            env.CallVoidMethod2(inst, void, 1245i16, null_mut());
            assert_fn_name("dynVoidMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetMethodID(class, "dynVoidMethod3", "(SLjava/lang/Object;D)V");
            let my_value = 88 as std::ffi::c_double;
            env.CallVoidMethod3(inst, void, 26225i16, global, my_value);
            assert_fn_name("dynVoidMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let void = env.GetMethodID(class, "dynVoidMethod3", "(SLjava/lang/Object;D)V");
            let my_value = 69.2 as std::ffi::c_double;
            env.CallVoidMethodA(inst, void, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_fn_name("dynVoidMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_object() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynObjectMethod0", "()Ljava/lang/Object;");
            let result = env.CallObjectMethod0(inst, meth);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("dynObjectMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynObjectMethod1", "(S)Ljava/lang/Object;");
            let result = env.CallObjectMethod1(inst, meth, 15i16);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("dynObjectMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynObjectMethod2", "(SLjava/lang/Object;)Ljava/lang/Object;");
            let result = env.CallObjectMethod2(inst, meth, 1245i16, null_mut());
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("dynObjectMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallObjectMethod3(inst, meth, 26225i16, global, my_value);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("dynObjectMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallObjectMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("dynObjectMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_boolean() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynBooleanMethod0", "()Z");
            let result = env.CallBooleanMethod0(inst, meth);
            assert_eq!(result, true);
            assert_fn_name("dynBooleanMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynBooleanMethod1", "(S)Z");
            let result = env.CallBooleanMethod1(inst, meth, 15i16);
            assert_eq!(result, true);
            assert_fn_name("dynBooleanMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynBooleanMethod2", "(SLjava/lang/Object;)Z");
            let result = env.CallBooleanMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, true);
            assert_fn_name("dynBooleanMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallBooleanMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, true);
            assert_fn_name("dynBooleanMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallBooleanMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, true);
            assert_fn_name("dynBooleanMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_byte() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynByteMethod0", "()B");
            let result = env.CallByteMethod0(inst, meth);
            assert_eq!(result, 1);
            assert_fn_name("dynByteMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynByteMethod1", "(S)B");
            let result = env.CallByteMethod1(inst, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("dynByteMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynByteMethod2", "(SLjava/lang/Object;)B");
            let result = env.CallByteMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("dynByteMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynByteMethod3", "(SLjava/lang/Object;D)B");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallByteMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("dynByteMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynByteMethod3", "(SLjava/lang/Object;D)B");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallByteMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("dynByteMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_char() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynCharMethod0", "()C");
            let result = env.CallCharMethod0(inst, meth);
            assert_eq!(result, 1);
            assert_fn_name("dynCharMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynCharMethod1", "(S)C");
            let result = env.CallCharMethod1(inst, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("dynCharMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynCharMethod2", "(SLjava/lang/Object;)C");
            let result = env.CallCharMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("dynCharMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynCharMethod3", "(SLjava/lang/Object;D)C");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallCharMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("dynCharMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynCharMethod3", "(SLjava/lang/Object;D)C");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallCharMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("dynCharMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_short() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynShortMethod0", "()S");
            let result = env.CallShortMethod0(inst, meth);
            assert_eq!(result, 1);
            assert_fn_name("dynShortMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynShortMethod1", "(S)S");
            let result = env.CallShortMethod1(inst, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("dynShortMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynShortMethod2", "(SLjava/lang/Object;)S");
            let result = env.CallShortMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("dynShortMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynShortMethod3", "(SLjava/lang/Object;D)S");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallShortMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("dynShortMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynShortMethod3", "(SLjava/lang/Object;D)S");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallShortMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("dynShortMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_int() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynIntMethod0", "()I");
            let result = env.CallIntMethod0(inst, meth);
            assert_eq!(result, 1);
            assert_fn_name("dynIntMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynIntMethod1", "(S)I");
            let result = env.CallIntMethod1(inst, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("dynIntMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynIntMethod2", "(SLjava/lang/Object;)I");
            let result = env.CallIntMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("dynIntMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynIntMethod3", "(SLjava/lang/Object;D)I");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallIntMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("dynIntMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynIntMethod3", "(SLjava/lang/Object;D)I");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallIntMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("dynIntMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_long() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynLongMethod0", "()J");
            let result = env.CallLongMethod0(inst, meth);
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynLongMethod1", "(S)J");
            let result = env.CallLongMethod1(inst, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynLongMethod2", "(SLjava/lang/Object;)J");
            let result = env.CallLongMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynLongMethod3", "(SLjava/lang/Object;D)J");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallLongMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynLongMethod3", "(SLjava/lang/Object;D)J");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallLongMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_float() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynFloatMethod0", "()F");
            let result = env.CallFloatMethod0(inst, meth);
            assert_eq!(result, 1f32);
            assert_fn_name("dynFloatMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynFloatMethod1", "(S)F");
            let result = env.CallFloatMethod1(inst, meth, 15i16);
            assert_eq!(result, 1f32);
            assert_fn_name("dynFloatMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynFloatMethod2", "(SLjava/lang/Object;)F");
            let result = env.CallFloatMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, 1f32);
            assert_fn_name("dynFloatMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynFloatMethod3", "(SLjava/lang/Object;D)F");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallFloatMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, 1f32);
            assert_fn_name("dynFloatMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynFloatMethod3", "(SLjava/lang/Object;D)F");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallFloatMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1f32);
            assert_fn_name("dynFloatMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_dyn_double() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynDoubleMethod0", "()D");
            let result = env.CallDoubleMethod0(inst, meth);
            assert_eq!(result, 1f64);
            assert_fn_name("dynDoubleMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynDoubleMethod1", "(S)D");
            let result = env.CallDoubleMethod1(inst, meth, 15i16);
            assert_eq!(result, 1f64);
            assert_fn_name("dynDoubleMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynDoubleMethod2", "(SLjava/lang/Object;)D");
            let result = env.CallDoubleMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, 1f64);
            assert_fn_name("dynDoubleMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynDoubleMethod3", "(SLjava/lang/Object;D)D");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallDoubleMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, 1f64);
            assert_fn_name("dynDoubleMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynDoubleMethod3", "(SLjava/lang/Object;D)D");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallDoubleMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1f64);
            assert_fn_name("dynDoubleMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_void() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let void0 = env.GetStaticMethodID(class, "staticVoidMethod0", "()V");
            env.CallStaticVoidMethod0(class, void0);
            assert_fn_name("staticVoidMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetStaticMethodID(class, "staticVoidMethod1", "(S)V");
            env.CallStaticVoidMethod1(class, void, 15i16);
            assert_fn_name("staticVoidMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetStaticMethodID(class, "staticVoidMethod2", "(SLjava/lang/Object;)V");
            env.CallStaticVoidMethod2(class, void, 1245i16, null_mut());
            assert_fn_name("staticVoidMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let void = env.GetStaticMethodID(class, "staticVoidMethod3", "(SLjava/lang/Object;D)V");
            let my_value = 88 as std::ffi::c_double;
            env.CallStaticVoidMethod3(class, void, 26225i16, global, my_value);
            assert_fn_name("staticVoidMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let void = env.GetStaticMethodID(class, "staticVoidMethod3", "(SLjava/lang/Object;D)V");
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
            let meth = env.GetStaticMethodID(class, "staticObjectMethod0", "()Ljava/lang/Object;");
            let result = env.CallStaticObjectMethod0(class, meth);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticObjectMethod1", "(S)Ljava/lang/Object;");
            let result = env.CallStaticObjectMethod1(class, meth, 15i16);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticObjectMethod2", "(SLjava/lang/Object;)Ljava/lang/Object;");
            let result = env.CallStaticObjectMethod2(class, meth, 1245i16, null_mut());
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticObjectMethod3(class, meth, 26225i16, global, my_value);
            assert!(!result.is_null());
            env.DeleteLocalRef(result);
            assert_fn_name("staticObjectMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticObjectMethod3", "(SLjava/lang/Object;D)Ljava/lang/Object;");
            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallStaticObjectMethodA(class, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
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
            let meth = env.GetStaticMethodID(class, "staticBooleanMethod0", "()Z");
            let result = env.CallStaticBooleanMethod0(class, meth);
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticBooleanMethod1", "(S)Z");
            let result = env.CallStaticBooleanMethod1(class, meth, 15i16);
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticBooleanMethod2", "(SLjava/lang/Object;)Z");
            let result = env.CallStaticBooleanMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticBooleanMethod3", "(SLjava/lang/Object;D)Z");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticBooleanMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, true);
            assert_fn_name("staticBooleanMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticBooleanMethod3", "(SLjava/lang/Object;D)Z");
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
            let meth = env.GetStaticMethodID(class, "staticByteMethod0", "()B");
            let result = env.CallStaticByteMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticByteMethod1", "(S)B");
            let result = env.CallStaticByteMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticByteMethod2", "(SLjava/lang/Object;)B");
            let result = env.CallStaticByteMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticByteMethod3", "(SLjava/lang/Object;D)B");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticByteMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticByteMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticByteMethod3", "(SLjava/lang/Object;D)B");
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
            let meth = env.GetStaticMethodID(class, "staticCharMethod0", "()C");
            let result = env.CallStaticCharMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticCharMethod1", "(S)C");
            let result = env.CallStaticCharMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticCharMethod2", "(SLjava/lang/Object;)C");
            let result = env.CallStaticCharMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticCharMethod3", "(SLjava/lang/Object;D)C");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticCharMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticCharMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticCharMethod3", "(SLjava/lang/Object;D)C");
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
            let meth = env.GetStaticMethodID(class, "staticShortMethod0", "()S");
            let result = env.CallStaticShortMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticShortMethod1", "(S)S");
            let result = env.CallStaticShortMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticShortMethod2", "(SLjava/lang/Object;)S");
            let result = env.CallStaticShortMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticShortMethod3", "(SLjava/lang/Object;D)S");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticShortMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticShortMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticShortMethod3", "(SLjava/lang/Object;D)S");
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
            let meth = env.GetStaticMethodID(class, "staticIntMethod0", "()I");
            let result = env.CallStaticIntMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticIntMethod1", "(S)I");
            let result = env.CallStaticIntMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticIntMethod2", "(SLjava/lang/Object;)I");
            let result = env.CallStaticIntMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticIntMethod3", "(SLjava/lang/Object;D)I");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticIntMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticIntMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticIntMethod3", "(SLjava/lang/Object;D)I");
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
            let meth = env.GetStaticMethodID(class, "staticLongMethod0", "()J");
            let result = env.CallStaticLongMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticLongMethod1", "(S)J");
            let result = env.CallStaticLongMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticLongMethod2", "(SLjava/lang/Object;)J");
            let result = env.CallStaticLongMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticLongMethod3", "(SLjava/lang/Object;D)J");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticLongMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticLongMethod3", "(SLjava/lang/Object;D)J");
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
            let meth = env.GetStaticMethodID(class, "staticFloatMethod0", "()F");
            let result = env.CallStaticFloatMethod0(class, meth);
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticFloatMethod1", "(S)F");
            let result = env.CallStaticFloatMethod1(class, meth, 15i16);
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticFloatMethod2", "(SLjava/lang/Object;)F");
            let result = env.CallStaticFloatMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticFloatMethod3", "(SLjava/lang/Object;D)F");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticFloatMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1f32);
            assert_fn_name("staticFloatMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticFloatMethod3", "(SLjava/lang/Object;D)F");
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
            let meth = env.GetStaticMethodID(class, "staticDoubleMethod0", "()D");
            let result = env.CallStaticDoubleMethod0(class, meth);
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticDoubleMethod1", "(S)D");
            let result = env.CallStaticDoubleMethod1(class, meth, 15i16);
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticDoubleMethod2", "(SLjava/lang/Object;)D");
            let result = env.CallStaticDoubleMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticDoubleMethod3", "(SLjava/lang/Object;D)D");
            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticDoubleMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1f64);
            assert_fn_name("staticDoubleMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticDoubleMethod3", "(SLjava/lang/Object;D)D");
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

    #[test]
    fn test_dyn_long_reflected() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();
            let inst = get_test_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetMethodID(class, "dynLongMethod0", "()J");
            let refl = env.ToReflectedMethod(class, meth, false);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let result = env.CallLongMethod0(inst, meth);
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynLongMethod1", "(S)J");
            let refl = env.ToReflectedMethod(class, meth, false);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let result = env.CallLongMethod1(inst, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynLongMethod2", "(SLjava/lang/Object;)J");
            let refl = env.ToReflectedMethod(class, meth, false);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let result = env.CallLongMethod2(inst, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetMethodID(class, "dynLongMethod3", "(SLjava/lang/Object;D)J");
            let refl = env.ToReflectedMethod(class, meth, false);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let my_value = 88 as std::ffi::c_double;
            let result = env.CallLongMethod3(inst, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetMethodID(class, "dynLongMethod3", "(SLjava/lang/Object;D)J");
            let refl = env.ToReflectedMethod(class, meth, false);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let my_value = 69.2 as std::ffi::c_double;
            let result = env.CallLongMethodA(inst, meth, [32695i16.into(), jtype::null(), my_value.into()].as_ptr());
            assert_eq!(result, 1);
            assert_fn_name("dynLongMethod3");
            assert_a(32695i16);
            assert_b(null_mut());
            assert_c(my_value);

            env.DeleteLocalRef(inst);
            env.DeleteGlobalRef(global);
            env.DeleteGlobalRef(class);
        }
    }

    #[test]
    fn test_static_long_reflected() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let env = get_env();
            let class = get_test_class();
            let meth = env.GetStaticMethodID(class, "staticLongMethod0", "()J");
            let refl = env.ToReflectedMethod(class, meth, true);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let result = env.CallStaticLongMethod0(class, meth);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod0");
            assert_a(0i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticLongMethod1", "(S)J");
            let refl = env.ToReflectedMethod(class, meth, true);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let result = env.CallStaticLongMethod1(class, meth, 15i16);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod1");
            assert_a(15i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticLongMethod2", "(SLjava/lang/Object;)J");
            let refl = env.ToReflectedMethod(class, meth, true);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let result = env.CallStaticLongMethod2(class, meth, 1245i16, null_mut());
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod2");
            assert_a(1245i16);
            assert_b(null_mut());
            assert_c(0f64);

            let meth = env.GetStaticMethodID(class, "staticLongMethod3", "(SLjava/lang/Object;D)J");
            let refl = env.ToReflectedMethod(class, meth, true);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

            let my_value = 88 as std::ffi::c_double;
            let result = env.CallStaticLongMethod3(class, meth, 26225i16, global, my_value);
            assert_eq!(result, 1);
            assert_fn_name("staticLongMethod3");
            assert_a(26225i16);
            assert_b(global);
            assert_c(my_value);

            let meth = env.GetStaticMethodID(class, "staticLongMethod3", "(SLjava/lang/Object;D)J");
            let refl = env.ToReflectedMethod(class, meth, true);
            let meth = env.FromReflectedMethod(refl);
            env.DeleteLocalRef(refl);

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
}
