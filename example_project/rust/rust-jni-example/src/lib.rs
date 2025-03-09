#![allow(non_snake_case)]

use std::io::Write;
use jni_simple::{*};
use std::os::raw::{c_void};
use std::ptr::{null, null_mut};
use std::thread;
use std::time::Duration;
use std::io::stdout;

#[no_mangle]
pub unsafe extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
    //All error codes are jint, never JNI_OK. See JNI documentation for their meaning when you handle them.
    //This is a Result<JNIEnv, jint>.
    let env : JNIEnv = vm.GetEnv(JNI_VERSION_1_8).unwrap();


    //This code does not check for failure or exceptions checks or "checks" for success in general.
    let sys = env.FindClass("java/lang/System");
    let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
    let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
    println!("RUST: JNI_OnLoad {}", nanos);
    stdout().flush().unwrap();

    JNI_VERSION_1_8
}

//Would be called from java. the signature in java is org.example.JNITest#test()
#[no_mangle]
pub unsafe extern "system" fn Java_org_example_JNITest_test(env: JNIEnv, _class: jclass) {
    //This code does not check for failure or exceptions checks or "checks" for success in general.
    let sys = env.FindClass("java/lang/System");
    let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
    let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
    println!("RUST: Java_org_example_JNITest_test {}", nanos);
    stdout().flush().unwrap();


    thread::spawn(|| {
        thread::sleep(Duration::from_millis(2000));

        //This can be done anywhere in the application at any time.
        let vms : JavaVM = *jni_simple::JNI_GetCreatedJavaVMs().unwrap() // error code is once again a jint.
            .first().unwrap(); //There can only be one JavaVM per process as per oracle spec.

        //You could also provide a thread name or thread group here.
        let mut n = JavaVMAttachArgs::new(JNI_VERSION_1_8, null(), null_mut());
        vms.AttachCurrentThread(&mut n).unwrap();
        let env = vms.GetEnv(JNI_VERSION_1_8).unwrap();
        let sys = env.FindClass("java/lang/System");
        let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
        let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
        println!("RUST thread delayed: Java_org_example_JNITest_test {}", nanos);
        stdout().flush().unwrap();
        vms.DetachCurrentThread()
    });
}