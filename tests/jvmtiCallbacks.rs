use jni_simple::jvmtiEventMode::JVMTI_ENABLE;
use jni_simple::{jboolean, jmethodID, jthread, jvalue, jvmtiCapabilities, jvmtiEvent, jvmtiEventCallbacks, load_jvm_from_java_home, JNIEnv, JNI_CreateJavaVM_with_string_args, JVMTIEnv, JavaVM, JNI_VERSION_1_8, JVMTI_VERSION_1_2};
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::OnceLock;

static DEBUGGER: OnceLock<JVMTIEnv> = OnceLock::new();
static COUNTER: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn shim_agent(vm: JavaVM, _options: *const char, _reserved: *mut c_void) -> i32 {
    let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");

    let mut cap = jvmtiCapabilities::default();
    cap.set_can_generate_method_exit_events(true);
    assert!(jvmti.AddCapabilities(&cap).is_ok());
    _= DEBUGGER.set(jvmti);
    0
}
extern "system" fn blah(_jvmti_env: JVMTIEnv, _jni_env: JNIEnv, _thread: jthread, _method: jmethodID, _was_popped_by_exception: jboolean, _return_value: jvalue) {
    COUNTER.fetch_add(1, SeqCst);
}


#[test]
pub fn test() {
    unsafe {
        load_jvm_from_java_home().expect("failed to load jvm");

        let ptr = shim_agent as usize;
        let args: Vec<String> = vec![format!("-agentpath:jvmti_shim/target/release/libjvmti_shim.so={ptr}")];

        let (_vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create java VM");
        let jvmti = DEBUGGER.get().copied().unwrap();

        let mut val = jvmtiEventCallbacks::default();
        val.MethodExit = Some(blah);
        assert!(jvmti.SetEventCallbacks(&val).is_ok());


        let sys = env.FindClass("java/lang/System");
        let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
        _= env.CallStaticLongMethodA(sys, nano_time, null());

        assert_eq!(COUNTER.load(SeqCst), 0);

        let g = jvmti.SetEventNotificationMode(JVMTI_ENABLE, jvmtiEvent::JVMTI_EVENT_METHOD_EXIT, null_mut());
        assert!(g.is_ok(), "{}", g.into_enum());
        _= env.CallStaticLongMethodA(sys, nano_time, null());
        assert_eq!(COUNTER.load(SeqCst), 1);
    }
}
