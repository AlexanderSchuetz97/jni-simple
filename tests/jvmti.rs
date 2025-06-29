use jni_simple::{
    jint, jniNativeInterface, jrawMonitorID, jvmtiCapabilities, load_jvm_from_java_home, load_jvm_from_library, JNIEnv, JNILinkage, JNI_CreateJavaVM_with_string_args, JVMTIEnv,
    JavaVM, JNI_VERSION_1_8, JVMTI_ERROR_NONE, JVMTI_VERSION_1_2,
};
use std::ffi::c_void;
use std::mem;
use std::ptr::null_mut;
use std::sync::OnceLock;

unsafe extern "C" fn shim_agent(vm: JavaVM, _options: *const char, _reserved: *mut c_void) -> i32 {
    println!("YES");
    let x = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");
    let mut v: jint = 0;
    let g = x.GetPhase(&mut v);
    println!("{:?}", g);
    println!("{:?}", v);
    let mut capa: jvmtiCapabilities = Default::default();
    let r = x.GetPotentialCapabilities(&mut capa);
    println!("{r} {}", capa);
    println!("{:?}", x);
    0
}

extern "system" fn blah(ti: JVMTIEnv, env: JNIEnv, _arg: *mut c_void) {
    unsafe {
        println!("blah");
        let n = env.GetVersion();
        let mut x = 0;
        ti.GetPhase(&mut x);
        println!("blah {} {}", n, x);
    }
}

static ORIGINAL_FUNCTIONS: OnceLock<jniNativeInterface> = OnceLock::new();

extern "system" fn hooked_get_version(env: JNIEnv) -> jint {
    println!("JNIEnv GetVersion will be called!");
    let guard = ORIGINAL_FUNCTIONS.get().unwrap();
    let result = unsafe { guard.get::<extern "system" fn(*mut c_void) -> jint>(JNILinkage::GetVersion)(env.vtable()) };
    println!("JNIEnv GetVersion returned {result}!");
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
        //load_jvm_from_java_home().expect("failed to load jvm");
        load_jvm_from_library("~/.jdks/openjdk-24.0.1/lib/server/libjvm.so").unwrap();

        let ptr = shim_agent as usize;

        //let args: Vec<String> = vec![format!("-agentpath:~/RustroverProjects/jni-simple/jvmti_shim/target/debug/libjvmti_shim.so={ptr}")];
        let args: Vec<String> = vec![];
        let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create java VM");
        let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");
        println!("{:?}", jvmti);
        let mut v: jint = 0;
        let g = jvmti.GetPhase(&mut v);
        println!("{:?}", g);
        println!("{:?}", v);

        let thread = env.FindClass("java/lang/Thread");
        let constr = env.GetMethodID(thread, "<init>", "()V");
        let _thread_obj = env.NewObject0(thread, constr);
        //        jvmti.RunAgentThread(thread_obj, blah, null_mut(), 1);
        //        thread::sleep(Duration::from_secs(5));
        let mut capa: jvmtiCapabilities = Default::default();
        let r = jvmti.GetPotentialCapabilities(&mut capa);
        println!("{r} {}", capa);

        install_hook(jvmti);

        let x = env.GetVersion();
        println!("FTAB {:?}", x);
    }
}
