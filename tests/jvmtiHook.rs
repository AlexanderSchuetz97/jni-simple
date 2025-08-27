#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::{
        jint, jniNativeInterface, load_jvm_from_java_home, JNIEnv, JNILinkage, JNI_CreateJavaVM_with_string_args, JVMTIEnv, JNI_VERSION_1_8, JVMTI_ERROR_NONE, JVMTI_VERSION_1_2,
    };
    use std::ffi::c_void;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::OnceLock;

    static ORIGINAL_FUNCTIONS: OnceLock<jniNativeInterface> = OnceLock::new();
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    extern "system" fn hooked_get_version(env: JNIEnv) -> jint {
        COUNTER.fetch_add(1, SeqCst);
        let guard = ORIGINAL_FUNCTIONS.get().unwrap();
        let result = unsafe { guard.get::<extern "system" fn(*mut c_void) -> jint>(JNILinkage::GetVersion)(env.vtable()) };
        result
    }

    fn install_hook(env: JVMTIEnv) {
        unsafe {
            _ = ORIGINAL_FUNCTIONS.get_or_init(|| {
                let mut iface = jniNativeInterface::new_uninit();
                assert_eq!(env.GetJNIFunctionTable(&mut iface), JVMTI_ERROR_NONE);
                iface
            });

            let mut iface = jniNativeInterface::new_uninit();
            assert_eq!(env.GetJNIFunctionTable(&mut iface), JVMTI_ERROR_NONE);
            iface.set(JNILinkage::GetVersion, hooked_get_version as _);
            assert_eq!(env.SetJNIFunctionTable(iface), JVMTI_ERROR_NONE);
        }
    }

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");
            let x = env.GetVersion();
            assert_eq!(0, COUNTER.load(SeqCst));
            install_hook(jvmti);
            let x2 = env.GetVersion();
            assert_eq!(1, COUNTER.load(SeqCst));
            assert_eq!(x, x2);
            let x2 = env.GetVersion();
            assert_eq!(2, COUNTER.load(SeqCst));
            assert_eq!(x, x2);
        }
    }
}
