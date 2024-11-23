#[cfg(feature = "loadjvm")]
pub mod test {
    use std::ptr::{null_mut};
    use jni_simple::*;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create java VM");
            let rt = env.GetObjectRefType(null_mut());
            assert_eq!(jobjectRefType::JNIInvalidRefType, rt);

            vm.DestroyJavaVM();
        }
    }
}