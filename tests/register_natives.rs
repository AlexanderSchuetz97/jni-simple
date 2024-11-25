#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::*;
    use std::ffi::{c_void, CString};
    use std::ptr::null_mut;

    unsafe extern "system" fn t1(env: JNIEnv, _: jclass, param: jobject) {
        assert!(!param.is_null());
        let data = env.GetStringUTFChars_as_string(param).unwrap();
        assert_eq!(data.as_str(), "test_string");
    }

    unsafe extern "system" fn t2(_env: JNIEnv, _: jclass, param: jdouble) {
        assert_eq!(param, 754.156f64);
    }

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create java VM");

            let class_blob = include_bytes!("../java_testcode/RegisterTest.class");

            let registered_class = env.DefineClass_from_slice("RegisterTest", null_mut(), class_blob.as_slice());
            let t1m = env.GetStaticMethodID(registered_class, "callTest", "(Ljava/lang/String;)V");
            let t2m = env.GetStaticMethodID(registered_class, "callTest", "(D)V");
            let test_string = env.NewStringUTF("test_string");

            env.CallStaticVoidMethod1(registered_class, t1m, test_string);
            assert!(env.ExceptionCheck());
            let exc = env.ExceptionOccurred();
            assert!(!exc.is_null());
            env.ExceptionClear();
            let exc_class = env.GetObjectClass(exc);
            env.DeleteLocalRef(exc);
            let class_class = env.GetObjectClass(exc_class);
            let get_name_method = env.GetMethodID(class_class, "getName", "()Ljava/lang/String;");
            env.DeleteLocalRef(class_class);
            let exc_class_name = env.CallObjectMethod0(exc_class, get_name_method);
            env.DeleteLocalRef(exc_class);
            let exc_class_name_str = env.GetStringUTFChars_as_string(exc_class_name).unwrap();
            env.DeleteLocalRef(exc_class_name);
            assert_eq!(exc_class_name_str.as_str(), "java.lang.UnsatisfiedLinkError");

            env.CallStaticVoidMethod1(registered_class, t2m, 754.156f64);
            assert!(env.ExceptionCheck());
            let exc = env.ExceptionOccurred();
            assert!(!exc.is_null());
            env.ExceptionClear();
            let exc_class = env.GetObjectClass(exc);
            env.DeleteLocalRef(exc);
            let exc_class_name = env.CallObjectMethod0(exc_class, get_name_method);
            env.DeleteLocalRef(exc_class);
            let exc_class_name_str = env.GetStringUTFChars_as_string(exc_class_name).unwrap();
            assert_eq!(exc_class_name_str.as_str(), "java.lang.UnsatisfiedLinkError");
            env.DeleteLocalRef(exc_class_name);

            let name = CString::new("test").unwrap();
            let sig1 = CString::new("(Ljava/lang/String;)V").unwrap();
            let sig2 = CString::new("(D)V").unwrap();

            let method1 = JNINativeMethod::new(name.as_ptr(), sig1.as_ptr(), t1 as *const c_void);
            let method2 = JNINativeMethod::new(name.as_ptr(), sig2.as_ptr(), t2 as *const c_void);

            assert_eq!(JNI_OK, env.RegisterNatives_slice(registered_class, &[method1, method2]));

            env.CallStaticVoidMethod1(registered_class, t2m, 754.156f64);
            assert!(!env.ExceptionCheck());

            env.CallStaticVoidMethod1(registered_class, t1m, test_string);
            assert!(!env.ExceptionCheck());

            env.UnregisterNatives(registered_class);

            env.CallStaticVoidMethod1(registered_class, t2m, 754.156f64);
            assert!(env.ExceptionCheck());
            let exc = env.ExceptionOccurred();
            assert!(!exc.is_null());
            env.ExceptionClear();
            let exc_class = env.GetObjectClass(exc);
            env.DeleteLocalRef(exc);
            let exc_class_name = env.CallObjectMethod0(exc_class, get_name_method);
            env.DeleteLocalRef(exc_class);
            let exc_class_name_str = env.GetStringUTFChars_as_string(exc_class_name).unwrap();
            assert_eq!(exc_class_name_str.as_str(), "java.lang.UnsatisfiedLinkError");
            env.DeleteLocalRef(exc_class_name);

            env.CallStaticVoidMethod1(registered_class, t1m, test_string);
            assert!(env.ExceptionCheck());
            let exc = env.ExceptionOccurred();
            assert!(!exc.is_null());
            env.ExceptionClear();
            let exc_class = env.GetObjectClass(exc);
            env.DeleteLocalRef(exc);
            let exc_class_name = env.CallObjectMethod0(exc_class, get_name_method);
            env.DeleteLocalRef(exc_class);
            let exc_class_name_str = env.GetStringUTFChars_as_string(exc_class_name).unwrap();
            assert_eq!(exc_class_name_str.as_str(), "java.lang.UnsatisfiedLinkError");
            env.DeleteLocalRef(exc_class_name);

            vm.DestroyJavaVM();
        }
    }
}
