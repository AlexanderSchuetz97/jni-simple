#[cfg(feature = "loadjvm")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::ptr::null_mut;
    use std::sync::{Arc, Condvar, Mutex};
    use sync_ptr::FromMutPtr;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let clz = env.FindClass("Ljava/lang/Object;");
            let local = env.AllocObject(clz);
            let global = env.NewGlobalRef(local);
            env.DeleteLocalRef(local);
            env.MonitorEnter(global);

            let l1 = Arc::new((Mutex::new(()), Condvar::new()));
            let l2 = l1.clone();

            let sp = global.as_sync_mut();

            let vm_clone = vm.clone();
            let jh = std::thread::spawn(move || {
                let global: jobject = sp.into();
                let env = vm_clone.AttachCurrentThread_str(JNI_VERSION_1_8, (), null_mut()).unwrap();
                env.MonitorEnter(global);
                let _g2 = l2.0.lock().unwrap();
                l2.1.notify_all();
                env.MonitorExit(global);
                let _ = vm_clone.DetachCurrentThread();
            });

            let g = l1.0.lock().unwrap();
            let (g, t) = l1.1.wait_timeout(g, std::time::Duration::from_secs(1)).unwrap();
            assert!(t.timed_out());
            env.MonitorExit(global);
            let (_g, t) = l1.1.wait_timeout(g, std::time::Duration::from_secs(5)).unwrap();
            assert!(!t.timed_out());
            jh.join().unwrap();
            env.DeleteGlobalRef(global);
            vm.DestroyJavaVM();
        }
    }
}
