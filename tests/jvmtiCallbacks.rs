#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::jvmtiEventMode::JVMTI_ENABLE;
    use jni_simple::{
        JNI_CreateJavaVM_with_string_args, JNI_VERSION_1_8, JNIEnv, JVMTI_VERSION_1_2, JVMTIEnv, JavaVM, jboolean, jmethodID, jthread, jvalue, jvmtiCapabilities, jvmtiEvent,
        jvmtiEventCallbacks, load_jvm_from_java_home,
    };
    use std::ffi::{CStr, c_void};
    use std::ptr::{null, null_mut};
    use std::sync::OnceLock;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::atomic::{AtomicI64, AtomicUsize};

    static DEBUGGER: OnceLock<JVMTIEnv> = OnceLock::new();
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    static LAST_VALUE: AtomicI64 = AtomicI64::new(0);

    unsafe extern "C" fn shim_agent(vm: JavaVM, _options: *const char, _reserved: *mut c_void) -> i32 {
        unsafe {
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");

            let mut cap = jvmtiCapabilities::default();
            cap.set_can_generate_method_exit_events(true);
            assert!(jvmti.AddCapabilities(&cap).is_ok());
            _ = DEBUGGER.set(jvmti);
            0
        }
    }
    extern "system" fn blah(jvmti_env: JVMTIEnv, _jni_env: JNIEnv, _thread: jthread, method: jmethodID, _was_popped_by_exception: jboolean, return_value: jvalue) {
        unsafe {
            COUNTER.fetch_add(1, SeqCst);
            let mut name = null_mut();
            assert!(jvmti_env.GetMethodName(method, &mut name, null_mut(), null_mut()).is_ok());
            let r_name = CStr::from_ptr(name).to_string_lossy().to_string();
            if r_name.as_str() == "nanoTime" {
                let long = return_value.long();
                LAST_VALUE.store(long, SeqCst);
            }
        }
    }

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");

            let ptr = shim_agent as usize;
            let args: Vec<String> = vec![format!("-agentpath:jvmti_shim/target/release/libjvmti_shim.so={ptr}")];

            let (_vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = DEBUGGER.get().copied().unwrap();

            let mut val = jvmtiEventCallbacks::default();
            val.MethodExit = Some(blah);
            assert!(jvmti.SetEventCallbacks(&val).is_ok());

            let sys = env.FindClass("java/lang/System");
            let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
            _ = env.CallStaticLongMethodA(sys, nano_time, null());

            assert_eq!(COUNTER.load(SeqCst), 0);

            let g = jvmti.SetEventNotificationMode(JVMTI_ENABLE, jvmtiEvent::JVMTI_EVENT_METHOD_EXIT, null_mut());
            assert!(g.is_ok(), "{}", g.into_enum());
            let val = env.CallStaticLongMethodA(sys, nano_time, null());
            #[cfg(feature = "asserts")]
            assert_eq!(COUNTER.load(SeqCst), 3); //The asserts feature also calls a bunch of methods, this 3 is not set in stone and can be changed.
            #[cfg(not(feature = "asserts"))]
            assert_eq!(COUNTER.load(SeqCst), 1);

            assert_eq!(LAST_VALUE.load(SeqCst), val);
        }
    }
}
