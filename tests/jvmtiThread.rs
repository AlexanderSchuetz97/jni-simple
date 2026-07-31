#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::*;
    use std::ptr::null_mut;

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("Failed to get jvmti env");

            let mut thread = null_mut();
            jvmti.GetCurrentThread(&raw mut thread).expect("failed get current thread");
            assert!(!thread.is_null());
            let mut thread_state = 0;
            jvmti.GetThreadState(thread, &raw mut thread_state).expect("failed to get thread state");
            assert_ne!(thread_state & JVMTI_THREAD_STATE_ALIVE, 0, "{}", thread_state);

            //This is probably impl specific... May fail for some jvm's!
            assert_ne!(thread_state & JVMTI_THREAD_STATE_RUNNABLE, 0, "{}", thread_state);

            let threads = jvmti.GetAllThreads_as_vec().expect("failed to get all threads");
            assert!(!threads.is_empty());

            let mut found = false;
            for iter_thr in threads {
                if found {
                    assert!(!env.IsSameObject(thread, iter_thr));
                } else {
                    found = env.IsSameObject(thread, iter_thr);
                }

                env.DeleteLocalRef(iter_thr);
            }

            assert!(found, "current thread not found in thread list");

            env.DeleteLocalRef(thread);
        }
    }
}
