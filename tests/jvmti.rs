use jni_simple::{jint, jvmtiCapabilities, load_jvm_from_library, JNIEnv, JNI_CreateJavaVM_with_string_args, JVMTIEnv, JavaVM, JNI_VERSION_1_8, JVMTI_VERSION_1_2};
use std::ffi::c_void;

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

//#[test]
pub fn test() {
    unsafe {
        load_jvm_from_library("~/.jdks/temurin-21.0.2/lib/server/libjvm.so").expect("failed to load jvm");

        let ptr = shim_agent as usize;

        let args: Vec<String> = vec![format!("-agentpath:~/RustroverProjects/jni-simple/jvmti_shim/target/debug/libjvmti_shim.so={ptr}")];
        //let args: Vec<String> = vec![];
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
    }
}
