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
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            if env.GetVersion() < JNI_VERSION_19 {
                vm.DestroyJavaVM();
                return;
            }

            let jvmti = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_19).expect("Failed to get jvmti env");
            let system_class = env.FindClass("java/lang/System");
            assert!(!system_class.is_null());
            let sys_mod = env.GetModule(system_class);
            assert!(!sys_mod.is_null());

            let mod_class = env.FindClass("java/lang/Module");
            assert!(!mod_class.is_null());
            let mod_get_name = env.GetMethodID(mod_class, "getName", "()Ljava/lang/String;");
            assert!(!mod_get_name.is_null());

            let class_class = env.FindClass("java/lang/Class");
            assert!(!class_class.is_null());
            let get_classloader = env.GetMethodID(class_class, "getClassLoader", "()Ljava/lang/ClassLoader;");
            assert!(!get_classloader.is_null());

            let system_classloader = env.CallObjectMethod0(system_class, get_classloader);
            assert!(system_classloader.is_null());

            let modules = jvmti.GetAllModules_as_vec().expect("Failed to get modules");
            assert!(!modules.is_empty());

            let mut found_sys_mod = false;

            for module in modules {
                if env.IsSameObject(sys_mod, module) {
                    found_sys_mod = true;
                }

                env.DeleteLocalRef(module);
            }

            assert!(found_sys_mod);

            let mod_name = env.CallObjectMethod0(sys_mod, mod_get_name);
            assert!(!mod_name.is_null());
            let mod_name_str = env.GetStringChars_as_string(mod_name).expect("Failed to get mod name");

            assert_eq!("java.base", mod_name_str);

            let mut sys_mod2 = null_mut();
            jvmti
                .GetNamedModule(system_classloader, "java/lang", &raw mut sys_mod2)
                .into_result()
                .expect("GetNamedModule failed");
            assert!(!sys_mod2.is_null());
            assert!(env.IsSameObject(sys_mod2, sys_mod));

            let class_blob = include_bytes!("../java_testcode/ThrowNew.class");
            let class_loaded = env.DefineClass_from_slice("ThrowNew", null_mut(), class_blob);
            assert!(!class_loaded.is_null());

            let alloc = env.AllocObject(class_loaded); //Force the class to initalize
            assert!(!alloc.is_null());
            env.DeleteLocalRef(alloc);

            let class_loaded_module = env.GetModule(class_loaded);
            assert!(!class_loaded_module.is_null());

            let class_loaded_mod_name = env.CallObjectMethod0(class_loaded_module, mod_get_name);
            assert!(class_loaded_mod_name.is_null());

            jvmti.AddModuleReads(sys_mod, class_loaded_module).into_result().expect("Failed to update module");
            jvmti
                .AddModuleOpens(sys_mod, "java.lang", class_loaded_module)
                .into_result()
                .expect("Failed to update module 2");
        }
    }
}
