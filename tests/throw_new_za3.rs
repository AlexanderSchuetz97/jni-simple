#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::ptr::null_mut;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let class_blob = include_bytes!("../java_testcode/ThrowNewZa3.class");
            let class_loaded = env.DefineClass_from_slice("ThrowNewZa3", null_mut(), class_blob);
            if class_loaded.is_null() {
                env.ExceptionDescribe();
                env.FatalError("failed to load class");
            }

            assert_eq!(JNI_OK, env.ThrowNew(class_loaded, ()));
            env.ExceptionDescribe();

            let field = env.GetStaticFieldID(class_loaded, "message", "Ljava/lang/String;");
            let obj = env.GetStaticObjectField(class_loaded, field);
            assert!(obj.is_null());

            env.DeleteLocalRef(class_loaded);
            vm.DestroyJavaVM();
        }
    }
}
