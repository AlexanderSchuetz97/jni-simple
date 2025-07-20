use jni_simple::{jvmtiPhase, load_jvm_from_java_home, JNI_CreateJavaVM_with_string_args, JVMTIEnv, JNI_VERSION_1_8, JVMTI_ERROR_INVALID_ENVIRONMENT, JVMTI_PHASE_LIVE, JVMTI_VERSION_1_2};

#[test]
pub fn test() {
    unsafe {
        load_jvm_from_java_home().expect("failed to load jvm");
        let args: Vec<String> = vec![];
        let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create java VM");
        let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");
        let mut p = jvmtiPhase::default();
        assert!(jvmti.GetPhase(&mut p).is_ok());
        assert_eq!(JVMTI_PHASE_LIVE, p);
        assert!(jvmti.DisposeEnvironment().is_ok());
        assert_eq!(JVMTI_ERROR_INVALID_ENVIRONMENT, jvmti.GetPhase(&mut p));
    }
}
