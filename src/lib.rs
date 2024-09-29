//! # jni-simple
//! This crate contains a simple dumb handwritten rust wrapper around the JNI (Java Native Interface) API.
//! It does absolutely no magic around the JNI Calls and lets you just use it as you would in C.
//!
//! If you are looking to start a jvm from rust then the entrypoints in this create are
//! init_dynamic_link, load_jvm_from_library, JNI_CreateJavaVM and JNI_GetCreatedJavaVMs.
//!
//! If you are looking to write a jni library in rust then the types JNIEnv and jclass, etc.
//! should be sufficient.
//!
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_void, CStr, CString};
use std::fmt::{Debug, Formatter};
use std::mem;
#[cfg(feature = "loadjvm")]
use std::path::{PathBuf};
use std::ptr::{null_mut};
#[cfg(feature = "asserts")]
use std::ptr::null;

use once_cell::sync::OnceCell;
use sync_ptr::{FromConstPtr, SyncConstPtr, SyncMutPtr};

pub const JNI_OK: jint = 0;

pub const JNI_COMMIT: jint = 1;

pub const JNI_ABORT: jint = 2;
pub const JNI_ERR: jint = -1;
pub const JNI_EDETACHED: jint = -2;
pub const JNI_EVERSION: jint = -3;
pub const JNI_ENOMEM: jint = -4;
pub const JNI_EEXIST: jint = -5;
pub const JNI_EINVAL: jint = -6;

pub const JNI_VERSION_1_1: jint = 0x00010001;
pub const JNI_VERSION_1_2: jint = 0x00010002;
pub const JNI_VERSION_1_4: jint = 0x00010004;
pub const JNI_VERSION_1_6: jint = 0x00010006;
pub const JNI_VERSION_1_8: jint = 0x00010008;
pub const JNI_VERSION_9: jint =   0x00090000;
pub const JNI_VERSION_10: jint =  0x000a0000;
pub const JNI_VERSION_19: jint =  0x00130000;
pub const JNI_VERSION_20: jint =  0x00140000;
pub const JNI_VERSION_21: jint = 0x00150000;

pub type jlong = i64;
pub type jint = i32;
pub type jsize = jint;
pub type jshort = i16;
pub type jchar = u16;
pub type jbyte = i8;
pub type jboolean = bool;

pub type jfloat = std::ffi::c_float;

pub type jdouble = std::ffi::c_double;

pub type jclass = *mut c_void;

pub type jobject = *mut c_void;

pub type jstring = jobject;

pub type jarray = jobject;

pub type jobjectArray = jarray;

pub type jbooleanArray = jarray;

pub type jbyteArray = jarray;

pub type jcharArray = jarray;

pub type jshortArray = jarray;

pub type jintArray = jarray;

pub type jlongArray = jarray;

pub type jfloatArray = jarray;

pub type jdoubleArray = jarray;

#[repr(C)]
#[derive(Debug)]
pub enum jobjectRefType {
    JNIInvalidRefType = 0,
    JNILocalRefType = 1,
    JNIGlobalRefType = 2,
    JNIWeakGlobalRefType = 3
}


mod private {
    pub trait Sealed {}
}

pub type jweak = jobject;

pub type jthrowable = jobject;

pub type jmethodID = jobject;
pub type jfieldID = jobject;

///
/// Marker trait for all types that are valid to use to make variadic JNI Up-calls with.
///
pub trait JType: private::Sealed+Into<jtype>+Clone+Copy {

    ///
    /// Returns a single character that equals the type's JNI signature.
    ///
    /// Boolean -> Z
    /// Byte -> B
    /// Short -> S
    /// Char -> C
    /// Int -> I
    /// Long -> J
    /// Float -> F
    /// Double -> D
    /// any java.lang.Object -> L
    ///
    ///
    fn jtype_id() -> char;
}
impl private::Sealed for jobject {}
impl JType for jobject {
    #[inline(always)]
    fn jtype_id() -> char {
        'L'
    }
}
impl private::Sealed for jboolean {}
impl JType for jboolean {

    #[inline(always)]
    fn jtype_id() -> char {
        'Z'
    }
}
impl private::Sealed for jbyte {}
impl JType for jbyte {
    #[inline(always)]
    fn jtype_id() -> char {
        'B'
    }
}
impl private::Sealed for jshort {}
impl JType for jshort {
    #[inline(always)]
    fn jtype_id() -> char {
        'S'
    }
}
impl private::Sealed for jchar {}
impl JType for jchar {
    #[inline(always)]
    fn jtype_id() -> char {
        'C'
    }
}
impl private::Sealed for jint {}
impl JType for jint {
    #[inline(always)]
    fn jtype_id() -> char {
        'I'
    }
}
impl private::Sealed for jlong {}
impl JType for jlong {
    #[inline(always)]
    fn jtype_id() -> char {
        'J'
    }
}
impl private::Sealed for jfloat {}
impl JType for jfloat {
    #[inline(always)]
    fn jtype_id() -> char {
        'F'
    }
}
impl private::Sealed for jdouble {}
impl JType for jdouble {
    #[inline(always)]
    fn jtype_id() -> char {
        'D'
    }
}


#[repr(C)]
#[derive(Clone, Copy)]
pub union jtype {
    long: jlong,
    int: jint,
    short: jshort,
    char: jchar,
    byte: jbyte,
    boolean: jboolean,
    float: jfloat,
    double: jdouble,
    object: jobject,
    class: jclass,
    throwable: jthrowable
}

impl Debug for jtype {
    #[inline(never)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            let long = std::ptr::read_unaligned(&self.long as *const jlong);
            let int = std::ptr::read_unaligned(&self.int as *const jint);
            let short = std::ptr::read_unaligned(&self.short as *const jshort);
            let byte = std::ptr::read_unaligned(&self.byte as *const jbyte);
            let float = std::ptr::read_unaligned(&self.float as *const jfloat);
            let double = std::ptr::read_unaligned(&self.double as *const jdouble);

            f.write_fmt(format_args!("jtype union[long=0x{:x} int=0x{:x} short=0x{:x} byte=0x{:x} float={:e} double={:e}]", long, int, short, byte, float, double))
        }
    }
}

impl jtype {

    ///
    /// Helper function to "create" a jtype with a null jobject.
    ///
    #[inline(always)]
    pub const fn null() -> jtype {
        #[cfg(target_pointer_width = "32")]
        {
            let mut jt = jtype {long: 0};
            jt.object = null_mut();
            jt
        }
        #[cfg(target_pointer_width = "64")]
        {
            jtype { object: null_mut() }
        }
    }

    #[inline(always)]
    pub const unsafe fn long(&self) -> jlong {
        self.long
    }

    #[inline(always)]
    pub const unsafe fn int(&self) -> jint {
        self.int
    }

    #[inline(always)]
    pub const unsafe fn short(&self) -> jshort {
        self.short
    }

    #[inline(always)]
    pub const unsafe fn char(&self) -> jchar {
        self.char
    }

    #[inline(always)]
    pub const unsafe fn byte(&self) -> jbyte {
        self.byte
    }

    #[inline(always)]
    pub const unsafe fn boolean(&self) -> jboolean {
        self.boolean
    }

    #[inline(always)]
    pub const unsafe fn float(&self) -> jfloat {
        self.float
    }

    #[inline(always)]
    pub const unsafe fn double(&self) -> jdouble {
        self.double
    }

    #[inline(always)]
    pub const unsafe fn object(&self) -> jobject {
        self.object
    }

    #[inline(always)]
    pub const unsafe fn class(&self) -> jclass {
        self.class
    }

    #[inline(always)]
    pub const unsafe fn throwable(&self) -> jthrowable {
        self.throwable
    }

    #[inline(always)]
    pub fn set<T: Into<jtype>>(&mut self, value: T) {
        *self = value.into();
    }
}

impl From<jlong> for jtype {
    fn from(value: jlong) -> Self {
        jtype { long: value }
    }
}

impl From<jobject> for jtype {

    #[cfg(target_pointer_width = "64")]
    fn from(value: jobject) -> Self {
        jtype { object: value }
    }

    #[cfg(target_pointer_width = "32")]
    fn from(value: jobject) -> Self {
        let mut jt = jtype {long: 0};
        jt.object = value;
        jt
    }
}
impl From<jint> for jtype {
    fn from(value: jint) -> Self {
        let mut jt = jtype {long: 0};
        jt.int = value;
        jt
    }
}

impl From<jshort> for jtype {
    fn from(value: jshort) -> Self {
        let mut jt = jtype {long: 0};
        jt.short = value;
        jt
    }
}

impl From<jbyte> for jtype {
    fn from(value: jbyte) -> Self {
        let mut jt = jtype {long: 0};
        jt.byte = value;
        jt
    }
}

impl From<jchar> for jtype {
    fn from(value: jchar) -> Self {
        let mut jt = jtype {long: 0};
        jt.char = value;
        jt
    }
}

impl From<jfloat> for jtype {
    fn from(value: jfloat) -> Self {
        let mut jt = jtype {long: 0};
        jt.float = value;
        jt
    }
}

impl From<jdouble> for jtype {
    fn from(value: jdouble) -> Self {
        jtype { double: value }
    }
}
impl From<jboolean> for jtype {
    fn from(value: jboolean) -> Self {
        let mut jt = jtype {long: 0};
        jt.boolean = value;
        jt
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct JNINativeMethod {
    name: *const c_char,
    signature: *const c_char,
    fnPtr: *const c_void
}

type JNIInvPtr = SyncMutPtr<*mut [*mut c_void; 10]>;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JavaVM {
    functions: JNIInvPtr
}

#[repr(C)]
#[derive(Debug)]
pub struct JavaVMAttachArgs {
    version: jint,
    name: *const c_char,
    group: jobject
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct JavaVMOption {
    optionString: *mut c_char,
    extraInfo: *mut c_void,
}

impl JavaVMOption {
    pub fn new(option_string: *mut c_char, extra_info: *mut c_void) -> JavaVMOption {
        JavaVMOption {
            optionString: option_string,
            extraInfo: extra_info,
        }
    }

    pub fn optionString(&self) -> *mut c_char {
        self.optionString
    }

    pub fn extraInfo(&self) -> *mut c_void {
        self.extraInfo
    }
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct JavaVMInitArgs {
    version: i32,
    nOptions: i32,
    options: *mut JavaVMOption,
    ignoreUnrecognized: u8,
}

impl JavaVMInitArgs {
    pub fn new(version: i32, n_options: i32, options: *mut JavaVMOption, ignore_unrecognized: u8) -> JavaVMInitArgs {
        JavaVMInitArgs {
            version,
            nOptions: n_options,
            options,
            ignoreUnrecognized: ignore_unrecognized,
        }
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn nOptions(&self) -> i32 {
        self.nOptions
    }

    pub fn options(&self) -> *mut JavaVMOption {
        self.options
    }

    pub fn ignoreUnrecognized(&self) -> u8 {
        self.ignoreUnrecognized
    }
}

type JNIEnvVTable = *mut *mut [*mut c_void; 235];

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct JNIEnv {
    vtable: JNIEnvVTable,
}

impl JNINativeMethod {
    pub fn new(name: *const c_char, signature: *const c_char, function_pointer: *const c_void) -> JNINativeMethod {
        JNINativeMethod {
            name,
            signature,
            fnPtr: function_pointer,
        }
    }

    pub fn name(&self) -> *const c_char {
        self.name
    }


    pub fn signature(&self) -> *const c_char {
        self.signature
    }


    pub fn fnPtr(&self) -> *const c_void {
        self.fnPtr
    }
}


impl JavaVMAttachArgs {
    pub fn new(version: jint, name: *const c_char, group: jobject) -> Self {
        Self { version, name, group }
    }


    pub fn version(&self) -> jint {
        self.version
    }
    pub fn name(&self) -> *const c_char {
        self.name
    }
    pub fn group(&self) -> jobject {
        self.group
    }
}


impl JNIEnv {

    #[inline(always)]
    unsafe fn jni<X>(&self, index: usize) -> X {
        unsafe {mem::transmute_copy(&(**self.vtable)[index])}
    }

    ///
    /// Returns the version of the JNI interface.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetVersion
    ///
    /// The returned value must be compared against a constant. (They start with JNI_VERSION_...)
    /// Not every java version has such a constant.
    /// Only java versions where a function in the JNI interface was added has one.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn is_at_least_java10(env: JNIEnv) -> bool {
    ///     env.GetVersion() >= JNI_VERSION_10
    /// }
    /// ```
    ///
    pub unsafe fn GetVersion(&self) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetVersion");
            self.check_no_exception("GetVersion");
        }
        self.jni::<extern "system" fn(JNIEnvVTable) -> jint>(4)(self.vtable)
    }


    ///
    /// Defines a class in the given classloader.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#DefineClass
    ///
    /// # Arguments
    /// * `name` - name of the class
    /// * `classloader` - handle to the classloader java object. This can be null if the current JNI classloader should be used.
    /// * `data` - the binary content of the compiled java .class file.
    ///
    /// # Returns
    /// A local ref handle to the java.lang.Class (jclass) object that was just defined.
    /// On error null is returned.
    ///
    /// # Throws Java Exception:
    /// * `ClassFormatError` - if the class data does not specify a valid class.
    /// * `ClassCircularityError` - if a class or interface would be its own superclass or superinterface.
    /// * `OutOfMemoryError` - if the system runs out of memory.
    /// * `SecurityException` - if the caller attempts to define a class in the "java" package tree.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// The `classloader` handle must be a valid handle if it is not null.
    /// `name` must be a valid pointer to a 0 terminated utf-8 string. It must not be null.
    ///
    /// # Example
    /// ```rust
    /// use std::ffi::CString;
    /// use std::ptr::null_mut;
    /// use jni_simple::{*};
    ///
    /// unsafe fn define_main_class(env: JNIEnv) -> jclass {
    ///     let class_blob = &[0u8]; // = include_bytes!("../my_java_project/src/main/java/org/example/Main.class");
    ///     let name = CString::new("org/example/Main").unwrap();
    ///     let class = env.DefineClass(name.as_ptr(), null_mut(), class_blob.as_slice());
    ///     if env.ExceptionCheck() {
    ///         env.ExceptionDescribe();
    ///         panic!("Failed to load main class check stderr for an error");
    ///     }
    ///     if class.is_null() {
    ///         panic!("Failed to load main class. JVM did not throw an exception!"); //Unlikely
    ///     }
    ///     class
    /// }
    /// ```
    ///
    pub unsafe fn DefineClass(&self, name: *const c_char, classloader: jobject, data: &[u8]) -> jclass {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("DefineClass");
            self.check_no_exception("DefineClass");
            //TODO check if classloader is valid or null
            assert!(!name.is_null(), "DefineClass name is null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, *const c_char, jobject, *const u8, i32) -> jclass>(5)
            (self.vtable, name, classloader, data.as_ptr(), data.len() as i32)
    }

    ///
    /// Defines a class in the given classloader.
    /// Convenience function that wraps DefineClass but takes a &str as name argument instead of a *const c_char.
    ///
    /// # Arguments
    /// * `name` - name of the class
    /// * `classloader` - handle to the classloader java object. This can be null if the current JNI classloader should be used.
    /// * `data` - the binary content of the compiled java .class file.
    ///
    /// # Panics
    /// if name contains a '0' byte
    ///
    /// # Returns
    /// A local ref handle to the java.lang.Class (jclass) object that was just defined.
    /// On error null is returned.
    ///
    /// # Throws Java Exception:
    /// * `ClassFormatError` - if the class data does not specify a valid class.
    /// * `ClassCircularityError` - if a class or interface would be its own superclass or superinterface.
    /// * `OutOfMemoryError` - if the system runs out of memory.
    /// * `SecurityException` - if the caller attempts to define a class in the "java" package tree.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// The `classloader` handle must be a valid handle if it is not null.
    ///
    /// # Example
    /// ```rust
    /// use std::ptr::null_mut;
    /// use jni_simple::{*};
    ///
    /// unsafe fn define_main_class(env: JNIEnv) -> jclass {
    ///     let class_blob = &[0u8]; // = include_bytes!("../my_java_project/src/main/java/org/example/Main.class");
    ///     let class = env.DefineClass_str("org/example/Main", null_mut(), class_blob.as_slice());
    ///     if env.ExceptionCheck() {
    ///         env.ExceptionDescribe();
    ///         panic!("Failed to load main class check stderr for an error");
    ///     }
    ///     if class.is_null() {
    ///         panic!("Failed to load main class. JVM did not throw an exception!"); //Unlikely
    ///     }
    ///     class
    /// }
    /// ```
    ///
    pub unsafe fn DefineClass_str(&self, name: &str, classloader: jobject, data: &[u8]) -> jclass {
        let str = CString::new(name).unwrap();
        self.DefineClass(str.as_ptr(), classloader, data)
    }

    ///
    /// Finds or loads a class.
    /// If the class was previously loaded by the current JNI Classloader then it is returned.
    /// If the class was not previously loaded then the current JNI Classloader will attempt to
    /// load it.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#FindClass
    ///
    /// # Arguments
    /// * `name` - name of the class in jni notation (i.e: "java/lang/Object")
    ///
    /// # Returns
    /// A local ref handle to the java.lang.Class (jclass) object.
    /// On error null is returned.
    ///
    /// # Throws Java Exception:
    /// * `ClassFormatError` - if the class data does not specify a valid class.
    /// * `ClassCircularityError` - if a class or interface would be its own superclass or superinterface.
    /// * `OutOfMemoryError` - if the system runs out of memory.
    /// * `NoClassDefFoundError` -  if no definition for a requested class or interface can be found.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `name` must be a valid pointer to a 0 terminated utf-8 string. It must not be null.
    ///
    /// # Example
    /// ```rust
    /// use std::ffi::CString;
    /// use jni_simple::{*};
    ///
    /// unsafe fn find_main_class(env: JNIEnv) -> jclass {
    ///     let name = CString::new("org/example/Main").unwrap();
    ///     let class = env.FindClass(name.as_ptr());
    ///     if env.ExceptionCheck() {
    ///         env.ExceptionDescribe();
    ///         panic!("Failed to find main class check stderr for an error");
    ///     }
    ///     if class.is_null() {
    ///         panic!("Failed to find main class. JVM did not throw an exception!"); //Unlikely
    ///     }
    ///     class
    /// }
    /// ```
    ///
    pub unsafe fn FindClass(&self, name: *const c_char) -> jclass {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("FindClass");
            self.check_no_exception("FindClass");
            assert!(!name.is_null(), "FindClass name is null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, *const c_char) -> jclass>(6)(self.vtable, name)
    }

    ///
    /// Finds or loads a class.
    /// If the class was previously loaded by the current JNI Classloader or any parent of it then the class is simply returned.
    /// If the class was not previously loaded then the current JNI Classloader will attempt to find and load it.
    ///
    /// # Arguments
    /// * `name` - name of the class in jni notation (i.e: "java/lang/Object")
    ///
    /// # Panics
    /// if `name` contains a 0 byte.
    ///
    /// # Returns
    /// A local ref handle to the java.lang.Class (jclass) object.
    /// On error null is returned.
    ///
    /// # Throws Java Exception:
    /// * `ClassFormatError` - if the class data does not specify a valid class.
    /// * `ClassCircularityError` - if a class or interface would be its own superclass or superinterface.
    /// * `OutOfMemoryError` - if the system runs out of memory.
    /// * `NoClassDefFoundError` -  if no definition for a requested class or interface can be found.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn find_main_class(env: JNIEnv) -> jclass {
    ///     let class = env.FindClass_str("org/example/Main");
    ///     if env.ExceptionCheck() {
    ///         env.ExceptionDescribe();
    ///         panic!("Failed to find main class check stderr for an error");
    ///     }
    ///     if class.is_null() {
    ///         panic!("Failed to find main class. JVM did not throw an exception!"); //Unlikely
    ///     }
    ///     class
    /// }
    /// ```
    ///
    pub unsafe fn FindClass_str(&self, name: &str) -> jclass {
        let str = CString::new(name).unwrap();
        self.FindClass(str.as_ptr())
    }

    ///
    /// Gets the superclass of the class `class`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetSuperclass
    ///
    /// # Arguments
    /// * `class` - handle to a class object. must not be null.
    ///
    /// # Returns
    /// A local ref handle to the superclass or null.
    /// If `class` refers to java.lang.Object class then null is returned.
    /// If `class` refers to any Interface then null is returned.
    ///
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `class` must be a valid non-null handle to a class object.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn has_parent(env: JNIEnv, class: jclass) -> bool {
    ///     if class.is_null() {
    ///         return false;
    ///     }
    ///     let local = env.NewLocalRef(class);
    ///     let parent_or_null = env.GetSuperclass(local);
    ///     env.DeleteLocalRef(local);
    ///     if parent_or_null.is_null() {
    ///         return false;
    ///     }
    ///     env.DeleteLocalRef(parent_or_null);
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetSuperclass(&self, class: jclass) -> jclass {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetSuperclass");
            self.check_no_exception("GetSuperclass");
            self.check_is_class("GetSuperclass", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass) -> jclass>(10)(self.vtable, class)
    }

    ///
    /// Determines whether an object of clazz1 can be safely cast to clazz2.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#IsAssignableFrom
    ///
    /// # Arguments
    /// * `class1` - handle to a class object. must not be null.
    /// * `class2` - handle to a class object. must not be null.
    ///
    /// # Returns
    /// true if either:
    /// * class1 and class2 refer to the same class.
    /// * class1 is a subclass of class2.
    /// * class1 has class2 as one of its interfaces.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `class1` and `class2` must be valid non-null handles to class objects.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn is_throwable_class(env: JNIEnv, class: jclass) -> bool {
    ///     let throwable_class = env.FindClass_str("java/lang/Throwable");
    ///     if throwable_class.is_null() {
    ///         env.ExceptionDescribe();
    ///         panic!("java/lang/Throwable not found! See stderr!");
    ///     }
    ///     let local = env.NewLocalRef(class);
    ///     if local.is_null() {
    ///         env.DeleteLocalRef(throwable_class);
    ///         return false;
    ///     }
    ///     let result = env.IsAssignableFrom(local, throwable_class);
    ///     env.DeleteLocalRef(local);
    ///     env.DeleteLocalRef(throwable_class);
    ///     result
    /// }
    /// ```
    ///
    pub unsafe fn IsAssignableFrom(&self, class1: jclass, class2: jclass) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("IsAssignableFrom");
            self.check_no_exception("IsAssignableFrom");
            self.check_is_class("IsAssignableFrom", class1);
            self.check_is_class("IsAssignableFrom", class2);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass, jclass) -> jboolean>(11)(self.vtable, class1, class2)
    }

    ///
    /// Throws a java.lang.Throwable. This is roughly equal to the throw keyword in Java.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Throw
    ///
    /// # Arguments
    /// * `throwable` - handle to an object which is instanceof java.lang.Throwable. must not be null.
    ///
    /// # Returns
    /// JNI_OK on success. a negative value on failure.
    ///
    /// ## If JNI_OK was returned
    /// The JVM will be throwing an exception as a result of this call.
    ///
    /// When the current thread is throwing an exception you may only call the following JNI functions:
    /// * ExceptionOccurred
    /// * ExceptionDescribe
    /// * ExceptionClear
    /// * ExceptionCheck
    /// * ReleaseStringChars
    /// * ReleaseStringUTFChars
    /// * ReleaseStringCritical
    /// * Release<Type>ArrayElements
    /// * ReleasePrimitiveArrayCritical
    /// * DeleteLocalRef
    /// * DeleteGlobalRef
    /// * DeleteWeakGlobalRef
    /// * MonitorExit
    /// * PushLocalFrame
    /// * PopLocalFrame
    ///
    /// Calling any other JNI function is UB.
    ///
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `throwable` must be a valid non-null handle to an object which is instanceof java.lang.Throwable.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn throw_null_pointer_exception(env: JNIEnv) {
    ///     let npe_class = env.FindClass_str("java/lang/NullPointerException");
    ///     if npe_class.is_null() {
    ///         env.ExceptionDescribe();
    ///         panic!("java/lang/NullPointerException not found!");
    ///     }
    ///     let npe_constructor = env.GetMethodID_str(npe_class, "<init>", "()V");
    ///     if npe_constructor.is_null() {
    ///         env.ExceptionDescribe();
    ///         env.DeleteLocalRef(npe_class);
    ///         panic!("java/lang/NullPointerException has no zero arg constructor!");
    ///     }
    ///
    ///     let npe_obj = env.NewObject0(npe_class, npe_constructor);
    ///     env.DeleteLocalRef(npe_class);
    ///     if npe_obj.is_null() {
    ///         env.ExceptionDescribe();
    ///         panic!("java/lang/NullPointerException failed to call zero arg constructor!");
    ///     }
    ///     env.Throw(npe_obj);
    ///     env.DeleteLocalRef(npe_obj);
    /// }
    /// ```
    ///
    pub unsafe fn Throw(&self, throwable: jthrowable) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("Throw");
            self.check_no_exception("Throw");
            assert!(!throwable.is_null(), "Throw throwable is null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jthrowable) -> jint>(13)(self.vtable, throwable)
    }

    ///
    /// Throws a new instance `class`. This is roughly equal to `throw new ...` in Java.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#ThrowNew
    ///
    /// # Arguments
    /// * `class` - handle to a non-abstract class instances of which can be cast to java.lang.Throwable. Must not be null.
    /// * `message` - the exception message. Must be null or a pointer to a 0 terminated utf-8 string.
    ///
    /// # Returns
    /// JNI_OK on success. a negative value on failure.
    ///
    /// ## If JNI_OK was returned
    /// The JVM will be throwing an exception as a result of this call.
    ///
    /// When the current thread is throwing an exception you may only call the following JNI functions:
    /// * ExceptionOccurred
    /// * ExceptionDescribe
    /// * ExceptionClear
    /// * ExceptionCheck
    /// * ReleaseStringChars
    /// * ReleaseStringUTFChars
    /// * ReleaseStringCritical
    /// * Release<Type>ArrayElements
    /// * ReleasePrimitiveArrayCritical
    /// * DeleteLocalRef
    /// * DeleteGlobalRef
    /// * DeleteWeakGlobalRef
    /// * MonitorExit
    /// * PushLocalFrame
    /// * PopLocalFrame
    ///
    /// Calling any other JNI function is UB.
    ///
    /// # Throws Java Exception:
    /// * NoSuchMethodError if the class has no suitable constructor for the argument supplied. Note: the return value remains JNI_OK!
    ///   - null `message`: no zero arg or one arg String constructor exists.
    ///   - non-null `message`: no one arg String constructor exists.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `class` must be a valid non-null handle to a class which is:
    /// * Not abstract
    /// * Is a descendant of java.lang.Throwable (instances can be cast to Throwable)
    ///
    /// `message` must be a pointer to a 0 terminated utf-8 string or null.
    ///
    /// # Example
    /// ```rust
    /// use std::ffi::CString;
    /// use std::ptr::null;
    /// use jni_simple::{*};
    ///
    /// unsafe fn throw_illegal_argument_exception(env: JNIEnv, message: Option<&str>) {
    ///     let npe_class = env.FindClass_str("java/lang/IllegalArgumentException");
    ///     if npe_class.is_null() {
    ///         env.ExceptionDescribe();
    ///         panic!("java/lang/IllegalArgumentException not found!");
    ///     }
    ///     match message {
    ///         None => {
    ///             env.ThrowNew(npe_class, null());
    ///         }
    ///         Some(message) => {
    ///             let message = CString::new(message).expect("message contains 0 byte!");
    ///             env.ThrowNew(npe_class, message.as_ptr());
    ///         }
    ///     }
    ///     env.DeleteLocalRef(npe_class);
    /// }
    /// ```
    ///
    pub unsafe fn ThrowNew(&self, class: jclass, message: *const c_char) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ThrowNew");
            self.check_no_exception("ThrowNew");
            self.check_is_exception_class("ThrowNew", class);
            self.check_is_not_abstract("ThrowNew", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass, *const c_char) -> jint>(14)(self.vtable, class, message)
    }


    ///
    /// Throws a new instance `class`. This is roughly equal to `throw new ...` in Java.
    ///
    /// # Arguments
    /// * `class` - handle to a non-abstract class instances of which can be cast to java.lang.Throwable. Must not be null.
    /// * `message` - the exception message.
    ///
    /// # Panics
    /// If `message` contains any 0 bytes.
    ///
    /// # Returns
    /// JNI_OK on success. a negative value on failure.
    ///
    /// ## If JNI_OK was returned
    /// The JVM will be throwing an exception as a result of this call.
    ///
    /// When the current thread is throwing an exception you may only call the following JNI functions:
    /// * ExceptionOccurred
    /// * ExceptionDescribe
    /// * ExceptionClear
    /// * ExceptionCheck
    /// * ReleaseStringChars
    /// * ReleaseStringUTFChars
    /// * ReleaseStringCritical
    /// * Release<Type>ArrayElements
    /// * ReleasePrimitiveArrayCritical
    /// * DeleteLocalRef
    /// * DeleteGlobalRef
    /// * DeleteWeakGlobalRef
    /// * MonitorExit
    /// * PushLocalFrame
    /// * PopLocalFrame
    ///
    /// Calling any other JNI function is UB.
    ///
    /// # Throws Java Exception:
    /// * NoSuchMethodError if no single argument String constructor exists in the given class.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `class` must be a valid non-null handle to a class which is:
    /// * Not abstract
    /// * Is a descendant of java.lang.Throwable (instances can be cast to Throwable)
    ///
    /// # Example
    /// ```rust
    /// use std::ptr::null;
    /// use jni_simple::{*};
    ///
    /// unsafe fn throw_illegal_argument_exception(env: JNIEnv, message: &str) {
    ///     let npe_class = env.FindClass_str("java/lang/IllegalArgumentException");
    ///     if npe_class.is_null() {
    ///         env.ExceptionDescribe();
    ///         panic!("java/lang/IllegalArgumentException not found!");
    ///     }
    ///     env.ThrowNew_str(npe_class, message);
    ///     env.DeleteLocalRef(npe_class);
    /// }
    /// ```
    ///
    ///
    pub unsafe fn ThrowNew_str(&self, clazz: jclass, message: &str) -> jint {
        let str = CString::new(message).unwrap();
        self.ThrowNew(clazz, str.as_ptr())
    }

    ///
    /// Returns a local reference to the exception currently being thrown.
    /// Calling this function does not clear the exception.
    /// It stays thrown until for example `ExceptionClear` is called.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#ExceptionOccurred
    ///
    /// # Returns
    /// A local ref to the throwable that is currently being thrown.
    /// null if no throwable is currently thrown.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn protected_call<R, F: FnOnce() -> R>(env: JNIEnv, call: F) -> Result<R, jthrowable> {
    ///     let result = call();
    ///     if env.ExceptionCheck() {
    ///         let exception = env.ExceptionOccurred();
    ///         env.ExceptionClear();
    ///         return Err(exception);
    ///     }
    ///
    ///     Ok(result)
    /// }
    ///
    /// unsafe fn try_to_find_my_class(env: JNIEnv) {
    ///     let my_class : Result<jclass, jthrowable> = protected_call(env, || env.FindClass_str("org/example/DoesntExist"));
    ///     if my_class.is_err() {
    ///         let _throwable_object : jthrowable = my_class.unwrap_err();
    ///         //Handle exception here, this would be the catch(Throw t) block of java
    ///         unimplemented!()
    ///     } else {
    ///         let _my_class : jclass = my_class.unwrap();
    ///         //Use class
    ///         unimplemented!()
    ///     }
    /// }
    ///
    ///
    pub unsafe fn ExceptionOccurred(&self) -> jthrowable {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ExceptionOccurred");
        }
        self.jni::<extern "system" fn(JNIEnvVTable) -> jthrowable>(15)(self.vtable)
    }

    ///
    /// Print the stacktrace and message currently thrown to STDOUT.
    /// A side effect of this function is that the exception is also cleared.
    /// This is roughly equivalent to calling `java.lang.Throwable#printStackTrace()` in java.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#ExceptionDescribe
    ///
    /// If no exception is currently thrown then this method is a no-op.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    ///
    /// unsafe fn test(env: JNIEnv) {
    ///     let my_class = env.FindClass_str("org/example/TestClass");
    ///     if my_class.is_null() {
    ///         env.ExceptionDescribe();
    ///         panic!("Class not found check stderr");
    ///     }
    ///     unimplemented!()
    /// }
    /// ```
    ///
    pub unsafe fn ExceptionDescribe(&self) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ExceptionDescribe");
        }
        self.jni::<extern "system" fn(JNIEnvVTable)>(16)(self.vtable);
    }

    ///
    /// Print the stacktrace and message currently thrown to STDOUT.
    /// A side effect of this function is that the exception is also cleared.
    /// This is roughly equivalent to calling `java.lang.Throwable#printStackTrace()` in java.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#ExceptionDescribe
    ///
    /// If no exception is currently thrown then this method is a no-op.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    ///
    /// unsafe fn test(env: JNIEnv) {
    ///     let mut my_class = env.FindClass_str("org/example/TestClass");
    ///     if my_class.is_null() {
    ///         env.ExceptionClear();
    ///         my_class = env.FindClass_str("org/example/FallbackClass");
    ///     }
    ///     unimplemented!()
    /// }
    /// ```
    ///
    pub unsafe fn ExceptionClear(&self) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ExceptionClear");
        }
        self.jni::<extern "system" fn(JNIEnvVTable)>(17)(self.vtable);
    }

    ///
    /// Raises a fatal error and does not expect the VM to recover. This function does not return.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#FatalError
    ///
    /// # Arguments
    /// * `msg` - message that should be present in the error report. 0 terminated utf-8. Must not be null.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// `msg` must be a non-null pointer to a valid 0 terminated utf-8 string.
    ///
    pub unsafe fn FatalError(&self, msg: *const c_char) {
        #[cfg(feature = "asserts")]
        {
            assert!(!msg.is_null(), "FatalError msg is null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, *const c_char)>(18)(self.vtable, msg);
        unreachable!("FatalError");
    }

    ///
    /// Raises a fatal error and does not expect the VM to recover. This function does not return.
    ///
    /// # Arguments
    /// * `message` - message that should be present in the jvm error report that will be printed to stdout/stderr.
    ///
    /// # Panics
    /// If `message` contains any 0 bytes.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    pub unsafe fn FatalError_str(&self, message: &str) {
        let str = CString::new(message).unwrap().into_raw();
        self.FatalError(str);
        unreachable!("FatalError");
    }

    ///
    /// Checks if an exception is thrown on the current thread.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#ExceptionCheck
    ///
    /// # Returns
    /// true if an exception is thrown on the current thread, false otherwise.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    pub unsafe fn ExceptionCheck(&self) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ExceptionCheck");
        }
        self.jni::<extern "system" fn(JNIEnvVTable) -> jboolean>(228)(self.vtable)
    }

    ///
    /// Creates a new global reference from an existing reference.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#NewGlobalRef
    ///
    /// # Arguments
    /// * `obj` - a valid reference or null.
    ///
    /// # Returns
    /// the newly created global reference or null.
    /// null is returned if:
    /// * the argument `obj` is null
    /// * the system ran out of memory
    /// * `obj` is a weak reference that has already been garbage collected.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `obj` must not refer to a reference that has already been deleted by calling DeleteLocalRef, DeleteGlobalRef, DeleteWeakGlobalRef
    ///
    pub unsafe fn NewGlobalRef(&self, obj: jobject) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewGlobalRef");
            self.check_no_exception("NewGlobalRef");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jobject>(21)(self.vtable, obj)
    }

    ///
    /// Deletes a global reference to an object allowing the garbage collector to free it if no more
    /// references to it exists.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#DeleteGlobalRef
    ///
    /// # Arguments
    /// * `obj` - a valid non-null global reference.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `obj` must not be null.
    /// `obj` must be a global reference.
    /// `obj` must not refer to an already deleted global reference. (Double free)
    ///
    pub unsafe fn DeleteGlobalRef(&self, obj: jobject) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("DeleteGlobalRef");
            assert!(!obj.is_null(), "DeleteGlobalRef obj is null");
            match self.GetObjectRefType(obj) {
                jobjectRefType::JNIInvalidRefType => panic!("DeleteGlobalRef invalid non null reference"),
                jobjectRefType::JNILocalRefType => panic!("DeleteGlobalRef local reference passed"),
                jobjectRefType::JNIWeakGlobalRefType => panic!("DeleteGlobalRef weak global reference passed"),
                _=> {}
            }
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject)>(22)(self.vtable, obj)
    }

    ///
    /// Deletes a local reference to an object allowing the garbage collector to free it if no more
    /// references to it exists.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#DeleteGlobalRef
    ///
    /// # Arguments
    /// * `obj` - a valid non-null local reference.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `obj` must not be null.
    /// `obj` must be a local reference.
    /// `obj` must not refer to an already deleted local reference. (Double free)
    ///
    pub unsafe fn DeleteLocalRef(&self, obj: jobject) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("DeleteLocalRef");
            assert!(!obj.is_null(), "DeleteLocalRef obj is null");
            match self.GetObjectRefType(obj) {
                jobjectRefType::JNIInvalidRefType => panic!("DeleteLocalRef invalid non null reference"),
                jobjectRefType::JNILocalRefType => {}
                jobjectRefType::JNIGlobalRefType => panic!("DeleteLocalRef global reference passed"),
                jobjectRefType::JNIWeakGlobalRefType => panic!("DeleteLocalRef weak global reference passed"),
            }
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject)>(23)(self.vtable, obj)
    }

    ///
    /// The jvm guarantees that a native method can have at least 16 local references.
    /// Creating any more than 16 local references without calling this function is effectively UB.
    /// This function instructs the JVM to ensure that at least
    /// `capacity` amount of local references are available for allocation.
    /// This function can be called multiple times to increase the amount of required locals.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#EnsureLocalCapacity
    ///
    ///
    /// # Arguments
    /// * `capacity` - amount of local references the jvm must provide. Must be larger than 0.
    ///
    /// # Returns
    /// 0 on success, negative value indicating the error.
    ///
    /// # Throws Java Exception
    /// * OutOfMemoryError - if the vm runs out of memory ensuring capacity. This is never the case when 0 is returned.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `capacity` must not be 0 or negative.
    ///
    /// ## Observed UB when more locals are allocated than ensured
    /// This behavior depends heavily on the jvm used and the arguments used to start it. This list is incomplete
    /// * Heap/Stack corruption.
    /// * JVM calls FatalError and aborts the process.
    /// * JVM Functions that would return a local reference return null.
    /// * JVM simply allocates more locals than ensured. (starting the jvm with -verbose:jni will log this)
    ///
    pub unsafe fn EnsureLocalCapacity(&self, capacity: jint) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("EnsureLocalCapacity");
            self.check_no_exception("EnsureLocalCapacity");
            assert!(capacity >= 0, "EnsureLocalCapacity capacity is negative");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jint) -> jint>(26)(self.vtable, capacity)
    }

    ///
    /// Creates a new local reference frame, in which at least a given number of local references can be created.
    /// Note that local references already created in previous local frames are still valid in the current local frame.
    /// This method should be called by code that is called from unknown code where it is not known if enough
    /// local capacity is available. This method is superior to just increasing the capacity by calling EnsureLocalCapacity
    /// because that requires at least a rough knowledge of how many locals the caller itself has used and still needs.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#PushLocalFrame
    ///
    ///
    /// # Arguments
    /// * `capacity` - amount of local references the jvm must provide. Must be larger than 0.
    ///
    /// # Returns
    /// 0 on success, negative value indicating the error.
    ///
    /// # Throws Java Exception
    /// * OutOfMemoryError - if the vm runs out of memory ensuring capacity. This is never the case when 0 is returned.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    /// Current thread is not currently throwing a Java exception.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#java_exceptions
    ///
    /// `capacity` must not be 0 or negative.
    ///
    /// returning back to java code without cleaning up all created local reference frames by calling `PopLocalFrame` is UB.
    ///
    /// ## Observed UB when more locals are allocated than ensured
    /// This behavior depends heavily on the jvm used and the arguments used to start it. This list is incomplete
    /// * Heap/Stack corruption.
    /// * JVM calls FatalError and aborts the process.
    /// * JVM Functions that would return a local reference return null.
    /// * JVM simply allocates more locals than ensured. (starting the jvm with -verbose:jni will log this)
    ///
    pub unsafe fn PushLocalFrame(&self, capacity: jint) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("PushLocalFrame");
            self.check_no_exception("PushLocalFrame");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jint) -> jint>(19)(self.vtable, capacity)
    }

    ///
    /// Pops a local reference frame created with PushLocalFrame
    /// All local references created within this reference frame are freed automatically
    /// and are no longer valid when this call returns.
    ///
    /// # Arguments
    /// * result - arbitrary local reference jobject that should be moved to the parent reference frame.
    /// this is similar to a "return" value and may be null if no such result is needed.
    /// the local reference this function returns is valid within the parent local reference frame.
    ///
    /// # Returns
    /// A valid local reference that points to the same object as the reference `result`. Is Null if `result` is Null.
    ///
    /// # Safety
    ///
    /// Current thread must not be detached from JNI.
    ///
    /// Current thread does not hold a critical reference.
    /// * https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#GetPrimitiveArrayCritical_ReleasePrimitiveArrayCritical
    ///
    ///
    pub unsafe fn PopLocalFrame(&self, result: jobject) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("PopLocalFrame");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jobject>(20)(self.vtable, result)
    }

    pub unsafe fn NewLocalRef(&self, obj: jobject) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewLocalRef");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jobject>(25)(self.vtable, obj)
    }

    pub unsafe fn NewWeakGlobalRef(&self, obj: jobject) -> jweak {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewWeakGlobalRef");
            self.check_no_exception("NewWeakGlobalRef");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jweak>(226)(self.vtable, obj)
    }

    pub unsafe fn DeleteWeakGlobalRef(&self, obj: jweak) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("DeleteWeakGlobalRef");
            assert!(!obj.is_null(), "DeleteWeakGlobalRef obj is null");
            match self.GetObjectRefType(obj) {
                jobjectRefType::JNIInvalidRefType => panic!("DeleteWeakGlobalRef invalid non null reference"),
                jobjectRefType::JNILocalRefType => panic!("DeleteWeakGlobalRef local reference passed"),
                jobjectRefType::JNIGlobalRefType => panic!("DeleteWeakGlobalRef strong global reference passed"),
                _=> {}
            }
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jobject)>(227)(self.vtable, obj);
    }

    pub unsafe fn AllocObject(&self, clazz: jclass) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("AllocObject");
            self.check_no_exception("AllocObject");
            self.check_is_class("AllocObject", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass) -> jobject>(27)(self.vtable, clazz)
    }

    pub unsafe fn NewObjectA(&self, clazz: jclass, constructor: jmethodID, args: *const jtype) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewObjectA");
            self.check_no_exception("NewObjectA");
            assert!(!constructor.is_null(), "NewObjectA constructor is null");
            self.check_is_class("NewObjectA", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass, jmethodID, *const jtype) -> jobject>(30)(self.vtable, clazz, constructor, args)
    }

    pub unsafe fn NewObject0(&self, clazz: jclass, constructor: jmethodID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewObject");
            self.check_no_exception("NewObject");
            assert!(!constructor.is_null(), "NewObject constructor is null");
            self.check_is_class("NewObject", clazz);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jclass, jmethodID) -> jobject>(28)(self.vtable, clazz, constructor)
    }

    pub unsafe fn NewObject1<A: JType>(&self, clazz: jclass, constructor: jmethodID, arg1: A) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewObject");
            self.check_no_exception("NewObject");
            assert!(!constructor.is_null(), "NewObject constructor is null");
            self.check_is_class("NewObject", clazz);
            self.check_parameter_types_constructor("NewObject", clazz, constructor, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jclass, jmethodID, ...) -> jobject>(28)(self.vtable, clazz, constructor, arg1)
    }

    pub unsafe fn NewObject2<A: JType, B: JType>(&self, clazz: jclass, constructor: jmethodID, arg1: A, arg2: B) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewObject");
            self.check_no_exception("NewObject");
            assert!(!constructor.is_null(), "NewObject constructor is null");
            self.check_is_class("NewObject", clazz);
            self.check_parameter_types_constructor("NewObject", clazz, constructor, arg1, 0, 2);
            self.check_parameter_types_constructor("NewObject", clazz, constructor, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jclass, jmethodID, ...) -> jobject>(28)(self.vtable, clazz, constructor, arg1, arg2)
    }

    pub unsafe fn NewObject3<A: JType, B: JType, C: JType>(&self, clazz: jclass, constructor: jmethodID, arg1: A, arg2: B, arg3: C) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewObject");
            self.check_no_exception("NewObject");
            assert!(!constructor.is_null(), "NewObject constructor is null");
            self.check_is_class("NewObject", clazz);
            self.check_parameter_types_constructor("NewObject", clazz, constructor, arg1, 0, 3);
            self.check_parameter_types_constructor("NewObject", clazz, constructor, arg2, 1, 3);
            self.check_parameter_types_constructor("NewObject", clazz, constructor, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jclass, jmethodID, ...) -> jobject>(28)(self.vtable, clazz, constructor, arg1, arg2, arg3)
    }

    pub unsafe fn GetObjectClass(&self, obj: jobject) -> jclass {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetObjectClass");
            self.check_no_exception("GetObjectClass");
            self.check_ref_obj("GetObjectClass", obj);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jobject>(31)(self.vtable, obj)
    }

    pub unsafe fn GetObjectRefType(&self, obj: jobject) -> jobjectRefType {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetObjectRefType");
            self.check_no_exception("GetObjectRefType");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jobjectRefType>(232)(self.vtable, obj)
    }

    pub unsafe fn IsInstanceOf(&self, obj: jobject, clazz: jclass) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("IsInstanceOf");
            self.check_no_exception("IsInstanceOf");
            self.check_is_class("IsInstanceOf", clazz);
            self.check_ref_obj_permit_null("IsInstanceOf", obj);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass) -> jboolean>(32)(self.vtable, obj, clazz)
    }

    pub unsafe fn IsSameObject(&self, obj1: jobject, obj2: jobject) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("IsSameObject");
            self.check_no_exception("IsSameObject");
            self.check_ref_obj_permit_null("IsSameObject obj1", obj1);
            self.check_ref_obj_permit_null("IsSameObject obj2", obj2);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jobject) -> jboolean>(24)(self.vtable, obj1, obj2)
    }

    pub unsafe fn GetFieldID(&self, clazz: jclass, name: *const c_char, sig: *const c_char) -> jfieldID {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetFieldID");
            self.check_no_exception("GetFieldID");
            assert!(!name.is_null(), "GetFieldID name is null");
            assert!(!sig.is_null(), "GetFieldID sig is null");
            self.check_is_class("GetFieldID", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass, *const c_char, *const c_char) -> jfieldID>(94)(self.vtable, clazz, name, sig)
    }

    pub unsafe fn GetFieldID_str(&self, class: jclass, name: &str, sig: &str) -> jfieldID {
        let nstr = CString::new(name).unwrap();
        let nsig = CString::new(sig).unwrap();
        self.GetFieldID(class, nstr.as_ptr(), nsig.as_ptr())
    }

    pub unsafe fn GetObjectField(&self, obj: jobject, fieldID: jfieldID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetObjectField");
            self.check_no_exception("GetObjectField");
            self.check_field_type_object("GetObjectField", obj, fieldID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jobject>(95)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetBooleanField(&self, obj: jobject, fieldID: jfieldID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetBooleanField");
            self.check_no_exception("GetBooleanField");
            self.check_field_type_object("GetBooleanField", obj, fieldID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jboolean>(96)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetByteField(&self, obj: jobject, fieldID: jfieldID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetByteField");
            self.check_no_exception("GetByteField");
            self.check_field_type_object("GetByteField", obj, fieldID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jbyte>(97)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetCharField(&self, obj: jobject, fieldID: jfieldID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetCharField");
            self.check_no_exception("GetCharField");
            self.check_field_type_object("GetCharField", obj, fieldID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jchar>(98)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetShortField(&self, obj: jobject, fieldID: jfieldID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetShortField");
            self.check_no_exception("GetShortField");
            self.check_field_type_object("GetShortField", obj, fieldID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jshort>(99)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetIntField(&self, obj: jobject, fieldID: jfieldID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetIntField");
            self.check_no_exception("GetIntField");
            self.check_field_type_object("GetIntField", obj, fieldID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jint>(100)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetLongField(&self, obj: jobject, fieldID: jfieldID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetLongField");
            self.check_no_exception("GetLongField");
            self.check_field_type_object("GetLongField", obj, fieldID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jlong>(101)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetFloatField(&self, obj: jobject, fieldID: jfieldID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetFloatField");
            self.check_no_exception("GetFloatField");
            self.check_field_type_object("GetFloatField", obj, fieldID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jfloat>(102)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetDoubleField(&self, obj: jobject, fieldID: jfieldID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetDoubleField");
            self.check_no_exception("GetDoubleField");
            self.check_field_type_object("GetDoubleField", obj, fieldID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jdouble>(103)(self.vtable, obj, fieldID)
    }

    pub unsafe fn SetObjectField(&self, obj: jobject, fieldID: jfieldID, value: jobject) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetObjectField");
            self.check_no_exception("SetObjectField");
            self.check_field_type_object("SetObjectField", obj, fieldID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jobject)>(104)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetBooleanField(&self, obj: jobject, fieldID: jfieldID, value: jboolean) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetBooleanField");
            self.check_no_exception("SetBooleanField");
            self.check_field_type_object("SetBooleanField", obj, fieldID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jboolean)>(105)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetByteField(&self, obj: jobject, fieldID: jfieldID, value: jbyte) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetByteField");
            self.check_no_exception("SetByteField");
            self.check_field_type_object("SetByteField", obj, fieldID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jbyte)>(106)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetCharField(&self, obj: jobject, fieldID: jfieldID, value: jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetCharField");
            self.check_no_exception("SetCharField");
            self.check_field_type_object("SetCharField", obj, fieldID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jchar)>(107)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetShortField(&self, obj: jobject, fieldID: jfieldID, value: jshort) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetShortField");
            self.check_no_exception("SetShortField");
            self.check_field_type_object("SetShortField", obj, fieldID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jshort)>(108)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetIntField(&self, obj: jobject, fieldID: jfieldID, value: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetIntField");
            self.check_no_exception("SetIntField");
            self.check_field_type_object("SetIntField", obj, fieldID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jint)>(109)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetLongField(&self, obj: jobject, fieldID: jfieldID, value: jlong) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetLongField");
            self.check_no_exception("SetLongField");
            self.check_field_type_object("SetLongField", obj, fieldID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jlong)>(110)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetFloatField(&self, obj: jobject, fieldID: jfieldID, value: jfloat) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetFloatField");
            self.check_no_exception("SetFloatField");
            self.check_field_type_object("SetFloatField", obj, fieldID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jfloat)>(111)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetDoubleField(&self, obj: jobject, fieldID: jfieldID, value: jdouble) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetDoubleField");
            self.check_no_exception("SetDoubleField");
            self.check_field_type_object("SetDoubleField", obj, fieldID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jdouble)>(112)(self.vtable, obj, fieldID, value)
    }



    pub unsafe fn GetMethodID(&self, class: jclass, name: *const c_char, sig: *const c_char) -> jmethodID {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetMethodID");
            self.check_no_exception("GetMethodID");
            assert!(!name.is_null(), "GetMethodID name is null");
            assert!(!sig.is_null(), "GetMethodID sig is null");
            self.check_is_class("GetMethodID", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, *const c_char, *const c_char) -> jmethodID>(33)(self.vtable, class, name, sig)
    }

    pub unsafe fn GetMethodID_str(&self, class: jclass, name: &str, sig: &str) -> jmethodID {
        let nstr = CString::new(name).unwrap();
        let nsig = CString::new(sig).unwrap();
        self.GetMethodID(class, nstr.as_ptr(), nsig.as_ptr())
    }

    pub unsafe fn CallVoidMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallVoidMethodA");
            self.check_no_exception("CallVoidMethodA");
            self.check_return_type_object("CallVoidMethodA", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype)>(63)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallVoidMethod0(&self, obj: jobject, methodID: jmethodID) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallVoidMethod");
            self.check_no_exception("CallVoidMethod");
            self.check_return_type_object("CallVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID)>(61)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallVoidMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallVoidMethod");
            self.check_no_exception("CallVoidMethod");
            self.check_return_type_object("CallVoidMethod", obj, methodID, "void");
            self.check_parameter_types_object("CallVoidMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...)>(61)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallVoidMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallVoidMethod");
            self.check_no_exception("CallVoidMethod");
            self.check_return_type_object("CallVoidMethod", obj, methodID, "void");
            self.check_parameter_types_object("CallVoidMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallVoidMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...)>(61)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallVoidMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallVoidMethod");
            self.check_no_exception("CallVoidMethod");
            self.check_return_type_object("CallVoidMethod", obj, methodID, "void");
            self.check_parameter_types_object("CallVoidMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallVoidMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallVoidMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...)>(61)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallObjectMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallObjectMethodA");
            self.check_no_exception("CallObjectMethodA");
            self.check_return_type_object("CallObjectMethodA", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallObjectMethod0(&self, obj: jobject, methodID: jmethodID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallObjectMethod");
            self.check_no_exception("CallObjectMethod");
            self.check_return_type_object("CallObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jobject>(34)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallObjectMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallObjectMethod");
            self.check_no_exception("CallObjectMethod");
            self.check_return_type_object("CallObjectMethod", obj, methodID, "object");
            self.check_parameter_types_object("CallObjectMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jobject>(34)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallObjectMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallObjectMethod");
            self.check_no_exception("CallObjectMethod");
            self.check_return_type_object("CallObjectMethod", obj, methodID, "object");
            self.check_parameter_types_object("CallObjectMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallObjectMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jobject>(34)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallObjectMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallObjectMethod");
            self.check_no_exception("CallObjectMethod");
            self.check_return_type_object("CallObjectMethod", obj, methodID, "object");
            self.check_parameter_types_object("CallObjectMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallObjectMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallObjectMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jobject>(34)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallBooleanMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallBooleanMethodA");
            self.check_no_exception("CallBooleanMethodA");
            self.check_return_type_object("CallBooleanMethodA", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jboolean>(39)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallBooleanMethod0(&self, obj: jobject, methodID: jmethodID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallBooleanMethod");
            self.check_no_exception("CallBooleanMethod");
            self.check_return_type_object("CallBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jboolean>(37)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallBooleanMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallBooleanMethod");
            self.check_no_exception("CallBooleanMethod");
            self.check_return_type_object("CallBooleanMethod", obj, methodID, "boolean");
            self.check_parameter_types_object("CallBooleanMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jboolean>(37)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallBooleanMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallBooleanMethod");
            self.check_no_exception("CallBooleanMethod");
            self.check_return_type_object("CallBooleanMethod", obj, methodID, "boolean");
            self.check_parameter_types_object("CallBooleanMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallBooleanMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jboolean>(37)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallBooleanMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallBooleanMethod");
            self.check_no_exception("CallBooleanMethod");
            self.check_return_type_object("CallBooleanMethod", obj, methodID, "boolean");
            self.check_parameter_types_object("CallBooleanMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallBooleanMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallBooleanMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jboolean>(37)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallByteMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallByteMethodA");
            self.check_no_exception("CallByteMethodA");
            self.check_return_type_object("CallByteMethodA", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jbyte>(42)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallByteMethod0(&self, obj: jobject, methodID: jmethodID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallByteMethod0");
            self.check_no_exception("CallByteMethod0");
            self.check_return_type_object("CallByteMethod0", obj, methodID, "byte");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jbyte>(40)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallByteMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallByteMethod1");
            self.check_no_exception("CallByteMethod1");
            self.check_return_type_object("CallByteMethod1", obj, methodID, "byte");
            self.check_parameter_types_object("CallByteMethod1", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jbyte>(40)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallByteMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallByteMethod2");
            self.check_no_exception("CallByteMethod2");
            self.check_return_type_object("CallByteMethod2", obj, methodID, "byte");
            self.check_parameter_types_object("CallByteMethod2", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallByteMethod2", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jbyte>(40)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallByteMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallByteMethod3");
            self.check_no_exception("CallByteMethod3");
            self.check_return_type_object("CallByteMethod3", obj, methodID, "byte");
            self.check_parameter_types_object("CallByteMethod3", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallByteMethod3", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallByteMethod3", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jbyte>(40)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallCharMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallCharMethodA");
            self.check_no_exception("CallCharMethodA");
            self.check_return_type_object("CallCharMethodA", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jchar>(45)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallCharMethod0(&self, obj: jobject, methodID: jmethodID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallCharMethod");
            self.check_no_exception("CallCharMethod");
            self.check_return_type_object("CallCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jchar>(43)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallCharMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallCharMethod");
            self.check_no_exception("CallCharMethod");
            self.check_return_type_object("CallCharMethod", obj, methodID, "char");
            self.check_parameter_types_object("CallCharMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jchar>(43)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallCharMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallCharMethod");
            self.check_no_exception("CallCharMethod");
            self.check_return_type_object("CallCharMethod", obj, methodID, "char");
            self.check_parameter_types_object("CallCharMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallCharMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jchar>(43)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallCharMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallCharMethod");
            self.check_no_exception("CallCharMethod");
            self.check_return_type_object("CallCharMethod", obj, methodID, "char");
            self.check_parameter_types_object("CallCharMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallCharMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallCharMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jchar>(43)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallShortMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallShortMethodA");
            self.check_no_exception("CallShortMethodA");
            self.check_return_type_object("CallShortMethodA", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jshort>(48)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallShortMethod0(&self, obj: jobject, methodID: jmethodID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallShortMethod");
            self.check_no_exception("CallShortMethod");
            self.check_return_type_object("CallShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jshort>(46)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallShortMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallShortMethod");
            self.check_no_exception("CallShortMethod");
            self.check_return_type_object("CallShortMethod", obj, methodID, "short");
            self.check_parameter_types_object("CallShortMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jshort>(46)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallShortMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallShortMethod");
            self.check_no_exception("CallShortMethod");
            self.check_return_type_object("CallShortMethod", obj, methodID, "short");
            self.check_parameter_types_object("CallShortMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallShortMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jshort>(46)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallShortMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallShortMethod");
            self.check_no_exception("CallShortMethod");
            self.check_return_type_object("CallShortMethod", obj, methodID, "short");
            self.check_parameter_types_object("CallShortMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallShortMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallShortMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jshort>(46)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallIntMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallIntMethodA");
            self.check_no_exception("CallIntMethodA");
            self.check_return_type_object("CallIntMethodA", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jint>(51)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallIntMethod0(&self, obj: jobject, methodID: jmethodID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallIntMethod");
            self.check_no_exception("CallIntMethod");
            self.check_return_type_object("CallIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jint>(49)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallIntMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallIntMethod");
            self.check_no_exception("CallIntMethod");
            self.check_return_type_object("CallIntMethod", obj, methodID, "int");
            self.check_parameter_types_object("CallIntMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jint>(49)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallIntMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallIntMethod");
            self.check_no_exception("CallIntMethod");
            self.check_return_type_object("CallIntMethod", obj, methodID, "int");
            self.check_parameter_types_object("CallIntMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallIntMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jint>(49)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallIntMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallIntMethod");
            self.check_no_exception("CallIntMethod");
            self.check_return_type_object("CallIntMethod", obj, methodID, "int");
            self.check_parameter_types_object("CallIntMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallIntMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallIntMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jint>(49)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallLongMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallLongMethodA");
            self.check_no_exception("CallLongMethodA");
            self.check_return_type_object("CallLongMethodA", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jlong>(54)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallLongMethod0(&self, obj: jobject, methodID: jmethodID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallLongMethod");
            self.check_no_exception("CallLongMethod");
            self.check_return_type_object("CallLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jlong>(52)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallLongMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallLongMethod");
            self.check_no_exception("CallLongMethod");
            self.check_return_type_object("CallLongMethod", obj, methodID, "long");
            self.check_parameter_types_object("CallLongMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jlong>(52)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallLongMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallLongMethod");
            self.check_no_exception("CallLongMethod");
            self.check_return_type_object("CallLongMethod", obj, methodID, "long");
            self.check_parameter_types_object("CallLongMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallLongMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jlong>(52)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallLongMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallLongMethod");
            self.check_no_exception("CallLongMethod");
            self.check_return_type_object("CallLongMethod", obj, methodID, "long");
            self.check_parameter_types_object("CallLongMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallLongMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallLongMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jlong>(52)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallFloatMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallFloatMethodA");
            self.check_no_exception("CallFloatMethodA");
            self.check_return_type_object("CallFloatMethodA", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jfloat>(57)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallFloatMethod0(&self, obj: jobject, methodID: jmethodID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallFloatMethod");
            self.check_no_exception("CallFloatMethod");
            self.check_return_type_object("CallFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jfloat>(55)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallFloatMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallFloatMethod");
            self.check_no_exception("CallFloatMethod");
            self.check_return_type_object("CallFloatMethod", obj, methodID, "float");
            self.check_parameter_types_object("CallFloatMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jfloat>(55)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallFloatMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallFloatMethod");
            self.check_no_exception("CallFloatMethod");
            self.check_return_type_object("CallFloatMethod", obj, methodID, "float");
            self.check_parameter_types_object("CallFloatMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallFloatMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jfloat>(55)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallFloatMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallFloatMethod");
            self.check_no_exception("CallFloatMethod");
            self.check_return_type_object("CallFloatMethod", obj, methodID, "float");
            self.check_parameter_types_object("CallFloatMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallFloatMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallFloatMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jfloat>(55)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallDoubleMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallDoubleMethodA");
            self.check_no_exception("CallDoubleMethodA");
            self.check_return_type_object("CallDoubleMethodA", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jdouble>(60)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallDoubleMethod0(&self, obj: jobject, methodID: jmethodID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallDoubleMethod");
            self.check_no_exception("CallDoubleMethod");
            self.check_return_type_object("CallDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jdouble>(58)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallDoubleMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallDoubleMethod");
            self.check_no_exception("CallDoubleMethod");
            self.check_return_type_object("CallDoubleMethod", obj, methodID, "double");
            self.check_parameter_types_object("CallDoubleMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jdouble>(58)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallDoubleMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallDoubleMethod");
            self.check_no_exception("CallDoubleMethod");
            self.check_return_type_object("CallDoubleMethod", obj, methodID, "double");
            self.check_parameter_types_object("CallDoubleMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallDoubleMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jdouble>(58)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallDoubleMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallDoubleMethod");
            self.check_no_exception("CallDoubleMethod");
            self.check_return_type_object("CallDoubleMethod", obj, methodID, "double");
            self.check_parameter_types_object("CallDoubleMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallDoubleMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallDoubleMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jdouble>(58)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }


    pub unsafe fn CallNonvirtualVoidMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualVoidMethodA");
            self.check_no_exception("CallNonvirtualVoidMethodA");
            self.check_return_type_object("CallNonvirtualVoidMethodA", obj, methodID, "void");
            self.check_is_class("CallNonvirtualVoidMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype)>(93)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualVoidMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualVoidMethod");
            self.check_no_exception("CallNonvirtualVoidMethod");
            self.check_return_type_object("CallNonvirtualVoidMethod", obj, methodID, "void");
            self.check_is_class("CallNonvirtualVoidMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID)>(91)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualVoidMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualVoidMethod");
            self.check_no_exception("CallNonvirtualVoidMethod");
            self.check_return_type_object("CallNonvirtualVoidMethod", obj, methodID, "void");
            self.check_is_class("CallNonvirtualVoidMethod", class);
            self.check_parameter_types_object("CallNonvirtualVoidMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...)>(91)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualVoidMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualVoidMethod");
            self.check_no_exception("CallNonvirtualVoidMethod");
            self.check_return_type_object("CallNonvirtualVoidMethod", obj, methodID, "void");
            self.check_is_class("CallNonvirtualVoidMethod", class);
            self.check_parameter_types_object("CallNonvirtualVoidMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualVoidMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...)>(91)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualVoidMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualVoidMethod");
            self.check_no_exception("CallNonvirtualVoidMethod");
            self.check_return_type_object("CallNonvirtualVoidMethod", obj, methodID, "void");
            self.check_is_class("CallNonvirtualVoidMethod", class);
            self.check_parameter_types_object("CallNonvirtualVoidMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualVoidMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualVoidMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...)>(91)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualObjectMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualObjectMethodA");
            self.check_no_exception("CallNonvirtualObjectMethodA");
            self.check_return_type_object("CallNonvirtualObjectMethodA", obj, methodID, "object");
            self.check_is_class("CallNonvirtualObjectMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jobject>(66)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualObjectMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualObjectMethod");
            self.check_no_exception("CallNonvirtualObjectMethod");
            self.check_return_type_object("CallNonvirtualObjectMethod", obj, methodID, "object");
            self.check_is_class("CallNonvirtualObjectMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jobject>(64)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualObjectMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualObjectMethod");
            self.check_no_exception("CallNonvirtualObjectMethod");
            self.check_return_type_object("CallNonvirtualObjectMethod", obj, methodID, "object");
            self.check_is_class("CallNonvirtualObjectMethod", class);
            self.check_parameter_types_object("CallNonvirtualObjectMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jobject>(64)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualObjectMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualObjectMethod");
            self.check_no_exception("CallNonvirtualObjectMethod");
            self.check_return_type_object("CallNonvirtualObjectMethod", obj, methodID, "object");
            self.check_is_class("CallNonvirtualObjectMethod", class);
            self.check_parameter_types_object("CallNonvirtualObjectMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualObjectMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jobject>(64)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualObjectMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualObjectMethod");
            self.check_no_exception("CallNonvirtualObjectMethod");
            self.check_return_type_object("CallNonvirtualObjectMethod", obj, methodID, "object");
            self.check_is_class("CallNonvirtualObjectMethod", class);
            self.check_parameter_types_object("CallNonvirtualObjectMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualObjectMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualObjectMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jobject>(64)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualBooleanMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualBooleanMethodA");
            self.check_no_exception("CallNonvirtualBooleanMethodA");
            self.check_return_type_object("CallNonvirtualBooleanMethodA", obj, methodID, "boolean");
            self.check_is_class("CallNonvirtualBooleanMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jboolean>(69)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualBooleanMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualBooleanMethod");
            self.check_no_exception("CallNonvirtualBooleanMethod");
            self.check_return_type_object("CallNonvirtualBooleanMethod", obj, methodID, "boolean");
            self.check_is_class("CallNonvirtualBooleanMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jboolean>(67)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualBooleanMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualBooleanMethod");
            self.check_no_exception("CallNonvirtualBooleanMethod");
            self.check_return_type_object("CallNonvirtualBooleanMethod", obj, methodID, "boolean");
            self.check_is_class("CallNonvirtualBooleanMethod", class);
            self.check_parameter_types_object("CallNonvirtualBooleanMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jboolean>(67)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualBooleanMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualBooleanMethod");
            self.check_no_exception("CallNonvirtualBooleanMethod");
            self.check_return_type_object("CallNonvirtualBooleanMethod", obj, methodID, "boolean");
            self.check_is_class("CallNonvirtualBooleanMethod", class);
            self.check_parameter_types_object("CallNonvirtualBooleanMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualBooleanMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jboolean>(67)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualBooleanMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualBooleanMethod");
            self.check_no_exception("CallNonvirtualBooleanMethod");
            self.check_return_type_object("CallNonvirtualBooleanMethod", obj, methodID, "boolean");
            self.check_is_class("CallNonvirtualBooleanMethod", class);
            self.check_parameter_types_object("CallNonvirtualBooleanMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualBooleanMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualBooleanMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jboolean>(67)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualByteMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualByteMethodA");
            self.check_no_exception("CallNonvirtualByteMethodA");
            self.check_return_type_object("CallNonvirtualByteMethodA", obj, methodID, "byte");
            self.check_is_class("CallNonvirtualByteMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jbyte>(72)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualByteMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualByteMethod");
            self.check_no_exception("CallNonvirtualByteMethod");
            self.check_return_type_object("CallNonvirtualByteMethod", obj, methodID, "byte");
            self.check_is_class("CallNonvirtualByteMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jbyte>(70)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualByteMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualByteMethod");
            self.check_no_exception("CallNonvirtualByteMethod");
            self.check_return_type_object("CallNonvirtualByteMethod", obj, methodID, "byte");
            self.check_is_class("CallNonvirtualByteMethod", class);
            self.check_parameter_types_object("CallNonvirtualByteMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jbyte>(70)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualByteMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualByteMethod");
            self.check_no_exception("CallNonvirtualByteMethod");
            self.check_return_type_object("CallNonvirtualByteMethod", obj, methodID, "byte");
            self.check_is_class("CallNonvirtualByteMethod", class);
            self.check_parameter_types_object("CallNonvirtualByteMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualByteMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jbyte>(70)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualByteMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualByteMethod");
            self.check_no_exception("CallNonvirtualByteMethod");
            self.check_return_type_object("CallNonvirtualByteMethod", obj, methodID, "byte");
            self.check_is_class("CallNonvirtualByteMethod", class);
            self.check_parameter_types_object("CallNonvirtualByteMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualByteMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualByteMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jbyte>(70)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualCharMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualCharMethodA");
            self.check_no_exception("CallNonvirtualCharMethodA");
            self.check_return_type_object("CallNonvirtualCharMethodA", obj, methodID, "char");
            self.check_is_class("CallNonvirtualCharMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jchar>(75)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualCharMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualCharMethod");
            self.check_no_exception("CallNonvirtualCharMethod");
            self.check_return_type_object("CallNonvirtualCharMethod", obj, methodID, "char");
            self.check_is_class("CallNonvirtualCharMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jchar>(73)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualCharMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualCharMethod");
            self.check_no_exception("CallNonvirtualCharMethod");
            self.check_return_type_object("CallNonvirtualCharMethod", obj, methodID, "char");
            self.check_is_class("CallNonvirtualCharMethod", class);
            self.check_parameter_types_object("CallNonvirtualCharMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jchar>(73)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualCharMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualCharMethod");
            self.check_no_exception("CallNonvirtualCharMethod");
            self.check_return_type_object("CallNonvirtualCharMethod", obj, methodID, "char");
            self.check_is_class("CallNonvirtualCharMethod", class);
            self.check_parameter_types_object("CallNonvirtualCharMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualCharMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jchar>(73)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualCharMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualCharMethod");
            self.check_no_exception("CallNonvirtualCharMethod");
            self.check_return_type_object("CallNonvirtualCharMethod", obj, methodID, "char");
            self.check_is_class("CallNonvirtualCharMethod", class);
            self.check_parameter_types_object("CallNonvirtualCharMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualCharMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualCharMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jchar>(73)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    
    pub unsafe fn CallNonvirtualShortMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualShortMethodA");
            self.check_no_exception("CallNonvirtualShortMethodA");
            self.check_return_type_object("CallNonvirtualShortMethodA", obj, methodID, "short");
            self.check_is_class("CallNonvirtualShortMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jshort>(78)(self.vtable, obj, class, methodID, args)
    }

    
    pub unsafe fn CallNonvirtualShortMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualShortMethod");
            self.check_no_exception("CallNonvirtualShortMethod");
            self.check_return_type_object("CallNonvirtualShortMethod", obj, methodID, "short");
            self.check_is_class("CallNonvirtualShortMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jshort>(76)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualShortMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualShortMethod");
            self.check_no_exception("CallNonvirtualShortMethod");
            self.check_return_type_object("CallNonvirtualShortMethod", obj, methodID, "short");
            self.check_is_class("CallNonvirtualShortMethod", class);
            self.check_parameter_types_object("CallNonvirtualShortMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jshort>(76)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualShortMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualShortMethod");
            self.check_no_exception("CallNonvirtualShortMethod");
            self.check_return_type_object("CallNonvirtualShortMethod", obj, methodID, "short");
            self.check_is_class("CallNonvirtualShortMethod", class);
            self.check_parameter_types_object("CallNonvirtualShortMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualShortMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jshort>(76)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualShortMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualShortMethod");
            self.check_no_exception("CallNonvirtualShortMethod");
            self.check_return_type_object("CallNonvirtualShortMethod", obj, methodID, "short");
            self.check_is_class("CallNonvirtualShortMethod", class);
            self.check_parameter_types_object("CallNonvirtualShortMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualShortMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualShortMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jshort>(76)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualIntMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualIntMethodA");
            self.check_no_exception("CallNonvirtualIntMethodA");
            self.check_return_type_object("CallNonvirtualIntMethodA", obj, methodID, "int");
            self.check_is_class("CallNonvirtualIntMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jint>(81)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualIntMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualIntMethod");
            self.check_no_exception("CallNonvirtualIntMethod");
            self.check_return_type_object("CallNonvirtualIntMethod", obj, methodID, "int");
            self.check_is_class("CallNonvirtualIntMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jint>(79)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualIntMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualIntMethod");
            self.check_no_exception("CallNonvirtualIntMethod");
            self.check_return_type_object("CallNonvirtualIntMethod", obj, methodID, "int");
            self.check_is_class("CallNonvirtualIntMethod", class);
            self.check_parameter_types_object("CallNonvirtualIntMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jint>(79)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualIntMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualIntMethod");
            self.check_no_exception("CallNonvirtualIntMethod");
            self.check_return_type_object("CallNonvirtualIntMethod", obj, methodID, "int");
            self.check_is_class("CallNonvirtualIntMethod", class);
            self.check_parameter_types_object("CallNonvirtualIntMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualIntMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jint>(79)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualIntMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualIntMethod");
            self.check_no_exception("CallNonvirtualIntMethod");
            self.check_return_type_object("CallNonvirtualIntMethod", obj, methodID, "int");
            self.check_is_class("CallNonvirtualIntMethod", class);
            self.check_parameter_types_object("CallNonvirtualIntMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualIntMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualIntMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jint>(79)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualLongMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualLongMethodA");
            self.check_no_exception("CallNonvirtualLongMethodA");
            self.check_return_type_object("CallNonvirtualLongMethodA", obj, methodID, "long");
            self.check_is_class("CallNonvirtualLongMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jlong>(84)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualLongMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualLongMethod");
            self.check_no_exception("CallNonvirtualLongMethod");
            self.check_return_type_object("CallNonvirtualLongMethod", obj, methodID, "long");
            self.check_is_class("CallNonvirtualLongMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jlong>(82)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualLongMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualLongMethod");
            self.check_no_exception("CallNonvirtualLongMethod");
            self.check_return_type_object("CallNonvirtualLongMethod", obj, methodID, "long");
            self.check_is_class("CallNonvirtualLongMethod", class);
            self.check_parameter_types_object("CallNonvirtualLongMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jlong>(82)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualLongMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualLongMethod");
            self.check_no_exception("CallNonvirtualLongMethod");
            self.check_return_type_object("CallNonvirtualLongMethod", obj, methodID, "long");
            self.check_is_class("CallNonvirtualLongMethod", class);
            self.check_parameter_types_object("CallNonvirtualLongMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualLongMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jlong>(82)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualLongMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualLongMethod");
            self.check_no_exception("CallNonvirtualLongMethod");
            self.check_return_type_object("CallNonvirtualLongMethod", obj, methodID, "long");
            self.check_is_class("CallNonvirtualLongMethod", class);
            self.check_parameter_types_object("CallNonvirtualLongMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualLongMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualLongMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jlong>(82)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualFloatMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualFloatMethodA");
            self.check_no_exception("CallNonvirtualFloatMethodA");
            self.check_return_type_object("CallNonvirtualFloatMethodA", obj, methodID, "float");
            self.check_is_class("CallNonvirtualFloatMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jfloat>(87)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualFloatMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualFloatMethod");
            self.check_no_exception("CallNonvirtualFloatMethod");
            self.check_return_type_object("CallNonvirtualFloatMethod", obj, methodID, "float");
            self.check_is_class("CallNonvirtualFloatMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jfloat>(85)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualFloatMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualFloatMethod");
            self.check_no_exception("CallNonvirtualFloatMethod");
            self.check_return_type_object("CallNonvirtualFloatMethod", obj, methodID, "float");
            self.check_is_class("CallNonvirtualFloatMethod", class);
            self.check_parameter_types_object("CallNonvirtualFloatMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jfloat>(85)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualFloatMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualFloatMethod");
            self.check_no_exception("CallNonvirtualFloatMethod");
            self.check_return_type_object("CallNonvirtualFloatMethod", obj, methodID, "float");
            self.check_is_class("CallNonvirtualFloatMethod", class);
            self.check_parameter_types_object("CallNonvirtualFloatMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualFloatMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jfloat>(85)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualFloatMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualFloatMethod");
            self.check_no_exception("CallNonvirtualFloatMethod");
            self.check_return_type_object("CallNonvirtualFloatMethod", obj, methodID, "float");
            self.check_is_class("CallNonvirtualFloatMethod", class);
            self.check_parameter_types_object("CallNonvirtualFloatMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualFloatMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualFloatMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jfloat>(85)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualDoubleMethodA(&self, obj: jobject, class: jclass, methodID: jmethodID, args: *const jtype) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualDoubleMethodA");
            self.check_no_exception("CallNonvirtualDoubleMethodA");
            self.check_return_type_object("CallNonvirtualDoubleMethodA", obj, methodID, "double");
            self.check_is_class("CallNonvirtualDoubleMethodA", class);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jclass, jmethodID, *const jtype) -> jdouble>(90)(self.vtable, obj, class, methodID, args)
    }

    pub unsafe fn CallNonvirtualDoubleMethod0(&self, obj: jobject, class: jclass, methodID: jmethodID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualDoubleMethod");
            self.check_no_exception("CallNonvirtualDoubleMethod");
            self.check_return_type_object("CallNonvirtualDoubleMethod", obj, methodID, "double");
            self.check_is_class("CallNonvirtualDoubleMethod", class);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID) -> jdouble>(88)(self.vtable, obj, class, methodID)
    }

    
    pub unsafe fn CallNonvirtualDoubleMethod1<A: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualDoubleMethod");
            self.check_no_exception("CallNonvirtualDoubleMethod");
            self.check_return_type_object("CallNonvirtualDoubleMethod", obj, methodID, "double");
            self.check_is_class("CallNonvirtualDoubleMethod", class);
            self.check_parameter_types_object("CallNonvirtualDoubleMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jdouble>(88)(self.vtable, obj, class, methodID, arg1)
    }

    
    pub unsafe fn CallNonvirtualDoubleMethod2<A: JType, B: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualDoubleMethod");
            self.check_no_exception("CallNonvirtualDoubleMethod");
            self.check_return_type_object("CallNonvirtualDoubleMethod", obj, methodID, "double");
            self.check_is_class("CallNonvirtualDoubleMethod", class);
            self.check_parameter_types_object("CallNonvirtualDoubleMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_object("CallNonvirtualDoubleMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jdouble>(88)(self.vtable, obj, class, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallNonvirtualDoubleMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, class: jclass, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallNonvirtualDoubleMethod");
            self.check_no_exception("CallNonvirtualDoubleMethod");
            self.check_return_type_object("CallNonvirtualDoubleMethod", obj, methodID, "double");
            self.check_is_class("CallNonvirtualDoubleMethod", class);
            self.check_parameter_types_object("CallNonvirtualDoubleMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_object("CallNonvirtualDoubleMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_object("CallNonvirtualDoubleMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jclass, jmethodID, ...) -> jdouble>(88)(self.vtable, obj, class, methodID, arg1, arg2, arg3)
    }


    pub unsafe fn GetStaticFieldID(&self, clazz: jclass, name: *const c_char, sig: *const c_char) -> jfieldID {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticFieldID");
            self.check_no_exception("GetStaticFieldID");
            assert!(!name.is_null(), "GetStaticFieldID name is null");
            assert!(!sig.is_null(), "GetStaticFieldID sig is null");
            self.check_is_class("GetStaticFieldID", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass, *const c_char, *const c_char) -> jfieldID>(144)(self.vtable, clazz, name, sig)
    }

    pub unsafe fn GetStaticFieldID_str(&self, class: jclass, name: &str, sig: &str) -> jfieldID {
        let nstr = CString::new(name).unwrap();
        let nsig = CString::new(sig).unwrap();
        self.GetStaticFieldID(class, nstr.as_ptr(), nsig.as_ptr())

    }

    pub unsafe fn GetStaticObjectField(&self, obj: jclass, fieldID: jfieldID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticObjectField");
            self.check_no_exception("GetStaticObjectField");
            self.check_field_type_static("GetStaticObjectField", obj, fieldID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jobject>(145)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetStaticBooleanField(&self, obj: jclass, fieldID: jfieldID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticBooleanField");
            self.check_no_exception("GetStaticBooleanField");
            self.check_field_type_static("GetStaticBooleanField", obj, fieldID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jboolean>(146)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetStaticByteField(&self, obj: jclass, fieldID: jfieldID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticByteField");
            self.check_no_exception("GetStaticByteField");
            self.check_field_type_static("GetStaticByteField", obj, fieldID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jbyte>(147)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetStaticCharField(&self, obj: jclass, fieldID: jfieldID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticCharField");
            self.check_no_exception("GetStaticCharField");
            self.check_field_type_static("GetStaticCharField", obj, fieldID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jchar>(148)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetStaticShortField(&self, obj: jclass, fieldID: jfieldID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticShortField");
            self.check_no_exception("GetStaticShortField");
            self.check_field_type_static("GetStaticShortField", obj, fieldID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jshort>(149)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetStaticIntField(&self, obj: jclass, fieldID: jfieldID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticIntField");
            self.check_no_exception("GetStaticIntField");
            self.check_field_type_static("GetStaticIntField", obj, fieldID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jint>(150)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetStaticLongField(&self, obj: jclass, fieldID: jfieldID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticLongField");
            self.check_no_exception("GetStaticLongField");
            self.check_field_type_static("GetStaticLongField", obj, fieldID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jlong>(151)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetStaticFloatField(&self, obj: jclass, fieldID: jfieldID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticFloatField");
            self.check_no_exception("GetStaticFloatField");
            self.check_field_type_static("GetStaticFloatField", obj, fieldID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jfloat>(152)(self.vtable, obj, fieldID)
    }

    pub unsafe fn GetStaticDoubleField(&self, obj: jclass, fieldID: jfieldID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticDoubleField");
            self.check_no_exception("GetStaticDoubleField");
            self.check_field_type_static("GetStaticDoubleField", obj, fieldID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID) -> jdouble>(153)(self.vtable, obj, fieldID)
    }

    pub unsafe fn SetStaticObjectField(&self, obj: jclass, fieldID: jfieldID, value: jobject) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticObjectField");
            self.check_no_exception("SetStaticObjectField");
            self.check_field_type_static("SetStaticObjectField", obj, fieldID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jobject)>(154)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetStaticBooleanField(&self, obj: jclass, fieldID: jfieldID, value: jboolean) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticBooleanField");
            self.check_no_exception("SetStaticBooleanField");
            self.check_field_type_static("SetStaticBooleanField", obj, fieldID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jboolean)>(155)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetStaticByteField(&self, obj: jclass, fieldID: jfieldID, value: jbyte) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticByteField");
            self.check_no_exception("SetStaticByteField");
            self.check_field_type_static("SetStaticByteField", obj, fieldID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jbyte)>(156)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetStaticCharField(&self, obj: jclass, fieldID: jfieldID, value: jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticCharField");
            self.check_no_exception("SetStaticCharField");
            self.check_field_type_static("SetStaticCharField", obj, fieldID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jchar)>(157)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetStaticShortField(&self, obj: jclass, fieldID: jfieldID, value: jshort) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticShortField");
            self.check_no_exception("SetStaticShortField");
            self.check_field_type_static("SetStaticShortField", obj, fieldID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jshort)>(158)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetStaticIntField(&self, obj: jclass, fieldID: jfieldID, value: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticIntField");
            self.check_no_exception("SetStaticIntField");
            self.check_field_type_static("SetStaticIntField", obj, fieldID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jint)>(159)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetStaticLongField(&self, obj: jclass, fieldID: jfieldID, value: jlong) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticLongField");
            self.check_no_exception("SetStaticLongField");
            self.check_field_type_static("SetStaticLongField", obj, fieldID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jlong)>(160)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetStaticFloatField(&self, obj: jclass, fieldID: jfieldID, value: jfloat) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticFloatField");
            self.check_no_exception("SetStaticFloatField");
            self.check_field_type_static("SetStaticFloatField", obj, fieldID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jfloat)>(161)(self.vtable, obj, fieldID, value)
    }

    pub unsafe fn SetStaticDoubleField(&self, obj: jclass, fieldID: jfieldID, value: jdouble) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetStaticDoubleField");
            self.check_no_exception("SetStaticDoubleField");
            self.check_field_type_static("SetStaticDoubleField", obj, fieldID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jfieldID, jdouble)>(162)(self.vtable, obj, fieldID, value)
    }




    pub unsafe fn GetStaticMethodID(&self, class: jclass, name: *const c_char, sig: *const c_char) -> jmethodID {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStaticMethodID");
            self.check_no_exception("GetStaticMethodID");
            self.check_is_class("GetStaticMethodID", class);
            assert!(!name.is_null(), "GetStaticMethodID name is null");
            assert!(!sig.is_null(), "GetStaticMethodID sig is null");
        }


        self.jni::<extern "system" fn(JNIEnvVTable, jobject, *const c_char, *const c_char) -> jmethodID>(113)(self.vtable, class, name, sig)
    }

    pub unsafe fn GetStaticMethodID_str(&self, class: jclass, name: &str, sig: &str) -> jmethodID {
        let nstr = CString::new(name).unwrap();
        let nsig = CString::new(sig).unwrap();
        self.GetStaticMethodID(class, nstr.as_ptr(), nsig.as_ptr())
    }


    pub unsafe fn CallStaticVoidMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticVoidMethodA");
            self.check_no_exception("CallStaticVoidMethodA");
            self.check_return_type_static("CallStaticVoidMethodA", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype)>(143)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticVoidMethod0(&self, obj: jobject, methodID: jmethodID) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticVoidMethod");
            self.check_no_exception("CallStaticVoidMethod");
            self.check_return_type_object("CallStaticVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID)>(141)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticVoidMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticVoidMethod");
            self.check_no_exception("CallStaticVoidMethod");
            self.check_return_type_object("CallStaticVoidMethod", obj, methodID, "void");
            self.check_parameter_types_static("CallStaticVoidMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...)>(141)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticVoidMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticVoidMethod");
            self.check_no_exception("CallStaticVoidMethod");
            self.check_return_type_object("CallStaticVoidMethod", obj, methodID, "void");
            self.check_parameter_types_static("CallStaticVoidMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticVoidMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...)>(141)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticVoidMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticVoidMethod");
            self.check_no_exception("CallStaticVoidMethod");
            self.check_return_type_object("CallStaticVoidMethod", obj, methodID, "void");
            self.check_parameter_types_static("CallStaticVoidMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticVoidMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticVoidMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...)>(141)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticObjectMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticObjectMethodA");
            self.check_no_exception("CallStaticObjectMethodA");
            self.check_return_type_static("CallStaticBooleanMethodA", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(116)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticObjectMethod0(&self, obj: jobject, methodID: jmethodID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticObjectMethod");
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jobject>(114)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticObjectMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticObjectMethod");
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "object");
            self.check_parameter_types_static("CallStaticObjectMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jobject>(114)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticObjectMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticObjectMethod");
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "object");
            self.check_parameter_types_static("CallStaticObjectMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticObjectMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jobject>(114)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticObjectMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticObjectMethod");
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "object");
            self.check_parameter_types_static("CallStaticObjectMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticObjectMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticObjectMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jobject>(114)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticBooleanMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticBooleanMethodA");
            self.check_no_exception("CallStaticBooleanMethodA");
            self.check_return_type_static("CallStaticBooleanMethodA", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jboolean>(119)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticBooleanMethod0(&self, obj: jobject, methodID: jmethodID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticBooleanMethod");
            self.check_no_exception("CallStaticBooleanMethod");
            self.check_return_type_object("CallStaticBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jboolean>(117)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticBooleanMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticBooleanMethod");
            self.check_no_exception("CallStaticBooleanMethod");
            self.check_return_type_object("CallStaticBooleanMethod", obj, methodID, "boolean");
            self.check_parameter_types_static("CallStaticBooleanMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jboolean>(117)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticBooleanMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticBooleanMethod");
            self.check_no_exception("CallStaticBooleanMethod");
            self.check_return_type_object("CallStaticBooleanMethod", obj, methodID, "boolean");
            self.check_parameter_types_static("CallStaticBooleanMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticBooleanMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jboolean>(117)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticBooleanMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticBooleanMethod");
            self.check_no_exception("CallStaticBooleanMethod");
            self.check_return_type_object("CallStaticBooleanMethod", obj, methodID, "boolean");
            self.check_parameter_types_static("CallStaticBooleanMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticBooleanMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticBooleanMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jboolean>(117)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticByteMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticByteMethodA");
            self.check_no_exception("CallStaticByteMethodA");
            self.check_return_type_static("CallStaticByteMethodA", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jbyte>(122)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticByteMethod0(&self, obj: jobject, methodID: jmethodID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticByteMethod");
            self.check_no_exception("CallStaticByteMethod");
            self.check_return_type_object("CallStaticByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jbyte>(120)(self.vtable, obj, methodID)
    }

    pub unsafe fn CallStaticByteMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticByteMethod");
            self.check_no_exception("CallStaticByteMethod");
            self.check_return_type_object("CallStaticByteMethod", obj, methodID, "byte");
            self.check_parameter_types_static("CallStaticByteMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jbyte>(120)(self.vtable, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticByteMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticByteMethod");
            self.check_no_exception("CallStaticByteMethod");
            self.check_return_type_object("CallStaticByteMethod", obj, methodID, "byte");
            self.check_parameter_types_static("CallStaticByteMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticByteMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jbyte>(120)(self.vtable, obj, methodID, arg1, arg2)
    }

    pub unsafe fn CallStaticByteMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticByteMethod");
            self.check_no_exception("CallStaticByteMethod");
            self.check_return_type_object("CallStaticByteMethod", obj, methodID, "byte");
            self.check_parameter_types_static("CallStaticByteMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticByteMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticByteMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jbyte>(120)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticCharMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticCharMethodA");
            self.check_no_exception("CallStaticCharMethodA");
            self.check_return_type_static("CallStaticCharMethodA", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jchar>(125)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticCharMethod0(&self, obj: jobject, methodID: jmethodID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticCharMethod");
            self.check_no_exception("CallStaticCharMethod");
            self.check_return_type_object("CallStaticCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jchar>(123)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticCharMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticCharMethod");
            self.check_no_exception("CallStaticCharMethod");
            self.check_return_type_object("CallStaticCharMethod", obj, methodID, "char");
            self.check_parameter_types_static("CallStaticCharMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jchar>(123)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticCharMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticCharMethod");
            self.check_no_exception("CallStaticCharMethod");
            self.check_return_type_object("CallStaticCharMethod", obj, methodID, "char");
            self.check_parameter_types_static("CallStaticCharMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticCharMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jchar>(123)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticCharMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticCharMethod");
            self.check_no_exception("CallStaticCharMethod");
            self.check_return_type_object("CallStaticCharMethod", obj, methodID, "char");
            self.check_parameter_types_static("CallStaticCharMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticCharMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticCharMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jchar>(123)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticShortMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticShortMethodA");
            self.check_no_exception("CallStaticShortMethodA");
            self.check_return_type_static("CallStaticShortMethodA", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jshort>(128)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticShortMethod0(&self, obj: jobject, methodID: jmethodID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticShortMethod");
            self.check_no_exception("CallStaticShortMethod");
            self.check_return_type_object("CallStaticShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jshort>(126)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticShortMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticShortMethod");
            self.check_no_exception("CallStaticShortMethod");
            self.check_return_type_object("CallStaticShortMethod", obj, methodID, "short");
            self.check_parameter_types_static("CallStaticShortMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jshort>(126)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticShortMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticShortMethod");
            self.check_no_exception("CallStaticShortMethod");
            self.check_return_type_object("CallStaticShortMethod", obj, methodID, "short");
            self.check_parameter_types_static("CallStaticShortMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticShortMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jshort>(126)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticShortMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticShortMethod");
            self.check_no_exception("CallStaticShortMethod");
            self.check_return_type_object("CallStaticShortMethod", obj, methodID, "short");
            self.check_parameter_types_static("CallStaticShortMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticShortMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticShortMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jshort>(126)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticIntMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticIntMethodA");
            self.check_no_exception("CallStaticIntMethodA");
            self.check_return_type_static("CallStaticIntMethodA", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jint>(131)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticIntMethod0(&self, obj: jobject, methodID: jmethodID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticIntMethod");
            self.check_no_exception("CallStaticIntMethod");
            self.check_return_type_object("CallStaticIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jint>(129)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticIntMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticIntMethod");
            self.check_no_exception("CallStaticIntMethod");
            self.check_return_type_object("CallStaticIntMethod", obj, methodID, "int");
            self.check_parameter_types_static("CallStaticIntMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jint>(129)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticIntMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticIntMethod");
            self.check_no_exception("CallStaticIntMethod");
            self.check_return_type_object("CallStaticIntMethod", obj, methodID, "int");
            self.check_parameter_types_static("CallStaticIntMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticIntMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jint>(129)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticIntMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticIntMethod");
            self.check_no_exception("CallStaticIntMethod");
            self.check_return_type_object("CallStaticIntMethod", obj, methodID, "int");
            self.check_parameter_types_static("CallStaticIntMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticIntMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticIntMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jint>(129)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticLongMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticLongMethodA");
            self.check_no_exception("CallStaticLongMethodA");
            self.check_return_type_static("CallStaticLongMethodA", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jlong>(134)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticLongMethod0(&self, obj: jobject, methodID: jmethodID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticLongMethod");
            self.check_no_exception("CallStaticLongMethod");
            self.check_return_type_object("CallStaticLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jlong>(132)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticLongMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticLongMethod");
            self.check_no_exception("CallStaticLongMethod");
            self.check_return_type_object("CallStaticLongMethod", obj, methodID, "long");
            self.check_parameter_types_static("CallStaticLongMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jlong>(132)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticLongMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticLongMethod");
            self.check_no_exception("CallStaticLongMethod");
            self.check_return_type_object("CallStaticLongMethod", obj, methodID, "long");
            self.check_parameter_types_static("CallStaticLongMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticLongMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jlong>(132)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticLongMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticLongMethod");
            self.check_no_exception("CallStaticLongMethod");
            self.check_return_type_object("CallStaticLongMethod", obj, methodID, "long");
            self.check_parameter_types_static("CallStaticLongMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticLongMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticLongMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jlong>(132)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticFloatMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticFloatMethodA");
            self.check_no_exception("CallStaticFloatMethodA");
            self.check_return_type_static("CallStaticFloatMethodA", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jfloat>(137)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticFloatMethod0(&self, obj: jobject, methodID: jmethodID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticFloatMethod");
            self.check_no_exception("CallStaticFloatMethod");
            self.check_return_type_object("CallStaticFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jfloat>(135)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticFloatMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticFloatMethod");
            self.check_no_exception("CallStaticFloatMethod");
            self.check_return_type_object("CallStaticFloatMethod", obj, methodID, "float");
            self.check_parameter_types_static("CallStaticFloatMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jfloat>(135)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticFloatMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticFloatMethod");
            self.check_no_exception("CallStaticFloatMethod");
            self.check_return_type_object("CallStaticFloatMethod", obj, methodID, "float");
            self.check_parameter_types_static("CallStaticFloatMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticFloatMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jfloat>(135)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticFloatMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticFloatMethod");
            self.check_no_exception("CallStaticFloatMethod");
            self.check_return_type_object("CallStaticFloatMethod", obj, methodID, "float");
            self.check_parameter_types_static("CallStaticFloatMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticFloatMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticFloatMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jfloat>(135)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticDoubleMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticDoubleMethodA");
            self.check_no_exception("CallStaticDoubleMethodA");
            self.check_return_type_static("CallStaticDoubleMethodA", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jdouble>(140)(self.vtable, obj, methodID, args)
    }

    pub unsafe fn CallStaticDoubleMethod0(&self, obj: jobject, methodID: jmethodID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticDoubleMethod");
            self.check_no_exception("CallStaticDoubleMethod");
            self.check_return_type_object("CallStaticDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID) -> jdouble>(138)(self.vtable, obj, methodID)
    }

    
    pub unsafe fn CallStaticDoubleMethod1<A: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticDoubleMethod");
            self.check_no_exception("CallStaticDoubleMethod");
            self.check_return_type_object("CallStaticDoubleMethod", obj, methodID, "double");
            self.check_parameter_types_static("CallStaticDoubleMethod", obj, methodID, arg1, 0, 1);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jdouble>(138)(self.vtable, obj, methodID, arg1)
    }

    
    pub unsafe fn CallStaticDoubleMethod2<A: JType, B: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticDoubleMethod");
            self.check_no_exception("CallStaticDoubleMethod");
            self.check_return_type_object("CallStaticDoubleMethod", obj, methodID, "double");
            self.check_parameter_types_static("CallStaticDoubleMethod", obj, methodID, arg1, 0, 2);
            self.check_parameter_types_static("CallStaticDoubleMethod", obj, methodID, arg2, 1, 2);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jdouble>(138)(self.vtable, obj, methodID, arg1, arg2)
    }

    
    pub unsafe fn CallStaticDoubleMethod3<A: JType, B: JType, C: JType>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("CallStaticDoubleMethod");
            self.check_no_exception("CallStaticDoubleMethod");
            self.check_return_type_object("CallStaticDoubleMethod", obj, methodID, "double");
            self.check_parameter_types_static("CallStaticDoubleMethod", obj, methodID, arg1, 0, 3);
            self.check_parameter_types_static("CallStaticDoubleMethod", obj, methodID, arg2, 1, 3);
            self.check_parameter_types_static("CallStaticDoubleMethod", obj, methodID, arg3, 2, 3);
        }
        self.jni::<extern "C" fn(JNIEnvVTable, jobject, jmethodID, ...) -> jdouble>(138)(self.vtable, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn NewString(&self, unicodeChars: *const jchar, len: jsize) -> jstring {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewString");
            self.check_no_exception("NewString");
            assert!(!unicodeChars.is_null(), "NewString string must not be null");
            assert!(len >= 0, "NewString len must not be negative");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, *const jchar, jsize) -> jstring>(163)(self.vtable, unicodeChars, len)
    }

    pub unsafe fn GetStringLength(&self, string: jstring) -> jsize {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStringLength");
            self.check_no_exception("GetStringLength");
            assert!(!string.is_null(), "GetStringLength string must not be null");
            self.check_if_arg_is_string("GetStringLength", string);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jstring) -> jsize>(164)(self.vtable, string)
    }

    pub unsafe fn GetStringChars(&self, string: jstring, isCopy: *mut jboolean) -> *const jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStringChars");
            self.check_no_exception("GetStringChars");
            assert!(!string.is_null(), "GetStringChars string must not be null");
            self.check_if_arg_is_string("GetStringChars", string);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jstring, *mut jboolean) -> *const jchar>(165)(self.vtable, string, isCopy)
    }

    pub unsafe fn ReleaseStringChars(&self, string: jstring, chars: *const jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseStringChars");
            assert!(!string.is_null(), "ReleaseStringChars string must not be null");
            assert!(!chars.is_null(), "ReleaseStringChars chars must not be null");
            self.check_if_arg_is_string("ReleaseStringChars", string);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jstring, *const jchar)>(166)(self.vtable, string, chars)
    }

    pub unsafe fn NewStringUTF(&self, bytes: *const c_char) -> jstring {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewStringUTF");
            self.check_no_exception("NewStringUTF");
            assert!(!bytes.is_null(), "NewStringUTF string must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, *const c_char) -> jstring>(167)(self.vtable, bytes)
    }

    pub unsafe fn NewStringUTF_str(&self, str: &str) -> jstring {
        let raw = CString::new(str).unwrap();
        let x =  self.NewStringUTF(raw.as_ptr());
        x
    }

    pub unsafe fn GetStringUTFLength(&self, string: jstring) -> jsize {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStringUTFLength");
            self.check_no_exception("GetStringUTFLength");
            assert!(!string.is_null(), "GetStringUTFLength string must not be null");
            self.check_if_arg_is_string("GetStringUTFLength", string);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jstring) -> jsize>(168)(self.vtable, string)
    }

    pub unsafe fn GetStringUTFChars(&self, string: jstring, isCopy: *mut jboolean) -> *const c_char {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStringUTFChars");
            assert!(!string.is_null(), "GetStringUTFChars string must not be null");
            self.check_if_arg_is_string("GetStringUTFChars", string);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jstring, *mut jboolean) -> *const c_char>(169)(self.vtable, string, isCopy)
    }

    ///
    /// Convenience method that calls GetStringUTFChars, copies the result
    /// into a rust String and then calls ReleaseStringUTFChars.
    ///
    /// If GetStringUTFChars fails then None is returned and ExceptionCheck should be performed.
    /// If parsing the String as utf-8 fails (it shouldn't) then None is returned.
    ///
    pub unsafe fn GetStringUTFChars_as_string(&self, string: jstring) -> Option<String> {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStringUTFChars_as_string");
            self.check_no_exception("GetStringUTFChars_as_string");
            assert!(!string.is_null(), "GetStringUTFChars_as_string string must not be null");
            self.check_if_arg_is_string("GetStringUTFChars_as_string", string);
        }

        let str = self.GetStringUTFChars(string, null_mut());
        if str.is_null() {
            return None;
        }

        let parsed = CStr::from_ptr(str).to_str();
        if parsed.is_err() {
            self.ReleaseStringUTFChars(string, str);
            return None;
        }

        let copy = parsed.map_err(|_| {}).unwrap().to_string();
        self.ReleaseStringUTFChars(string, str);
        Some(copy)
    }

    pub unsafe fn ReleaseStringUTFChars(&self, string: jstring, utf: *const c_char) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseStringUTFChars");
            assert!(!string.is_null(), "ReleaseStringUTFChars string must not be null");
            assert!(!utf.is_null(), "ReleaseStringUTFChars utf must not be null");
            self.check_if_arg_is_string("ReleaseStringUTFChars", string);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jstring, *const c_char)>(170)(self.vtable, string, utf)
    }

    pub unsafe fn GetStringRegion(&self, string: jstring, start: jsize, len: jsize, buffer: *mut jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStringRegion");
            self.check_no_exception("GetStringRegion");
            assert!(!string.is_null(), "GetStringRegion string must not be null");
            assert!(!buffer.is_null(), "GetStringRegion buffer must not be null");
            self.check_if_arg_is_string("GetStringRegion", string);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jstring, jsize, jsize, *mut jchar)>(220)(self.vtable, string, start, len, buffer)
    }

    pub unsafe fn GetStringUTFRegion(&self, string: jstring, start: jsize, len: jsize, buffer: *mut c_char) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetStringUTFRegion");
            self.check_no_exception("GetStringUTFRegion");
            assert!(!string.is_null(), "GetStringUTFRegion string must not be null");
            self.check_if_arg_is_string("GetStringUTFRegion", string);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jstring, jsize, jsize, *mut c_char)>(221)(self.vtable, string, start, len, buffer)
    }

    #[cfg(feature = "asserts")]
    thread_local! {
        //The "Critical Section" created by GetStringCritical has a lot of restrictions placed upon it.
        //This attempts to track "some" of them on a best effort basis.
        static CRITICAL_STRINGS: std::cell::RefCell<std::collections::HashMap<*const jchar, usize>> = std::cell::RefCell::new(std::collections::HashMap::new());
    }

    ///
    /// Obtains a critical pointer into a primitive java String.
    /// This pointer must be released by calling ReleaseStringCritical.
    /// No other JNI functions can be called in the current thread.
    /// The only exception being multiple consecutive calls to GetStringCritical & GetPrimitiveArrayCritical to obtain multiple critical
    /// pointers at the same time.
    ///
    /// This method will return NULL to indicate error.
    /// The JVM will most likely throw an Exception, probably an OOMError.
    /// If you obtain multiple critical pointers, you MUST release all successfully obtained critical pointers
    /// before being able to check for the exception.
    ///
    /// Special care must be taken to avoid blocking the current thread with a dependency on another JVM thread.
    /// I.e. Do not read from a pipe that is filled by another JVM thread for example.
    ///
    /// It is also ill-advised to hold onto critical pointers for long periods of time even if no dependency on another JVM Thread is made.
    /// The JVM may decide among other things to suspend garbage collection while a critical pointer is held.
    /// So reading from a Socket with a long timeout while holding a critical pointer is unlikely to be a good idea.
    /// As it may cause unintended side effects in the rest of the JVM (like running out of memory because the GC doesn't run)
    ///
    /// Failure to release critical pointers before returning execution back to Java Code should be treated as UB
    /// even tho the JVM spec fails to mention this detail.
    ///
    /// Releasing critical pointers in another thread other than the thread that created it should be treated as UB
    /// even tho the JVM spec only mentions this detail indirectly.
    ///
    /// I recommend against using this method for almost every use case.
    /// Due to newer JVM's using UTF-8 internal representation this method is likely slower than
    /// just copying out the UTF-8 string directly for newer JVMs.
    ///
    /// # Safety
    /// Writing to the returned `*const jchar` in any way is UB.
    ///
    pub unsafe fn GetStringCritical(&self, string: jstring, isCopy: *mut jboolean) -> *const jchar {
        #[cfg(feature = "asserts")]
        {
            assert!(!string.is_null(), "GetStringCritical string must not be null");
            Self::CRITICAL_POINTERS.with(|set| {
                if set.borrow().is_empty() {
                    Self::CRITICAL_STRINGS.with(|strings| {
                        if strings.borrow().is_empty() {
                            //We can only do this check if we have not yet obtained a unreleased critical on the current thread.
                            //For subsequent calls we cannot do this check.
                            self.check_no_exception("GetStringCritical");
                            self.check_if_arg_is_string("GetStringCritical", string);
                        }
                    });
                }
            });
        }

        let crit = self.jni::<extern "system" fn(JNIEnvVTable, jstring, *mut jboolean) -> *const jchar>(224)(self.vtable, string, isCopy);

        #[cfg(feature = "asserts")]
        {
            if !crit.is_null() {
                Self::CRITICAL_STRINGS.with(|set| {
                    let mut rm = set.borrow_mut();
                    let n = rm.remove(&crit).unwrap_or(0)+1;
                    rm.insert(crit, n);
                });
            }
        }

        crit
    }



    pub unsafe fn ReleaseStringCritical(&self, string: jstring, cstring: *const jchar) {
        #[cfg(feature = "asserts")]
        {
            assert!(!string.is_null(), "ReleaseStringCritical string must not be null");
            assert!(!cstring.is_null(), "ReleaseStringCritical cstring must not be null");
            Self::CRITICAL_STRINGS.with(|set| {
                let mut rm = set.borrow_mut();
                let mut n = rm.remove(&cstring).expect("ReleaseStringCritical cstring is not valid");
                if n == 0 {
                    unreachable!();
                }

                n -= 1;

                if n >= 1 {
                    rm.insert(cstring, n);
                }
            });
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jstring, *const jchar)>(225)(self.vtable, string, cstring)
    }


    pub unsafe fn GetArrayLength(&self, array: jarray) -> jsize {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetArrayLength");
            self.check_no_exception("GetArrayLength");
            assert!(!array.is_null(), "GetArrayLength array must not be null");
            self.check_is_array(array, "GetArrayLength");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jarray) -> jsize>(171)(self.vtable, array)
    }

    pub unsafe fn NewObjectArray(&self, len: jsize, elementClass: jclass, initialElement: jobject) -> jobjectArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewObjectArray");
            self.check_no_exception("NewObjectArray");
            assert!(!elementClass.is_null(), "NewObjectArray elementClass must not be null");
            assert!(len >= 0, "NewObjectArray len mot not be negative {}", len);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize, jclass, jobject) -> jobjectArray>(172)(self.vtable, len, elementClass, initialElement)
    }

    pub unsafe fn GetObjectArrayElement(&self, array: jobjectArray, index: jsize) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetObjectArrayElement");
            self.check_no_exception("GetObjectArrayElement");
            assert!(!array.is_null(), "GetObjectArrayElement array must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jobjectArray, jsize) -> jobject>(173)(self.vtable, array, index)
    }

    pub unsafe fn SetObjectArrayElement(&self, array: jobjectArray, index: jsize, value: jobject) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetObjectArrayElement");
            self.check_no_exception("SetObjectArrayElement");
            assert!(!array.is_null(), "SetObjectArrayElement array must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jobjectArray, jsize, jobject)>(174)(self.vtable, array, index, value);
    }

    pub unsafe fn NewBooleanArray(&self, size: jsize) -> jbooleanArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewBooleanArray");
            self.check_no_exception("NewBooleanArray");
            assert!(size >= 0, "NewBooleanArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize) -> jobject>(175)(self.vtable, size)
    }

    pub unsafe fn NewByteArray(&self, size: jsize) -> jbyteArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewByteArray");
            self.check_no_exception("NewByteArray");
            assert!(size >= 0, "NewByteArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize) -> jbyteArray>(176)(self.vtable, size)
    }

    pub unsafe fn NewCharArray(&self, size: jsize) -> jcharArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewCharArray");
            self.check_no_exception("NewCharArray");
            assert!(size >= 0, "NewCharArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize) -> jcharArray>(177)(self.vtable, size)
    }

    pub unsafe fn NewShortArray(&self, size: jsize) -> jshortArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewShortArray");
            self.check_no_exception("NewShortArray");
            assert!(size >= 0, "NewShortArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize) -> jshortArray>(178)(self.vtable, size)
    }

    pub unsafe fn NewIntArray(&self, size: jsize) -> jintArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewIntArray");
            self.check_no_exception("NewIntArray");
            assert!(size >= 0, "NewIntArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize) -> jintArray>(179)(self.vtable, size)
    }

    pub unsafe fn NewLongArray(&self, size: jsize) -> jlongArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewLongArray");
            self.check_no_exception("NewLongArray");
            assert!(size >= 0, "NewLongArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize) -> jlongArray>(180)(self.vtable, size)
    }

    pub unsafe fn NewFloatArray(&self, size: jsize) -> jfloatArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewFloatArray");
            self.check_no_exception("NewFloatArray");
            assert!(size >= 0, "NewFloatArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize) -> jfloatArray>(181)(self.vtable, size)
    }

    pub unsafe fn NewDoubleArray(&self, size: jsize) -> jdoubleArray {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewDoubleArray");
            self.check_no_exception("NewDoubleArray");
            assert!(size >= 0, "NewDoubleArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jsize) -> jdoubleArray>(182)(self.vtable, size)
    }

    pub unsafe fn GetBooleanArrayElements(&self, array: jbooleanArray, is_copy: *mut jboolean) -> *mut jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetBooleanArrayElements");
            self.check_no_exception("GetBooleanArrayElements");
            assert!(!array.is_null(), "GetBooleanArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, *mut jboolean) -> *mut jboolean>(183)(self.vtable, array, is_copy)
    }

    pub unsafe fn GetByteArrayElements(&self, array: jbyteArray, is_copy: *mut jboolean) -> *mut jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetByteArrayElements");
            self.check_no_exception("GetByteArrayElements");
            assert!(!array.is_null(), "GetByteArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbyteArray, *mut jboolean) -> *mut jbyte>(184)(self.vtable, array, is_copy)
    }

    pub unsafe fn GetCharArrayElements(&self, array: jcharArray, is_copy: *mut jboolean) -> *mut jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetCharArrayElements");
            self.check_no_exception("GetCharArrayElements");
            assert!(!array.is_null(), "GetCharArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jcharArray, *mut jboolean) -> *mut jchar>(185)(self.vtable, array, is_copy)
    }

    pub unsafe fn GetShortArrayElements(&self, array: jshortArray, is_copy: *mut jboolean) -> *mut jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetShortArrayElements");
            self.check_no_exception("GetShortArrayElements");
            assert!(!array.is_null(), "GetShortArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jshortArray, *mut jboolean) -> *mut jshort>(186)(self.vtable, array, is_copy)
    }

    pub unsafe fn GetIntArrayElements(&self, array: jintArray, is_copy: *mut jboolean) -> *mut jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetIntArrayElements");
            self.check_no_exception("GetIntArrayElements");
            assert!(!array.is_null(), "GetIntArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jintArray, *mut jboolean) -> *mut jint>(187)(self.vtable, array, is_copy)
    }

    pub unsafe fn GetLongArrayElements(&self, array: jlongArray, is_copy: *mut jboolean) -> *mut jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetLongArrayElements");
            self.check_no_exception("GetLongArrayElements");
            assert!(!array.is_null(), "GetLongArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jlongArray, *mut jboolean) -> *mut jlong>(188)(self.vtable, array, is_copy)
    }

    pub unsafe fn GetFloatArrayElements(&self, array: jfloatArray, is_copy: *mut jboolean) -> *mut jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetFloatArrayElements");
            self.check_no_exception("GetFloatArrayElements");
            assert!(!array.is_null(), "GetFloatArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jfloatArray, *mut jboolean) -> *mut jfloat>(189)(self.vtable, array, is_copy)
    }

    pub unsafe fn GetDoubleArrayElements(&self, array: jdoubleArray, is_copy: *mut jboolean) -> *mut jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetDoubleArrayElements");
            self.check_no_exception("GetDoubleArrayElements");
            assert!(!array.is_null(), "GetDoubleArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jdoubleArray, *mut jboolean) -> *mut jdouble>(190)(self.vtable, array, is_copy)
    }

    pub unsafe fn ReleaseBooleanArrayElements(&self, array: jbooleanArray, elems: *mut jboolean, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseBooleanArrayElements");
            assert!(!array.is_null(), "ReleaseBooleanArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseBooleanArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseBooleanArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, *mut jboolean, jint)>(191)(self.vtable, array, elems, mode);
    }

    pub unsafe fn ReleaseByteArrayElements(&self, array: jbyteArray, elems: *mut jbyte, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseByteArrayElements");
            assert!(!array.is_null(), "ReleaseByteArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseByteArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseByteArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbyteArray, *mut jbyte, jint)>(192)(self.vtable, array, elems, mode);
    }

    pub unsafe fn ReleaseCharArrayElements(&self, array: jcharArray, elems: *mut jchar, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseCharArrayElements");
            assert!(!array.is_null(), "ReleaseCharArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseCharArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseCharArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jcharArray, *mut jchar, jint)>(193)(self.vtable, array, elems, mode);
    }

    pub unsafe fn ReleaseShortArrayElements(&self, array: jshortArray, elems: *mut jshort, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseShortArrayElements");
            assert!(!array.is_null(), "ReleaseShortArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseShortArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseShortArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jshortArray, *mut jshort, jint)>(194)(self.vtable, array, elems, mode);
    }

    pub unsafe fn ReleaseIntArrayElements(&self, array: jintArray, elems: *mut jint, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseIntArrayElements");
            assert!(!array.is_null(), "ReleaseIntArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseIntArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseIntArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jintArray, *mut jint, jint)>(195)(self.vtable, array, elems, mode);
    }

    pub unsafe fn ReleaseLongArrayElements(&self, array: jlongArray, elems: *mut jlong, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseLongArrayElements");
            assert!(!array.is_null(), "ReleaseLongArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseLongArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseLongArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jlongArray, *mut jlong, jint)>(196)(self.vtable, array, elems, mode);
    }

    pub unsafe fn ReleaseFloatArrayElements(&self, array: jfloatArray, elems: *mut jfloat, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseFloatArrayElements");
            assert!(!array.is_null(), "ReleaseFloatArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseFloatArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseFloatArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jfloatArray, *mut jfloat, jint)>(197)(self.vtable, array, elems, mode);
    }

    pub unsafe fn ReleaseDoubleArrayElements(&self, array: jdoubleArray, elems: *mut jdouble, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ReleaseDoubleArrayElements");
            assert!(!array.is_null(), "ReleaseDoubleArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseDoubleArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseDoubleArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jdoubleArray, *mut jdouble, jint)>(198)(self.vtable, array, elems, mode);
    }

    ///
    /// Copies data from the jbooleanArray `array` starting from the given `start` index into the memory pointed to by `buf`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines
    ///
    /// # Arguments
    /// * `array` - handle to a Java jbooleanArray
    /// * `start` - the index of the first element to copy in the Java jbooleanArray
    /// * `len` - amount of data to be copied
    /// * `buf` - pointer to memory where the data should be copied to
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is written into `buf` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jbooleanArray.
    /// `buf` must be valid non-null pointer to memory with enough capacity to store `len` bytes.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jbooleanArray, chunk_buffer: &mut [bool], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///     env.GetBooleanArrayRegion(array, chunk_offset as jsize, chunk_buffer.len() as jsize, chunk_buffer.as_mut_ptr());
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetBooleanArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *mut jboolean) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetBooleanArrayRegion");
            self.check_no_exception("GetBooleanArrayRegion");
            assert!(!array.is_null(), "GetBooleanArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetBooleanArrayRegion buf must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *mut jboolean)>(199)(self.vtable, array, start, len, buf);
    }

    ///
    /// Copies data from the jbyteArray `array` starting from the given `start` index into the memory pointed to by `buf`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines
    ///
    /// # Arguments
    /// * `array` - handle to a Java jbyteArray
    /// * `start` - the index of the first element to copy in the Java jbyteArray
    /// * `len` - amount of data to be copied
    /// * `buf` - pointer to memory where the data should be copied to
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is written into `buf` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jbyteArray.
    /// `buf` must be valid non-null pointer to memory with enough capacity to store `len` bytes.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jbyteArray, chunk_buffer: &mut [i8], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetByteArrayRegion(array, chunk_offset as jsize, chunk_buffer.len() as jsize, chunk_buffer.as_mut_ptr());
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetByteArrayRegion(&self, array: jbyteArray, start: jsize, len: jsize, buf: *mut jbyte) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetByteArrayRegion");
            self.check_no_exception("GetByteArrayRegion");
            assert!(!array.is_null(), "GetByteArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetByteArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *mut jbyte)>(200)(self.vtable, array, start, len, buf);
    }

    ///
    /// Copies data from the jbyteArray `array` starting from the given `start` index into the slice `buf`.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jbyteArray.
    /// * `start` - the index of the first element to copy in the Java jbyteArray
    /// * `buf` - the slice to copy data into
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside buf if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jbyteArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jbyteArray, chunk_buffer: &mut [i8], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetByteArrayRegion_into_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetByteArrayRegion_into_slice(&self, array: jbyteArray, start: jsize, buf: &mut [i8]) {
        self.GetByteArrayRegion(array, start, buf.len() as jsize, buf.as_mut_ptr());
    }

    ///
    /// Copies data from the slice `buf` into the jbyteArray `array` starting at the given `start` index.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jbyteArray.
    /// * `start` - the index where the first element should be coped into in the Java jybteArray
    /// * `buf` - the slice where data is copied from
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside `array` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jbyteArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_rust_to_java(env: JNIEnv,
    ///         array: jbyteArray, chunk_buffer: &[i8], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.SetByteArrayRegion_from_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn SetByteArrayRegion_from_slice(&self, array: jbyteArray, start: jsize, buf: &[i8]) {
        self.SetByteArrayRegion(array, start, buf.len() as jsize, buf.as_ptr());
    }

    ///
    /// Copies data from a Java jbyteArray `array` into a new Vec<i8>
    ///
    /// # Arguments
    /// * `array` - handle to a Java jbyteArray.
    /// * `start` - the index of the first element to copy in the Java jbyteArray
    /// * `len` - the amount of data that should be copied. If `None` then all remaining elements in the array are copied.
    ///
    /// If `len` is `Some` and negative or 0 then an empty Vec<i8> is returned.
    ///
    /// # Returns:
    /// a new Vec<i8> that contains the copied data.
    ///
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` was Some and is larger than the amount of remaining elements in the array.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or `start` is >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside the returned Vec<i8> if this function throws an exception
    /// * Data partially written
    /// * No data written
    ///
    /// It is only guaranteed that this function never returns uninitialized memory.
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jbyteArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_entire_java_array_to_rust(env: JNIEnv, array: jbyteArray) -> Vec<i8> {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///     env.GetByteArrayRegion_as_vec(array, 0, None)
    /// }
    /// ```
    ///
    pub unsafe fn GetByteArrayRegion_as_vec(&self, array: jbyteArray, start: jsize, len: Option<jsize>) -> Vec<i8> {
        let len = len.unwrap_or_else(||self.GetArrayLength(array)-start);
        if len < 0 {
           return Vec::new();
        }
        let mut data = vec![0i8; len as usize];
        self.GetByteArrayRegion_into_slice(array, start, data.as_mut_slice());
        data
    }

    ///
    /// Copies data from the jcharArray `array` starting from the given `start` index into the memory pointed to by `buf`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines
    ///
    /// # Arguments
    /// * `array` - handle to a Java jcharArray
    /// * `start` - the index of the first element to copy in the Java jcharArray
    /// * `len` - amount of data to be copied
    /// * `buf` - pointer to memory where the data should be copied to
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is written into `buf` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jcharArray.
    /// `buf` must be valid non-null pointer to memory with enough capacity and proper alignment to store `len` jchar's.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jcharArray, chunk_buffer: &mut [u16], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetCharArrayRegion(array, chunk_offset as jsize, chunk_buffer.len() as jsize, chunk_buffer.as_mut_ptr());
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetCharArrayRegion(&self, array: jcharArray, start: jsize, len: jsize, buf: *mut jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetCharArrayRegion");
            self.check_no_exception("GetCharArrayRegion");
            assert!(!array.is_null(), "GetCharArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetCharArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jchar>()), "GetCharArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *mut jchar)>(201)(self.vtable, array, start, len, buf);
    }

    ///
    /// Copies data from the jcharArray `array` starting from the given `start` index into the slice `buf`.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jcharArray.
    /// * `start` - the index of the first element to copy in the Java jcharArray
    /// * `buf` - the slice to copy data into
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside buf if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jcharArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jcharArray, chunk_buffer: &mut [u16], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetCharArrayRegion_into_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetCharArrayRegion_into_slice(&self, array: jcharArray, start: jsize, buf: &mut [u16]) {
        self.GetCharArrayRegion(array, start, buf.len() as jsize, buf.as_mut_ptr());
    }

    ///
    /// Copies data from the slice `buf` into the jcharArray `array` starting at the given `start` index.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jcharArray.
    /// * `start` - the index where the first element should be coped into in the Java jcharArray
    /// * `buf` - the slice where data is copied from
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside `array` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jcharArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_rust_to_java(env: JNIEnv,
    ///         array: jcharArray, chunk_buffer: &[u16], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.SetCharArrayRegion_from_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn SetCharArrayRegion_from_slice(&self, array: jcharArray, start: jsize, buf: &[u16]) {
        self.SetCharArrayRegion(array, start, buf.len() as jsize, buf.as_ptr());
    }

    ///
    /// Copies data from a Java jcharArray `array` into a new Vec<u16>
    ///
    /// # Arguments
    /// * `array` - handle to a Java jcharArray.
    /// * `start` - the index of the first element to copy in the Java jcharArray
    /// * `len` - the amount of data that should be copied. If `None` then all remaining elements in the array are copied.
    ///
    /// If `len` is `Some` and negative or 0 then an empty Vec<u16> is returned.
    ///
    /// # Returns:
    /// a new Vec<u16> that contains the copied data.
    ///
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` was Some and is larger than the amount of remaining elements in the array.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or `start` is >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside the returned Vec<u16> if this function throws an exception
    /// * Data partially written
    /// * No data written
    ///
    /// It is only guaranteed that this function never returns uninitialized memory.
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jbyteArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_entire_java_array_to_rust(env: JNIEnv, array: jcharArray) -> Vec<u16> {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///     env.GetCharArrayRegion_as_vec(array, 0, None)
    /// }
    /// ```
    ///
    pub unsafe fn GetCharArrayRegion_as_vec(&self, array: jcharArray, start: jsize, len: Option<jsize>) -> Vec<u16> {
        let len = len.unwrap_or_else(||self.GetArrayLength(array)-start);
        if len < 0 {
            return Vec::new();
        }
        let mut data = vec![0u16; len as usize];
        self.GetCharArrayRegion_into_slice(array, start, data.as_mut_slice());
        data
    }

    ///
    /// Copies data from the jshortArray `array` starting from the given `start` index into the memory pointed to by `buf`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines
    ///
    /// # Arguments
    /// * `array` - handle to a Java jshortArray
    /// * `start` - the index of the first element to copy in the Java jshortArray
    /// * `len` - amount of data to be copied
    /// * `buf` - pointer to memory where the data should be copied to
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is written into `buf` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jshortArray.
    /// `buf` must be valid non-null pointer to memory with enough capacity and proper alignment to store `len` jshort's.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jshortArray, chunk_buffer: &mut [i16], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetShortArrayRegion(array, chunk_offset as jsize, chunk_buffer.len() as jsize, chunk_buffer.as_mut_ptr());
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetShortArrayRegion(&self, array: jshortArray, start: jsize, len: jsize, buf: *mut jshort) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetShortArrayRegion");
            self.check_no_exception("GetShortArrayRegion");
            assert!(!array.is_null(), "GetShortArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetShortArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jshort>()), "GetShortArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *mut jshort)>(202)(self.vtable, array, start, len, buf);
    }

    ///
    /// Copies data from the jshortArray `array` starting from the given `start` index into the slice `buf`.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jshortArray.
    /// * `start` - the index of the first element to copy in the Java jshortArray
    /// * `buf` - the slice to copy data into
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside buf if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jshortArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jshortArray, chunk_buffer: &mut [i16], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetShortArrayRegion_into_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetShortArrayRegion_into_slice(&self, array: jshortArray, start: jsize, buf: &mut [i16]) {
        self.GetShortArrayRegion(array, start, buf.len() as jsize, buf.as_mut_ptr());
    }

    ///
    /// Copies data from the slice `buf` into the jshortArray `array` starting at the given `start` index.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jshortArray.
    /// * `start` - the index where the first element should be coped into in the Java jshortArray
    /// * `buf` - the slice where data is copied from
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside `array` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jshortArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_rust_to_java(env: JNIEnv,
    ///         array: jshortArray, chunk_buffer: &[i16], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.SetShortArrayRegion_from_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn SetShortArrayRegion_from_slice(&self, array: jshortArray, start: jsize, buf: &[i16]) {
        self.SetShortArrayRegion(array, start, buf.len() as jsize, buf.as_ptr());
    }

    ///
    /// Copies data from a Java jshortArray `array` into a new Vec<i16>
    ///
    /// # Arguments
    /// * `array` - handle to a Java jshortArray.
    /// * `start` - the index of the first element to copy in the Java jshortArray
    /// * `len` - the amount of data that should be copied. If `None` then all remaining elements in the array are copied.
    ///
    /// If `len` is `Some` and negative or 0 then an empty Vec<i16> is returned.
    ///
    /// # Returns:
    /// a new Vec<i16> that contains the copied data.
    ///
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` was Some and is larger than the amount of remaining elements in the array.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or `start` is >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside the returned Vec<i16> if this function throws an exception
    /// * Data partially written
    /// * No data written
    ///
    /// It is only guaranteed that this function never returns uninitialized memory.
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jshortArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_entire_java_array_to_rust(env: JNIEnv, array: jshortArray) -> Vec<i16> {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///     env.GetShortArrayRegion_as_vec(array, 0, None)
    /// }
    /// ```
    ///
    pub unsafe fn GetShortArrayRegion_as_vec(&self, array: jshortArray, start: jsize, len: Option<jsize>) -> Vec<i16> {
        let len = len.unwrap_or_else(||self.GetArrayLength(array)-start);
        if len < 0 {
            return Vec::new();
        }
        let mut data = vec![0i16; len as usize];
        self.GetShortArrayRegion_into_slice(array, start, data.as_mut_slice());
        data
    }

    ///
    /// Copies data from the jintArray `array` starting from the given `start` index into the memory pointed to by `buf`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines
    ///
    /// # Arguments
    /// * `array` - handle to a Java jintArray
    /// * `start` - the index of the first element to copy in the Java jintArray
    /// * `len` - amount of data to be copied
    /// * `buf` - pointer to memory where the data should be copied to
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is written into `buf` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jintArray.
    /// `buf` must be valid non-null pointer to memory with enough capacity and proper alignment to store `len` jint's.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jintArray, chunk_buffer: &mut [i32], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetIntArrayRegion(array, chunk_offset as jsize, chunk_buffer.len() as jsize, chunk_buffer.as_mut_ptr());
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetIntArrayRegion(&self, array: jintArray, start: jsize, len: jsize, buf: *mut jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetIntArrayRegion");
            self.check_no_exception("GetIntArrayRegion");
            assert!(!array.is_null(), "GetIntArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetIntArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jint>()), "GetIntArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *mut jint)>(203)(self.vtable, array, start, len, buf);
    }

    ///
    /// Copies data from the jintArray `array` starting from the given `start` index into the slice `buf`.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jintArray.
    /// * `start` - the index of the first element to copy in the Java jintArray
    /// * `buf` - the slice to copy data into
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside buf if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jintArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jintArray, chunk_buffer: &mut [i32], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetIntArrayRegion_into_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetIntArrayRegion_into_slice(&self, array: jshortArray, start: jsize, buf: &mut [i32]) {
        self.GetIntArrayRegion(array, start, buf.len() as jsize, buf.as_mut_ptr());
    }

    ///
    /// Copies data from the slice `buf` into the jintArray `array` starting at the given `start` index.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jintArray.
    /// * `start` - the index where the first element should be coped into in the Java jintArray
    /// * `buf` - the slice where data is copied from
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside `array` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jintArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_rust_to_java(env: JNIEnv,
    ///         array: jintArray, chunk_buffer: &[i32], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.SetIntArrayRegion_from_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn SetIntArrayRegion_from_slice(&self, array: jintArray, start: jsize, buf: &[i32]) {
        self.SetIntArrayRegion(array, start, buf.len() as jsize, buf.as_ptr());
    }

    ///
    /// Copies data from a Java jintArray `array` into a new Vec<i32>
    ///
    /// # Arguments
    /// * `array` - handle to a Java jintArray.
    /// * `start` - the index of the first element to copy in the Java jintArray
    /// * `len` - the amount of data that should be copied. If `None` then all remaining elements in the array are copied.
    ///
    /// If `len` is `Some` and negative or 0 then an empty Vec<i16> is returned.
    ///
    /// # Returns:
    /// a new Vec<i32> that contains the copied data.
    ///
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` was Some and is larger than the amount of remaining elements in the array.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or `start` is >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside the returned Vec<i32> if this function throws an exception
    /// * Data partially written
    /// * No data written
    ///
    /// It is only guaranteed that this function never returns uninitialized memory.
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jintArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_entire_java_array_to_rust(env: JNIEnv, array: jintArray) -> Vec<i32> {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///     env.GetIntArrayRegion_as_vec(array, 0, None)
    /// }
    /// ```
    ///
    pub unsafe fn GetIntArrayRegion_as_vec(&self, array: jintArray, start: jsize, len: Option<jsize>) -> Vec<i32> {
        let len = len.unwrap_or_else(||self.GetArrayLength(array)-start);
        if len < 0 {
            return Vec::new();
        }
        let mut data = vec![0i32; len as usize];
        self.GetIntArrayRegion_into_slice(array, start, data.as_mut_slice());
        data
    }

    ///
    /// Copies data from the jlongArray `array` starting from the given `start` index into the memory pointed to by `buf`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines
    ///
    /// # Arguments
    /// * `array` - handle to a Java jlongArray
    /// * `start` - the index of the first element to copy in the Java jlongArray
    /// * `len` - amount of data to be copied
    /// * `buf` - pointer to memory where the data should be copied to
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is written into `buf` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jlongArray.
    /// `buf` must be valid non-null pointer to memory with enough capacity and proper alignment to store `len` jlong's.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jlongArray, chunk_buffer: &mut [i64], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetLongArrayRegion(array, chunk_offset as jsize, chunk_buffer.len() as jsize, chunk_buffer.as_mut_ptr());
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetLongArrayRegion(&self, array: jlongArray, start: jsize, len: jsize, buf: *mut jlong) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetLongArrayRegion");
            self.check_no_exception("GetLongArrayRegion");
            assert!(!array.is_null(), "GetLongArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetLongArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jlong>()), "GetLongArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *mut jlong)>(204)(self.vtable, array, start, len, buf);
    }

    ///
    /// Copies data from the jlongArray `array` starting from the given `start` index into the slice `buf`.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jlongArray.
    /// * `start` - the index of the first element to copy in the Java jlongArray
    /// * `buf` - the slice to copy data into
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside buf if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jlongArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jlongArray, chunk_buffer: &mut [i64], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetLongArrayRegion_into_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetLongArrayRegion_into_slice(&self, array: jlongArray, start: jsize, buf: &mut [i64]) {
        self.GetLongArrayRegion(array, start, buf.len() as jsize, buf.as_mut_ptr());
    }

    ///
    /// Copies data from the slice `buf` into the jlongArray `array` starting at the given `start` index.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jlongArray.
    /// * `start` - the index where the first element should be coped into in the Java jlongArray
    /// * `buf` - the slice where data is copied from
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside `array` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jlongArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_rust_to_java(env: JNIEnv,
    ///         array: jlongArray, chunk_buffer: &[i64], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.SetLongArrayRegion_from_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn SetLongArrayRegion_from_slice(&self, array: jlongArray, start: jsize, buf: &[i64]) {
        self.SetLongArrayRegion(array, start, buf.len() as jsize, buf.as_ptr());
    }

    ///
    /// Copies data from a Java jlongArray `array` into a new Vec<i64>
    ///
    /// # Arguments
    /// * `array` - handle to a Java jlongArray.
    /// * `start` - the index of the first element to copy in the Java jlongArray
    /// * `len` - the amount of data that should be copied. If `None` then all remaining elements in the array are copied.
    ///
    /// If `len` is `Some` and negative or 0 then an empty Vec<i64> is returned.
    ///
    /// # Returns:
    /// a new Vec<i64> that contains the copied data.
    ///
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` was Some and is larger than the amount of remaining elements in the array.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or `start` is >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside the returned Vec<i64> if this function throws an exception
    /// * Data partially written
    /// * No data written
    ///
    /// It is only guaranteed that this function never returns uninitialized memory.
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jlongArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_entire_java_array_to_rust(env: JNIEnv, array: jlongArray) -> Vec<i64> {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///     env.GetLongArrayRegion_as_vec(array, 0, None)
    /// }
    /// ```
    ///
    pub unsafe fn GetLongArrayRegion_as_vec(&self, array: jlongArray, start: jsize, len: Option<jsize>) -> Vec<i64> {
        let len = len.unwrap_or_else(||self.GetArrayLength(array)-start);
        if len < 0 {
            return Vec::new();
        }
        let mut data = vec![0i64; len as usize];
        self.GetLongArrayRegion_into_slice(array, start, data.as_mut_slice());
        data
    }

    ///
    /// Copies data from the jfloatArray `array` starting from the given `start` index into the memory pointed to by `buf`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines
    ///
    /// # Arguments
    /// * `array` - handle to a Java jfloatArray
    /// * `start` - the index of the first element to copy in the Java jfloatArray
    /// * `len` - amount of data to be copied
    /// * `buf` - pointer to memory where the data should be copied to
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is written into `buf` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jfloatArray.
    /// `buf` must be valid non-null pointer to memory with enough capacity and proper alignment to store `len` jfloat's.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jfloatArray, chunk_buffer: &mut [jfloat], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetFloatArrayRegion(array, chunk_offset as jsize, chunk_buffer.len() as jsize, chunk_buffer.as_mut_ptr());
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetFloatArrayRegion(&self, array: jfloatArray, start: jsize, len: jsize, buf: *mut jfloat) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetFloatArrayRegion");
            self.check_no_exception("GetFloatArrayRegion");
            assert!(!array.is_null(), "GetFloatArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetFloatArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jfloat>()), "GetFloatArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *mut jfloat)>(205)(self.vtable, array, start, len, buf);
    }

    ///
    /// Copies data from the jfloatArray `array` starting from the given `start` index into the slice `buf`.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jfloatArray.
    /// * `start` - the index of the first element to copy in the Java jfloatArray
    /// * `buf` - the slice to copy data into
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside buf if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jfloatArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jfloatArray, chunk_buffer: &mut [jfloat], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetFloatArrayRegion_into_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetFloatArrayRegion_into_slice(&self, array: jfloatArray, start: jsize, buf: &mut [jfloat]) {
        self.GetFloatArrayRegion(array, start, buf.len() as jsize, buf.as_mut_ptr());
    }

    ///
    /// Copies data from the slice `buf` into the jfloatArray `array` starting at the given `start` index.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jfloatArray.
    /// * `start` - the index where the first element should be coped into in the Java jfloatArray
    /// * `buf` - the slice where data is copied from
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside `array` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jfloatArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_rust_to_java(env: JNIEnv,
    ///         array: jfloatArray, chunk_buffer: &[jfloat], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.SetFloatArrayRegion_from_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn SetFloatArrayRegion_from_slice(&self, array: jfloatArray, start: jsize, buf: &[jfloat]) {
        self.SetFloatArrayRegion(array, start, buf.len() as jsize, buf.as_ptr());
    }

    ///
    /// Copies data from a Java jfloatArray `array` into a new Vec<f32>
    ///
    /// # Arguments
    /// * `array` - handle to a Java jfloatArray.
    /// * `start` - the index of the first element to copy in the Java jfloatArray
    /// * `len` - the amount of data that should be copied. If `None` then all remaining elements in the array are copied.
    ///
    /// If `len` is `Some` and negative or 0 then an empty Vec<f32> is returned.
    ///
    /// # Returns:
    /// a new Vec<f32> that contains the copied data.
    ///
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` was Some and is larger than the amount of remaining elements in the array.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or `start` is >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside the returned Vec<f32> if this function throws an exception
    /// * Data partially written
    /// * No data written
    ///
    /// It is only guaranteed that this function never returns uninitialized memory.
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jfloatArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_entire_java_array_to_rust(env: JNIEnv, array: jfloatArray) -> Vec<f32> {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///     env.GetFloatArrayRegion_as_vec(array, 0, None)
    /// }
    /// ```
    ///
    pub unsafe fn GetFloatArrayRegion_as_vec(&self, array: jfloatArray, start: jsize, len: Option<jsize>) -> Vec<f32> {
        let len = len.unwrap_or_else(||self.GetArrayLength(array)-start);
        if len < 0 {
            return Vec::new();
        }
        let mut data = vec![0f32; len as usize];
        self.GetFloatArrayRegion_into_slice(array, start, data.as_mut_slice());
        data
    }

    ///
    /// Copies data from the jdoubleArray `array` starting from the given `start` index into the memory pointed to by `buf`.
    ///
    /// https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/functions.html#Get_PrimitiveType_ArrayRegion_routines
    ///
    /// # Arguments
    /// * `array` - handle to a Java jdoubleArray
    /// * `start` - the index of the first element to copy in the Java jdoubleArray
    /// * `len` - amount of data to be copied
    /// * `buf` - pointer to memory where the data should be copied to
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is written into `buf` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jdoubleArray.
    /// `buf` must be valid non-null pointer to memory with enough capacity and proper alignment to store `len` jdouble's.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jdoubleArray, chunk_buffer: &mut [jdouble], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetDoubleArrayRegion(array, chunk_offset as jsize, chunk_buffer.len() as jsize, chunk_buffer.as_mut_ptr());
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetDoubleArrayRegion(&self, array: jdoubleArray, start: jsize, len: jsize, buf: *mut jdouble) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetDoubleArrayRegion");
            self.check_no_exception("GetDoubleArrayRegion");
            assert!(!array.is_null(), "GetDoubleArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetDoubleArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jdouble>()), "GetDoubleArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *mut jdouble)>(206)(self.vtable, array, start, len, buf);
    }

    ///
    /// Copies data from the jdoubleArray `array` starting from the given `start` index into the slice `buf`.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jdoubleArray.
    /// * `start` - the index of the first element to copy in the Java jdoubleArray
    /// * `buf` - the slice to copy data into
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside buf if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jdoubleArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_java_to_rust(env: JNIEnv,
    ///         array: jdoubleArray, chunk_buffer: &mut [jdouble], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.GetDoubleArrayRegion_into_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn GetDoubleArrayRegion_into_slice(&self, array: jdoubleArray, start: jsize, buf: &mut [jdouble]) {
        self.GetDoubleArrayRegion(array, start, buf.len() as jsize, buf.as_mut_ptr());
    }

    ///
    /// Copies data from the slice `buf` into the jfloatArray `array` starting at the given `start` index.
    ///
    /// # Arguments
    /// * `array` - handle to a Java jfloatArray.
    /// * `start` - the index where the first element should be coped into in the Java jfloatArray
    /// * `buf` - the slice where data is copied from
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if the slice `buf` is larger than the amount of remaining elements in the `array`.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside `array` if this function throws an exception.
    /// * Data partially written
    /// * No data written
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jfloatArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_chunk_from_rust_to_java(env: JNIEnv,
    ///         array: jfloatArray, chunk_buffer: &[jdouble], chunk_offset: usize) -> bool {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///
    ///     env.SetDoubleArrayRegion_from_slice(array, chunk_offset as jsize, chunk_buffer);
    ///     if env.ExceptionCheck() {
    ///         //ArrayIndexOutOfBoundsException
    ///         env.ExceptionClear();
    ///         return false;
    ///     }
    ///     true
    /// }
    /// ```
    ///
    pub unsafe fn SetDoubleArrayRegion_from_slice(&self, array: jdoubleArray, start: jsize, buf: &[jdouble]) {
        self.SetDoubleArrayRegion(array, start, buf.len() as jsize, buf.as_ptr());
    }

    ///
    /// Copies data from a Java jdoubleArray `array` into a new Vec<f64>
    ///
    /// # Arguments
    /// * `array` - handle to a Java jdoubleArray.
    /// * `start` - the index of the first element to copy in the Java jdoubleArray
    /// * `len` - the amount of data that should be copied. If `None` then all remaining elements in the array are copied.
    ///
    /// If `len` is `Some` and negative or 0 then an empty Vec<f64> is returned.
    ///
    /// # Returns:
    /// a new Vec<f64> that contains the copied data.
    ///
    ///
    /// # Throws Java Exception:
    /// * `ArrayIndexOutOfBoundsException` - if `len` was Some and is larger than the amount of remaining elements in the array.
    /// * `ArrayIndexOutOfBoundsException` - if `start` is negative or `start` is >= env.GetArrayLength(array)
    ///
    /// It is JVM implementation specific what is stored inside the returned Vec<f64> if this function throws an exception
    /// * Data partially written
    /// * No data written
    ///
    /// It is only guaranteed that this function never returns uninitialized memory.
    ///
    /// # Safety
    /// `array` must be a valid non-null reference to a jdoubleArray.
    ///
    /// # Example
    /// ```rust
    /// use jni_simple::{*};
    ///
    /// unsafe fn copy_entire_java_array_to_rust(env: JNIEnv, array: jdoubleArray) -> Vec<jdouble> {
    ///     if array.is_null() {
    ///         panic!("Java Array is null")
    ///     }
    ///     env.GetDoubleArrayRegion_as_vec(array, 0, None)
    /// }
    /// ```
    ///
    pub unsafe fn GetDoubleArrayRegion_as_vec(&self, array: jdoubleArray, start: jsize, len: Option<jsize>) -> Vec<f64> {
        let len = len.unwrap_or_else(||self.GetArrayLength(array)-start);
        if len < 0 {
            return Vec::new();
        }
        let mut data = vec![0f64; len as usize];
        self.GetDoubleArrayRegion_into_slice(array, start, data.as_mut_slice());
        data
    }

    pub unsafe fn SetBooleanArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jboolean) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetBooleanArrayRegion");
            self.check_no_exception("SetBooleanArrayRegion");
            assert!(!array.is_null(), "SetBooleanArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetBooleanArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *const jboolean)>(207)(self.vtable, array, start, len, buf);
    }

    pub unsafe fn SetByteArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jbyte) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetByteArrayRegion");
            self.check_no_exception("SetByteArrayRegion");
            assert!(!array.is_null(), "SetByteArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetByteArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *const jbyte)>(208)(self.vtable, array, start, len, buf);
    }

    pub unsafe fn SetCharArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetCharArrayRegion");
            self.check_no_exception("SetCharArrayRegion");
            assert!(!array.is_null(), "SetCharArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetCharArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jchar>()), "SetCharArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *const jchar)>(209)(self.vtable, array, start, len, buf);
    }

    pub unsafe fn SetShortArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jshort) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetShortArrayRegion");
            self.check_no_exception("SetShortArrayRegion");
            assert!(!array.is_null(), "SetShortArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetShortArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jshort>()), "SetShortArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *const jshort)>(210)(self.vtable, array, start, len, buf);
    }

    pub unsafe fn SetIntArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetIntArrayRegion");
            self.check_no_exception("SetIntArrayRegion");
            assert!(!array.is_null(), "SetIntArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetIntArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jint>()), "SetIntArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *const jint)>(211)(self.vtable, array, start, len, buf);
    }

    pub unsafe fn SetLongArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jlong) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetLongArrayRegion");
            self.check_no_exception("SetLongArrayRegion");
            assert!(!array.is_null(), "SetLongArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetLongArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jlong>()), "SetLongArrayRegion buf pointer is not aligned");
        }


        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *const jlong)>(212)(self.vtable, array, start, len, buf);
    }

    pub unsafe fn SetFloatArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jfloat) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetFloatArrayRegion");
            self.check_no_exception("SetFloatArrayRegion");
            assert!(!array.is_null(), "SetFloatArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetFloatArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jfloat>()), "SetFloatArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *const jfloat)>(213)(self.vtable, array, start, len, buf);
    }

    pub unsafe fn SetDoubleArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jdouble) {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("SetDoubleArrayRegion");
            self.check_no_exception("SetDoubleArrayRegion");
            assert!(!array.is_null(), "SetDoubleArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetDoubleArrayRegion buf must not be null");
            assert_eq!(0, buf.align_offset(align_of::<jdouble>()), "SetDoubleArrayRegion buf pointer is not aligned");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jbooleanArray, jsize, jsize, *const jdouble)>(214)(self.vtable, array, start, len, buf);
    }


    #[cfg(feature = "asserts")]
    thread_local! {
        //The "Critical Section" created by GetPrimitiveArrayCritical has a lot of restrictions placed upon it.
        //This attempts to track "some" of them on a best effort basis.
        static CRITICAL_POINTERS: std::cell::RefCell<std::collections::HashMap<*mut c_void, usize>> = std::cell::RefCell::new(std::collections::HashMap::new());
    }



    ///
    /// Obtains a critical pointer into a primitive java array.
    /// This pointer must be released by calling ReleasePrimitiveArrayCritical.
    /// No other JNI functions can be called in the current thread.
    /// The only exception being multiple consecutive calls to GetPrimitiveArrayCritical & GetStringCritical to obtain multiple critical
    /// pointers at the same time.
    ///
    /// This method will return NULL to indicate error.
    /// The JVM will most likely throw an Exception, probably an OOMError.
    /// If you obtain multiple critical pointers, you MUST release all successfully obtained critical pointers
    /// before being able to check for the exception.
    ///
    /// Special care must be taken to avoid blocking the current thread with a dependency on another JVM thread.
    /// I.e. Do not read from a pipe that is filled by another JVM thread for example.
    ///
    /// It is also ill-advised to hold onto critical pointers for long periods of time even if no dependency on another JVM Thread is made.
    /// The JVM may decide among other things to suspend garbage collection while a critical pointer is held.
    /// So reading from a Socket with a long timeout while holding a critical pointer is unlikely to be a good idea.
    /// As it may cause unintended side effects in the rest of the JVM (like running out of memory because the GC doesn't run)
    ///
    /// Failure to release critical pointers before returning execution back to Java Code should be treated as UB
    /// even tho the JVM spec fails to mention this detail.
    ///
    /// Releasing critical pointers in another thread other than the thread that created it should be treated as UB
    /// even tho the JVM spec only mentions this detail indirectly.
    ///
    /// I recommend against using this method for almost every use case as using either Set/Get array region or direct NIO buffers
    /// is a better choice. One use case I can think of where this method is a valid choice
    /// is performing pixel manipulations on the int[]/byte[] inside a large existing BufferedImage.
    ///
    pub unsafe fn GetPrimitiveArrayCritical(&self, array: jarray, isCopy: *mut jboolean) -> *mut c_void {
        #[cfg(feature = "asserts")]
        {
            Self::CRITICAL_POINTERS.with(|set| {
                if set.borrow().is_empty() {
                    Self::CRITICAL_STRINGS.with(|strings| {
                        if strings.borrow().is_empty() {
                            //We can only do this check if we have not yet obtained a unreleased critical on the current thread.
                            //For subsequent calls we cannot do this check.
                            self.check_no_exception("GetPrimitiveArrayCritical");
                        }
                    });
                }
            });
            assert!(!array.is_null(), "GetPrimitiveArrayCritical jarray must not be null");
        }

        let crit = self.jni::<extern "system" fn(JNIEnvVTable, jarray, *mut jboolean) -> *mut c_void>(222)(self.vtable, array, isCopy);

        #[cfg(feature = "asserts")]
        {
            if !crit.is_null() {
                Self::CRITICAL_POINTERS.with(|set| {
                    let mut rm = set.borrow_mut();
                    let n = rm.remove(&crit).unwrap_or(0)+1;
                    rm.insert(crit, n);
                });
            }
        }

        crit
    }

    pub unsafe fn ReleasePrimitiveArrayCritical(&self, array: jarray, carray: *mut c_void, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleasePrimitiveArrayCritical jarray must not be null");
            assert!(!carray.is_null(), "ReleasePrimitiveArrayCritical carray must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleasePrimitiveArrayCritical mode is invalid {}", mode);
            Self::CRITICAL_POINTERS.with(|set| {
                let mut rm = set.borrow_mut();
                let mut n = rm.remove(&carray).expect("ReleasePrimitiveArrayCritical carray is not valid");
                if n == 0 {
                    unreachable!();
                }

                if mode != JNI_COMMIT {
                    //JNI_COMMIT does not release the pointer. It's a noop for non-copied pointers.
                    n -= 1;
                }

                if n >= 1 {
                    rm.insert(carray, n);
                }
            });
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jarray, *mut c_void, jint)>(223)(self.vtable, array, carray, mode);
    }

    pub unsafe fn RegisterNatives_slice(&self, clazz: jclass, methods : &[JNINativeMethod]) -> jint {
        self.RegisterNatives(clazz, methods.as_ptr(), methods.len() as jint)
    }

    pub unsafe fn RegisterNatives(&self, clazz: jclass, methods : *const JNINativeMethod, size: jint) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("RegisterNatives");
            self.check_no_exception("RegisterNatives");
            assert!(!clazz.is_null(), "RegisterNatives class must not be null");
            assert!(size>0, "RegisterNatives size must be greater than 0");
            let sl = std::slice::from_raw_parts(methods, size as usize);
            for s in 0..sl.len() {
                let cur = &sl[s];
                assert!(!cur.name.is_null(), "RegisterNatives JNINativeMethod[{}],name is null", s);
                assert!(!cur.signature.is_null(), "RegisterNatives JNINativeMethod[{}].signature is null", s);
                assert!(!cur.fnPtr.is_null(), "RegisterNatives JNINativeMethod[{}].fnPtr is null", s);
            }
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jclass, *const JNINativeMethod, jint) -> jint>(215)(self.vtable, clazz, methods, size)
    }

    pub unsafe fn UnregisterNatives(&self, clazz: jclass) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("UnregisterNatives");
            self.check_no_exception("UnregisterNatives");
            assert!(!clazz.is_null(), "UnregisterNatives class must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jclass) -> jint>(216)(self.vtable, clazz)
    }

    pub unsafe fn MonitorEnter(&self, obj: jobject) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("MonitorEnter");
            self.check_no_exception("MonitorEnter");
            assert!(!obj.is_null(), "MonitorEnter object must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jint>(217)(self.vtable, obj)
    }

    pub unsafe fn MonitorExit(&self, obj: jobject) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("MonitorExit");
            assert!(!obj.is_null(), "MonitorExit object must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jint>(218)(self.vtable, obj)
    }

    pub unsafe fn NewDirectByteBuffer(&self, address: *mut c_void, capacity: jlong) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("NewDirectByteBuffer");
            self.check_no_exception("NewDirectByteBuffer");
            assert!(!address.is_null(), "NewDirectByteBuffer address must not be null");
            assert!(capacity >= 0, "NewDirectByteBuffer capacity must not be negative {}", capacity);
            assert!(capacity <= jint::MAX as jlong, "NewDirectByteBuffer capacity is too big, its larger than Integer.MAX_VALUE {}", capacity);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, *mut c_void, jlong) -> jobject>(229)(self.vtable, address, capacity)
    }

    pub unsafe fn GetDirectBufferAddress(&self, buf: jobject) -> *mut c_void {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetDirectBufferAddress");
            self.check_no_exception("GetDirectBufferAddress");
            assert!(!buf.is_null(), "GetDirectBufferAddress buffer must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> *mut c_void>(230)(self.vtable, buf)
    }

    pub unsafe fn GetDirectBufferCapacity(&self, buf: jobject) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetDirectBufferCapacity");
            self.check_no_exception("GetDirectBufferCapacity");
            assert!(!buf.is_null(), "GetDirectBufferCapacity buffer must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jlong>(231)(self.vtable, buf)
    }

    pub unsafe fn FromReflectedMethod(&self, method: jobject) -> jmethodID {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("FromReflectedMethod");
            self.check_no_exception("FromReflectedMethod");
            assert!(!method.is_null(), "FromReflectedMethod method must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jmethodID>(7)(self.vtable, method)
    }

    pub unsafe fn ToReflectedMethod(&self, cls: jclass, jmethodID: jmethodID, isStatic: jboolean) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ToReflectedMethod");
            self.check_no_exception("ToReflectedMethod");
            assert!(!cls.is_null(), "ToReflectedMethod class must not be null");
            assert!(!jmethodID.is_null(), "ToReflectedMethod method must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass, jmethodID, jboolean) -> jobject>(9)(self.vtable, cls, jmethodID, isStatic)
    }

    pub unsafe fn FromReflectedField(&self, field: jobject) -> jfieldID {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("FromReflectedField");
            self.check_no_exception("FromReflectedField");
            assert!(!field.is_null(), "FromReflectedField field must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jfieldID>(8)(self.vtable, field)
    }

    pub unsafe fn ToReflectedField(&self, cls: jclass, jfieldID: jfieldID, isStatic: jboolean) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("ToReflectedField");
            self.check_no_exception("ToReflectedField");
            assert!(!cls.is_null(), "ToReflectedField class must not be null");
            assert!(!jfieldID.is_null(), "ToReflectedField field must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jclass, jfieldID, jboolean) -> jobject>(12)(self.vtable, cls, jfieldID, isStatic)
    }

    pub unsafe fn GetJavaVM(&self) -> Result<JavaVM, jint> {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetJavaVM");
            self.check_no_exception("GetJavaVM");
        }
        let mut r : JNIInvPtr = SyncMutPtr::null();
        let res = self.jni::<extern "system" fn(JNIEnvVTable, *mut JNIInvPtr) -> jint>(219)(self.vtable, &mut r);
        if res != 0 {
            return Err(res);
        }
        if r.is_null() {
            panic!("GetJavaVM returned 0 but did not set JVM pointer");
        }
        Ok(JavaVM { functions: r})
    }

    pub unsafe fn GetModule(&self, cls: jclass) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("GetModule");
            self.check_no_exception("GetModule");
            assert!(self.GetVersion() >= JNI_VERSION_9);
        }

        self.jni::<extern "system" fn(JNIEnvVTable, jclass) -> jobject>(233)(self.vtable, cls)
    }

    pub unsafe fn IsVirtualThread(&self, thread: jobject) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_not_critical("IsVirtualThread");
            assert!(self.GetVersion() >= JNI_VERSION_21);
        }
        self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jboolean>(234)(self.vtable, thread)
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_not_critical(&self, context: &str) {
        Self::CRITICAL_POINTERS.with(|set| {
            let sz = set.borrow_mut().len();
            if sz != 0 {
                panic!("{} cannot be called now, because there are {} critical pointers into primitive arrays that have not been released by the current thread.", context, sz);
            }
        });
        Self::CRITICAL_STRINGS.with(|set| {
            let sz = set.borrow_mut().len();
            if sz != 0 {
                panic!("{} cannot be called now, because there are {} critical pointers into strings that have not been released by the current thread.", context, sz);
            }
        });
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_is_array(&self, obj: jobject, context: &str) {
        if obj.is_null() {
            panic!("{} cannot check if arg is array because arg is null", context);
        }
        let cl = self.GetObjectClass(obj);
        if cl.is_null() {
            panic!("{} arg.getClass() is null?", context);
        }
        let clazz = self.GetObjectClass(cl);
        if clazz.is_null() {
            panic!("{} Class#getClass() is null?", context);
        }

        let is_array = self.GetMethodID_str(clazz, "isArray", "()Z");
        let r = self.CallBooleanMethod0(cl, is_array);
        if self.ExceptionCheck() {
            self.ExceptionDescribe();
            panic!("{} Class#isArray() is throws?", context);
        }

        if !r {
            panic!("{} arg is not an array", context);
        }

        self.DeleteLocalRef(cl);
        self.DeleteLocalRef(clazz);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_no_exception(&self, context: &str) {
        if !self.ExceptionCheck() {
            return;
        }

        self.ExceptionDescribe();
        panic!("{} exception is thrown and not handled", context);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_ref_obj_permit_null(&self, context: &str, obj: jobject) {
        if obj.is_null() {
            return;
        }
        match self.GetObjectRefType(obj) {
            jobjectRefType::JNIInvalidRefType => panic!("{} ref is invalid", context),
            _=> {}
        }
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_ref_obj(&self, context: &str, obj: jobject) {
        assert!(!obj.is_null(), "{} ref is null", context);
        match self.GetObjectRefType(obj) {
            jobjectRefType::JNIInvalidRefType => panic!("{} ref is invalid", context),
            jobjectRefType::JNIWeakGlobalRefType => {
                //This bad practice, but sadly sometimes valid.
                //I.e. caller holds a strong reference and "knows" the weak ref cannot be GC'ed during the call.
                //Good practice would be to use the strong ref to make the call but sadly JVM doesn't enforce this.
                //This is just best effort really since we have absolutely NO clue when the GC will run.
                //TODO call System.gc() here?
                assert!(!self.IsSameObject(obj, null_mut()), "{} weak reference that has already been garbage collected", context);
            },
            _=> {}
        }
    }


    #[cfg(feature = "asserts")]
    unsafe fn check_is_exception_class(&self, context: &str, obj: jclass) {
        self.check_is_class(context, obj);
        let throwable_cl = self.FindClass_str("java/lang/Throwable");
        assert!(!throwable_cl.is_null(), "{} java/lang/Throwable not found???", context);
        assert!(self.IsAssignableFrom(obj, throwable_cl), "{} class is not throwable", context);
        self.DeleteLocalRef(throwable_cl);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_is_not_abstract(&self, context: &str, obj: jclass) {
        self.check_is_class(context, obj);
        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let meth = self.GetMethodID_str(class_cl, "getModifiers", "()I");
        assert!(!meth.is_null(), "{} java/lang/Class#getModifiers not found???", context);
        let mods = self.CallIntMethod0(obj, meth);
        self.DeleteLocalRef(class_cl);
        if self.ExceptionCheck() {
            self.ExceptionDescribe();
            panic!("{} java/lang/Class#getModifiers throws?", context);
        }

        let mod_cl = self.FindClass_str("java/lang/reflect/Modifier");
        assert!(!mod_cl.is_null(), "{} java/lang/reflect/Modifier not found???", context);
        let mod_fl = self.GetStaticFieldID_str(mod_cl, "ABSTRACT", "I");
        assert!(!mod_fl.is_null(), "{} java/lang/reflect/Modifier.ABSTRACT not found???", context);
        let amod = self.GetStaticIntField(mod_cl, mod_fl);
        self.DeleteLocalRef(mod_cl);

        if mods & amod != 0 {
            panic!("{} class is abstract", context);
        }
    }



    #[cfg(feature = "asserts")]
    unsafe fn check_is_class(&self, context: &str, obj: jclass) {
        assert!(!obj.is_null(), "{} class is null", context);
        self.check_ref_obj(context, obj);

        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        //GET OBJECT CLASS
        let tcl = self.jni::<extern "system" fn(JNIEnvVTable, jobject) -> jobject>(31)(self.vtable, obj);
        assert!(self.IsSameObject(tcl, class_cl), "{} not a class!", context);
        self.DeleteLocalRef(tcl);
        self.DeleteLocalRef(class_cl);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_if_arg_is_string(&self, src: &str, jobject: jobject) {
        if jobject.is_null() {
            return;
        }

        let clazz = self.GetObjectClass(jobject);
        assert!(!clazz.is_null(), "{} string.class is null?", src);
        let str_class = self.FindClass_str("java/lang/String");
        assert!(!str_class.is_null(), "{} java/lang/String not found?", src);
        assert!(self.IsSameObject(clazz, str_class), "{} Non string passed to GetStringCritical", src);
        self.DeleteLocalRef(clazz);
        self.DeleteLocalRef(str_class);
    }


    #[cfg(feature = "asserts")]
    unsafe fn check_field_type_static(&self, context: &str, obj: jclass, fieldID: jfieldID, ty: &str) {
        self.check_is_class(context, obj);
        assert!(!fieldID.is_null(), "{} fieldID is null", context);
        let f = self.ToReflectedField(obj, fieldID, true);
        assert!(!f.is_null(), "{} -> ToReflectedField returned null", context);
        let field_cl = self.FindClass_str("java/lang/reflect/Field");
        assert!(!f.is_null(), "{} java/lang/reflect/Method not found???", context);
        let field_rtyp = self.GetMethodID_str(field_cl, "getType", "()Ljava/lang/Class;");
        assert!(!field_rtyp.is_null(), "{} java/lang/reflect/Field#getType not found???", context);
        //CallObjectMethodA
        let rtc = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, f, field_rtyp, null());
        assert!(!rtc.is_null(), "{} java/lang/reflect/Field#getType returned null???", context);
        self.DeleteLocalRef(field_cl);
        self.DeleteLocalRef(f);
        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, rtc, class_name, null());
        assert!(!name_str.is_null(), "{} java/lang/Class#getName returned null??? Class has no name???", context);
        self.DeleteLocalRef(rtc);
        let the_name = self.GetStringUTFChars_as_string(name_str).expect(format!("{} failed to get/parse classname???", context).as_str());
        self.DeleteLocalRef(class_cl);
        self.DeleteLocalRef(name_str);
        if the_name.as_str().eq(ty) {
            return;
        }

        if ty.eq("object") {
            match the_name.as_str() {
                "long" | "int" | "short" | "byte" | "char" | "float" | "double" | "boolean" => {
                    panic!("{} type of field is {} but expected object", context, the_name);
                }
                _=> {
                    return;
                }
            }
        }

        panic!("{} type of field is {} but expected {}", context, the_name, ty);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_return_type_static(&self, context: &str, obj: jclass, methodID: jmethodID, ty: &str) {
        self.check_is_class(context, obj);
        assert!(!methodID.is_null(), "{} methodID is null", context);
        let m = self.ToReflectedMethod(obj, methodID, true);
        assert!(!m.is_null(), "{} -> ToReflectedMethod returned null", context);
        let meth_cl = self.FindClass_str("java/lang/reflect/Method");
        assert!(!m.is_null(), "{} java/lang/reflect/Method not found???", context);
        let meth_rtyp = self.GetMethodID_str(meth_cl, "getReturnType", "()Ljava/lang/Class;");
        assert!(!meth_rtyp.is_null(), "{} java/lang/reflect/Method#getReturnType not found???", context);
        //CallObjectMethodA
        let rtc = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, m, meth_rtyp, null());
        self.DeleteLocalRef(meth_cl);
        self.DeleteLocalRef(m);
        if rtc.is_null(){
            if ty.eq("void") {
                return;
            }

            panic!("{} return type of method is void but expected {}", context, ty);
        }
        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, rtc, class_name, null());
        assert!(!name_str.is_null(), "{} java/lang/Class#getName returned null??? Class has no name???", context);
        self.DeleteLocalRef(rtc);
        let the_name = self.GetStringUTFChars_as_string(name_str).expect(format!("{} failed to get/parse classname???", context).as_str());
        self.DeleteLocalRef(class_cl);
        self.DeleteLocalRef(name_str);
        if the_name.as_str().eq(ty) {
            return;
        }

        if ty.eq("object") {
            match the_name.as_str() {
                "void" | "long" | "int" | "short" | "byte" | "char" | "float" | "double" | "boolean" => {
                    panic!("{} return type of method is {} but expected object", context, the_name);
                }
                _=> {
                    return;
                }
            }
        }

        panic!("{} return type of method is {} but expected {}", context, the_name, ty);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_parameter_types_static<T: JType>(&self, context: &str, clazz: jclass, methodID: jmethodID, param1: T, idx: jsize, count: jsize) {
        self.check_is_class(context, clazz);
        assert!(!methodID.is_null(), "{} methodID is null", context);
        let java_method = self.ToReflectedMethod(clazz, methodID, true);
        assert!(!java_method.is_null(), "{} -> ToReflectedMethod returned null", context);
        let meth_cl = self.FindClass_str("java/lang/reflect/Method");
        assert!(!java_method.is_null(), "{} java/lang/reflect/Method not found???", context);
        let meth_params = self.GetMethodID_str(meth_cl, "getParameterTypes", "()[Ljava/lang/Class;");
        assert!(!meth_params.is_null(), "{} java/lang/reflect/Method#getParameterTypes not found???", context);

        //CallObjectMethodA
        let parameter_array = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, java_method, meth_params, null());
        self.DeleteLocalRef(meth_cl);
        self.DeleteLocalRef(java_method);
        assert!(!parameter_array.is_null(), "{} java/lang/reflect/Method#getParameterTypes return null???", context);
        let parameter_count = self.GetArrayLength(parameter_array);
        assert_eq!(parameter_count, count, "{} wrong number of method parameters", context);
        let param1_class = self.GetObjectArrayElement(parameter_array, idx);
        assert!(!param1_class.is_null(), "{} java/lang/reflect/Method#getParameterTypes[{}] is null???", context, idx);
        self.DeleteLocalRef(parameter_array);

        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        let class_is_primitive = self.GetMethodID_str(class_cl, "isPrimitive", "()Z");
        assert!(!class_is_primitive.is_null(), "{} java/lang/Class#isPrimitive not found???", context);

        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, param1_class, class_name, null());
        assert!(!name_str.is_null(), "{} java/lang/Class#getName returned null??? Class has no name???", context);
        //CallBooleanMethodA
        let param1_is_primitive = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jboolean>(39)(self.vtable, param1_class, class_is_primitive, null());

        let the_name = self.GetStringUTFChars_as_string(name_str).expect(format!("{} failed to get/parse classname???", context).as_str());
        self.DeleteLocalRef(class_cl);
        self.DeleteLocalRef(name_str);

        match T::jtype_id() {
            'Z' => assert_eq!("boolean", the_name, "{} param{} wrong type. Method has {} but passed boolean", context, idx, the_name),
            'B' => assert_eq!("byte", the_name, "{} param{} wrong type. Method has {} but passed byte", context, idx, the_name),
            'S' => assert_eq!("short", the_name, "{} param{} wrong type. Method has {} but passed short", context, idx, the_name),
            'C' => assert_eq!("char", the_name, "{} param{} wrong type. Method has {} but passed char", context, idx, the_name),
            'I' => assert_eq!("int", the_name, "{} param{} wrong type. Method has {} but passed int", context, idx, the_name),
            'J' => assert_eq!("long", the_name, "{} param{} wrong type. Method has {} but passed long", context, idx, the_name),
            'F' => assert_eq!("float", the_name, "{} param{} wrong type. Method has {} but passed float", context, idx, the_name),
            'D' => assert_eq!("double", the_name, "{} param{} wrong type. Method has {} but passed double", context, idx, the_name),
            'L' => {
                assert!(!param1_is_primitive, "{} param{} wrong type. Method has {} but passed an object or null", context, idx, the_name);
                let jt : jtype = param1.into();
                let obj = jt.object;
                if !obj.is_null() {
                    assert!(self.IsInstanceOf(obj, param1_class), "{} param{} wrong type. Method has {} but passed an object that is not null and not instanceof", context, idx, the_name);
                }
            }
            _=> unreachable!("{}", T::jtype_id())
        }

        self.DeleteLocalRef(param1_class);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_parameter_types_constructor<T: JType>(&self, context: &str, clazz: jclass, methodID: jmethodID, param1: T, idx: jsize, count: jsize) {
        self.check_ref_obj(context, clazz);
        assert!(!clazz.is_null(), "{} obj.class is null??", context);
        assert!(!methodID.is_null(), "{} methodID is null", context);
        let java_method = self.ToReflectedMethod(clazz, methodID, false);
        assert!(!java_method.is_null(), "{} -> ToReflectedMethod returned null", context);
        let meth_cl = self.FindClass_str("java/lang/reflect/Method");
        assert!(!java_method.is_null(), "{} java/lang/reflect/Method not found???", context);
        let meth_params = self.GetMethodID_str(meth_cl, "getParameterTypes", "()[Ljava/lang/Class;");
        assert!(!meth_params.is_null(), "{} java/lang/reflect/Method#getParameterTypes not found???", context);

        //CallObjectMethodA
        let parameter_array = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, java_method, meth_params, null());
        self.DeleteLocalRef(meth_cl);
        self.DeleteLocalRef(java_method);
        assert!(!parameter_array.is_null(), "{} java/lang/reflect/Method#getParameterTypes return null???", context);
        let parameter_count = self.GetArrayLength(parameter_array);
        assert_eq!(parameter_count, count, "{} wrong number of method parameters", context);
        let param1_class = self.GetObjectArrayElement(parameter_array, idx);
        assert!(!param1_class.is_null(), "{} java/lang/reflect/Method#getParameterTypes[{}] is null???", context, idx);
        self.DeleteLocalRef(parameter_array);

        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        let class_is_primitive = self.GetMethodID_str(class_cl, "isPrimitive", "()Z");
        assert!(!class_is_primitive.is_null(), "{} java/lang/Class#isPrimitive not found???", context);

        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, param1_class, class_name, null());
        assert!(!name_str.is_null(), "{} java/lang/Class#getName returned null??? Class has no name???", context);
        //CallBooleanMethodA
        let param1_is_primitive = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jboolean>(39)(self.vtable, param1_class, class_is_primitive, null());

        let the_name = self.GetStringUTFChars_as_string(name_str).expect(format!("{} failed to get/parse classname???", context).as_str());
        self.DeleteLocalRef(class_cl);
        self.DeleteLocalRef(name_str);

        match T::jtype_id() {
            'Z' => assert_eq!("boolean", the_name, "{} param{} wrong type. Method has {} but passed boolean", context, idx, the_name),
            'B' => assert_eq!("byte", the_name, "{} param{} wrong type. Method has {} but passed byte", context, idx, the_name),
            'S' => assert_eq!("short", the_name, "{} param{} wrong type. Method has {} but passed short", context, idx, the_name),
            'C' => assert_eq!("char", the_name, "{} param{} wrong type. Method has {} but passed char", context, idx, the_name),
            'I' => assert_eq!("int", the_name, "{} param{} wrong type. Method has {} but passed int", context, idx, the_name),
            'J' => assert_eq!("long", the_name, "{} param{} wrong type. Method has {} but passed long", context, idx, the_name),
            'F' => assert_eq!("float", the_name, "{} param{} wrong type. Method has {} but passed float", context, idx, the_name),
            'D' => assert_eq!("double", the_name, "{} param{} wrong type. Method has {} but passed double", context, idx, the_name),
            'L' => {
                assert!(!param1_is_primitive, "{} param{} wrong type. Method has {} but passed an object or null", context, idx, the_name);
                let jt : jtype = param1.into();
                let obj = jt.object;
                if !obj.is_null() {
                    assert!(self.IsInstanceOf(obj, param1_class), "{} param{} wrong type. Method has {} but passed an object that is not null and not instanceof", context, idx, the_name);
                }
            }
            _=> unreachable!("{}", T::jtype_id())
        }

        self.DeleteLocalRef(param1_class);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_parameter_types_object<T: JType>(&self, context: &str, obj: jobject, methodID: jmethodID, param1: T, idx: jsize, count: jsize) {
        assert!(!obj.is_null(), "{} obj is null", context);
        self.check_ref_obj(context, obj);
        let clazz = self.GetObjectClass(obj);
        assert!(!clazz.is_null(), "{} obj.class is null??", context);
        assert!(!methodID.is_null(), "{} methodID is null", context);
        let java_method = self.ToReflectedMethod(clazz, methodID, false);
        assert!(!java_method.is_null(), "{} -> ToReflectedMethod returned null", context);
        self.DeleteLocalRef(clazz);
        let meth_cl = self.FindClass_str("java/lang/reflect/Method");
        assert!(!java_method.is_null(), "{} java/lang/reflect/Method not found???", context);
        let meth_params = self.GetMethodID_str(meth_cl, "getParameterTypes", "()[Ljava/lang/Class;");
        assert!(!meth_params.is_null(), "{} java/lang/reflect/Method#getParameterTypes not found???", context);

        //CallObjectMethodA
        let parameter_array = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, java_method, meth_params, null());
        self.DeleteLocalRef(meth_cl);
        self.DeleteLocalRef(java_method);
        assert!(!parameter_array.is_null(), "{} java/lang/reflect/Method#getParameterTypes return null???", context);
        let parameter_count = self.GetArrayLength(parameter_array);
        assert_eq!(parameter_count, count, "{} wrong number of method parameters", context);
        let param1_class = self.GetObjectArrayElement(parameter_array, idx);
        assert!(!param1_class.is_null(), "{} java/lang/reflect/Method#getParameterTypes[{}] is null???", context, idx);
        self.DeleteLocalRef(parameter_array);

        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        let class_is_primitive = self.GetMethodID_str(class_cl, "isPrimitive", "()Z");
        assert!(!class_is_primitive.is_null(), "{} java/lang/Class#isPrimitive not found???", context);

        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, param1_class, class_name, null());
        assert!(!name_str.is_null(), "{} java/lang/Class#getName returned null??? Class has no name???", context);
        //CallBooleanMethodA
        let param1_is_primitive = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jboolean>(39)(self.vtable, param1_class, class_is_primitive, null());

        let the_name = self.GetStringUTFChars_as_string(name_str).expect(format!("{} failed to get/parse classname???", context).as_str());
        self.DeleteLocalRef(class_cl);
        self.DeleteLocalRef(name_str);

        match T::jtype_id() {
            'Z' => assert_eq!("boolean", the_name, "{} param{} wrong type. Method has {} but passed boolean", context, idx, the_name),
            'B' => assert_eq!("byte", the_name, "{} param{} wrong type. Method has {} but passed byte", context, idx, the_name),
            'S' => assert_eq!("short", the_name, "{} param{} wrong type. Method has {} but passed short", context, idx, the_name),
            'C' => assert_eq!("char", the_name, "{} param{} wrong type. Method has {} but passed char", context, idx, the_name),
            'I' => assert_eq!("int", the_name, "{} param{} wrong type. Method has {} but passed int", context, idx, the_name),
            'J' => assert_eq!("long", the_name, "{} param{} wrong type. Method has {} but passed long", context, idx, the_name),
            'F' => assert_eq!("float", the_name, "{} param{} wrong type. Method has {} but passed float", context, idx, the_name),
            'D' => assert_eq!("double", the_name, "{} param{} wrong type. Method has {} but passed double", context, idx, the_name),
            'L' => {
                assert!(!param1_is_primitive, "{} param{} wrong type. Method has {} but passed an object or null", context, idx, the_name);
                let jt : jtype = param1.into();
                let obj = jt.object;
                if !obj.is_null() {
                    assert!(self.IsInstanceOf(obj, param1_class), "{} param{} wrong type. Method has {} but passed an object that is not null and not instanceof", context, idx, the_name);
                }
            }
            _=> unreachable!("{}", T::jtype_id())
        }

        self.DeleteLocalRef(param1_class);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_return_type_object(&self, context: &str, obj: jobject, methodID: jmethodID, ty: &str) {
        assert!(!obj.is_null(), "{} obj is null", context);
        self.check_ref_obj(context, obj);
        let clazz = self.GetObjectClass(obj);
        assert!(!clazz.is_null(), "{} obj.class is null??", context);
        assert!(!methodID.is_null(), "{} methodID is null", context);
        let m = self.ToReflectedMethod(clazz, methodID, false);
        self.DeleteLocalRef(clazz);
        assert!(!m.is_null(), "{} -> ToReflectedMethod returned null", context);
        let meth_cl = self.FindClass_str("java/lang/reflect/Method");
        assert!(!m.is_null(), "{} java/lang/reflect/Method not found???", context);
        let meth_rtyp = self.GetMethodID_str(meth_cl, "getReturnType", "()Ljava/lang/Class;");
        assert!(!meth_rtyp.is_null(), "{} java/lang/reflect/Method#getReturnType not found???", context);
        //CallObjectMethodA
        let rtc = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, m, meth_rtyp, null());
        self.DeleteLocalRef(meth_cl);
        self.DeleteLocalRef(m);
        if rtc.is_null(){
            if ty.eq("void") {
                return;
            }

            panic!("{} return type of method is void but expected {}", context, ty);
        }
        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, rtc, class_name, null());
        assert!(!name_str.is_null(), "{} java/lang/Class#getName returned null??? Class has no name???", context);
        self.DeleteLocalRef(rtc);
        let the_name = self.GetStringUTFChars_as_string(name_str).expect(format!("{} failed to get/parse classname???", context).as_str());
        self.DeleteLocalRef(class_cl);
        self.DeleteLocalRef(name_str);
        if the_name.as_str().eq(ty) {
            return;
        }

        if ty.eq("object") {
            match the_name.as_str() {
                "void" | "long" | "int" | "short" | "byte" | "char" | "float" | "double" | "boolean" => {
                    panic!("{} return type of method is {} but expected object", context, the_name);
                }
                _=> {
                    return;
                }
            }
        }

        panic!("{} return type of method is {} but expected {}", context, the_name, ty);
    }

    #[cfg(feature = "asserts")]
    unsafe fn check_field_type_object(&self, context: &str, obj: jclass, fieldID: jfieldID, ty: &str) {
        assert!(!obj.is_null(), "{} obj is null", context);
        let clazz = self.GetObjectClass(obj);
        assert!(!clazz.is_null(), "{} obj.class is null??", context);
        assert!(!fieldID.is_null(), "{} fieldID is null", context);
        let f = self.ToReflectedField(clazz, fieldID, false);
        assert!(!f.is_null(), "{} -> ToReflectedField returned null", context);
        let field_cl = self.FindClass_str("java/lang/reflect/Field");
        assert!(!f.is_null(), "{} java/lang/reflect/Method not found???", context);
        let field_rtyp = self.GetMethodID_str(field_cl, "getType", "()Ljava/lang/Class;");
        assert!(!field_rtyp.is_null(), "{} java/lang/reflect/Field#getType not found???", context);
        //CallObjectMethodA
        let rtc = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, f, field_rtyp, null());
        assert!(!rtc.is_null(), "{} java/lang/reflect/Field#getType returned null???", context);
        self.DeleteLocalRef(field_cl);
        self.DeleteLocalRef(f);
        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvVTable, jobject, jmethodID, *const jtype) -> jobject>(36)(self.vtable, rtc, class_name, null());
        assert!(!name_str.is_null(), "{} java/lang/Class#getName returned null??? Class has no name???", context);
        self.DeleteLocalRef(rtc);
        let the_name = self.GetStringUTFChars_as_string(name_str).expect(format!("{} failed to get/parse classname???", context).as_str());
        self.DeleteLocalRef(class_cl);
        self.DeleteLocalRef(name_str);
        if the_name.as_str().eq(ty) {
            return;
        }

        if ty.eq("object") {
            match the_name.as_str() {
                "long" | "int" | "short" | "byte" | "char" | "float" | "double" | "boolean" => {
                    panic!("{} type of field is {} but expected object", context, the_name);
                }
                _=> {
                    return;
                }
            }
        }

        panic!("{} type of field is {} but expected {}", context, the_name, ty);
    }

}

type JNI_CreateJavaVM = extern "C" fn(*mut JNIInvPtr, *mut JNIEnv, *mut JavaVMInitArgs) -> jint;
type JNI_GetCreatedJavaVMs = extern "C" fn(*mut JNIInvPtr, jsize, *mut jsize) -> jint;

#[derive(Debug, Copy, Clone)]
struct JNIDynamicLink {
    JNI_CreateJavaVM: SyncConstPtr<c_void>,
    JNI_GetCreatedJavaVMs: SyncConstPtr<c_void>,
}
unsafe impl Sync for JNIDynamicLink {}
unsafe impl Send for JNIDynamicLink {}

impl JNIDynamicLink {
    pub fn new(JNI_CreateJavaVM: *const c_void, JNI_GetCreatedJavaVMs: *const c_void) -> Self {
        if JNI_GetCreatedJavaVMs.is_null() {
            panic!("JNI_GetCreatedJavaVMs is null");
        }

        if JNI_CreateJavaVM.is_null() {
            panic!("JNI_CreateJavaVM is null");
        }

        unsafe {
            Self {
                JNI_CreateJavaVM: JNI_CreateJavaVM.as_sync_const(),
                JNI_GetCreatedJavaVMs: JNI_GetCreatedJavaVMs.as_sync_const()
            }
        }
    }

    pub fn JNI_CreateJavaVM(&self) -> JNI_CreateJavaVM {
        unsafe {mem::transmute(self.JNI_CreateJavaVM.inner())}
    }
    pub fn JNI_GetCreatedJavaVMs(&self) -> JNI_GetCreatedJavaVMs {
        unsafe {mem::transmute(self.JNI_GetCreatedJavaVMs.inner())}
    }
}



static LINK: OnceCell<JNIDynamicLink> = OnceCell::new();

///
/// Call this function to initialize the dynamic linking to the jvm to use the provided function pointers to
/// create the jvm. If this function is called more than once then it is a noop, since it is not possible to create
/// more than one jvm per process.
///
pub fn init_dynamic_link(JNI_CreateJavaVM: *const c_void, JNI_GetCreatedJavaVMs: *const c_void) {
    _= LINK.set(JNIDynamicLink::new(JNI_CreateJavaVM, JNI_GetCreatedJavaVMs));
}

///
/// Returns true if the jvm was loaded by either calling load_jvm_from_library or init_dynamic_link.
///
pub fn is_jvm_loaded() -> bool {
    LINK.get().is_some()
}

///
/// Convenience method to load the jvm from a path to libjvm.so or jvm.dll.
/// On success this method does NOT close the handle to the shared object.
/// This is usually fine because unloading the jvm is not supported anyway.
/// If you do not desire this then use init_dynamic_link.
///
#[cfg(feature = "loadjvm")]
pub unsafe fn load_jvm_from_library(path: &str) -> Result<(), String> {
    use std::sync::atomic::{AtomicBool, Ordering};
    let latch = AtomicBool::new(false);

    LINK.get_or_try_init(|| {
        latch.store(true, Ordering::SeqCst);
        let lib = libloading::Library::new(path)
            .map_err(|e| format!("Failed to load jvm from {} reason: {}", path, e))?;

        let JNI_CreateJavaVM_ptr = lib.get::<JNI_CreateJavaVM>(b"JNI_CreateJavaVM\0")
            .map_err(|e| format!("Failed to load jvm from {} reason: JNI_CreateJavaVM -> {}", path, e))?
            .try_as_raw_ptr()
            .ok_or_else(|| format!("Failed to load jvm from {} reason: JNI_CreateJavaVM -> failed to get raw ptr", path))?;

        if JNI_CreateJavaVM_ptr.is_null() {
            return Err(format!("Failed to load jvm from {} reason: JNI_CreateJavaVM not found", path))
        }

        let JNI_GetCreatedJavaVMs_ptr = lib.get::<JNI_GetCreatedJavaVMs>(b"JNI_GetCreatedJavaVMs\0")
            .map_err(|e| format!("Failed to load jvm from {} reason: JNI_GetCreatedJavaVMs -> {}", path, e))?
            .try_as_raw_ptr()
            .ok_or_else(|| format!("Failed to load jvm from {} reason: JNI_CreateJavaVM -> failed to get raw ptr", path))?;

        if JNI_GetCreatedJavaVMs_ptr.is_null() {
            return Err(format!("Failed to load jvm from {} reason: JNI_GetCreatedJavaVMs not found", path))
        }

        //We are good to go!
        mem::forget(lib);
        Ok(JNIDynamicLink::new(JNI_CreateJavaVM_ptr, JNI_GetCreatedJavaVMs_ptr))
    })?;

    if !latch.load(Ordering::SeqCst) {
        return Err("JVM already loaded".to_string());
    }

    Ok(())
}


///
/// Convenience method to load the jvm from the JAVA_HOME environment variable
/// that is commonly set on Windows by End-User Java Setups,
/// or on linux by distribution package installers.
///
/// If JAVA_HOME is not set or doesn't point to a known layout of a JVM installation
/// then this function returns an error.
///
#[cfg(feature = "loadjvm")]
pub unsafe fn load_jvm_from_java_home() -> Result<(), String> {
    //All (most) jvm layouts that I am aware of on windows+linux.
    const COMMON_LIBJVM_PATHS: &[&[&str]] = &[
        &["lib", "server", "libjvm.so"], //LINUX JAVA 11+
        &["jre", "lib", "amd64", "server", "libjvm.so"], //LINUX JDK JAVA <= 8 amd64
        &["lib", "amd64", "server", "libjvm.so"], //LINUX JRE JAVA <= 8 amd64
        &["jre", "lib", "aarch32", "server", "libjvm.so"], //LINUX JDK JAVA <= 8 arm 32
        &["lib", "aarch32", "server", "libjvm.so"], //LINUX JRE JAVA <= 8 arm 32
        &["jre", "lib", "aarch64", "server", "libjvm.so"], //LINUX JDK JAVA <= 8 arm 64
        &["lib", "aarch64", "server", "libjvm.so"], //LINUX JRE JAVA <= 8 arm 64
        &["jre", "bin", "server", "jvm.dll"], //WINDOWS JDK <= 8
        &["bin", "server", "jvm.dll"], //WINDOWS JRE <= 8 AND WINDOWS JDK/JRE 11+
    ];

    let java_home = std::env::var("JAVA_HOME")
        .map_err(|_| "JAVA_HOME is not set or invalid".to_string())?;


    for parts in COMMON_LIBJVM_PATHS {
        let mut buf = PathBuf::from(java_home.as_str());
        for part in *parts {
            buf.push(part);
        }

        if buf.try_exists().unwrap_or(false) {
            let full_path = buf.to_str()
                .ok_or_else(|| format!("JAVA_HOME {} is invalid", java_home))?;

            return load_jvm_from_library(full_path);
        }
    }

    Err(format!("JAVA_HOME {} is invalid", java_home))
}

fn get_link() -> &'static JNIDynamicLink {
    LINK.get().expect("jni_simple::init_dynamic_link not called")
}

///
/// Returns the created JavaVMs.
/// This will only ever return 1 (or 0) JavaVM according to Oracle Documentation.
///
/// Will panic if the JVM shared library has not been loaded yet.
///
pub unsafe fn JNI_GetCreatedJavaVMs() -> Result<Vec<JavaVM>, jint> {
    let link = get_link();

    //NOTE: Oracle spec says this will only ever yield 1 JVM.
    //I will worry about this when it actually becomes a problem
    let mut buf : [JNIInvPtr; 64] = [SyncMutPtr::null(); 64];
    let mut count : jint = 0;
    let res = link.JNI_GetCreatedJavaVMs()(buf.as_mut_ptr(), 64, &mut count);
    if res != JNI_OK {
        return Err(res);
    }

    if count < 0 {
        panic!("JNI_GetCreatedJavaVMs did set count to < 0 : {}", count);
    }

    let mut result_vec : Vec<JavaVM> = Vec::with_capacity(count as usize);
    for i in 0 .. count as usize {
        let ptr = buf[i];
        if ptr.is_null() {
            panic!("JNI_GetCreatedJavaVMs VM #{} is null! count is {}", i, count);
        }

        result_vec.push(JavaVM { functions: ptr});
    }

    Ok(result_vec)
}

///
/// Directly calls JNI_CreateJavaVM with the provided arguments.
/// Will panic if the JVM shared library has not been loaded yet.
///
pub unsafe fn JNI_CreateJavaVM(arguments: *mut JavaVMInitArgs) -> Result<(JavaVM, JNIEnv), jint> {
    #[cfg(feature = "asserts")]
    {
        assert!(!arguments.is_null(), "JNI_CreateJavaVM arguments must not be null");
    }
    let link = get_link();

    let mut jvm : JNIInvPtr = SyncMutPtr::null();
    let mut env : JNIEnv = JNIEnv {
        vtable: null_mut(),
    };

    let res = link.JNI_CreateJavaVM()(&mut jvm, &mut env, arguments);
    if res != JNI_OK {
        return Err(res);
    }

    if jvm.is_null() {
        panic!("JNI_CreateJavaVM returned JNI_OK but the JavaVM pointer is null");
    }

    if env.vtable.is_null() {
        panic!("JNI_CreateJavaVM returned JNI_OK but the JNIEnv pointer is null");
    }

    Ok((JavaVM{ functions: jvm }, env))
}

///
/// Convenience function to call JNI_CreateJavaVM with a simple list of String arguments.
/// These arguments are almost identical to the command line arguments used to start the jvm with the java binary.
/// Some options differ slightly. Consult the JNI Invocation API documentation for more information.
///
/// Will panic if the JVM shared library has not been loaded yet.
///
pub unsafe fn JNI_CreateJavaVM_with_string_args(version: jint, arguments: &Vec<String>) -> Result<(JavaVM, JNIEnv), jint> {
    struct DropGuard(*mut c_char);
    impl Drop for DropGuard {
        fn drop(&mut self) {
            unsafe { _ = CString::from_raw(self.0); }
        }
    }

    let mut vm_args: Vec<JavaVMOption> = Vec::with_capacity(arguments.len());
    let mut dealloc_list = Vec::with_capacity(arguments.len());
    for arg in arguments {
        let jvm_arg = CString::new(arg.as_str()).unwrap().into_raw();
        dealloc_list.push(DropGuard(jvm_arg));

        vm_args.push(JavaVMOption{
            optionString: jvm_arg,
            extraInfo: null_mut(),
        });
    }

    let mut args = JavaVMInitArgs {
        version,
        nOptions: vm_args.len() as i32,
        options: vm_args.as_mut_ptr(),
        ignoreUnrecognized: 1,
    };

    let result = JNI_CreateJavaVM(&mut args);
    drop(dealloc_list);
    result
}



impl JavaVM {

    #[inline]
    unsafe fn jnx<X>(&self, index: usize) -> X {
        unsafe {mem::transmute_copy(&(**self.functions.inner())[index])}
    }

    ///
    /// Attaches the current thread to the JVM as a normal thread.
    /// If a thread name is provided then it will be used as the java name of the current thread.
    ///
    pub unsafe fn AttachCurrentThread_str(&self, version: jint, thread_name: Option<&str>, thread_group: jobject) -> Result<JNIEnv, jint> {
        if thread_name.is_some() {
            let cstr = CString::new(thread_name.unwrap()).unwrap().into_raw();
            let mut args = JavaVMAttachArgs::new(version, cstr, thread_group);
            let result = self.AttachCurrentThread(&mut args);
            _=CString::from_raw(cstr);
            return result;
        }

        let mut args = JavaVMAttachArgs::new(version, null_mut(), thread_group);
        self.AttachCurrentThread(&mut args)
    }

    pub unsafe fn AttachCurrentThread(&self, args: *mut JavaVMAttachArgs) -> Result<JNIEnv, jint> {
        #[cfg(feature = "asserts")]
        {
            assert!(!args.is_null(), "AttachCurrentThread args must not be null");
        }
        let mut envptr : JNIEnvVTable = null_mut();

        let result = self.jnx::<extern "system" fn(JNIInvPtr, *mut JNIEnvVTable, *mut JavaVMAttachArgs) -> jint>(4)
            (self.functions, &mut envptr, args);
        if result != JNI_OK {
            return Err(result);
        }

        if envptr.is_null() {
            panic!("AttachCurrentThread returned JNI_OK but did not set the JNIEnv pointer!");
        }

        Ok(JNIEnv{ vtable: envptr})
    }

    ///
    /// Attaches the current thread to the JVM as a daemon thread.
    /// If a thread name is provided then it will be used as the java name of the current thread.
    ///
    pub unsafe fn AttachCurrentThreadAsDaemon_str(&self, version: jint, thread_name: Option<&str>, thread_group: jobject) -> Result<JNIEnv, jint> {
        if thread_name.is_some() {
            let cstr = CString::new(thread_name.unwrap()).unwrap().into_raw();
            let mut args = JavaVMAttachArgs::new(version, cstr, thread_group);
            let result = self.AttachCurrentThreadAsDaemon(&mut args);
            _=CString::from_raw(cstr);
            return result;
        }

        let mut args = JavaVMAttachArgs::new(version, null_mut(), thread_group);
        self.AttachCurrentThreadAsDaemon(&mut args)
    }

    pub unsafe fn AttachCurrentThreadAsDaemon(&self, args: *mut JavaVMAttachArgs) -> Result<JNIEnv, jint> {
        #[cfg(feature = "asserts")]
        {
            assert!(!args.is_null(), "AttachCurrentThreadAsDaemon args must not be null");
        }
        let mut envptr : JNIEnvVTable = null_mut();

        let result = self.jnx::<extern "system" fn(JNIInvPtr, *mut JNIEnvVTable, *mut JavaVMAttachArgs) -> jint>(7)
            (self.functions, &mut envptr, args);

        if result != JNI_OK {
            return Err(result);
        }

        if envptr.is_null() {
            panic!("AttachCurrentThreadAsDaemon returned JNI_OK but did not set the JNIEnv pointer!");
        }

        Ok(JNIEnv{ vtable: envptr})
    }

    ///
    /// Gets the JNIEnv for the current thread.
    ///
    pub unsafe fn GetEnv(&self, jni_version: jint) -> Result<JNIEnv, jint> {
        let mut envptr : JNIEnvVTable = null_mut();

        let result = self.jnx::<extern "system" fn(JNIInvPtr, *mut JNIEnvVTable, jint) -> jint>(6)
            (self.functions, &mut envptr, jni_version);

        if result != JNI_OK {
            return Err(result);
        }

        if envptr.is_null() {
            panic!("GetEnv returned JNI_OK but did not set the JNIEnv pointer!");
        }

        Ok(JNIEnv{ vtable: envptr})
    }

    ///
    /// Detaches the current thread from the jvm.
    /// This should only be called on functions that were attached with AttachCurrentThread or AttachCurrentThreadAsDaemon.
    ///
    pub unsafe fn DetachCurrentThread(&self) -> jint {
        self.jnx::<extern "system" fn(JNIInvPtr) -> jint>(5)(self.functions)
    }


    ///
    /// This function will block until all java threads have completed and then destroy the JVM.
    /// It should not be called from a method that is called from the JVM.
    ///
    pub unsafe fn DestroyJavaVM(&self) {
        self.jnx::<extern "system" fn(JNIInvPtr) -> ()>(3)(self.functions);
    }


}

#[cfg(test)]
#[test]
fn test_sync() {
    static_assertions::assert_impl_all!(JavaVM: Sync);
    static_assertions::assert_impl_all!(JavaVM: Send);

    static_assertions::assert_not_impl_all!(JNIEnv: Sync);
    static_assertions::assert_not_impl_all!(JNIEnv: Send);
}