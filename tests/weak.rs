#[cfg(feature = "loadjvm")]
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
            let clz = env.FindClass("Ljava/lang/Object;");
            let local = env.AllocObject(clz);
            let global = env.NewGlobalRef(local);
            let weak = env.NewWeakGlobalRef(local);
            let weak_class = env.GetObjectClass(weak);
            let weak_to_local = env.NewLocalRef(weak);
            match env.GetObjectRefType(weak_class) {
                jobjectRefType::JNILocalRefType => {}
                _ => panic!("{:?}", env.GetObjectRefType(weak_class)),
            }
            match env.GetObjectRefType(weak_to_local) {
                jobjectRefType::JNILocalRefType => {}
                _ => panic!("{:?}", env.GetObjectRefType(weak_to_local)),
            }

            assert!(env.IsInstanceOf(local, weak_class));
            assert!(env.IsSameObject(weak_class, clz));
            assert!(env.IsSameObject(weak, weak_to_local));
            assert!(env.IsInstanceOf(global, weak_class));
            assert!(env.IsInstanceOf(weak, weak_class));

            match env.GetObjectRefType(weak) {
                jobjectRefType::JNIWeakGlobalRefType => {}
                _ => panic!("{:?}", env.GetObjectRefType(weak)),
            }
            match env.GetObjectRefType(local) {
                jobjectRefType::JNILocalRefType => {}
                _ => panic!("{:?}", env.GetObjectRefType(local)),
            }
            match env.GetObjectRefType(global) {
                jobjectRefType::JNIGlobalRefType => {}
                _ => panic!("{:?}", env.GetObjectRefType(global)),
            }
            match env.GetObjectRefType(null_mut()) {
                jobjectRefType::JNIInvalidRefType => {}
                _ => panic!("{:?}", env.GetObjectRefType(null_mut())),
            }

            env.DeleteWeakGlobalRef(weak);
            env.DeleteGlobalRef(global);
            env.DeleteLocalRef(local);
            env.DeleteLocalRef(weak_class);
            env.DeleteLocalRef(clz);

            vm.DestroyJavaVM();
        }
    }
}
