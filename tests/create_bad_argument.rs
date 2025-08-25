#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::*;
    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");

            let args: Vec<String> = vec!["-Xmx1SAD".to_string()]; //This option is recognized but intentionally invalid.

            let error_code = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).unwrap_err();
            assert_eq!(error_code, JNI_EINVAL);
            let args: Vec<String> = vec!["-Xmx128M".to_string()];
            let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            vm.DestroyJavaVM();
        }
    }
}
