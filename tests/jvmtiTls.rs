#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use std::ptr::null_mut;
    use jni_simple::{load_jvm_from_java_home, JNI_CreateJavaVM_with_string_args, JVMTIEnv, JNI_VERSION_1_8, JVMTI_VERSION_1_2};

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("Failed to get jvmti env");
            let ptr = core::ptr::dangling_mut();

            let mut thread = null_mut();
            jvmti.GetCurrentThread(&raw mut thread).expect("failed to get current thread");
            jvmti.SetThreadLocalStorage(thread, ptr).expect("failed to set current thread local storage");
            let mut got = null_mut();
            jvmti.GetThreadLocalStorage(thread, &raw mut got).expect("failed to get current thread local storage");
            assert_eq!(ptr, got);
        }
    }
}