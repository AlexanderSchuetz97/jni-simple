#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::*;

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("Failed to get jvmti env");
            let mut cap = jvmtiCapabilities::default();
            cap.set_can_get_source_file_name(true);
            jvmti.AddCapabilities(&raw const cap).expect("Failed to add cap to get source file name");

            let system = env.FindClass("java/lang/System");
            assert!(!system.is_null());

            let sfn = jvmti.GetSourceFileName_as_string(system).expect("failed to get source file name");
            assert_eq!(sfn, "System.java");

            jvmti.RelinquishCapabilities(&raw const cap).expect("Failed to relinquish cap");
            let err = jvmti.GetSourceFileName_as_string(system).expect_err("this should have failed");
            assert_eq!(JVMTI_ERROR_MUST_POSSESS_CAPABILITY, err);
        }
    }
}