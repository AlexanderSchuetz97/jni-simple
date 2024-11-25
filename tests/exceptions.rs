#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::*;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create java VM");
            let throwable = env.FindClass("Ljava/lang/Throwable;");
            let throwable_constructor = env.GetMethodID(throwable, "<init>", "()V");
            let throwable_get_message = env.GetMethodID(throwable, "getMessage", "()Ljava/lang/String;");
            let throwable_instance = env.NewObject0(throwable, throwable_constructor);
            assert!(!env.ExceptionCheck());
            env.Throw(throwable_instance);
            assert!(env.ExceptionCheck());
            let throwable_thrown = env.ExceptionOccurred();
            assert!(!throwable_thrown.is_null());
            assert!(env.ExceptionCheck());
            env.ExceptionDescribe();
            assert!(!env.ExceptionCheck());
            env.ExceptionClear();
            assert!(!env.ExceptionCheck());
            env.ExceptionDescribe();
            assert!(!env.ExceptionCheck());
            env.Throw(throwable_instance);
            assert!(env.ExceptionCheck());
            env.ExceptionClear();
            assert!(!env.ExceptionCheck());
            let should_be_null = env.ExceptionOccurred();
            assert!(should_be_null.is_null());
            assert!(env.IsSameObject(throwable_thrown, throwable_instance));

            assert_eq!(JNI_OK, env.ThrowNew(throwable, "Some Error"));
            assert!(env.ExceptionCheck());
            let different_obj = env.ExceptionOccurred();
            env.ExceptionClear();
            assert!(!env.ExceptionCheck());
            assert!(!env.IsSameObject(throwable_thrown, different_obj));

            let message = env.CallObjectMethod0(different_obj, throwable_get_message);
            let rust_msg = env.GetStringUTFChars_as_string(message).unwrap();
            assert_eq!(rust_msg, "Some Error");

            assert_eq!(JNI_OK, env.ThrowNew(throwable, ()));

            assert!(env.ExceptionCheck());
            let another_obj = env.ExceptionOccurred();
            env.ExceptionClear();
            assert!(!env.ExceptionCheck());

            let message = env.CallObjectMethod0(another_obj, throwable_get_message);
            assert!(message.is_null());
            assert!(!env.IsSameObject(throwable_thrown, another_obj));
            assert!(!env.IsSameObject(different_obj, another_obj));

            vm.DestroyJavaVM();
        }
    }
}
