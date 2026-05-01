#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::{
        JNI_CreateJavaVM_with_string_args, JNI_OK, JNI_VERSION_1_8, JVMTI_ERROR_NONE, JVMTI_THREAD_STATE_SUSPENDED, JVMTI_VERSION_1_2, JVMTIEnv, jobjectRefType, jthread,
        jvmtiCapabilities, load_jvm_from_java_home,
    };
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::SeqCst;
    use std::thread;
    use std::time::Duration;
    use sync_ptr::SyncMutPtr;

    static THREAD_HANDLE: Mutex<Vec<SyncMutPtr<c_void>>> = Mutex::new(Vec::new());
    static RUNNING: AtomicBool = AtomicBool::new(true);
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

            let mut jhs = Vec::new();

            for _ in 0..10 {
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
                        let mut g = THREAD_HANDLE.lock().unwrap();
                        g.push(SyncMutPtr::new(thread_glob));
                        drop(g);

                        let sys = env.FindClass("java/lang/System");
                        let get_prop = env.GetStaticMethodID(sys, "getProperty", "(Ljava/lang/String;)Ljava/lang/String;");
                        assert!(!get_prop.is_null());
                        let js = env.NewString_from_str("os.name");

                        loop {
                            if !RUNNING.load(SeqCst) {
                                env.DeleteLocalRef(js);
                                env.DeleteLocalRef(sys);
                                assert_eq!(vm.DetachCurrentThread(), JNI_OK);
                                return;
                            }
                            thread::sleep(Duration::from_millis(10));
                            let k = env.CallStaticObjectMethod1(sys, get_prop, js);
                            if !k.is_null() {
                                env.DeleteLocalRef(k);
                            }
                        }
                    })
                };

                jhs.push(jh);
            }

            let thread_handles = loop {
                thread::sleep(Duration::from_millis(100));
                for n in &jhs {
                    assert!(!n.is_finished());
                }
                let g = THREAD_HANDLE.lock().unwrap();
                if g.len() == jhs.len() {
                    break g.iter().map(SyncMutPtr::inner).collect::<Vec<jthread>>();
                }
            };

            let mut errors = vec![JVMTI_ERROR_NONE; thread_handles.len()];

            jvmti
                .SuspendThreadList(thread_handles.len() as _, thread_handles.as_ptr(), errors.as_mut_ptr())
                .expect("failed to call SuspendThreadList");
            for e in &errors {
                assert!(e.is_ok(), "{}", e);
            }

            thread::sleep(Duration::from_millis(1000));

            for h in &thread_handles {
                let mut ts = 0;
                jvmti.GetThreadState(*h, &raw mut ts).expect("Failed to get thread state");
                assert_ne!(ts & JVMTI_THREAD_STATE_SUSPENDED, 0, "{}", ts);
            }
            jvmti
                .ResumeThreadList(thread_handles.len() as _, thread_handles.as_ptr(), errors.as_mut_ptr())
                .expect("failed to ResumeThreadList");
            for e in &errors {
                assert!(e.is_ok(), "{}", e);
            }

            for h in thread_handles {
                let mut ts = 0;
                jvmti.GetThreadState(h, &raw mut ts).expect("Failed to get thread state");
                assert_eq!(ts & JVMTI_THREAD_STATE_SUSPENDED, 0, "{}", ts);
                env.DeleteGlobalRef(h);
            }

            RUNNING.store(false, SeqCst);
            for n in jhs {
                assert!(n.join().is_ok());
            }

            vm.DestroyJavaVM();
        }
    }
}
