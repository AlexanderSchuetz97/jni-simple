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
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");

            let classes = jvmti.GetLoadedClasses_as_vec().expect("Failed to get loaded classes");
            assert!(!classes.is_empty());
            for c in classes {
                assert_eq!(env.GetObjectRefType(c), jobjectRefType::JNILocalRefType);
                env.DeleteLocalRef(c);
            }
        }
    }
}
