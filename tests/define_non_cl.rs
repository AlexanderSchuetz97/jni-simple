#[cfg(feature = "loadjvm")]
#[cfg(feature = "asserts")]
pub mod test {
    use jni_simple::*;
    use std::panic;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec!["-Xcheck:jni".to_string()];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let cl = env.FindClass("java/lang/Object");
            assert!(!cl.is_null());
            let class_blob = include_bytes!("../java_testcode/ThrowNewZa.class");
            panic::catch_unwind(|| {
                //cl is not a classloader. This is UB in the JVM.
                env.DefineClass_from_slice("ThrowNewZa", cl, class_blob);
            })
            .expect_err("Error expected");
            vm.DestroyJavaVM();
        }
    }
}
