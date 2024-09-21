#[cfg(feature = "loadjvm")]
pub mod test {
    use jni_simple::*;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");

            let args: Vec<String> = vec![];

            let (_, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args)
                .expect("failed to create jvm");

            let array_list_class = env.FindClass_str("java/util/ArrayList");
            let array_list_constructor = env.GetMethodID_str(array_list_class, "<init>", "()V");
            let array_list_instance = env.NewObject0(array_list_class, array_list_constructor);
            let abstract_list_class = env.GetSuperclass(array_list_class);
            assert!(!abstract_list_class.is_null());
            assert!(env.IsInstanceOf(array_list_instance, abstract_list_class));
            assert!(!env.IsInstanceOf(abstract_list_class, array_list_class));
            assert!(!env.IsAssignableFrom(abstract_list_class, array_list_class));
            assert!(env.IsAssignableFrom(array_list_class, abstract_list_class));

        }
    }
}