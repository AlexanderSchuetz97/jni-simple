#[cfg(all(feature = "loadjvm", feature = "std"))]
mod test {
    use jni_simple::{LoadFromJavaHomeError, LoadFromJavaHomeFolderError, LoadFromLibraryError};
    use std::fs;

    #[test]
    fn test() {
        #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
        let path = "jvmti_shim/target/release/libjvmti_shim.so";
        #[cfg(target_os = "windows")]
        let path = "jvmti_shim\\target\\release\\jvmti_shim.dll";
        #[cfg(target_os = "macos")]
        let path = "jvmti_shim/target/release/libjvmti_shim.dylib";

        let err = unsafe { jni_simple::load_jvm_from_library(path).expect_err("successfully loaded jvm with bad shared object") };

        match &err {
            LoadFromLibraryError::JNICreateJavaVmNotFound { .. } => {
                //OK
            }
            LoadFromLibraryError::JNIGetCreatedJavaVMsNotFound { .. } => {
                //OK
            }
            _ => panic!("Did not expect {}", err),
        }

        let err = unsafe { jni_simple::load_jvm_from_java_home_folder("jvmti_shim").expect_err("successfully loaded jvm with bad java home") };

        assert!(matches!(err, LoadFromJavaHomeFolderError::UnknownJavaHomeLayout));

        #[cfg(unix)]
        if fs::exists("/tmp").unwrap_or_default() {
            //Circular symbolic link, an all-time classic.
            _ = fs::remove_dir("/tmp/jni_simple_test_dummy_link");
            _ = fs::remove_dir("/tmp/jni_simple_test_dummy_link2");
            _ = fs::remove_file("/tmp/jni_simple_test_dummy_link");
            _ = fs::remove_file("/tmp/jni_simple_test_dummy_link2");

            fs::create_dir("/tmp/jni_simple_test_dummy_link").expect("failed to create dir");
            std::os::unix::fs::symlink("/tmp/jni_simple_test_dummy_link", "/tmp/jni_simple_test_dummy_link2").expect("failed to create symlink 2");
            fs::remove_dir("/tmp/jni_simple_test_dummy_link").expect("failed to remove dir");
            std::os::unix::fs::symlink("/tmp/jni_simple_test_dummy_link2", "/tmp/jni_simple_test_dummy_link").expect("failed to create symlink 1");

            let err =
                unsafe { jni_simple::load_jvm_from_java_home_folder("/tmp/jni_simple_test_dummy_link/does_not_exist").expect_err("successfully loaded jvm with bad java home") };

            match &err {
                LoadFromJavaHomeFolderError::IOError(e) => {
                    let ets = e.kind().to_string();
                    //FileSystemLoop is still unstable as of 1.88
                    assert!(ets.contains("loop"), "{}", ets);
                }
                _ => panic!("unexpected error: {err}"),
            }

            _ = fs::remove_dir("/tmp/jni_simple_test_dummy_link");
            _ = fs::remove_dir("/tmp/jni_simple_test_dummy_link2");
            _ = fs::remove_file("/tmp/jni_simple_test_dummy_link");
            _ = fs::remove_file("/tmp/jni_simple_test_dummy_link2");
        }

        unsafe {
            jni_simple::load_jvm_from_java_home().expect("Failed to load jvm");
        }

        let err = unsafe { jni_simple::load_jvm_from_java_home().expect_err("successfully loaded jvm twice") };

        assert!(matches!(err, LoadFromJavaHomeError::AlreadyLoaded));
    }
}
