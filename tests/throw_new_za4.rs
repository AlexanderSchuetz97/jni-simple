#[cfg(feature = "loadjvm")]
#[cfg(feature = "asserts")]
pub mod test {
    use std::panic;
    use jni_simple::*;
    use std::ptr::{null, null_mut};

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec!["-Xcheck:jni".to_string()];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create java VM");
            let class_blob = include_bytes!("../java_testcode/ThrowNewZa4.class");
            let class_loaded = env.DefineClass_str("ThrowNewZa4", null_mut(), class_blob);
            if class_loaded.is_null() {
                env.ExceptionDescribe();
                env.FatalError_str("failed to load class");
            }

            let n = panic::catch_unwind(|| {
                env.ThrowNew_str(class_loaded, "test")
            });
            assert!(n.is_err());

            env.DeleteLocalRef(class_loaded);
            vm.DestroyJavaVM();
        }
    }
}