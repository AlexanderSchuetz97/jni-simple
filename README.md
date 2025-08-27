# jni-simple

This crate contains a simple dumb handwritten rust binding for the JNI (Java Native Interface) API.
It does absolutely no magic around the JNI Calls and lets you just use them as you would in C.

In addition to JNI, this crate also provides a similar simplistic binding for the JVMTI (Java VM Tool Interface) API.
The JVMTI Api can be used to write, for example, a Java Agent (like a Java debugger) in Rust or perform similar deep instrumentation with the JVM.
Most applications do not need JVMTI.

## Examples
### Loading a JVM from a shared object file or dll
Note: this example assumes the `loadjvm` feature is enabled!
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

        // Works on Both but requires the JAVA_HOME environment variable to be set.
        // This is usually done by the java installer.
        // Fails if JAVA_HOME is not set.
        //jni_simple::load_jvm_from_java_home().expect("failed to load jvm");
        

        //Adjust JVM version and arguments here; args are just like the args you pass on the command line.
        //You could provide your classpath here, for example, or configure the jvm heap size.
        //Default arguments (none) will do for this example.
        let args : Vec<String> = vec![];
        let (_jvm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &args).expect("failed to create jvm");

        //This code does not check for failure or exceptions checks or "checks" for success in general.
        let sys = env.FindClass("java/lang/System");
        let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
        let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
        //Calls System.nanoTime() and prints the result
        println!("{}", nanos);
    }
}
```
### Writing a JNI shared library that implements a native method
The complete version of this example (including the java code) can be found in the repository inside the example_project folder.
Note: this example assumes the `dynlink` feature is enabled!
```rust
#![allow(non_snake_case)]

use std::io::Write;
use jni_simple::{*};
use std::os::raw::{c_void};
use std::ptr::{null, null_mut};
use std::thread;
use std::time::Duration;
use std::io::stdout;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
    unsafe {
        //All error codes are jint, never JNI_OK. See JNI documentation for their meaning when you handle them.
        //This is a Result<JNIEnv, jint>.
        let env: JNIEnv = vm.GetEnv(JNI_VERSION_1_8).unwrap();


        //This code does not check for failure or exceptions checks or "checks" for success in general.
        let sys = env.FindClass("java/lang/System");
        let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
        let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
        println!("RUST: JNI_OnLoad {}", nanos);
        stdout().flush().unwrap();

        return JNI_VERSION_1_8;
    }
}

//Would be called from java. the signature in java is org.example.JNITest#test()
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_example_JNITest_test(env: JNIEnv, _class: jclass) {
  unsafe {
    //This code does not check for failure or exceptions checks or "checks" for success in general.
    let sys = env.FindClass("java/lang/System");
    let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
    let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
    println!("RUST: Java_org_example_JNITest_test {}", nanos);
    stdout().flush().unwrap();


    thread::spawn(|| unsafe {
      thread::sleep(Duration::from_millis(2000));

      //This can be done anywhere in the application at any time.
      let vms : JavaVM = jni_simple::JNI_GetCreatedJavaVMs_first().unwrap() // the error code is once again a jint.
              .unwrap(); //There can only be one JavaVM per process as per oracle spec.

      //You could also provide a thread name or thread group here.
      let mut attach_args = JavaVMAttachArgs::new(JNI_VERSION_1_8, null(), null_mut());
      vms.AttachCurrentThread(&mut attach_args).unwrap();
      let env = vms.GetEnv::<JNIEnv>(JNI_VERSION_1_8).unwrap();
      let sys = env.FindClass("java/lang/System");
      let nano_time = env.GetStaticMethodID(sys, "nanoTime", "()J");
      let nanos = env.CallStaticLongMethodA(sys, nano_time, null());
      println!("RUST thread delayed: Java_org_example_JNITest_test {}", nanos);
      stdout().flush().unwrap();
      vms.DetachCurrentThread();
    });
  }
}
```
### Writing a Java Agent (JVMTI)
Note: this example assumes the `dynlink` feature is enabled!

```toml
crate-type = ["cdylib"]
```

```bash
java -agentpath:absolute_path_to_your_compiled_agent.so.dll -jar myjar.jar
```

```rust
use jni_simple::{*};

use std::ffi::c_void;
use jni_simple::*;

#[unsafe(no_mangle)]
unsafe extern "system" fn Agent_OnLoad(vm: JavaVM, _command_line_options: *const char, _: *mut c_void) -> i32 {
  unsafe {
    let Ok(jvmti) = vm.GetEnv::<JVMTIEnv>(JVMTI_VERSION_1_2) else {
      eprintln!("Agent_OnLoad failed to get JVMTI_VERSION_1_2 environment");
      return -1;
    };

    /// This is just an example, do whatever you want to do with JVMTI here.
    let mut cap = jvmtiCapabilities::default();
    let err = jvmti.GetPotentialCapabilities(&mut cap);
    if err != JVMTI_ERROR_NONE {
      eprintln!("Agent_OnLoad failed to get potential capabilities error {err}");
      return -1;
    }

    if !cap.can_redefine_any_class() {
      eprintln!("Agent_OnLoad error: JavaVM cannot redefine classes");
      return -1;
    }

    //....
    0
  }
}
```

### Using JVMTI from JNI
Note: It's probably not a good idea to do this in productive code, but can be useful in some debugging situations.
Note: this example assumes the `dynlink` feature is enabled!
```rust
#![allow(non_snake_case)]

use std::ffi::c_void;
use jni_simple::{*};

//This could also be an ordinary native method invoked from java.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut c_void) -> jint {
  unsafe {
    //All error codes are jint, never JNI_OK. See JNI documentation for their meaning when you handle them.
    //This is a Result<JVMTIEnv, jint>.
    let jvmti : JVMTIEnv = vm.GetEnv(JVMTI_VERSION_1_2).unwrap();
    let mut cap = jvmtiCapabilities::default();
    let _err = jvmti.GetPotentialCapabilities(&mut cap);
    // Note that this JVMTI environment has much fewer capabilities than its agent counterpart, 
    // as the JVM may have already performed optimizations that makes some more advanced operations impossible 
    // by the time our jni library is loaded.
    // ...
    // Do what you need to do with jvmti here.

    return JNI_VERSION_1_8;
  }
}
```

## Main goals of this crate

### Dont pretend that JNI is "safe"
JNI is inherently unsafe (from a rust point of view), and any attempt to enforce safety will lead to 
performance or API complexity issues. All JNI methods provided by this crate are 
marked as "unsafe" as they should be.

### Simple type system of JNI is kept as is
All types like jobject, jstring, jarray,... which are opaque handles represented as pointers in C are 
represented as raw opaque pointers in Rust that are type aliases of each other. 
This essentially makes them just hint to the user and doesn't enforce any type safety as that would sometimes
be a big hindrance when working with JNI. 

##### Why?
For example, conversion between jobject and jobjectArray or any other array type is required for some jni methods and trivial to do in C, 
yet using the existing `jni` crate this requires convoluted casts that probably have side effects (like calling IsInstanceOf) for 'safety' reasons.
Using `jni-simple` (this crate) you don't even need a cast. Naturally, passing the wrong type of object to the JVM is UB, hence the need for unsafe.

### Designed for runtime dynamic linking of the JVM
The Problem: The existing jni crate depends on the jni-sys crate which requires the JVM to be resolvable by the dynamic linker.
There are 2 ways to do this. The first is to statically link the JVM into the binary, this is rarely done, 
very cumbersome and poorly documented. The other is to provide the JVM on the linker path so ldd can find it, 
but I have never seen this occur in the real world either.

This crate is developed for the more common use case that the JVM is available somewhere on the system. 
It leaves it up to the user of the crate to write the necessary code to find and load the JVM. 

This allows for maximum flexibility when writing a launcher app which, for example, 
may first download and extract a JVM from the internet.
As should be obvious, when writing a native library (cdylib) that does not launch the JVM itself and 
is instead loaded by `System.load` or `System.loadLibrary` then this is irrelevant.

## Test coverage and maturity
### JNI
Test coverage is near 100%.

This means that all functions are at least called by a test, 
and for some even most edge cases are tested.

The implementation is mature and has been used in production without any known issues.

### JVMTI
Test coverage is very low.

The overall maturity of the JVMTI Api provided in the current version of jni-simple is low.
Expect bugs that will cause undefined behavior in functions that I have not yet tested.
(due to human error when manually translating the jvmti specification into rust code)

I have mostly prepared this JVMTI implementation ahead of time for when I need it,
so it may take some time until everything is tested and fixed.
Naturally, this has no impact on JNI and should not affect many people 
as JVMTI is not needed for 99% of programs.

If you encounter a bug and need an urgent fix, then open up an issue on GitHub or make a pull request.

Naturally, if you wish to submit tests, then that would also be appreciated.

## Features

### std
This feature is enabled by default!
Adds support for some types in the rust standard library.

If you wish to compile your library/launcher app without the rust standard library, then disable default features.

### loadjvm
This feature is not enabled by default!
Requires the std feature.

This feature provides functions to dynamically link the jvm using the `libloading` crate 
from a string containing the absolute path to `libjvm.so` or `jvm.dll`.

Note: If you do not want to use the `libloading` crate but still start the JVM then there are methods provided to 
load the JVM from a pointer to JNI_CreateJavaVM function instead of a dll/so file. 
Do that if you want to do dynamic linking yourself using `dlopen` or `LoadLibraryA` for example.

Note: This feature should not be used when writing a library that is loaded by `System.load` or `System.loadLibrary`. 
It would just add a dependency that is not needed.

### dynlink
This feature is not enabled by default!

If this feature is enabled, it is assumed that the symbols exported by the JVM can be found by the dynamic linker by itself.
When this feature is enabled, all functions that "load" the jvm become a noop. 

This feature should be enabled when making a shared library that is loaded by the JVM using `System.load`, `System.loadLibrary`, 
or when writing a java agent that is loaded by specifying the `-agentlib` VM argument when starting the VM 
either from the command line or the JNI Invoker interface. 
In those cases, the dynamic linker is guaranteed to be able to find the symbols exported by the JVM.

Note: This feature should not be used when writing a jvm launcher application.

Note: Enabling both `dynlink` and `loadjvm` makes no sense, it just adds the `libloading` crate as an unnecessary dependency.

### asserts
This feature is not enabled by default!
Requires the std feature.

This feature enables assertions in the code. This is useful for debugging and testing purposes.
These checks will cause a big performance hit and should not be used in production builds.

I would not even recommend using this feature for normal debugging builds 
unless you are specifically debugging issues that occur in relation to UB with the JNI interface.
There is no need to enable this feature when you are just debugging a problem that occurs in pure rust code.

Enabling this feature for automated unit/integration tests that perform complicated JNI calls can be a good idea if you
can tolerate the performance penalty of enabling this feature.
Enabling this feature for benchmarks is not recommended as it falsifies the results of the benchmark.

This feature should NOT be used with the jvm launch option `-Xcheck:jni` 
as the assertions contain calls to `env.ExceptionCheck()` before nearly every normal call to the jvm, 
which will fool the JVM into thinking that your user code checks for exceptions, which it may not do. 
It will also generate many false negatives as some 'assertion' code does not call `env.ExceptionCheck()`
as the only realistic scenario for some calls that are made to fail is for the JVM to run out of memory.

I recommend using this feature before or after you have tested your code with `-Xcheck:jni` depending 
on what problem your troubleshooting. The assertions are generally much better at detecting things like null pointers 
or invalid parameters than the JVM checks, while the JVM checks are able to catch missing exception checks or JVM Local Stack overflows better.

The asserts are implemented using rust panics. 
If you compile your project with unwinding panics, the panics can be caught. 
It is not recommended to continue or try to "recover" from these panics as the 
assertions do NOT perform cleanup actions when a panic occurs. 
You will leak JVM locals or leave the JVM in an otherwise unrecoverable state if you continue. 
I recommend aborting the processes on such a panic as such a panic only occurs  
if the rust code had triggered UB in the JVM in the absence of the assertions.
This can either be done by calling abort when "catching" the panic or compiling your rust code with panic=abort
if you do not need to catch panics anywhere in your rust code.

Info: There are no asserts present in the JVMTI implementation as JVMTI has more robust checking implemented 
in the java vm itself, forgoing the need to do any checks ourselves. Most of the potential for UB in JVMTI is impossible
to check anyway, because the API heavily relies on callbacks which are passed to JVMTI as raw function pointers.
Checking those for validity is not really possible without changing the API at a higher level, which is not a goal of this crate.
In addition to that, jvmti is not needed for most use cases, so I suspect that it will see little use.

## Further Info

### String handling

Many JNI functions accept a zero-terminated utf-8 string as a parameter. 
It is possible to call all those functions with all commonly used rust string types as well as raw *const c_char pointers.

The following types are supported:
- &str
- String
- &String
- &Cow\<&str\>
- CString
- &CString
- &CStr
- &OsStr
- OsString
- &OsString
- &[u8]
- Vec\<u8\>
- \* const c_char
  - \* const u8
  - \* const i8
- \* mut c_char
  - \* mut u8
  - \* mut i8

Because JNI expects a zero terminated string, some types are copied if necessary, and a zero byte is appended.
Raw pointer types are assumed to already be zero terminated and are passed as is.
If a string already contains a 0-byte, then it is not copied and passed to jni as is. Should the 0-byte be somewhere
in the middle of the string, then this 0-byte effectively truncates the string at that point for the jni call.

Example:
```rust
#![allow(non_snake_case)]
use std::ptr::null;
use std::ffi::c_char;
use jni_simple::{*};

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_some_package_ClassName_method(env: JNIEnv, class: jclass) {
  unsafe {
    env.FindClass("java/lang/String");
    env.FindClass("java/lang/String".as_bytes());
    env.FindClass("java/lang/String".to_string());
    env.FindClass("java/lang/String\0".as_ptr());
    env.FindClass("java/lang/String\0garbage data that is ignored");

    let exception = env.FindClass("java/lang/Exception");
    env.ThrowNew(exception, "This is a message");

    //This fn accepts different types of raw pointers, so we unfortunately have to specify the type here.
    env.ThrowNew(exception, null::<c_char>());
    //To make this a bit shorter, you can also use the unit type.
    //This is equivalent to passing null::<c_char>()
    env.ThrowNew(exception, ());


    // !!! WARNING THIS WOULD BE UB !!!
    //This is UB because the string is not zero terminated and passed as a raw pointer and therefore assumed to be zero terminated.
    env.FindClass("java/lang/String".as_ptr());
  }
}
```


### Variadic up-calls
Currently, variadic up-calls into JVM code are only implemented for 0 to 3 parameters.
(Do not confuse this with java "Variadic" methods, 
for JNI they are just a single parameter that is an Array)

JNI provides 3 ways of up-calling into JVM code:
1. CallStatic(TYPE)Method(class, methodID, ...) 
2. CallStatic(TYPE)MethodA(class, methodID, jtype*) 
3. CallStatic(TYPE)MethodV(class, methodID, va_list)

Substitute (TYPE) for the return type of the java method called. For example, "Object" or "Int".

This crate only implements variants 1 and 2.
This crate does not implement variant 3 because "va_list" cannot be created inside rust code.

Variant 2 is relatively straight forward and fully supported by this crate. 
This means you can call ANY java method using Variant 2.

Variant 1 has a Variadic parameter. This is what Variadic up-calls refer to.
Rust does support this but only for explicit extern "C" functions and not for any
functions implemented in rust itself. To call Variant 1 this crate provides concrete
implementations to call this Variadic function with 0, 1, 2 and 3 parameters.
This should cover 99% of your up-call needs. 
To call methods with more than 3 parameters, simply use Variant 2.

As you can see below, calling Variant 2 is a bit unwieldy, 
so it is probably the better choice to use Variant 1 of up-calling for most smaller functions.
Example:
```rust
#![allow(non_snake_case)]
use std::ptr::null;
use jni_simple::{*};

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_some_package_ClassName_method(env: JNIEnv, class: jclass) {
    let meth0 = env.GetStaticMethodID(class, "methodWith0IntParams", "()V");
    let meth1 = env.GetStaticMethodID(class, "methodWith1IntParams", "(I)V");
    let meth2 = env.GetStaticMethodID(class, "methodWith2IntParams", "(II)V");
    let meth3 = env.GetStaticMethodID(class, "methodWith3IntParams", "(III)V");
    //for example, public static void methodWith4IntParams(int a, int b, int c, int d) {}
    let meth4 = env.GetStaticMethodID(class, "methodWith4IntParams", "(IIII)V");

    //Variant 1: Variadic up-calls:
    //BE CAREFUL, this method is sensitive to the difference between i32/i16/i8 etc. 
    //So always specify the type so that it matches the type of the Java Method.
    //Letting the compiler choose the type may or may not work!
    //Passing a different argument than what the Java Method has is UB!
    //A sealed trait ensures that only parameters that the JVM can understand can be passed here
    //So for example, accidentally passing a &str to these methods will not compile.
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
    env.CallStaticVoidMethodA(class, meth1, jtypes!(1i32).as_ptr());
    env.CallStaticVoidMethodA(class, meth2, jtypes!(1i32, 2i32).as_ptr());
    env.CallStaticVoidMethodA(class, meth3, jtypes!(1i32, 2i32, 3i32).as_ptr());
    env.CallStaticVoidMethodA(class, meth4, jtypes!(1i32, 2i32, 3i32, 4i32).as_ptr());
    //There is no "practical" limit to how large you could make this array/vec.
    //For reference the jtypes! macro resolves to this
    let macro_result : [jtype; 4] = jtypes!(1i32, 2i32, 3i32, 4i32);
    //And is just short for:
    let macro_result : [jtype; 4] = [jtype::from(1i32), jtype::from(2i32), jtype::from(3i32), jtype::from(4i32)];
}
```

