#[cfg(feature = "loadjvm")]
mod test {
    use std::ptr::{null, null_mut};
    use std::sync::Mutex;
    use jni_simple::*;

    static MUTEX: Mutex<()> = Mutex::new(());

    unsafe fn load_it() -> (JavaVM, JNIEnv){
        //Cargo runs the tests on different threads.
        let _lock = MUTEX.lock().unwrap();
        if !jni_simple::is_jvm_loaded() {
            // On linux/unix:
            jni_simple::load_jvm_from_library("/usr/lib/jvm/java-11-openjdk-amd64/lib/server/libjvm.so")
                .expect("failed to load jvm");

            // On windows:
            //    jni_simple::load_jvm_from_library("C:\\Program Files\\Java\\jdk-17.0.1\\jre\\bin\\server\\jvm.dll")
            //        .expect("failed to load jvm");
        }

        let thr = JNI_GetCreatedJavaVMs().expect("failed to get jvm");
        if thr.is_empty() {
            //Adjust JVM version and arguments here, args are just like the args you pass on the command line.
            //You could provide your classpath here for example or configure the jvm heap size.
            //Default arguments (none) will do for this example.
            let args : Vec<String> = vec![];
            return JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create jvm");
        }

        let jvm = thr.first().unwrap().clone();
        let env = jvm.GetEnv(JNI_VERSION_1_8);
        if env.is_err() && env.unwrap_err() == JNI_EDETACHED {
            let env = jvm.AttachCurrentThread_str(JNI_VERSION_1_8, None, null_mut()).expect("failed to attach thread");
            return (jvm, env);
        }
        return (jvm, env.unwrap());
    }



    #[test]
    fn test() {
        unsafe {
            let (_jvm, env) = load_it();

            //This code does not check for failure or exceptions checks or "checks" for success in general.
            let sys = env.FindClass_str("java/lang/System");
            let nano_time = env.GetStaticMethodID_str(sys, "nanoTime", "()J");
            let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
            //Calls System.nanoTime() and prints the result
            println!("{}", nanos);
        }
    }



    #[test]
    fn test_call() {
        unsafe {
            let (_jvm, env) = load_it();

            //This code does not check for failure or exceptions checks or "checks" for success in general.
            let sys = env.FindClass_str("java/lang/System");
            let get_prop = env.GetStaticMethodID_str(sys, "getProperty", "(Ljava/lang/String;)Ljava/lang/String;");

            let str = env.NewStringUTF_str("os.name");
            let obj = env.CallStaticObjectMethodA(sys, get_prop, [str.into()].as_ptr());
            assert!(!obj.is_null());
            let uw = env.GetStringUTFChars_as_string(obj).unwrap().to_lowercase();
            env.DeleteLocalRef(obj);
            env.DeleteLocalRef(str);

            #[cfg(target_os = "linux")]
            assert_eq!(uw, "linux");
            #[cfg(target_os = "windows")]
            assert_eq!(uw, "windows");


            let set_prop = env.GetStaticMethodID_str(sys, "setProperty", "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;");
            let str = env.NewStringUTF_str("some_prop");
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
