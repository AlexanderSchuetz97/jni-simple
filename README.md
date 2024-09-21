# jni-simple

This crate contains a simple dumb handwritten rust wrapper around the JNI (Java Native Interface) API.
It does absolutely no magic around the JNI Calls and lets you just use them as you would in C.

### Examples
#### Loading a JVM on from a shared object file or dll
Note: this example assumes the loadjvm feature is enabled!
```rust
use jni_simple::{*};
use std::ptr::null;

#[test]
fn test() {
    unsafe {

        // On linux/unix:
        jni_simple::load_jvm_from_library("/usr/lib/jvm/java-11-openjdk-amd64/lib/server/libjvm.so")
            .expect("failed to load jvm");
       
        // On windows:
        //    jni_simple::load_jvm_from_library("C:\\Program Files\\Java\\jdk-17.0.1\\jre\\bin\\server\\jvm.dll")
        //        .expect("failed to load jvm");

        // Works on Both, but requires JAVA_HOME environment variable to be set.
        // This is usually done by the java installer.
        // Fails if JAVA_HOME is not set.
        //jni_simple::load_jvm_from_java_home().expect("failed to load jvm");
        

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
```
#### Writing a JNI shared library that implements a native method
The complete version of this example can be found in the repository inside the example_project folder.
```rust
#![allow(non_snake_case)]

use std::io::Write;
use jni_simple::{*};
use std::os::raw::{c_void};
use std::ptr::{null, null_mut};
use std::thread;
use std::time::Duration;
use std::io::stdout;

//Optional: Only needed if you need to spawn "rust" threads that need to interact with the JVM.
extern "system" {
    fn JNI_CreateJavaVM(invoker: *mut c_void, env: *mut c_void, initargs: *mut c_void) -> jint;
    fn JNI_GetCreatedJavaVMs(array: *mut c_void, len: jsize, out: *mut jsize) -> jint;
}

#[no_mangle]
pub unsafe extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
    //Optional: Only needed if you need to spawn "rust" threads that need to interact with the JVM.
    jni_simple::init_dynamic_link(JNI_CreateJavaVM as *mut c_void, JNI_GetCreatedJavaVMs as *mut c_void);

    //All error codes are jint, never JNI_OK. See JNI documentation for their meaning when you handle them.
    //This is a Result<JNIEnv, jint>.
    let env : JNIEnv = vm.GetEnv(JNI_VERSION_1_8).unwrap();


    //This code does not check for failure or exceptions checks or "checks" for success in general.
    let sys = env.FindClass_str("java/lang/System");
    let nano_time = env.GetStaticMethodID_str(sys, "nanoTime", "()J");
    let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
    println!("RUST: JNI_OnLoad {}", nanos);
    stdout().flush().unwrap();

    return JNI_VERSION_1_8;
}

//Would be called from java. the signature in java is org.example.JNITest#test()
#[no_mangle]
pub unsafe extern "system" fn Java_org_example_JNITest_test(env: JNIEnv, _class: jclass) {
    //This code does not check for failure or exceptions checks or "checks" for success in general.
    let sys = env.FindClass_str("java/lang/System");
    let nano_time = env.GetStaticMethodID_str(sys, "nanoTime", "()J");
    let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
    println!("RUST: Java_org_example_JNITest_test {}", nanos);
    stdout().flush().unwrap();


    thread::spawn(|| {
        thread::sleep(Duration::from_millis(2000));

        //This can be done anywhere in the application at any time.
        let vms : JavaVM = jni_simple::JNI_GetCreatedJavaVMs().unwrap() // error code is once again a jint.
            .first().unwrap().clone(); //There can only be one JavaVM per process as per oracle spec.

        //You could also provide a thread name or thread group here.
        let mut n = JavaVMAttachArgs::new(JNI_VERSION_1_8, null(), null_mut());
        vms.AttachCurrentThread(&mut n).unwrap();
        let env = vms.GetEnv(JNI_VERSION_1_8).unwrap();
        let sys = env.FindClass_str("java/lang/System");
        let nano_time = env.GetStaticMethodID_str(sys, "nanoTime", "()J");
        let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
        println!("RUST thread delayed: Java_org_example_JNITest_test {}", nanos);
        stdout().flush().unwrap();
        vms.DetachCurrentThread();
    });
}
```

### Main goals of this crate

#### Dont pretend that JNI is "safe"
JNI is inherently unsafe (from a rust point of view) and any attempt to enforce safety will lead to 
performance or API complexity issues. All JNI methods provided by this crate are 
marked as "unsafe" as they should be.

#### Simple type system of JNI is kept as is
All types like jobject, jstring, jarray,... which are opaque handles represented as pointers in C are 
represented as raw opaque pointers in Rust that are type aliases of each other. 
This essentially makes them just hint to the user and doesn't enforce any type safety as that would sometimes
be a big hindrance when working with JNI.

#### Designed for runtime dynamic linking of the JVM
The Problem: The existing jni crate depends on the jni-sys crate which requires the JVM to be resolvable by the dynamic linker.
There are 2 ways to do this. The first is to statically link the JVM into the binary, this is rarely done, 
very cumbersome and poorly documented. The other is to provide the JVM on the linker path so ldd can find it, 
but I have never seen this occur in the real world either.

This crate is developed for the more common use case that the JVM is available somewhere on the system and leaves it up to the user of 
the crate to write the necessary code to find and load the JVM. 

This allows for maximum flexibility when writing a launcher app which for example may first download a JVM from the internet.
As should be obvious, when writing a native library that does not launch the JVM itself and 
is loaded by `System.load` or `System.loadLibrary` then this is irrelevant.

### Features

#### loadjvm
This feature provides functions to dynamically link the jvm using the `libloading` crate 
from a string containing the absolute path to `libjvm.so` or `jvm.dll`.

Note: If you do not want to use the `libloading` create but still start the JVM then there are methods provided to 
load the JVM from a pointer to JNI_CreateJavaVM function instead of a dll/so file. 
Do that if you want to do dynamic linking yourself using `dlopen` or `LoadLibraryA` for example.

Note: This feature should not be used when writing a library that is loaded by `System.load` or `System.loadLibrary`. 
It would just add a dependency that is not needed.

#### asserts
This feature enables assertions in the code. This is useful for debugging and testing purposes.
These checks will cause a big performance hit and should not be used in production builds.

I would not even recommend using this feature for normal debugging builds 
unless you are specifically debugging issues that occur in relation to the JNI interface.
There is no need to enable this feature when you are just debugging a problem that occurs in pure rust.

This feature should NOT be used with the jvm launch option `-Xcheck:jni` 
as the assertions contain calls to `env.ExceptionCheck()` which will fool the JVM into thinking 
that your user code checks for exceptions, which it may not do. 

I recommend using this feature before or after you have tested your code with `-Xcheck:jni` depending 
on what problem your troubleshooting. The assertions are generally much better at detecting things like null pointers 
or invalid parameters than the JVM checks, while the JVM checks are able to catch missing exception checks or JVM Local Stack overflows better.

Since asserts are implemented using unwinding panics the panics can be caught. 
It is not recommended to continue or try to "recover" from this as the 
assertions do NOT perform cleanup actions when a panic occurs so you will leak JVM memory.
I recommend aborting the processes on such a panic as such a panic only occurs if the rust code had triggered UB in the JVM.
This can either be done by calling abort when "catching" the panic or compiling your rust code with panic=abort

### Further Info
### Variadic up-calls
Currently, variadic up-calls into JVM code are only implemented for 0 to 3 parameters.
(Do not confuse this with java "Variadic" methods, 
for JNI they are just a single parameter that is an Array)

JNI provides 3 ways of up-calling into JVM code:
1. CallStatic(TYPE)Method(class, methodID, ...) 
2. CallStatic(TYPE)MethodA(class, methodID, jtype*) 
3. CallStatic(TYPE)MethodV(class, methodID, va_list)

Substitute (TYPE) for the return type of the java method called. For example "Object" or "Int".

This crate only implements variant 1 and 2.
Variant 3 is not implemented by this crate because "va_list" cannot be created inside rust code.

Variant 2 is relatively straight forward and fully supported by this crate. 
This means you can call ANY java method using Variant 2.

Variant 1 has a Variadic parameter. This is Variadic up-calls refers to.
Rust does support this but only for explicit extern "C" functions and not for any
functions implemented in rust itself. To call Variant 1 this crate provides concrete
implementations to call this Variadic function with 0, 1, 2 and 3 parameters.
This should cover 99% of your up-call needs. 
To call methods with more than 3 parameters simply use Variant 2.

As you can see calling Variant 2 is a bit unwieldy for so for most smaller functions using
Variant 1 of up-calling is probably the better choice.
Example:
```rust
use std::ptr::null;
use jni_simple::{*};

#[no_mangle]
pub unsafe extern "system" fn Java_some_package_ClassName_method(env: JNIEnv, class: jclass) {
    let meth0 = env.GetStaticMethodID_str(class, "methodWith0IntParams", "()V");
    let meth1 = env.GetStaticMethodID_str(class, "methodWith1IntParams", "(I)V");
    let meth2 = env.GetStaticMethodID_str(class, "methodWith2IntParams", "(II)V");
    let meth3 = env.GetStaticMethodID_str(class, "methodWith3IntParams", "(III)V");
    //for example: public static void methodWith4IntParams(int a, int b, int c, int d) {}
    let meth4 = env.GetStaticMethodID_str(class, "methodWith4IntParams", "(IIII)V");

    //Variant 1: Variadic up-calls:
    //BE CAREFUL, this method is sensitive to difference between i32/i16/i8 etc. 
    //So always specify the type so that it matches the type of the Java Method.
    //Letting the compiler choose the may or may not work!
    //Passing a different argument than what the Java Method has is UB!
    //A sealed trait ensures that only parameters that the JVM can understand can be passed here
    //So for example accidentally passing a &str to these methods will not compile.
    env.CallStaticVoidMethod0(class, meth0);
    env.CallStaticVoidMethod1(class, meth1, 1i32);
    env.CallStaticVoidMethod2(class, meth2, 1i32, 2i32);
    env.CallStaticVoidMethod3(class, meth3, 1i32, 2i32, 3i32);
    //meth4 cannot be called with a Variadic up-call because it has 4 parameters!

    //Variant 2: Calls with a pointer to an array of jtype
    //The array is of type [jtype; N] so the pointer passed is a *const jtype.
    //jtype is a union defined in this crate where Into<jtype> 
    //is implemented for all types that can be passed to java as a parameter.
    //You could also use a Vec<jtype> to obtain your pointer to the jtype's!
    env.CallStaticVoidMethodA(class, meth0, null());
    env.CallStaticVoidMethodA(class, meth1, [1i32.into()].as_ptr());
    env.CallStaticVoidMethodA(class, meth2, [1i32.into(), 1i32.into()].as_ptr());
    env.CallStaticVoidMethodA(class, meth3, [1i32.into(), 1i32.into(), 1i32.into()].as_ptr());
    env.CallStaticVoidMethodA(class, meth4, [1i32.into(), 1i32.into(), 1i32.into(), 1i32.into()].as_ptr());
    //There is no "practical" limit to how large you could make this array/vec.
}
```
