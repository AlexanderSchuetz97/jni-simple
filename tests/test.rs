#[cfg(feature = "loadjvm")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::ptr::{null, null_mut};
    use std::sync::Mutex;

    //Cargo runs the tests on different threads.
    static MUTEX: Mutex<()> = Mutex::new(());

    unsafe fn load_it() -> (JavaVM, JNIEnv) {
        unsafe {
            if !is_jvm_loaded() {
                load_jvm_from_java_home().expect("failed to load jvm");
            }

            let thr = JNI_GetCreatedJavaVMs_first().expect("failed to get jvm");
            if thr.is_none() {
                //Adjust JVM version and arguments here; args are just like the args you pass on the command line.
                //You could provide your classpath here, for example, or configure the jvm heap size.
                //Default arguments (none) will do for this example.
                let args: Vec<String> = vec![];
                return JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create jvm");
            }

            let jvm = thr.unwrap().clone();
            let env = jvm.GetEnv(JNI_VERSION_1_8);
            if env.is_err() && env.unwrap_err() == JNI_EDETACHED {
                let env = jvm.AttachCurrentThread_str(JNI_VERSION_1_8, (), null_mut()).expect("failed to attach thread");
                return (jvm, env);
            }
            (jvm, env.unwrap())
        }
    }

    #[test]
    fn test() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let (_jvm, env) = load_it();

            //This code does not check for failure or exceptions checks or "checks" for success in general.
            let sys = env.FindClass("java/lang/System");
            let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
            let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
            //Calls System.nanoTime() and prints the result
            println!("{}", nanos);
        }
    }

    #[test]
    fn test_call2() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let (_jvm, env) = load_it();

            //This code does not check for failure or exceptions checks or "checks" for success in general.
            let sys = env.FindClass("java/lang/System");
            let get_prop = env.GetStaticMethodID(sys, "getProperty", "(Ljava/lang/String;)Ljava/lang/String;");

            let str = env.NewStringUTF("os.name");
            let obj = env.CallStaticObjectMethod1(sys, get_prop, str);
            let uw = env.GetStringUTFChars_as_string(obj).unwrap().to_lowercase();
            env.DeleteLocalRef(obj);
            env.DeleteLocalRef(str);

            #[cfg(target_os = "linux")]
            assert_eq!(uw, "linux");
            #[cfg(target_os = "windows")]
            assert!(uw.contains("windows"), "{}", &uw);
            #[cfg(target_os = "macos")]
            assert!(uw.contains("mac"), "{}", &uw);
            #[cfg(target_os = "freebsd")]
            assert_eq!(uw, "freebsd");

            let set_prop = env.GetStaticMethodID(sys, "setProperty", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;");
            let str = env.NewStringUTF("some_prop2");
            let obj = env.CallStaticObjectMethod1(sys, get_prop, str);
            assert!(obj.is_null());
            let obj = env.CallStaticObjectMethod2(sys, set_prop, str, str);
            assert!(obj.is_null());
            let obj = env.CallStaticObjectMethod1(sys, get_prop, str);
            assert!(!obj.is_null());
            let uw = env.GetStringUTFChars_as_string(obj).unwrap().to_lowercase();
            env.DeleteLocalRef(obj);
            env.DeleteLocalRef(str);
            assert_eq!(uw, "some_prop2");
        }
    }

    #[test]
    fn test_call() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let (_jvm, env) = load_it();

            //This code does not check for failure or exceptions checks or "checks" for success in general.
            let sys = env.FindClass("java/lang/System");
            let get_prop = env.GetStaticMethodID(sys, "getProperty", "(Ljava/lang/String;)Ljava/lang/String;");

            let str = env.NewStringUTF("os.name");
            let obj = env.CallStaticObjectMethodA(sys, get_prop, [str.into()].as_ptr());
            assert!(!obj.is_null());
            let uw = env.GetStringUTFChars_as_string(obj).unwrap().to_lowercase();
            env.DeleteLocalRef(obj);
            env.DeleteLocalRef(str);

            #[cfg(target_os = "linux")]
            assert_eq!(uw, "linux");
            #[cfg(target_os = "windows")]
            assert!(uw.contains("windows"), "{}", &uw);
            #[cfg(target_os = "macos")]
            assert!(uw.contains("mac"), "{}", &uw);
            #[cfg(target_os = "freebsd")]
            assert_eq!(uw, "freebsd");

            let set_prop = env.GetStaticMethodID(sys, "setProperty", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;");
            let str = env.NewStringUTF("some_prop");
            let obj = env.CallStaticObjectMethodA(sys, get_prop, [str.into()].as_ptr());
            assert!(obj.is_null());
            let obj = env.CallStaticObjectMethodA(sys, set_prop, [str.into(), str.into()].as_ptr());
            assert!(obj.is_null());
            let obj = env.CallStaticObjectMethodA(sys, get_prop, [str.into()].as_ptr());
            assert!(!obj.is_null());
            let uw = env.GetStringUTFChars_as_string(obj).unwrap().to_lowercase();
            env.DeleteLocalRef(obj);
            env.DeleteLocalRef(str);
            assert_eq!(uw, "some_prop");
        }
    }
}
