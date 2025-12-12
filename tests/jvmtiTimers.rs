#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::*;
    use std::thread;

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("Failed to get jvmti env");

            let mut cap = jvmtiCapabilities::default();
            cap.set_can_get_current_thread_cpu_time(true);
            cap.set_can_get_thread_cpu_time(true);

            if !jvmti.AddCapabilities(&raw const cap).is_ok() {
                println!("Cannot run jvmtiTimers test because the jvm does not permit adding this capability");
            }

            let mut tm = 0;
            jvmti.GetCurrentThreadCpuTime(&raw mut tm).into_result().expect("failed to get current thread cpu time");
            assert_ne!(tm, 0);
            thread::sleep(std::time::Duration::from_millis(100));
            let mut tm2 = 0;
            jvmti.GetCurrentThreadCpuTime(&raw mut tm2).into_result().expect("failed to get current thread cpu time");
            assert_ne!(tm2, 0);
            assert_ne!(tm2, tm);

            jvmti.GetTime(&mut tm).into_result().expect("failed to get time");
            assert_ne!(tm2, 0);
            thread::sleep(std::time::Duration::from_millis(100));
            jvmti.GetTime(&mut tm2).into_result().expect("failed to get time");
            assert_ne!(tm2, 0);
            assert_ne!(tm2, tm);

            let mut timer_info = jvmtiTimerInfo::default();
            jvmti.GetTimerInfo(&raw mut timer_info).into_result().expect("failed to get timer info");
            assert_ne!(timer_info, jvmtiTimerInfo::default());
            assert!(
                timer_info.kind == JVMTI_TIMER_ELAPSED || timer_info.kind == JVMTI_TIMER_TOTAL_CPU || timer_info.kind == JVMTI_TIMER_USER_CPU,
                "{}",
                timer_info.kind
            )
        }
    }
}
