#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::{
        JNI_CreateJavaVM_with_string_args, JNI_OK, JNI_VERSION_1_8, JVMTI_THREAD_STATE_SUSPENDED, JVMTI_VERSION_1_2, JVMTIEnv, jobjectRefType, jvmtiCapabilities,
        load_jvm_from_java_home,
    };
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::atomic::AtomicPtr;
    use std::sync::atomic::Ordering::SeqCst;
    use std::thread;
    use std::time::Duration;

    static THREAD_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("Failed to get jvmti env");
            let mut cap = jvmtiCapabilities::default();
            cap.set_can_suspend(true);

            jvmti.AddCapabilities(&raw const cap).expect("Failed to add capabilities");

            let jh = {
                let jvmti = jvmti.clone();
                let vm = vm.clone();
                thread::spawn(move || {
                    let env = vm.AttachCurrentThread_str(JNI_VERSION_1_8, (), null_mut()).expect("failed to get jni env");
                    let mut thread = null_mut();
                    jvmti.GetCurrentThread(&raw mut thread).expect("failed to get jvmti thread");
                    assert_eq!(env.GetObjectRefType(thread), jobjectRefType::JNILocalRefType);
                    let thread_glob = env.NewGlobalRef(thread);
                    env.DeleteLocalRef(thread);
                    assert!(!thread.is_null());
                    THREAD_HANDLE.store(thread_glob, SeqCst);
                    jvmti.SuspendThread(thread_glob).expect("failed to suspend");
                    assert_eq!(vm.DetachCurrentThread(), JNI_OK);
                })
            };

            let thread_glob = loop {
                assert!(!jh.is_finished());
                thread::sleep(Duration::from_millis(20));
                let thread_glob = THREAD_HANDLE.load(SeqCst);
                if thread_glob.is_null() {
                    continue;
                }
                break thread_glob;
            };

            thread::sleep(Duration::from_secs(1));

            assert!(!jh.is_finished());
            let mut ts = 0;
            jvmti.GetThreadState(thread_glob, &raw mut ts).expect("Failed to get thread state");
            assert_ne!(ts & JVMTI_THREAD_STATE_SUSPENDED, 0, "{}", ts);
            jvmti.ResumeThread(thread_glob).expect("failed to resume");
            assert!(jh.join().is_ok());
            env.DeleteGlobalRef(thread_glob);
            vm.DestroyJavaVM();
        }
    }
}
