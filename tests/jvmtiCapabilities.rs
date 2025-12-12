#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::*;

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");

            let mut got_cap = jvmtiCapabilities::default();
            got_cap.set_can_get_thread_cpu_time(true);

            jvmti.GetCapabilities(&raw mut got_cap).expect("failed to get capabilities");
            assert!(!got_cap.can_get_thread_cpu_time());

            let mut sl = [0; jvmtiCapabilities::size()];
            got_cap.copy_to_slice(&mut sl);
            for n in sl {
                assert_eq!(n, 0, "{:?}", got_cap);
            }

            let mut cap = jvmtiCapabilities::default();
            assert!(jvmti.GetPotentialCapabilities(&raw mut cap).is_ok());
            let mut raw_copy = vec![0u8; jvmtiCapabilities::size()];
            cap.copy_to_slice(raw_copy.as_mut_slice());
            let mut cap2 = jvmtiCapabilities::default();
            cap2.copy_from_slice(raw_copy.as_slice());

            assert_eq!(cap, cap2);

            assert_eq!(cap.to_string(), cap2.to_string());

            println!("{}", cap);

            if cap.can_generate_early_vmstart() {
                //Ordinary hotspot jvm does not have this capability in this context.
                println!("Cannot do the rest of the test because you jvm has more capabilities than expected...");
                return;
            }

            cap2.set_can_generate_early_vmstart(true);
            assert_ne!(cap, cap2);
            assert_ne!(cap.to_string(), cap2.to_string());

            cap2.set_can_generate_early_vmstart(false);

            assert_eq!(cap, cap2);
            assert_eq!(cap.to_string(), cap2.to_string());

            let mut cap = jvmtiCapabilities::default();
            cap.set_can_generate_early_vmstart(true);
            assert_eq!(jvmti.AddCapabilities(&cap), JVMTI_ERROR_NOT_AVAILABLE);
        }
    }
}
