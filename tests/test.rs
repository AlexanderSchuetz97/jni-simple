#[cfg(feature = "loadjvm")]
mod test {
    use std::ptr::null;

    use jni_simple::*;

    #[test]
    fn test() {
        unsafe {

            // On linux/unix:
            jni_simple::load_jvm_from_library("/usr/lib/jvm/java-11-openjdk-amd64/lib/server/libjvm.so")
                .expect("failed to load jvm");

            // On windows:
            //    jni_simple::load_jvm_from_library("C:\\Program Files\\Java\\jdk-17.0.1\\jre\\bin\\server\\jvm.dll")
            //        .expect("failed to load jvm");


            //Adjust JVM version and arguments here, args are just like the args you pass on the command line.
            //You could provide your classpath here for example or configure the jvm heap size.
            //Default arguments (none) will do for this example.
            let args : Vec<String> = vec![];
            let (_jvm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create jvm");

            //This code does not check for failure or exceptions checks or "checks" for success in general.
            let sys = env.FindClass_str("java/lang/System");
            let nano_time = env.GetStaticMethodID_str(sys, "nanoTime", "()J");
            let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
            //Calls System.nanoTime() and prints the result
            println!("{}", nanos);
        }
    }


}
