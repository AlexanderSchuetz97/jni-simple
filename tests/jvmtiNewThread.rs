#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::{JNI_CreateJavaVM_with_string_args, JNI_VERSION_1_8, JVMTI_VERSION_1_2, JVMTIEnv, load_jvm_from_java_home};
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::SeqCst;
    use std::thread;
    use std::thread::ThreadId;
    use std::time::Duration;

    static STATE: Mutex<Option<ThreadId>> = Mutex::new(None);
    static FAILED: AtomicBool = AtomicBool::new(false);
    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("Failed to get jvmti env");
            let thread_class = env.FindClass("java/lang/Thread");
            let thread_constructor = env.GetMethodID(thread_class, "<init>", "()V");
            let virign_thread = env.NewObject0(thread_class, thread_constructor);

            let mut jvmti_version = 0;
            assert!(jvmti.GetVersionNumber(&raw mut jvmti_version).is_ok());
            let jni_version = env.GetVersion();

            jvmti
                .RunAgentThread_fn(virign_thread, 1, move |jvmti, env| {
                    let jni_version_2 = env.GetVersion();
                    let mut jvmti_version_2 = 0;
                    if jvmti.GetVersionNumber(&raw mut jvmti_version_2).is_err() {
                        FAILED.store(true, SeqCst);
                    }

                    if jvmti_version_2 != jvmti_version {
                        FAILED.store(true, SeqCst);
                    }

                    if jni_version_2 != jni_version {
                        FAILED.store(true, SeqCst);
                    }

                    let mut g = STATE.lock().unwrap();
                    *g = Some(std::thread::current().id());
                    drop(g);
                })
                .expect("Failed to start thread");

            let oid = loop {
                thread::sleep(Duration::from_millis(100));
                if FAILED.load(SeqCst) {
                    panic!("Agent thread failed");
                }
                let st = STATE.lock().unwrap();
                if st.is_none() {
                    continue;
                }
                break st.unwrap().clone();
            };

            if FAILED.load(SeqCst) {
                panic!("Agent thread failed");
            }

            assert_ne!(thread::current().id(), oid);
        }
    }
}
