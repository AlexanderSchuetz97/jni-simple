#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");

            let args: Vec<String> = vec!["-Drusttest=12345".to_string()];

            let (_, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create jvm");

            let sys = env.FindClass("java/lang/System");
            let get_prop = env.GetStaticMethodID(sys, "getProperty", "(Ljava/lang/String;)Ljava/lang/String;");

            let str = env.NewStringUTF("rusttest");
            let obj = env.CallStaticObjectMethodA(sys, get_prop, [str.into()].as_ptr());
            assert!(!obj.is_null());
            let uw = env.GetStringUTFChars_as_string(obj).unwrap().to_lowercase();
            assert_eq!("12345", uw.as_str());
            env.DeleteLocalRef(obj);
            env.DeleteLocalRef(str);
        }
    }
}
