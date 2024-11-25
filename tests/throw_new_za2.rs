#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::*;
    use std::ptr::{null_mut};

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec!["-Xcheck:jni".to_string()];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create java VM");
            let class_blob = include_bytes!("../java_testcode/ThrowNewZa2.class");
            let class_loaded = env.DefineClass("ThrowNewZa2", null_mut(), class_blob);
            if class_loaded.is_null() {
                env.ExceptionDescribe();
                env.FatalError("failed to load class");
            }

            assert_eq!(JNI_OK, env.ThrowNew(class_loaded, "Test Message"));
            env.ExceptionDescribe();

            let field = env.GetStaticFieldID(class_loaded, "message", "Ljava/lang/String;");
            let obj = env.GetStaticObjectField(class_loaded, field);
            assert!(!obj.is_null());
            let str = env.GetStringUTFChars_as_string(obj).unwrap();
            env.DeleteLocalRef(obj);
            assert_eq!(str.as_str(), "Test Message");

            assert_eq!(JNI_OK, env.ThrowNew(class_loaded, ()));
            env.ExceptionDescribe();
            let obj = env.GetStaticObjectField(class_loaded, field);
            assert!(!obj.is_null());

            let str = env.GetStringUTFChars_as_string(obj).unwrap();
            assert_eq!(str.as_str(), "called");
            env.DeleteLocalRef(obj);
            env.DeleteLocalRef(class_loaded);
            vm.DestroyJavaVM();
        }
    }
}
