#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::*;
    use std::thread;

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("Failed to get jvmti env");

            let mut nproc = 0;
            jvmti.GetAvailableProcessors(&mut nproc).expect("failed to get nproc");
            assert_ne!(nproc, 0);
            assert!(nproc > 0);

            //TODO not 100% sure about this assertion.
            assert_eq!(thread::available_parallelism().map(|n| n.get()).unwrap_or_else(|e| nproc as usize), nproc as usize);
        }
    }
}
