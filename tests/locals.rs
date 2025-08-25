#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::*;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");

            let args: Vec<String> = vec![];

            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create jvm");

            let clazz = env.FindClass("java/lang/Object");
            assert_eq!(JNI_OK, env.EnsureLocalCapacity(128));
            assert_eq!(JNI_OK, env.PushLocalFrame(128));
            let obj = env.AllocObject(clazz);
            let n = env.NewGlobalRef(obj);
            let r = env.PopLocalFrame(obj);
            assert!(env.IsSameObject(r, n));
            vm.DestroyJavaVM();
        }
    }
}
