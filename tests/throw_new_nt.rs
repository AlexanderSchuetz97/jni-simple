#[cfg(feature = "loadjvm")]
#[cfg(feature = "asserts")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::panic;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec!["-Xcheck:jni".to_string()];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let obj_cl = env.FindClass("java/lang/Object");

            let n = panic::catch_unwind(|| env.ThrowNew(obj_cl, ()));
            assert!(n.is_err());

            env.DeleteLocalRef(obj_cl);
            vm.DestroyJavaVM();
        }
    }
}
