#[cfg(feature = "loadjvm")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::ptr::null_mut;
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::Duration;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let version = env.GetVersion();
            match version {
                JNI_VERSION_1_8 | JNI_VERSION_9 | JNI_VERSION_10 | JNI_VERSION_19 | JNI_VERSION_20 | JNI_VERSION_21 | JNI_VERSION_24 => (),
                _ => {
                    panic!("Invalid or unknown JVM JNI version {}. This test is only aware of versions up to 24. If the jvm is newer than this then point your JAVA_HOME to a jvm version >= 8 and <= 24", version);
                }
            }

            let vm_clone = vm.clone();
            std::thread::spawn(move || {
                assert_eq!(JNI_EDETACHED, vm_clone.GetEnv::<JNIEnv>(JNI_VERSION_1_8).unwrap_err());
                let _env = vm_clone.AttachCurrentThread_str(JNI_VERSION_1_8, (), null_mut()).unwrap();
                assert!(vm_clone.GetEnv::<JNIEnv>(JNI_VERSION_1_8).is_ok());
                let _env = vm_clone.AttachCurrentThread_str(JNI_VERSION_1_8, (), null_mut()).unwrap();
                assert_eq!(JNI_OK, vm_clone.DetachCurrentThread());
                assert_eq!(JNI_EDETACHED, vm_clone.GetEnv::<JNIEnv>(JNI_VERSION_1_8).unwrap_err());
                let env = vm_clone.AttachCurrentThread_str(JNI_VERSION_1_8, "HelloWorld", null_mut()).unwrap();
                let n = env.FindClass("java/lang/Thread");
                let gt = env.GetStaticMethodID(n, "currentThread", "()Ljava/lang/Thread;");
                let gn = env.GetMethodID(n, "getName", "()Ljava/lang/String;");
                let thread = env.CallStaticObjectMethod0(n, gt);
                let thread_name_j = env.CallObjectMethod0(thread, gn);
                let jn = env.GetStringUTFChars_as_string(thread_name_j).unwrap();
                assert_eq!("HelloWorld", jn.as_str());
                env.DeleteLocalRef(thread_name_j);
                env.DeleteLocalRef(thread);
                env.DeleteLocalRef(n);
                assert_eq!(JNI_OK, vm_clone.DetachCurrentThread());
                assert_eq!(JNI_OK, vm_clone.DetachCurrentThread());
            })
            .join()
            .unwrap();

            let vm_clone = env.GetJavaVM().unwrap();

            let l1 = Arc::new((Mutex::new(()), Condvar::new(), Condvar::new()));
            let l2 = l1.clone();
            let l3 = l2.clone();

            let guard = l1.0.lock().unwrap();

            std::thread::spawn(move || {
                let guard = l2.0.lock().unwrap();
                let _env = vm_clone.AttachCurrentThreadAsDaemon_str(JNI_VERSION_1_8, (), null_mut()).unwrap();
                assert!(vm_clone.GetEnv::<JNIEnv>(JNI_VERSION_1_8).is_ok());
                l2.1.notify_all();
                let _guard = l2.1.wait(guard).unwrap();
            });

            std::thread::spawn(move || {
                let env = vm_clone.AttachCurrentThreadAsDaemon_str(JNI_VERSION_1_8, "HelloWorld", null_mut()).unwrap();
                let n = env.FindClass("java/lang/Thread");
                let gt = env.GetStaticMethodID(n, "currentThread", "()Ljava/lang/Thread;");
                let gn = env.GetMethodID(n, "getName", "()Ljava/lang/String;");
                let thread = env.CallStaticObjectMethod0(n, gt);
                let thread_name_j = env.CallObjectMethod0(thread, gn);
                let jn = env.GetStringUTFChars_as_string(thread_name_j).unwrap();
                assert_eq!("HelloWorld", jn.as_str());
                assert_eq!(JNI_OK, vm_clone.DetachCurrentThread());
            })
            .join()
            .unwrap();

            let guard = l1.1.wait(guard).unwrap();
            let vm_clone = vm.clone();
            let jh = std::thread::spawn(move || {
                let _guard = l3.0.lock().unwrap();
                vm_clone.DestroyJavaVM();
                l3.2.notify_all();
            });
            assert_eq!(JNI_OK, vm_clone.DetachCurrentThread());

            let n = JNI_GetCreatedJavaVMs_first().unwrap();
            assert!(!n.is_none());

            let (_guard, tm) = l1.2.wait_timeout(guard, Duration::from_millis(5000)).unwrap();
            assert_eq!(false, tm.timed_out());
            jh.join().unwrap();
            l1.1.notify_all();
            let n = JNI_GetCreatedJavaVMs_first().unwrap();
            assert!(n.is_none());

            //This is a bit of "imagination" but j8 has this behavior.
            assert_eq!(JNI_ERR, vm.AttachCurrentThread_str(JNI_VERSION_1_8, (), null_mut()).unwrap_err());
        }
    }
}
