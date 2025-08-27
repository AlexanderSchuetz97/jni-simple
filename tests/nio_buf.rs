#[cfg(feature = "loadjvm")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::ptr::null_mut;
    use std::sync::Mutex;

    //Cargo runs the tests on different threads.
    static MUTEX: Mutex<()> = Mutex::new(());

    unsafe fn get_env() -> JNIEnv {
        if !is_jvm_loaded() {
            load_jvm_from_java_home().expect("failed to load jvm");
        }

        let thr = JNI_GetCreatedJavaVMs_first().expect("failed to get jvm");
        if thr.is_none() {
            //let args: Vec<String> = vec!["-Xcheck:jni".to_string()];
            //let args: Vec<String> = vec!["-Xint".to_string()];
            let args: Vec<String> = vec![];

            let (_, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create jvm");
            return env;
        }

        let jvm = thr.unwrap().clone();
        let env = jvm.GetEnv(JNI_VERSION_1_8);
        let env = env.unwrap_or_else(|c| {
            if c != JNI_EDETACHED {
                panic!("JVM ERROR {}", c);
            }

            jvm.AttachCurrentThread_str(JNI_VERSION_1_8, (), null_mut()).expect("failed to attach thread")
        });

        env
    }

    #[test]
    fn test_nio_buffer_from_rust() {
        let _lock = MUTEX.lock().unwrap();
        unsafe {
            let env = get_env();
            let mut some_buffer = [123i8; 512];
            let ptr = some_buffer.as_mut_ptr().cast();
            let dir_buf = env.NewDirectByteBuffer(ptr, some_buffer.len() as jlong);
            assert!(!dir_buf.is_null());
            assert_eq!(512, env.GetDirectBufferCapacity(dir_buf));
            assert_eq!(ptr, env.GetDirectBufferAddress(dir_buf));

            let buf_class = env.FindClass("java/nio/ByteBuffer");
            assert!(env.IsInstanceOf(dir_buf, buf_class));
            let get_next_byte = env.GetMethodID(buf_class, "get", "()B");
            let set_next_byte = env.GetMethodID(buf_class, "put", "(B)Ljava/nio/ByteBuffer;");
            let set_position = env.GetMethodID(buf_class, "position", "(I)Ljava/nio/Buffer;");

            for i in 0..512 {
                let b = env.CallByteMethod0(dir_buf, get_next_byte);
                assert_eq!(b, 123);
                assert_eq!(some_buffer[i], 123);
            }

            env.DeleteLocalRef(env.CallObjectMethod1(dir_buf, set_position, 0i32));

            for i in 0..512 {
                env.DeleteLocalRef(env.CallObjectMethod1(dir_buf, set_next_byte, i as i8));
                assert_eq!(some_buffer[i], i as i8);
            }

            env.DeleteLocalRef(dir_buf);
        }
    }
}
