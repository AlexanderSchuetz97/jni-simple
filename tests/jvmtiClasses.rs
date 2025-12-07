#[cfg(not(miri))]
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub mod test {
    use jni_simple::*;
    use std::ffi::CStr;
    use std::ptr::null_mut;

    #[test]
    pub fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec![];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2).expect("failed to get JVMTI environment");

            let system_class = env.FindClass("java/lang/System");
            assert!(!system_class.is_null());

            let mut found_my_class = false;

            let classes = jvmti.GetLoadedClasses_as_vec().expect("Failed to get loaded classes");
            assert!(!classes.is_empty());
            for c in classes {
                assert_eq!(env.GetObjectRefType(c), jobjectRefType::JNILocalRefType);
                if env.IsSameObject(c, system_class) {
                    found_my_class = true;
                }
                env.DeleteLocalRef(c);
            }

            assert!(found_my_class);

            let mut found_nano_time = false;

            let methods = jvmti.GetClassMethods_as_vec(system_class).expect("Failed to get methods");
            assert!(!methods.is_empty());
            for c in methods {
                let mut name = null_mut();
                assert!(jvmti.GetMethodName(c, &raw mut name, null_mut(), null_mut()).is_ok());
                let name_copy = CStr::from_ptr(name).to_string_lossy().to_string();
                assert!(jvmti.Deallocate(name).is_ok());
                //println!("{}", name_copy);
                if name_copy == "nanoTime" {
                    found_nano_time = true;
                }
            }

            assert!(found_nano_time);

            let fields = jvmti.GetClassFields_as_vec(system_class).expect("Failed to get fields");
            assert!(!fields.is_empty());
            let mut found_stdout = false;
            for c in fields {
                let mut name = null_mut();
                assert!(jvmti.GetFieldName(system_class, c, &raw mut name, null_mut(), null_mut()).is_ok());
                let name_copy = CStr::from_ptr(name).to_string_lossy().to_string();
                assert!(jvmti.Deallocate(name).is_ok());
                let mut mods = 0;
                assert!(jvmti.GetFieldModifiers(system_class, c, &raw mut mods).is_ok());

                if name_copy == "out" && (mods & (REFLECT_MODIFIER_STATIC | REFLECT_MODIFIER_PUBLIC)) != 0 {
                    found_stdout = true;
                }
            }

            assert!(found_stdout);

            let list_class = env.FindClass("java/util/List");
            assert!(!list_class.is_null());

            let interfaces = jvmti
                .GetImplementedInterfaces_as_vec(list_class)
                .expect("Failed to get implemented interfaces of java/util/List");
            assert!(!interfaces.is_empty());

            let mut collection_found = false;
            let mut names = Vec::new();

            for iface in interfaces {
                let mut class_name = null_mut();
                assert!(jvmti.GetClassSignature(iface, &raw mut class_name, null_mut()).is_ok());
                let name = CStr::from_ptr(class_name).to_string_lossy().to_string();
                assert!(jvmti.Deallocate(class_name).is_ok());
                if name == "Ljava/util/Collection;" || name == "Ljava/util/SequencedCollection;" {
                    collection_found = true;
                }
                names.push(name);
                env.DeleteLocalRef(iface);
            }

            assert!(
                collection_found,
                "Got superinterfaces of java/util/List = {}. Maybe in a future jdk they changed the interface hierarchy?",
                names.join(", ")
            );

            let class_blob = include_bytes!("../java_testcode/ThrowNew.class");
            let class_loaded = env.DefineClass_from_slice("ThrowNew", null_mut(), class_blob);
            assert!(!class_loaded.is_null());

            let mut minor = 154543645;
            let mut major = 55555552;
            assert!(jvmti.GetClassVersionNumbers(class_loaded, &raw mut minor, &raw mut major).is_ok());

            assert_eq!(minor, 0);
            assert_eq!(major, 52); //java8

            let alloc = env.AllocObject(class_loaded); //Ensure that the jvm actually initializes the class.

            let mut status = 0;
            assert!(jvmti.GetClassStatus(class_loaded, &raw mut status).is_ok());
            assert_eq!(status, JVMTI_CLASS_STATUS_INITIALIZED | JVMTI_CLASS_STATUS_PREPARED | JVMTI_CLASS_STATUS_VERIFIED);

            let mut modifiers = 0;
            assert!(jvmti.GetClassModifiers(class_loaded, &raw mut modifiers).is_ok());

            assert_ne!(modifiers & REFLECT_MODIFIER_PUBLIC, 0, "{modifiers}");
            assert_ne!(modifiers & REFLECT_MODIFIER_SYNCHRONIZED, 0, "{modifiers}"); //TODO figure out why

            env.DeleteLocalRef(alloc);
            env.DeleteLocalRef(class_loaded);

            let mut loader = std::ptr::dangling_mut();
            assert!(jvmti.GetClassLoader(system_class, &raw mut loader).is_ok());
            assert!(loader.is_null()); //Bootstrap loader means set to null, possibly different depending on JVM impl.
        }
    }
}
