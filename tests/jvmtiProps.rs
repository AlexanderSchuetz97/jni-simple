use jni_simple::*;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ptr::null_mut;

#[test]
pub fn test() {
    unsafe {
        load_jvm_from_java_home().expect("failed to load jvm");
        let args: Vec<String> = vec![];
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
        }

        println!("{:?}", map);

        assert!(map.contains_key("java.vm.version"));
        assert_eq!(map.get("java.class.path").unwrap().as_str(), "");
    }
}
