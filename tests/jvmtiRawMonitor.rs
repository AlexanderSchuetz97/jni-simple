#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::*;
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::atomic::{AtomicBool, AtomicPtr};
    use std::thread;
    use std::time::Duration;

    #[test]
    pub fn test() {
        unsafe {
            static WATCHDOG: AtomicBool = AtomicBool::new(false);
            static MONITOR: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

            let wd = thread::spawn(|| {
                for _ in 0..60 {
                    thread::sleep(Duration::from_millis(1000));
                    if WATCHDOG.load(SeqCst) {
                        return;
                    }
                }
                eprintln!("Test took to long aborting!");
                std::process::abort();
            });

            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");

            let mut raw: jrawMonitorID = null_mut();
            assert!(jvmti.CreateRawMonitor("raw monitor test", &raw mut raw).is_ok());
            assert!(jvmti.DestroyRawMonitor(raw).is_ok());

            assert!(jvmti.CreateRawMonitor("raw monitor test", &MONITOR).is_ok());
            assert!(!MONITOR.load(SeqCst).is_null());

            assert!(jvmti.RawMonitorEnter(&MONITOR).is_ok());

            let jh = thread::spawn(move || {
                let _env = vm.AttachCurrentThread_str(JNI_VERSION_1_8, "child", null_mut()).expect("failed to attach child");
                assert!(jvmti.RawMonitorEnter(&MONITOR).is_ok());
                thread::sleep(Duration::from_millis(100));
                assert!(jvmti.RawMonitorExit(&MONITOR).is_ok());
                _ = vm.DetachCurrentThread();
            });

            thread::sleep(Duration::from_millis(1000));
            assert!(!jh.is_finished());

            assert!(jvmti.RawMonitorExit(&MONITOR).is_ok());

            jh.join().expect("child failed");

            assert!(jvmti.RawMonitorEnter(&MONITOR).is_ok());

            let jh = thread::spawn(move || {
                let _env = vm.AttachCurrentThread_str(JNI_VERSION_1_8, "child", null_mut()).expect("failed to attach child");
                assert!(jvmti.RawMonitorEnter(&MONITOR).is_ok());
                assert!(jvmti.RawMonitorNotifyAll(&MONITOR).is_ok());
                assert!(jvmti.RawMonitorExit(&MONITOR).is_ok());
                _ = vm.DetachCurrentThread();
            });

            assert!(jvmti.RawMonitorWait(&MONITOR).is_ok());
            jh.join().expect("child failed");

            assert!(jvmti.RawMonitorExit(&MONITOR).is_ok());

            assert!(jvmti.RawMonitorEnter(&MONITOR).is_ok());

            let jh = thread::spawn(move || {
                let _env = vm.AttachCurrentThread_str(JNI_VERSION_1_8, "child", null_mut()).expect("failed to attach child");
                assert!(jvmti.RawMonitorEnter(&MONITOR).is_ok());
                assert!(jvmti.RawMonitorNotify(&MONITOR).is_ok());
                assert!(jvmti.RawMonitorExit(&MONITOR).is_ok());
                _ = vm.DetachCurrentThread();
            });

            assert!(jvmti.RawMonitorWait(&MONITOR).is_ok());
            jh.join().expect("child failed");

            assert!(jvmti.RawMonitorExit(&MONITOR).is_ok());

            assert!(jvmti.DestroyRawMonitor(&MONITOR).is_ok());
            WATCHDOG.store(true, SeqCst);
            wd.join().expect("wd failed");
        }
    }
}
