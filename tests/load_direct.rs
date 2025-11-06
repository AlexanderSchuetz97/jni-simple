#[cfg(feature = "loadjvm")]
mod test {
    use jni_simple::{JNI_CreateJavaVM_with_string_args, JNI_VERSION_1_8, load_jvm_from_library};
    use std::ptr::null;

    pub unsafe fn load_it() {
        let java_home = std::env::var("JAVA_HOME").expect("JAVA_HOME not set");
        ///All (most) jvm layouts that I am aware of on windows+linux+macos.
        const COMMON_LIBJVM_PATHS: &[&[&str]] = &[
            &["lib", "server", "libjvm.so"],                   //LINUX JAVA 11+
            &["jre", "lib", "amd64", "server", "libjvm.so"],   //LINUX JDK JAVA <= 8 amd64
            &["lib", "amd64", "server", "libjvm.so"],          //LINUX JRE JAVA <= 8 amd64
            &["jre", "lib", "aarch32", "server", "libjvm.so"], //LINUX JDK JAVA <= 8 arm 32
            &["lib", "aarch32", "server", "libjvm.so"],        //LINUX JRE JAVA <= 8 arm 32
            &["jre", "lib", "aarch64", "server", "libjvm.so"], //LINUX JDK JAVA <= 8 arm 64
            &["lib", "aarch64", "server", "libjvm.so"],        //LINUX JRE JAVA <= 8 arm 64
            //
            &["jre", "bin", "server", "jvm.dll"], //WINDOWS JDK <= 8
            &["bin", "server", "jvm.dll"],        //WINDOWS JRE <= 8 AND WINDOWS JDK/JRE 11+
            //
            &["jre", "lib", "server", "libjvm.dylib"],                     //MACOS Java <= 8
            &["Contents", "Home", "jre", "lib", "server", "libjvm.dylib"], //MACOS Java <= 8
            &["lib", "server", "libjvm.dylib"],                            //MACOS Java 11+
            &["Contents", "Home", "lib", "server", "libjvm.dylib"],        //MACOS Java 11+
        ];

        for parts in COMMON_LIBJVM_PATHS {
            let mut buf = std::path::PathBuf::from(&java_home);
            for part in *parts {
                buf.push(part);
            }

            if buf.try_exists().unwrap_or(false) {
                let full_path = buf.to_str().expect("failed to convert path to string");
                unsafe {
                    load_jvm_from_library(full_path).expect("Loading JVM failed");
                    return;
                }
            }
        }

        panic!("Loading JVM from {} failed", java_home);
    }

    #[test]
    fn test() {
        unsafe {
            load_it();

            let args: Vec<String> = vec![];
            let (_, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args, false).expect("failed to create jvm");

            let sys = env.FindClass("java/lang/System");
            let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
            let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
            assert_ne!(nanos, 0);
        }
    }
}
