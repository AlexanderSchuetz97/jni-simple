#[cfg(feature = "loadjvm")]
#[cfg(not(feature = "asserts"))]
#[cfg(feature = "std")]
#[cfg(not(miri))]
pub mod test {
    use jni_simple::*;
    use std::ptr::null_mut;

    #[test]
    fn test() {
        unsafe {
            load_jvm_from_java_home().expect("failed to load jvm");
            let args: Vec<String> = vec!["-Xcheck:jni".to_string(), "-Xint".to_string()];
            let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create java VM");
            let class_blob = include_bytes!("../java_testcode/BoolTest.class");
            let class_loaded = env.DefineClass_from_slice("BoolTest", null_mut(), class_blob);
            if class_loaded.is_null() {
                env.ExceptionDescribe();
                env.FatalError("failed to load class");
            }

            let array_field = env.GetStaticFieldID(class_loaded, "ARRAY", "[Z");
            assert!(!array_field.is_null());
            let bool1_meth = env.GetStaticMethodID(class_loaded, "bool1", "()Z");
            assert!(!bool1_meth.is_null());
            let bool2_meth = env.GetStaticMethodID(class_loaded, "bool2", "()Z");
            assert!(!bool2_meth.is_null());
            let test_it_meth = env.GetStaticMethodID(class_loaded, "testIt", "()I");
            assert!(!test_it_meth.is_null());

            let array_instance = env.GetStaticObjectField(class_loaded, array_field);
            assert!(!array_instance.is_null());

            let k = env.CallStaticBooleanMethod0(class_loaded, bool1_meth);
            assert!(!env.ExceptionCheck()); //Suppresses warning from jvm
            assert!(!k);

            let k = env.CallStaticBooleanMethod0(class_loaded, bool2_meth);
            assert!(!env.ExceptionCheck()); //Suppresses warning from jvm
            assert!(!k);

            let k = env.CallStaticIntMethod0(class_loaded, test_it_meth);
            assert!(!env.ExceptionCheck()); //Suppresses warning from jvm
            assert_eq!(k, 0);

            let values = [false, true];
            env.SetBooleanArrayRegion(array_instance, 0, 2, values.as_ptr());
            let bools = env.GetBooleanArrayRegion_as_vec(array_instance, 0, None);
            assert_eq!(bools[0].as_wide(), 0);
            assert_eq!(bools[1].as_wide(), 1);

            let values = [jboolean::from_wide(44), jboolean::from_wide(83)];
            env.SetBooleanArrayRegion(array_instance, 0, 2, values.as_ptr());

            let bools = env.GetBooleanArrayRegion_as_vec(array_instance, 0, None);
            assert!(bools[0].as_bool());
            assert!(bools[1].as_bool());
            if bools[0].as_wide() != 44 {
                eprintln!("The JVM automatically narrows the values on access.");
                vm.DestroyJavaVM();
                return;
            }
            assert_eq!(bools[1].as_wide(), 83);

            let k = env.CallStaticIntMethod0(class_loaded, test_it_meth);
            assert!(!env.ExceptionCheck()); //Suppresses warning from jvm
            if k != 2 {
                eprintln!("The JVM treats wide bool8 values as false in branching logic");
                assert_eq!(k, 0);
            }

            let k1 = env.CallStaticBooleanMethod0(class_loaded, bool1_meth);
            assert!(!env.ExceptionCheck()); //Suppresses warning from jvm
            let k2 = env.CallStaticBooleanMethod0(class_loaded, bool2_meth);
            assert!(!env.ExceptionCheck()); //Suppresses warning from jvm

            if !k1 {
                //HOTSPOT JVM JAVA 25 GOES HERE!
                //I am not convinced we didnt cause UB in the JVM here as the branching behavior of the hotspot JVM is not consistent.
                //On return it narrows it to false, in branching however it assumes true.
                //It also appears to return true if the wide value is not divisible by true.
                //I think what the jvm is doing is using the JNZ instruction for branching
                //And on method return it returns the value using an "AND 0x1" mask.
                eprintln!("The JVM treats wide bool8 values as false when used as a return type. k1 = {k1} k2 = {k2} ");
                vm.DestroyJavaVM();
                return;
            }

            assert!(k2);

            vm.DestroyJavaVM();
        }
    }
}
