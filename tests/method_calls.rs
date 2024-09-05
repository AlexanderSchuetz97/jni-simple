#[cfg(feature = "loadjvm")]
pub mod test {
    use std::ptr::{null_mut};
    use std::sync::Mutex;
    use jni_simple::*;


    //Cargo runs the tests on different threads.
    static MUTEX: Mutex<()> = Mutex::new(());

    unsafe fn load_it() -> (JavaVM, JNIEnv, jclass) {
        if !jni_simple::is_jvm_loaded() {
            // On linux/unix:
            jni_simple::load_jvm_from_library("/usr/lib/jvm/java-11-openjdk-amd64/lib/server/libjvm.so")
                .expect("failed to load jvm");

            // On windows:
            //    jni_simple::load_jvm_from_library("C:\\Program Files\\Java\\jdk-17.0.1\\jre\\bin\\server\\jvm.dll")
            //        .expect("failed to load jvm");
        }

        let thr = JNI_GetCreatedJavaVMs().expect("failed to get jvm");
        if thr.is_empty() {
            let args: Vec<String> = vec![];//vec!["-Xcheck:jni".to_string()];
            let (jvm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create jvm");
            let class_blob = include_bytes!("../java_testcode/MethodCalls.class");
            let class_loaded = env.DefineClass_str("MethodCalls", null_mut(), class_blob);
            if class_loaded.is_null() {
                env.ExceptionDescribe();
                env.FatalError_str("failed to load class");
            }

            return (jvm, env, class_loaded);
        }

        let jvm = thr.first().unwrap().clone();
        let env = jvm.GetEnv(JNI_VERSION_1_8);
        let env = env.unwrap_or_else(|c| {
            if c != JNI_EDETACHED {
                panic!("JVM ERROR {}", c);
            }

            jvm.AttachCurrentThread_str(JNI_VERSION_1_8, None, null_mut()).expect("failed to attach thread")
        });

        let class_loaded = env.FindClass_str("MethodCalls");

        (jvm, env, class_loaded)
    }

    unsafe fn reset_it() {
        let (_jvm, env, class) = load_it();
        let reset = env.GetStaticMethodID_str(class, "reset", "()V");
        env.CallStaticVoidMethod0(class, reset);
        env.DeleteLocalRef(class);
    }

    unsafe fn assert_fn_name(name: &str) {
        let (_jvm, env, class) = load_it();
        let name_field = env.GetStaticFieldID_str(class, "name", "Ljava/lang/String;");
        let name_obj = env.GetStaticObjectField(class, name_field);
        if name_obj.is_null() {
            panic!("assert_fn_name expected {} got null", name);
        }
        let got = env.GetStringUTFChars_as_string(name_obj).expect("failed to get string");
        env.DeleteLocalRef(name_obj);
        env.DeleteLocalRef(class);
        assert_eq!(name, got.as_str());
    }

    unsafe fn assert_a(v: i16) {
        let (_jvm, env, class) = load_it();
        let field = env.GetStaticFieldID_str(class, "a", "S");
        let value = env.GetStaticShortField(class, field);
        env.DeleteLocalRef(class);
        assert_eq!(v, value);
    }

    unsafe fn new_global_obj() -> jobject {
        let (_jvm, env, class) = load_it();
        env.DeleteLocalRef(class);
        let class = env.FindClass_str("java/lang/Object");
        let meth = env.GetMethodID_str(class, "<init>", "()V");
        let obj = env.NewObjectA(class, meth, null_mut());
        let gref = env.NewGlobalRef(obj);
        env.DeleteLocalRef(obj);
        env.DeleteLocalRef(class);
        gref
    }
    unsafe fn assert_b(v: jobject) {
        let (_jvm, env, class) = load_it();
        let field = env.GetStaticFieldID_str(class, "b", "Ljava/lang/Object;");
        let value = env.GetStaticObjectField(class, field);
        env.DeleteLocalRef(class);
        if v.is_null() {
            assert!(value.is_null());
            return;
        }
        assert!(!value.is_null());
        assert!(env.IsSameObject(v, value));
        env.DeleteLocalRef(value);
    }

    unsafe fn assert_c(v: std::ffi::c_double) {
        let (_jvm, env, class) = load_it();
        let field = env.GetStaticFieldID_str(class, "c", "D");
        let value = env.GetStaticDoubleField(class, field);
        env.DeleteLocalRef(class);
        assert_eq!(v, value);
    }

    #[test]
    fn test_static_void() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            reset_it();
            let global = new_global_obj();

            let (_jvm, env, class) = load_it();
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
        }
    }
}