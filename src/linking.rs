use crate::{JNI_OK, JNIEnv, JNIInvPtr, JavaVM, JavaVMInitArgs, JavaVMOption, jint};
use alloc::ffi::CString;
#[cfg(feature = "loadjvm")]
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::{c_char, c_void};
use core::ptr::null_mut;
use sync_ptr::SyncMutPtr;

#[cfg(not(feature = "dynlink"))]
use crate::jsize;

#[cfg(not(feature = "dynlink"))]
use sync_ptr::{SyncFnPtr, sync_fn_ptr_from_addr};

/// Module that contains the dll/so imports from the JVM.
/// This module should only be used when writing a library that is loaded by the JVM
/// using `System.load` or `System.loadLibrary`
#[cfg(feature = "dynlink")]
mod dynlink {
    use crate::{JNIEnv, JNIInvPtr, JavaVMInitArgs, jint, jsize};

    unsafe extern "system" {
        pub fn JNI_CreateJavaVM(invoker: *mut JNIInvPtr, env: *mut JNIEnv, initargs: *mut JavaVMInitArgs) -> jint;
        pub fn JNI_GetCreatedJavaVMs(array: *mut JNIInvPtr, len: jsize, out: *mut jsize) -> jint;
    }
}

/// type signature for the extern fn in the jvm
#[cfg(not(feature = "dynlink"))]
type JNI_CreateJavaVM = unsafe extern "C" fn(*mut JNIInvPtr, *mut JNIEnv, *mut JavaVMInitArgs) -> jint;

/// type signature for the extern fn in the jvm
#[cfg(not(feature = "dynlink"))]
type JNI_GetCreatedJavaVMs = unsafe extern "C" fn(*mut JNIInvPtr, jsize, *mut jsize) -> jint;

/// Data holder for the raw JVM function pointers.
#[cfg(not(feature = "dynlink"))]
#[derive(Debug, Copy, Clone)]
struct JNIDynamicLink {
    /// raw function ptr to `JNI_CreateJavaVM`
    JNI_CreateJavaVM: SyncFnPtr<JNI_CreateJavaVM>,
    /// raw function ptr to `JNI_GetCreatedJavaVMs`
    JNI_GetCreatedJavaVMs: SyncFnPtr<JNI_GetCreatedJavaVMs>,
}

#[cfg(not(feature = "dynlink"))]
impl JNIDynamicLink {
    /// Constructor with the two pointers
    pub fn new(JNI_CreateJavaVM: *const c_void, JNI_GetCreatedJavaVMs: *const c_void) -> Self {
        assert!(!JNI_GetCreatedJavaVMs.is_null(), "JNI_GetCreatedJavaVMs is null");
        assert!(!JNI_CreateJavaVM.is_null(), "JNI_CreateJavaVM is null");

        unsafe {
            Self {
                JNI_CreateJavaVM: sync_fn_ptr_from_addr!(JNI_CreateJavaVM, JNI_CreateJavaVM),
                JNI_GetCreatedJavaVMs: sync_fn_ptr_from_addr!(JNI_GetCreatedJavaVMs, JNI_GetCreatedJavaVMs),
            }
        }
    }

    /// Get the `JNI_GetCreatedJavaVMs` function pointer
    pub fn JNI_CreateJavaVM(&self) -> JNI_CreateJavaVM {
        self.JNI_CreateJavaVM.unwrap()
    }

    /// Get the `JNI_GetCreatedJavaVMs` function pointer
    pub fn JNI_GetCreatedJavaVMs(&self) -> JNI_GetCreatedJavaVMs {
        self.JNI_GetCreatedJavaVMs.unwrap()
    }
}

#[cfg(feature = "std")]
#[cfg(not(feature = "dynlink"))]
/// Standard library-based synchronization to prevent loading the jvm multiple times.
mod std_link {
    use crate::linking::JNIDynamicLink;

    /// Static state
    static LINK: std::sync::RwLock<Option<JNIDynamicLink>> = std::sync::RwLock::new(None);

    /// Writeable exclusive access to the static state
    pub fn link_write() -> std::sync::RwLockWriteGuard<'static, Option<JNIDynamicLink>> {
        LINK.write().unwrap_or_else(|e| {
            LINK.clear_poison();
            e.into_inner()
        })
    }

    /// Readable shared access to the static state
    pub fn link_read() -> std::sync::RwLockReadGuard<'static, Option<JNIDynamicLink>> {
        LINK.read().unwrap_or_else(|e| {
            LINK.clear_poison();
            e.into_inner()
        })
    }
}

#[cfg(feature = "std")]
#[cfg(not(feature = "dynlink"))]
use std_link::{link_read, link_write};

#[cfg(not(feature = "std"))]
#[cfg(not(feature = "dynlink"))]
/// Spin-lock-based synchronization to prevent loading the jvm multiple times.
mod spin_link {
    // The reason why I implemented this myself instead of using the spin crate is
    // because it would add spin as a dependency when compiling for std.
    // If I make spin optional, then selecting default-features=false won't compile unless
    // the user selects the "spin" feature manually.

    use crate::linking::JNIDynamicLink;
    use core::cell::UnsafeCell;
    use core::ops::{Deref, DerefMut};
    use core::sync::atomic::AtomicUsize;
    use core::sync::atomic::Ordering::SeqCst;

    /// Wrapper for `UnsafeCell` that can be put into static.
    struct UCellWrapper(UnsafeCell<Option<JNIDynamicLink>>);
    unsafe impl Send for UCellWrapper {}
    unsafe impl Sync for UCellWrapper {}

    /// `usize::MAX` / 2 or larger means the writer has it locked
    /// 0 is unlocked
    /// smaller than `usize::MAX` / 2: number of readers locked.
    static LOCK: AtomicUsize = AtomicUsize::new(0);

    /// The static state.
    static LINK: UCellWrapper = UCellWrapper(UnsafeCell::new(None));

    /// See LOCK above
    const USIZE_HALF: usize = usize::MAX / 2;

    /// Immutable guard to the global state
    pub(super) struct SpinLockGuard;
    impl Deref for SpinLockGuard {
        type Target = Option<JNIDynamicLink>;

        fn deref(&self) -> &Self::Target {
            unsafe { &*LINK.0.get() }
        }
    }

    impl Drop for SpinLockGuard {
        fn drop(&mut self) {
            let r = LOCK.fetch_sub(1, SeqCst);
            debug_assert!(r != 0);
        }
    }

    /// Mutable guard to the global state
    pub(super) struct SpinLockGuardMut;

    impl Deref for SpinLockGuardMut {
        type Target = Option<JNIDynamicLink>;

        fn deref(&self) -> &Self::Target {
            unsafe { &*LINK.0.get() }
        }
    }

    impl DerefMut for SpinLockGuardMut {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *LINK.0.get() }
        }
    }

    impl Drop for SpinLockGuardMut {
        fn drop(&mut self) {
            let r = LOCK.fetch_sub(USIZE_HALF, SeqCst);
            debug_assert!(r >= USIZE_HALF);
        }
    }

    /// Writeable exclusive access to the static state
    pub fn link_write() -> SpinLockGuardMut {
        loop {
            if LOCK.compare_exchange(0, USIZE_HALF, SeqCst, SeqCst).is_ok() {
                return SpinLockGuardMut;
            }

            core::hint::spin_loop();
        }
    }

    /// Readable shared access to the static state
    pub fn link_read() -> SpinLockGuard {
        loop {
            // Safety: if this overflows, we are boned,
            // but I don't think any os can spawn usize::threads,
            // so we are pretty safe.
            if LOCK.fetch_add(1, SeqCst) < USIZE_HALF - 1 {
                return SpinLockGuard;
            }
            LOCK.fetch_sub(1, SeqCst);

            core::hint::spin_loop();
        }
    }
}

#[cfg(not(feature = "std"))]
#[cfg(not(feature = "dynlink"))]
use spin_link::{link_read, link_write};

///
/// Call this function to initialize the dynamic linking to the jvm to use the provided function pointers to
/// create the jvm.
///
/// If this function is called more than once then it is a noop, since it is not possible to create
/// more than one jvm per process.
///
#[cfg(not(feature = "dynlink"))]
#[must_use]
pub fn init_dynamic_link(JNI_CreateJavaVM: *const c_void, JNI_GetCreatedJavaVMs: *const c_void) -> bool {
    let mut guard = link_write();
    if guard.is_none() {
        return false;
    }

    *guard = Some(JNIDynamicLink::new(JNI_CreateJavaVM, JNI_GetCreatedJavaVMs));
    true
}

///
/// Call this function to initialize the dynamic linking to the jvm to use the provided function pointers to
/// create the jvm.
///
/// If this function is called more than once then it is a noop, since it is not possible to create
/// more than one jvm per process.
///
#[cfg(feature = "dynlink")]
#[allow(clippy::missing_const_for_fn)]
#[must_use]
pub fn init_dynamic_link(_: *const c_void, _: *const c_void) -> bool {
    //NOOP, because the dynamic linker already must have preloaded the jvm for linking to succeed.
    false
}

///
/// Returns true if the jvm was loaded by either calling `load_jvm_from_library` or `init_dynamic_link`.
///
#[cfg(not(feature = "dynlink"))]
#[must_use]
pub fn is_jvm_loaded() -> bool {
    link_read().is_some()
}

/// Returns the static dynamic link or panic
/// # Panics
/// if the dynamic link was not initialized.
#[cfg(not(feature = "dynlink"))]
fn get_link() -> JNIDynamicLink {
    link_read().expect("jni_simple::init_dynamic_link not called")
}

///
/// Returns true if the jvm was loaded by either calling `load_jvm_from_library` or `init_dynamic_link`.
///
#[cfg(feature = "dynlink")]
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn is_jvm_loaded() -> bool {
    true
}

///
/// Convenience method to load the jvm from a path to libjvm.so or jvm.dll.
///
/// On success this method does NOT close the handle to the shared object.
/// This is usually fine because unloading the jvm is not supported anyway.
/// If you do not desire this then use `init_dynamic_link`.
///
/// # Errors
/// if loading the library fails without crashing the process, then a String describing the reason why is returned as an error.
///
/// # Safety
/// The Safety of this fn depends on the shared object that will be loaded as a result of this call.
///
#[cfg(feature = "loadjvm")]
#[cfg(not(feature = "dynlink"))]
pub unsafe fn load_jvm_from_library(path: &str) -> Result<(), String> {
    let mut guard = link_write();
    if guard.is_some() {
        drop(guard);
        return Err("JVM already loaded".to_string());
    }

    unsafe {
        let lib = libloading::Library::new(path).map_err(|e| alloc::format!("Failed to load jvm from {path} reason: {e}"))?;

        let JNI_CreateJavaVM_ptr = lib
            .get::<JNI_CreateJavaVM>(b"JNI_CreateJavaVM\0")
            .map_err(|e| alloc::format!("Failed to load jvm from {path} reason: JNI_CreateJavaVM -> {e}"))?
            .try_as_raw_ptr()
            .ok_or_else(|| alloc::format!("Failed to load jvm from {path} reason: JNI_CreateJavaVM -> failed to get raw ptr"))?;

        if JNI_CreateJavaVM_ptr.is_null() {
            return Err(alloc::format!("Failed to load jvm from {path} reason: JNI_CreateJavaVM not found"));
        }

        let JNI_GetCreatedJavaVMs_ptr = lib
            .get::<JNI_GetCreatedJavaVMs>(b"JNI_GetCreatedJavaVMs\0")
            .map_err(|e| alloc::format!("Failed to load jvm from {path} reason: JNI_GetCreatedJavaVMs -> {e}"))?
            .try_as_raw_ptr()
            .ok_or_else(|| alloc::format!("Failed to load jvm from {path} reason: JNI_CreateJavaVM -> failed to get raw ptr"))?;

        if JNI_GetCreatedJavaVMs_ptr.is_null() {
            return Err(alloc::format!("Failed to load jvm from {path} reason: JNI_GetCreatedJavaVMs not found"));
        }

        //We are good to go!
        core::mem::forget(lib);
        *guard = Some(JNIDynamicLink::new(JNI_CreateJavaVM_ptr, JNI_GetCreatedJavaVMs_ptr));
        drop(guard);
    }

    Ok(())
}

///
/// Convenience method to load the jvm from a path to libjvm.so, jvm.dll or libjvm.dylib.
///
/// On success this method does NOT close the handle to the shared object.
/// This is usually fine because unloading the jvm is not supported anyway.
/// If you do not desire this then use `init_dynamic_link`.
///
/// # Errors
/// if loading the library fails without crashing the process then a String describing the reason why is returned as an error.
///
/// # Safety
/// The Safety of this fn depends on the shared object that will be loaded as a result of this call.
///
#[cfg(feature = "loadjvm")]
#[cfg(feature = "dynlink")]
pub unsafe fn load_jvm_from_library(_: &str) -> Result<(), String> {
    Err("JVM already loaded".to_string())
}

/// Convenience method to load the jvm from the `JAVA_HOME` environment variable
/// that is commonly set on Windows by End-User Java Setups,
/// or on linux by distribution package installers.
///
/// # Errors
/// If `JAVA_HOME` is not set or doesn't point to a known layout of a JVM installation or cant be read
/// then this function returns an error.
///
/// # Safety
/// The Safety of this fn depends on the shared object that will be loaded as a result of this call.
///
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub unsafe fn load_jvm_from_java_home() -> Result<(), String> {
    let java_home = std::env::var("JAVA_HOME").map_err(|_| "JAVA_HOME is not set or invalid".to_string())?;
    unsafe { load_jvm_from_java_home_folder(&java_home) }
}

/// Convenience method to load the jvm from a given path to a java installation.
/// Info: The `java_home` parameter should refer to a path of a folder, which directly contains the "bin" or "jre" folder.
///
/// # Errors
/// If `java_home` doesn't refer to a known layout of a JVM installation or cant be read
/// then this function returns an error.
///
/// # Safety
/// The Safety of this fn depends on the shared object that will be loaded as a result of this call.
#[cfg(feature = "loadjvm")]
#[cfg(feature = "std")]
pub unsafe fn load_jvm_from_java_home_folder(java_home: &str) -> Result<(), String> {
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
        let mut buf = std::path::PathBuf::from(java_home);
        for part in *parts {
            buf.push(part);
        }

        if buf.try_exists().unwrap_or(false) {
            let full_path = buf.to_str().ok_or_else(|| alloc::format!("JAVA_HOME {java_home} is invalid"))?;

            unsafe {
                return load_jvm_from_library(full_path);
            }
        }
    }

    Err(alloc::format!("JAVA_HOME {java_home} is invalid"))
}

///
/// Returns the created `JavaVMs` in the given `vms` slice.
/// All remaining elements in the slice are set to None.
/// The count of returned `JavaVMs` is returned in the result.
///
/// If the given slice is smaller than the amount of created `JavaVMs` then
/// this function does not error and simply returns the amount
/// of space in the slice that would have been needed.
///
/// If this function returns an Err then the slice is untouched.
///
/// # Note
/// This will probably only ever return 1 (or 0) `JavaVM`s according to Oracle Documentation
/// as the hotspot jvm does not support more than 1 JVM per process.
///
/// # Errors
/// JNI implementation specific error constants like `JNI_EINVAL`
///
/// # Panics
/// Will panic if the JVM shared library has not been loaded yet.
/// If the JVM's `JNI_GetCreatedJavaVMs` method returns unexpected values
///
/// # Safety
/// The Safety of this fn is implementation dependant.
///
pub unsafe fn JNI_GetCreatedJavaVMs(vms: &mut [Option<JavaVM>]) -> Result<usize, jint> {
    #[cfg(not(feature = "dynlink"))]
    let link = get_link().JNI_GetCreatedJavaVMs();
    #[cfg(feature = "dynlink")]
    let link = dynlink::JNI_GetCreatedJavaVMs;

    //NOTE: Oracle spec says this will only ever yield 1 JVM.
    //I will worry about this when it actually becomes a problem
    let mut buf: [JNIInvPtr; 64] = [SyncMutPtr::null(); 64];
    let mut count: jint = 0;
    let res = unsafe { link(buf.as_mut_ptr(), 64, &raw mut count) };
    if res != JNI_OK {
        return Err(res);
    }

    let count = usize::try_from(count).expect("JNI_GetCreatedJavaVMs did set count to < 0");

    for (i, env) in buf.into_iter().enumerate().take(count) {
        assert!(!env.is_null(), "JNI_GetCreatedJavaVMs VM #{i} is null! count is {count}");
    }

    for (i, target) in vms.iter_mut().enumerate() {
        if i >= count {
            *target = None;
            continue;
        }

        *target = Some(JavaVM { vtable: buf[i] });
    }

    Ok(count)
}
///
/// Returns the first created `JavaVM` or None in the result.
///
/// Usually there is only 1 created or 0 created `JavaVM`'s in any given process.
/// This function acts as a convenience function that only returns the first and probably only `JavaVM`.
///
/// # Errors
/// JNI implementation specific error constants like `JNI_EINVAL`
///
/// # Panics
/// Will panic if the JVM shared library has not been loaded yet.
/// If the JVM's `JNI_GetCreatedJavaVMs` method returns unexpected values
///
/// # Safety
/// The Safety of this fn is implementation dependant.
///
pub unsafe fn JNI_GetCreatedJavaVMs_first() -> Result<Option<JavaVM>, jint> {
    unsafe {
        let mut vm = [None];
        _ = JNI_GetCreatedJavaVMs(vm.as_mut())?;
        Ok(vm[0])
    }
}

///
/// Directly calls `JNI_CreateJavaVM` with the provided arguments.
///
/// # Errors
/// JNI implementation specific error constants like `JNI_EINVAL`
///
/// # Panics
/// Will panic if the JVM shared library has not been loaded yet.
/// Will panic if the JVM shared library retruned unexpected values.
///
/// # Safety
/// The Safety of this fn is implementation dependant.
/// On Hotspot JVM's this fn cannot be called successfully more than once.
/// Subsequent calls are undefined behaviour.
///
pub unsafe fn JNI_CreateJavaVM(arguments: *mut JavaVMInitArgs) -> Result<(JavaVM, JNIEnv), jint> {
    #[cfg(feature = "asserts")]
    {
        assert!(!arguments.is_null(), "JNI_CreateJavaVM arguments must not be null");
    }

    #[cfg(not(feature = "dynlink"))]
    let link = get_link().JNI_CreateJavaVM();
    #[cfg(feature = "dynlink")]
    let link = dynlink::JNI_CreateJavaVM;

    let mut jvm: JNIInvPtr = SyncMutPtr::null();
    let mut env: JNIEnv = JNIEnv { vtable: null_mut() };

    let res = unsafe { link(&raw mut jvm, &raw mut env, arguments) };
    if res != JNI_OK {
        return Err(res);
    }

    assert!(!jvm.is_null(), "JNI_CreateJavaVM returned JNI_OK but the JavaVM pointer is null");

    assert!(!env.vtable.is_null(), "JNI_CreateJavaVM returned JNI_OK but the JNIEnv pointer is null");

    Ok((JavaVM { vtable: jvm }, env))
}

///
/// Convenience function to call `JNI_CreateJavaVM` with a simple list of String arguments.
///
/// These arguments are almost identical to the command line arguments used to start the jvm with the java binary.
/// Some options differ slightly. Consult the JNI Invocation API documentation for more information.
///
/// # Errors
/// JNI implementation specific error constants like `JNI_EINVAL`
///
/// # Panics
/// Will panic if the JVM shared library has not been loaded yet.
/// Will panic if more than `jsize::MAX` arguments are passed to the vm. (The JVM itself is likely to just die earlier)
/// If any argument contains a 0 byte in the string.
///
/// # Safety
/// The Safety of this fn is implementation dependant.
/// On Hotspot JVM's this fn cannot be called successfully more than once.
/// Subsequent calls are undefined behaviour.
///
/// # Example
/// ```rust
/// use std::ptr::null_mut;
/// use jni_simple::*;
///
///
/// //This example fn is roughly equivalent to "java -Xint -Xmx1G -Djava.class.path={absolute_path_to_jar_file} {main_class}" on the command line.
/// unsafe fn launch_jvm(absolute_path_to_jar_file: &str, main_class: &str) -> ! {
///     #[cfg(all(feature = "loadjvm", feature = "std"))] //Only needed due to doctest!
///     load_jvm_from_java_home().expect("Failed to load jvm");
///
///     let (vm, env) = JNI_CreateJavaVM_with_string_args(JNI_VERSION_1_8, &[
///          "-Xint".to_string(),
///          "-Xmx1G".to_string(),
///          format!("-Djava.class.path={absolute_path_to_jar_file}")
///     ], false).expect("Failed to start jvm");
///
///     let main_class = env.FindClass(main_class);
///     if env.ExceptionCheck() {
///         //Main class not found
///         env.ExceptionDescribe();
///         return std::process::exit(-1);
///     }
///
///     let main_method = env.GetStaticMethodID(main_class, "main","([Ljava/lang/String)V");
///     if env.ExceptionCheck() {
///         //no static main(String[] args) method in the main class.
///         env.ExceptionDescribe();
///         return std::process::exit(-1);
///     }
///
///     let string_class = env.FindClass("java/lang/String");
///     if env.ExceptionCheck() {
///         //Unlikely, java.lang.String not found.
///         env.ExceptionDescribe();
///         return std::process::exit(-1);
///     }
///
///     let main_method_string_parameter_array = env.NewObjectArray(0, string_class, null_mut());
///      if env.ExceptionCheck() {
///         //Unlikely jvm ran out of memory when creating "new String[0];"
///         env.ExceptionDescribe();
///         return std::process::exit(-1);
///     }
///
///     env.CallStaticVoidMethod1(main_class, main_method, main_method_string_parameter_array);
///     if env.ExceptionCheck() {
///         //Main method threw an exception
///         env.ExceptionDescribe();
///         return std::process::exit(-1);
///     }
///
///     //Block until all non deamon java threads the main method has started are done.
///     vm.DestroyJavaVM();
///
///     //Exit the process with success.
///     std::process::exit(0)
/// }
/// ```
///
pub unsafe fn JNI_CreateJavaVM_with_string_args<T: AsRef<str>>(version: jint, arguments: &[T], ignore_unrecognized_options: bool) -> Result<(JavaVM, JNIEnv), jint> {
    unsafe {
        /// inner helper struct to ensure that the `CStrings` are free'd in any case.
        struct DropGuard(*mut c_char);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                unsafe {
                    _ = CString::from_raw(self.0);
                }
            }
        }

        let mut vm_args: Vec<JavaVMOption> = Vec::with_capacity(arguments.len());
        let mut dealloc_list = Vec::with_capacity(arguments.len());
        for arg in arguments {
            let jvm_arg = CString::new(arg.as_ref()).expect("Argument contains 0 byte").into_raw();
            dealloc_list.push(DropGuard(jvm_arg));

            vm_args.push(JavaVMOption {
                optionString: jvm_arg,
                extraInfo: null_mut(),
            });
        }

        let mut args = JavaVMInitArgs {
            version,
            nOptions: i32::try_from(vm_args.len()).expect("Too many arguments"),
            options: vm_args.as_mut_ptr(),
            ignoreUnrecognized: u8::from(ignore_unrecognized_options),
        };

        let result = JNI_CreateJavaVM(&raw mut args);
        drop(dealloc_list);
        result
    }
}
