#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::*;
    use std::ptr::null_mut;

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("Failed to get jvmti env");

            let mut memory = null_mut();
            assert!(jvmti.Allocate(64, &raw mut memory).is_ok());
            assert!(!memory.is_null());

            std::ptr::write_bytes(memory, 0, 64);
            std::slice::from_raw_parts_mut(memory, 64).fill(1);
            assert_eq!(1, std::ptr::read_volatile(memory));
            assert_eq!(1, std::ptr::read_volatile(memory.wrapping_add(5)));
            assert!(jvmti.Deallocate(memory).is_ok());
        }
    }
}
