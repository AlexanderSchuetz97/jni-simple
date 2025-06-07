use jni_simple::*;
use std::ffi::c_void;

#[unsafe(no_mangle)]
extern "system" fn Agent_OnLoad(vm: JavaVM, _command_line_options: *const char, _: *mut c_void) -> i32 {
    unsafe {
        let Ok(jvmti) = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2) else {
            println!("Agent_OnLoad failed to get JVMTI_VERSION_1_2 environment");
            return -1;
        };

        let mut cap = jvmtiCapabilities::default();
        let err = jvmti.GetPotentialCapabilities(&mut cap);
        if err != JVMTI_ERROR_NONE {
            println!("Agent_OnLoad failed to get potential capabilities error {err}");
            return -1;
        }

        if !cap.can_redefine_any_class() {
            println!("Agent_OnLoad error: JavaVM cannot redefine classes");
            return -1;
        }

        //....
        0
    }
}
