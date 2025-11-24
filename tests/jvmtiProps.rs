#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "freebsd", target_os = "netbsd"))]
pub mod test {
    use jni_simple::*;
    use std::collections::HashMap;
    use std::ffi::{CStr, c_void};
    use std::ptr::null_mut;

    unsafe extern "C" fn shim_agent(vm: JavaVM, _options: *const char, _reserved: *mut c_void) -> i32 {
        unsafe {
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");
            let v = jvmti.SetSystemProperty("java.class.path", ".");
            if !v.is_ok() {
                //If this is JVMTI_ERROR_NOT_AVAILABLE then I have to find a different writable property here.
                eprintln!("failed to set JVMTI system property {}", v);
                println!("failed to set JVMTI system property {}", v);
                return 1;
            }
            0
        }
    }

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let ptr = shim_agent as usize;

            #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
            let args: Vec<String> = vec![format!("-agentpath:jvmti_shim/target/release/libjvmti_shim.so={ptr}")];
            #[cfg(target_os = "windows")]
            let args: Vec<String> = vec![format!("-agentpath:jvmti_shim\\target\\release\\jvmti_shim.dll={ptr}")];
            #[cfg(target_os = "macos")]
            let args: Vec<String> = vec![format!("-agentpath:jvmti_shim/target/release/libjvmti_shim.dylib={ptr}")];

            let (vm, _env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");
            let mut count = 0i32;
            let mut data = null_mut();

            assert!(jvmti.GetSystemProperties(&mut count, &mut data).is_ok());
            assert!(count > 0);
            let mut map = HashMap::new();
            for st in std::slice::from_raw_parts_mut(data, count as usize) {
                let mut value = null_mut();
                assert!(jvmti.GetSystemProperty(*st, &mut value).is_ok());
                map.insert(CStr::from_ptr(*st).to_string_lossy().to_string(), CStr::from_ptr(value).to_string_lossy().to_string());
                assert!(jvmti.Deallocate(value).is_ok());
            }

            println!("{:?}", map);

            assert!(map.contains_key("java.vm.version"));
            assert_eq!(map.get("java.class.path").unwrap().as_str(), ".");
        }
    }
}
