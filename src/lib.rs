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
#![allow(dead_code)]

use std::ffi::{c_char, c_void, CStr, CString};
use std::mem;
use std::ptr::{null_mut};
#[cfg(feature = "asserts")]
use std::ptr::null;

use once_cell::sync::OnceCell;

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
pub enum jobjectRefType {
    JNIInvalidRefType = 0,
    JNILocalRefType = 1,
    JNIGlobalRefType = 2,
    JNIWeakGlobalRefType = 3
}

pub type jweak = jobject;

pub type jthrowable = jobject;

pub type jmethodID = jobject;
pub type jfieldID = jobject;

#[repr(C)]
pub union jtype {
    pub long: jlong,
    pub int: jint,
    pub short: jshort,
    pub char: jchar,
    pub byte: jbyte,
    pub boolean: jboolean,
    pub float: jfloat,
    pub double: jdouble,
    pub object: jobject,
    pub class: jclass,
    pub throwable: jthrowable
}

impl jtype {

    ///
    /// Helper function to "create" a jtype with a null jobject.
    ///
    pub const fn null() -> jtype {
        jtype { object: null_mut() }
    }
}

impl Into<jtype> for jlong {
    fn into(self) -> jtype {
        jtype { long: self }
    }
}

impl Into<jtype> for jobject {
    fn into(self) -> jtype {
        jtype { object: self }
    }
}
impl Into<jtype> for jint {
    fn into(self) -> jtype {
        jtype { int: self }
    }
}

impl Into<jtype> for jshort {
    fn into(self) -> jtype {
        jtype { short: self }
    }
}

impl Into<jtype> for jchar {
    fn into(self) -> jtype {
        jtype { char: self }
    }
}

impl Into<jtype> for jfloat {
    fn into(self) -> jtype {
        jtype { float: self }
    }
}

impl Into<jtype> for jdouble {
    fn into(self) -> jtype {
        jtype { double: self }
    }
}
impl Into<jtype> for jboolean {
    fn into(self) -> jtype {
        jtype { boolean: self }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct JNINativeMethod {
    name: *const c_char,
    signature: *const c_char,
    fnPtr: *const c_void
}

type JNIInvPtr = *mut *mut [*mut c_void; 10];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JavaVM {
    functions: JNIInvPtr
}

unsafe impl Send for JavaVM {}
unsafe impl Sync for JavaVM {}

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

type JNIEnvPtr = *mut *mut [*mut c_void; 235];

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct JNIEnv {
    functions: JNIEnvPtr,
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
        unsafe {mem::transmute_copy(&(**self.functions)[index])}
    }

    pub unsafe fn GetVersion(&self) -> jint {
        self.jni::<extern "system" fn(JNIEnvPtr) -> jint>(4)(self.functions)
    }

    pub unsafe fn DefineClass(&self, name: *const c_char, classloader: jobject, data: &[u8]) -> jclass {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("DefineClass");
            assert!(!name.is_null(), "DefineClass name is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, *const c_char, jobject, *const u8, i32) -> jclass>(5)
            (self.functions, name, classloader, data.as_ptr(), data.len() as i32)
    }

    pub unsafe fn DefineClass_str(&self, name: &str, classloader: jobject, data: &[u8]) -> jclass {
        let str = CString::new(name).unwrap();
        self.DefineClass(str.as_ptr(), classloader, data)
    }

    pub unsafe fn FindClass(&self, name: *const c_char) -> jclass {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("FindClass");
            assert!(!name.is_null(), "FindClass name is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, *const c_char) -> jclass>(6)(self.functions, name)
    }

    pub unsafe fn FindClass_str(&self, name: &str) -> jclass {
        let str = CString::new(name).unwrap();
        self.FindClass(str.as_ptr())
    }

    pub unsafe fn GetSuperclass(&self, clazz: jclass) -> jclass {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetSuperclass");
            self.check_is_class("GetSuperclass", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass) -> jclass>(10)(self.functions, clazz)
    }

    pub unsafe fn IsAssignableFrom(&self, clazz1: jclass, clazz2: jclass) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("IsAssignableFrom");
            self.check_is_class("IsAssignableFrom", clazz1);
            self.check_is_class("IsAssignableFrom", clazz2);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass, jclass) -> jboolean>(11)(self.functions, clazz1, clazz2)
    }

    pub unsafe fn Throw(&self, obj: jthrowable) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("Throw");
            assert!(!obj.is_null(), "Throw throwable is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jthrowable) -> jint>(13)(self.functions, obj)
    }

    pub unsafe fn ThrowNew(&self, clazz: jclass, message: *const c_char) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("ThrowNew");
            self.check_is_class("ThrowNew", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass, *const c_char) -> jint>(14)(self.functions, clazz, message)
    }

    pub unsafe fn ThrowNew_str(&self, clazz: jclass, message: &str) -> jint {
        let str = CString::new(message).unwrap();
        self.ThrowNew(clazz, str.as_ptr())
    }

    pub unsafe fn ExceptionOccurred(&self) -> jthrowable {
        self.jni::<extern "system" fn(JNIEnvPtr) -> jthrowable>(15)(self.functions)
    }

    pub unsafe fn ExceptionDescribe(&self) {
        self.jni::<extern "system" fn(JNIEnvPtr)>(16)(self.functions);
    }

    pub unsafe fn ExceptionClear(&self) {
        self.jni::<extern "system" fn(JNIEnvPtr)>(17)(self.functions);
    }

    pub unsafe fn FatalError(&self, msg: *const c_char) {
        #[cfg(feature = "asserts")]
        {
            assert!(!msg.is_null(), "FatalError msg is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, *const c_char)>(18)(self.functions, msg);
        unreachable!("FatalError");
    }

    pub unsafe fn FatalError_str(&self, message: &str) {
        let str = CString::new(message).unwrap().into_raw();
        self.FatalError(str);
        unreachable!("FatalError");
    }


    pub unsafe fn ExceptionCheck(&self) -> jboolean {
        self.jni::<extern "system" fn(JNIEnvPtr) -> jboolean>(228)(self.functions)
    }

    pub unsafe fn NewGlobalRef(&self, obj: jobject) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewGlobalRef");
            assert!(!obj.is_null(), "NewGlobalRef obj is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jobject>(21)(self.functions, obj)
    }

    pub unsafe fn DeleteGlobalRef(&self, obj: jobject) {
        #[cfg(feature = "asserts")]
        {
            assert!(!obj.is_null(), "DeleteGlobalRef obj is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject)>(22)(self.functions, obj)
    }

    pub unsafe fn DeleteLocalRef(&self, obj: jobject) {
        #[cfg(feature = "asserts")]
        {
            assert!(!obj.is_null(), "DeleteLocalRef obj is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject)>(23)(self.functions, obj)
    }

    pub unsafe fn EnsureLocalCapacity(&self, capacity: jint) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("EnsureLocalCapacity");
            assert!(capacity >= 0, "EnsureLocalCapacity capacity is negative");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jint) -> jint>(26)(self.functions, capacity)
    }

    pub unsafe fn PushLocalFrame(&self, capacity: jint) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("PushLocalFrame");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jint) -> jint>(19)(self.functions, capacity)
    }

    pub unsafe fn PopLocalFrame(&self, result: jobject) -> jobject {
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jobject>(20)(self.functions, result)
    }

    pub unsafe fn NewLocalRef(&self, obj: jobject) -> jobject {
        #[cfg(feature = "asserts")]
        {
            assert!(!obj.is_null(), "NewLocalRef obj is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jobject>(25)(self.functions, obj)
    }

    pub unsafe fn NewWeakGlobalRef(&self, obj: jobject) -> jweak {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewWeakGlobalRef");
            assert!(!obj.is_null(), "NewWeakGlobalRef obj is null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jweak>(226)(self.functions, obj)
    }

    pub unsafe fn DeleteWeakGlobalRef(&self, obj: jweak) {
        #[cfg(feature = "asserts")]
        {
            assert!(!obj.is_null(), "DeleteWeakGlobalRef obj is null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jobject)>(227)(self.functions, obj);
    }

    pub unsafe fn AllocObject(&self, clazz: jclass) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("AllocObject");
            self.check_is_class("AllocObject", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass) -> jobject>(27)(self.functions, clazz)
    }

    pub unsafe fn NewObjectA(&self, clazz: jclass, constructor: jmethodID, args: *const jtype) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewObjectA");
            assert!(!constructor.is_null(), "NewObjectA constructor is null");
            self.check_is_class("NewObjectA", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass, jmethodID, *const jtype) -> jobject>(30)(self.functions, clazz, constructor, args)
    }

    pub unsafe fn GetObjectClass(&self, obj: jobject) -> jclass {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetObjectClass");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jobject>(31)(self.functions, obj)
    }

    pub unsafe fn GetObjectRefType(&self, obj: jobject) -> jobjectRefType {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetObjectRefType");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jobjectRefType>(232)(self.functions, obj)
    }

    pub unsafe fn IsInstanceOf(&self, obj: jobject, clazz: jclass) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("IsInstanceOf");
            self.check_is_class("IsInstanceOf", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jclass) -> jboolean>(32)(self.functions, obj, clazz)
    }

    pub unsafe fn IsSameObject(&self, obj1: jobject, obj2: jobject) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("IsSameObject");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jobject) -> jboolean>(24)(self.functions, obj1, obj2)
    }

    pub unsafe fn GetFieldID(&self, clazz: jclass, name: *const c_char, sig: *const c_char) -> jfieldID {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetFieldID");
            assert!(!name.is_null(), "GetFieldID name is null");
            assert!(!sig.is_null(), "GetFieldID sig is null");
            self.check_is_class("GetFieldID", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass, *const c_char, *const c_char) -> jfieldID>(94)(self.functions, clazz, name, sig)
    }

    pub unsafe fn GetFieldID_str(&self, class: jclass, name: &str, sig: &str) -> jfieldID {
        let nstr = CString::new(name).unwrap();
        let nsig = CString::new(sig).unwrap();
        self.GetFieldID(class, nstr.as_ptr(), nsig.as_ptr())
    }

    pub unsafe fn GetObjectField(&self, obj: jobject, fieldID: jfieldID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetObjectField");
            self.check_field_type_object("GetObjectField", obj, fieldID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jobject>(95)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetBooleanField(&self, obj: jobject, fieldID: jfieldID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetBooleanField");
            self.check_field_type_object("GetBooleanField", obj, fieldID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jboolean>(96)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetByteField(&self, obj: jobject, fieldID: jfieldID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetByteField");
            self.check_field_type_object("GetByteField", obj, fieldID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jbyte>(97)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetCharField(&self, obj: jobject, fieldID: jfieldID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetCharField");
            self.check_field_type_object("GetCharField", obj, fieldID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jchar>(98)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetShortField(&self, obj: jobject, fieldID: jfieldID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetShortField");
            self.check_field_type_object("GetShortField", obj, fieldID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jshort>(99)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetIntField(&self, obj: jobject, fieldID: jfieldID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetIntField");
            self.check_field_type_object("GetIntField", obj, fieldID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jint>(100)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetLongField(&self, obj: jobject, fieldID: jfieldID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetLongField");
            self.check_field_type_object("GetLongField", obj, fieldID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jlong>(101)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetFloatField(&self, obj: jobject, fieldID: jfieldID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetFloatField");
            self.check_field_type_object("GetFloatField", obj, fieldID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jfloat>(102)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetDoubleField(&self, obj: jobject, fieldID: jfieldID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetDoubleField");
            self.check_field_type_object("GetDoubleField", obj, fieldID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jdouble>(103)(self.functions, obj, fieldID)
    }

    pub unsafe fn SetObjectField(&self, obj: jobject, fieldID: jfieldID, value: jobject) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetObjectField");
            self.check_field_type_object("SetObjectField", obj, fieldID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jobject)>(104)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetBooleanField(&self, obj: jobject, fieldID: jfieldID, value: jboolean) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetBooleanField");
            self.check_field_type_object("SetBooleanField", obj, fieldID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jboolean)>(105)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetByteField(&self, obj: jobject, fieldID: jfieldID, value: jbyte) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetByteField");
            self.check_field_type_object("SetByteField", obj, fieldID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jbyte)>(106)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetCharField(&self, obj: jobject, fieldID: jfieldID, value: jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetCharField");
            self.check_field_type_object("SetCharField", obj, fieldID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jchar)>(107)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetShortField(&self, obj: jobject, fieldID: jfieldID, value: jshort) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetShortField");
            self.check_field_type_object("SetShortField", obj, fieldID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jshort)>(108)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetIntField(&self, obj: jobject, fieldID: jfieldID, value: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetIntField");
            self.check_field_type_object("SetIntField", obj, fieldID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jint)>(109)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetLongField(&self, obj: jobject, fieldID: jfieldID, value: jlong) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetLongField");
            self.check_field_type_object("SetLongField", obj, fieldID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jlong)>(110)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetFloatField(&self, obj: jobject, fieldID: jfieldID, value: jfloat) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetFloatField");
            self.check_field_type_object("SetFloatField", obj, fieldID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jfloat)>(111)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetDoubleField(&self, obj: jobject, fieldID: jfieldID, value: jdouble) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetDoubleField");
            self.check_field_type_object("SetDoubleField", obj, fieldID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jdouble)>(112)(self.functions, obj, fieldID, value)
    }



    pub unsafe fn GetMethodID(&self, class: jclass, name: *const c_char, sig: *const c_char) -> jmethodID {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetMethodID");
            assert!(!name.is_null(), "GetMethodID name is null");
            assert!(!sig.is_null(), "GetMethodID sig is null");
            self.check_is_class("GetMethodID", class);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, *const c_char, *const c_char) -> jmethodID>(33)(self.functions, class, name, sig)
    }

    pub unsafe fn GetMethodID_str(&self, class: jclass, name: &str, sig: &str) -> jmethodID {
        let nstr = CString::new(name).unwrap();
        let nsig = CString::new(sig).unwrap();
        self.GetMethodID(class, nstr.as_ptr(), nsig.as_ptr())
    }

    pub unsafe fn CallVoidMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallVoidMethodA");
            self.check_return_type_object("CallVoidMethodA", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype)>(63)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallVoidMethod0(&self, obj: jobject, methodID: jmethodID) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallVoidMethod");
            self.check_return_type_object("CallVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID)>(61)(self.functions, obj, methodID)
    }

    pub unsafe fn CallVoidMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallVoidMethod");
            self.check_return_type_object("CallVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A)>(61)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallVoidMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallVoidMethod");
            self.check_return_type_object("CallVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B)>(61)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallVoidMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallVoidMethod");
            self.check_return_type_object("CallVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C)>(61)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallObjectMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallObjectMethodA");
            self.check_return_type_object("CallObjectMethodA", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallObjectMethod0(&self, obj: jobject, methodID: jmethodID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallObjectMethod");
            self.check_return_type_object("CallObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jobject>(34)(self.functions, obj, methodID)
    }

    pub unsafe fn CallObjectMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallObjectMethod");
            self.check_return_type_object("CallObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jobject>(34)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallObjectMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallObjectMethod");
            self.check_return_type_object("CallObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jobject>(34)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallObjectMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallObjectMethod");
            self.check_return_type_object("CallObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jobject>(34)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallBooleanMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallBooleanMethodA");
            self.check_return_type_object("CallBooleanMethodA", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jboolean>(39)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallBooleanMethod0(&self, obj: jobject, methodID: jmethodID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallBooleanMethod");
            self.check_return_type_object("CallBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jboolean>(37)(self.functions, obj, methodID)
    }

    pub unsafe fn CallBooleanMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallBooleanMethod");
            self.check_return_type_object("CallBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jboolean>(37)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallBooleanMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallBooleanMethod");
            self.check_return_type_object("CallBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jboolean>(37)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallBooleanMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallBooleanMethod");
            self.check_return_type_object("CallBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jboolean>(37)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallByteMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallByteMethodA");
            self.check_return_type_object("CallByteMethodA", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jbyte>(42)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallByteMethod0(&self, obj: jobject, methodID: jmethodID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallByteMethod0");
            self.check_return_type_object("CallByteMethod0", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jbyte>(40)(self.functions, obj, methodID)
    }

    pub unsafe fn CallByteMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallByteMethod1");
            self.check_return_type_object("CallByteMethod1", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jbyte>(40)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallByteMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallByteMethod2");
            self.check_return_type_object("CallByteMethod2", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jbyte>(40)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallByteMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallByteMethod3");
            self.check_return_type_object("CallByteMethod3", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jbyte>(40)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallCharMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallCharMethodA");
            self.check_return_type_object("CallCharMethodA", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jchar>(45)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallCharMethod0(&self, obj: jobject, methodID: jmethodID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallCharMethod");
            self.check_return_type_object("CallCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jchar>(43)(self.functions, obj, methodID)
    }

    pub unsafe fn CallCharMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallCharMethod");
            self.check_return_type_object("CallCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jchar>(43)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallCharMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallCharMethod");
            self.check_return_type_object("CallCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jchar>(43)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallCharMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallCharMethod");
            self.check_return_type_object("CallCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jchar>(43)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallShortMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallShortMethodA");
            self.check_return_type_object("CallShortMethodA", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jshort>(48)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallShortMethod0(&self, obj: jobject, methodID: jmethodID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallShortMethod");
            self.check_return_type_object("CallShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jshort>(46)(self.functions, obj, methodID)
    }

    pub unsafe fn CallShortMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallShortMethod");
            self.check_return_type_object("CallShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jshort>(46)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallShortMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallShortMethod");
            self.check_return_type_object("CallShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jshort>(46)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallShortMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallShortMethod");
            self.check_return_type_object("CallShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jshort>(46)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallIntMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallIntMethodA");
            self.check_return_type_object("CallIntMethodA", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jint>(51)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallIntMethod0(&self, obj: jobject, methodID: jmethodID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallIntMethod");
            self.check_return_type_object("CallIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jint>(49)(self.functions, obj, methodID)
    }

    pub unsafe fn CallIntMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallIntMethod");
            self.check_return_type_object("CallIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jint>(49)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallIntMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallIntMethod");
            self.check_return_type_object("CallIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jint>(49)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallIntMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallIntMethod");
            self.check_return_type_object("CallIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jint>(49)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallLongMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallLongMethodA");
            self.check_return_type_object("CallLongMethodA", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jlong>(54)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallLongMethod0(&self, obj: jobject, methodID: jmethodID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallLongMethod");
            self.check_return_type_object("CallLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jlong>(52)(self.functions, obj, methodID)
    }

    pub unsafe fn CallLongMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallLongMethod");
            self.check_return_type_object("CallLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jlong>(52)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallLongMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallLongMethod");
            self.check_return_type_object("CallLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jlong>(52)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallLongMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallLongMethod");
            self.check_return_type_object("CallLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jlong>(52)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallFloatMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallFloatMethodA");
            self.check_return_type_object("CallFloatMethodA", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jfloat>(57)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallFloatMethod0(&self, obj: jobject, methodID: jmethodID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallFloatMethod");
            self.check_return_type_object("CallFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jfloat>(55)(self.functions, obj, methodID)
    }

    pub unsafe fn CallFloatMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallFloatMethod");
            self.check_return_type_object("CallFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jfloat>(55)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallFloatMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallFloatMethod");
            self.check_return_type_object("CallFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jfloat>(55)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallFloatMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallFloatMethod");
            self.check_return_type_object("CallFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jfloat>(55)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallDoubleMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallDoubleMethodA");
            self.check_return_type_object("CallDoubleMethodA", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jdouble>(60)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallDoubleMethod0(&self, obj: jobject, methodID: jmethodID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallDoubleMethod");
            self.check_return_type_object("CallDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jdouble>(58)(self.functions, obj, methodID)
    }

    pub unsafe fn CallDoubleMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallDoubleMethod");
            self.check_return_type_object("CallDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jdouble>(58)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallDoubleMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallDoubleMethod");
            self.check_return_type_object("CallDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jdouble>(58)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallDoubleMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallDoubleMethod");
            self.check_return_type_object("CallDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jdouble>(58)(self.functions, obj, methodID, arg1, arg2, arg3)
    }


    pub unsafe fn CallNonvirtualVoidMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualVoidMethodA");
            self.check_return_type_object("CallNonvirtualVoidMethodA", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype)>(93)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualVoidMethod0(&self, obj: jobject, methodID: jmethodID) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualVoidMethod");
            self.check_return_type_object("CallNonvirtualVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID)>(91)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualVoidMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualVoidMethod");
            self.check_return_type_object("CallNonvirtualVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A)>(91)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualVoidMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualVoidMethod");
            self.check_return_type_object("CallNonvirtualVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B)>(91)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualVoidMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualVoidMethod");
            self.check_return_type_object("CallNonvirtualVoidMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C)>(91)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualObjectMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualObjectMethodA");
            self.check_return_type_object("CallNonvirtualObjectMethodA", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(66)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualObjectMethod0(&self, obj: jobject, methodID: jmethodID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualObjectMethod");
            self.check_return_type_object("CallNonvirtualObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jobject>(64)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualObjectMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualObjectMethod");
            self.check_return_type_object("CallNonvirtualObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jobject>(64)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualObjectMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualObjectMethod");
            self.check_return_type_object("CallNonvirtualObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jobject>(64)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualObjectMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualObjectMethod");
            self.check_return_type_object("CallNonvirtualObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jobject>(64)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualBooleanMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualBooleanMethodA");
            self.check_return_type_object("CallNonvirtualBooleanMethodA", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jboolean>(69)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualBooleanMethod0(&self, obj: jobject, methodID: jmethodID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualBooleanMethod");
            self.check_return_type_object("CallNonvirtualBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jboolean>(67)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualBooleanMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualBooleanMethod");
            self.check_return_type_object("CallNonvirtualBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jboolean>(67)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualBooleanMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualBooleanMethod");
            self.check_return_type_object("CallNonvirtualBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jboolean>(67)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualBooleanMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualBooleanMethod");
            self.check_return_type_object("CallNonvirtualBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jboolean>(67)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualByteMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualByteMethodA");
            self.check_return_type_object("CallNonvirtualByteMethodA", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jbyte>(72)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualByteMethod0(&self, obj: jobject, methodID: jmethodID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualByteMethod");
            self.check_return_type_object("CallNonvirtualByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jbyte>(70)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualByteMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualByteMethod");
            self.check_return_type_object("CallNonvirtualByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jbyte>(70)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualByteMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualByteMethod");
            self.check_return_type_object("CallNonvirtualByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jbyte>(70)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualByteMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualByteMethod");
            self.check_return_type_object("CallNonvirtualByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jbyte>(70)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualCharMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualCharMethodA");
            self.check_return_type_object("CallNonvirtualCharMethodA", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jchar>(75)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualCharMethod0(&self, obj: jobject, methodID: jmethodID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualCharMethod");
            self.check_return_type_object("CallNonvirtualCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jchar>(73)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualCharMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualCharMethod");
            self.check_return_type_object("CallNonvirtualCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jchar>(73)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualCharMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualCharMethod");
            self.check_return_type_object("CallNonvirtualCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jchar>(73)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualCharMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualCharMethod");
            self.check_return_type_object("CallNonvirtualCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jchar>(73)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualShortMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualShortMethodA");
            self.check_return_type_object("CallNonvirtualShortMethodA", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jshort>(78)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualShortMethod0(&self, obj: jobject, methodID: jmethodID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualShortMethod");
            self.check_return_type_object("CallNonvirtualShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jshort>(76)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualShortMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualShortMethod");
            self.check_return_type_object("CallNonvirtualShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jshort>(76)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualShortMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualShortMethod");
            self.check_return_type_object("CallNonvirtualShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jshort>(76)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualShortMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualShortMethod");
            self.check_return_type_object("CallNonvirtualShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jshort>(76)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualIntMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualIntMethodA");
            self.check_return_type_object("CallNonvirtualIntMethodA", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jint>(81)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualIntMethod0(&self, obj: jobject, methodID: jmethodID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualIntMethod");
            self.check_return_type_object("CallNonvirtualIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jint>(79)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualIntMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualIntMethod");
            self.check_return_type_object("CallNonvirtualIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jint>(79)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualIntMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualIntMethod");
            self.check_return_type_object("CallNonvirtualIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jint>(79)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualIntMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualIntMethod");
            self.check_return_type_object("CallNonvirtualIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jint>(79)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualLongMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualLongMethodA");
            self.check_return_type_object("CallNonvirtualLongMethodA", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jlong>(84)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualLongMethod0(&self, obj: jobject, methodID: jmethodID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualLongMethod");
            self.check_return_type_object("CallNonvirtualLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jlong>(82)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualLongMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualLongMethod");
            self.check_return_type_object("CallNonvirtualLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jlong>(82)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualLongMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualLongMethod");
            self.check_return_type_object("CallNonvirtualLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jlong>(82)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualLongMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualLongMethod");
            self.check_return_type_object("CallNonvirtualLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jlong>(82)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualFloatMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualFloatMethodA");
            self.check_return_type_object("CallNonvirtualFloatMethodA", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jfloat>(87)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualFloatMethod0(&self, obj: jobject, methodID: jmethodID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualFloatMethod");
            self.check_return_type_object("CallNonvirtualFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jfloat>(85)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualFloatMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualFloatMethod");
            self.check_return_type_object("CallNonvirtualFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jfloat>(85)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualFloatMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualFloatMethod");
            self.check_return_type_object("CallNonvirtualFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jfloat>(85)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualFloatMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualFloatMethod");
            self.check_return_type_object("CallNonvirtualFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jfloat>(85)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallNonvirtualDoubleMethodA(&self, obj: jobject, methodID: jmethodID, args: *const jtype) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualDoubleMethodA");
            self.check_return_type_object("CallNonvirtualDoubleMethodA", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jdouble>(90)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallNonvirtualDoubleMethod0(&self, obj: jobject, methodID: jmethodID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualDoubleMethod");
            self.check_return_type_object("CallNonvirtualDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jdouble>(88)(self.functions, obj, methodID)
    }

    pub unsafe fn CallNonvirtualDoubleMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualDoubleMethod");
            self.check_return_type_object("CallNonvirtualDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jdouble>(88)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallNonvirtualDoubleMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualDoubleMethod");
            self.check_return_type_object("CallNonvirtualDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jdouble>(88)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallNonvirtualDoubleMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallNonvirtualDoubleMethod");
            self.check_return_type_object("CallNonvirtualDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jdouble>(88)(self.functions, obj, methodID, arg1, arg2, arg3)
    }


    pub unsafe fn GetStaticFieldID(&self, clazz: jclass, name: *const c_char, sig: *const c_char) -> jfieldID {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticFieldID");
            assert!(!name.is_null(), "GetStaticFieldID name is null");
            assert!(!sig.is_null(), "GetStaticFieldID sig is null");
            self.check_is_class("GetStaticFieldID", clazz);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass, *const c_char, *const c_char) -> jfieldID>(144)(self.functions, clazz, name, sig)
    }

    pub unsafe fn GetStaticFieldID_str(&self, class: jclass, name: &str, sig: &str) -> jfieldID {
        let nstr = CString::new(name).unwrap();
        let nsig = CString::new(sig).unwrap();
        self.GetStaticFieldID(class, nstr.as_ptr(), nsig.as_ptr())

    }

    pub unsafe fn GetStaticObjectField(&self, obj: jclass, fieldID: jfieldID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticObjectField");
            self.check_field_type_static("GetStaticObjectField", obj, fieldID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jobject>(145)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetStaticBooleanField(&self, obj: jclass, fieldID: jfieldID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticBooleanField");
            self.check_field_type_static("GetStaticBooleanField", obj, fieldID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jboolean>(146)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetStaticByteField(&self, obj: jclass, fieldID: jfieldID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticByteField");
            self.check_field_type_static("GetStaticByteField", obj, fieldID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jbyte>(147)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetStaticCharField(&self, obj: jclass, fieldID: jfieldID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticCharField");
            self.check_field_type_static("GetStaticCharField", obj, fieldID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jchar>(148)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetStaticShortField(&self, obj: jclass, fieldID: jfieldID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticShortField");
            self.check_field_type_static("GetStaticShortField", obj, fieldID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jshort>(149)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetStaticIntField(&self, obj: jclass, fieldID: jfieldID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticIntField");
            self.check_field_type_static("GetStaticIntField", obj, fieldID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jint>(150)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetStaticLongField(&self, obj: jclass, fieldID: jfieldID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticLongField");
            self.check_field_type_static("GetStaticLongField", obj, fieldID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jlong>(151)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetStaticFloatField(&self, obj: jclass, fieldID: jfieldID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticFloatField");
            self.check_field_type_static("GetStaticFloatField", obj, fieldID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jfloat>(152)(self.functions, obj, fieldID)
    }

    pub unsafe fn GetStaticDoubleField(&self, obj: jclass, fieldID: jfieldID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticDoubleField");
            self.check_field_type_static("GetStaticDoubleField", obj, fieldID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID) -> jdouble>(153)(self.functions, obj, fieldID)
    }

    pub unsafe fn SetStaticObjectField(&self, obj: jclass, fieldID: jfieldID, value: jobject) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticObjectField");
            self.check_field_type_static("SetStaticObjectField", obj, fieldID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jobject)>(154)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetStaticBooleanField(&self, obj: jclass, fieldID: jfieldID, value: jboolean) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticBooleanField");
            self.check_field_type_static("SetStaticBooleanField", obj, fieldID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jboolean)>(155)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetStaticByteField(&self, obj: jclass, fieldID: jfieldID, value: jbyte) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticByteField");
            self.check_field_type_static("SetStaticByteField", obj, fieldID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jbyte)>(156)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetStaticCharField(&self, obj: jclass, fieldID: jfieldID, value: jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticCharField");
            self.check_field_type_static("SetStaticCharField", obj, fieldID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jchar)>(157)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetStaticShortField(&self, obj: jclass, fieldID: jfieldID, value: jshort) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticShortField");
            self.check_field_type_static("SetStaticShortField", obj, fieldID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jshort)>(158)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetStaticIntField(&self, obj: jclass, fieldID: jfieldID, value: jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticIntField");
            self.check_field_type_static("SetStaticIntField", obj, fieldID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jint)>(159)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetStaticLongField(&self, obj: jclass, fieldID: jfieldID, value: jlong) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticLongField");
            self.check_field_type_static("SetStaticLongField", obj, fieldID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jlong)>(160)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetStaticFloatField(&self, obj: jclass, fieldID: jfieldID, value: jfloat) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticFloatField");
            self.check_field_type_static("SetStaticFloatField", obj, fieldID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jfloat)>(161)(self.functions, obj, fieldID, value)
    }

    pub unsafe fn SetStaticDoubleField(&self, obj: jclass, fieldID: jfieldID, value: jdouble) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetStaticDoubleField");
            self.check_field_type_static("SetStaticDoubleField", obj, fieldID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jfieldID, jdouble)>(162)(self.functions, obj, fieldID, value)
    }




    pub unsafe fn GetStaticMethodID(&self, class: jclass, name: *const c_char, sig: *const c_char) -> jmethodID {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticMethodID");
            self.check_is_class("GetStaticMethodID", class);
            assert!(!name.is_null(), "GetStaticMethodID name is null");
            assert!(!sig.is_null(), "GetStaticMethodID sig is null");
        }


        self.jni::<extern "system" fn(JNIEnvPtr, jobject, *const c_char, *const c_char) -> jmethodID>(113)(self.functions, class, name, sig)
    }

    pub unsafe fn GetStaticMethodID_str(&self, class: jclass, name: &str, sig: &str) -> jmethodID {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStaticMethodID_str");
            self.check_is_class("GetStaticMethodID_str", class);
        }
        let nstr = CString::new(name).unwrap();
        let nsig = CString::new(sig).unwrap();
        self.GetStaticMethodID(class, nstr.as_ptr(), nsig.as_ptr())
    }


    pub unsafe fn CallStaticVoidMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticVoidMethodA");
            self.check_return_type_static("CallStaticVoidMethodA", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype)>(143)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticVoidMethod0(&self, obj: jobject, methodID: jmethodID) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID)>(141)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticVoidMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A)>(141)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticVoidMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B)>(141)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticVoidMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "void");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C)>(141)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticObjectMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethodA");
            self.check_return_type_static("CallStaticBooleanMethodA", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(116)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticObjectMethod0(&self, obj: jobject, methodID: jmethodID) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jobject>(114)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticObjectMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jobject>(114)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticObjectMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jobject>(114)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticObjectMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticObjectMethod");
            self.check_return_type_object("CallStaticObjectMethod", obj, methodID, "object");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jobject>(114)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticBooleanMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticBooleanMethodA");
            self.check_return_type_static("CallStaticBooleanMethodA", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jboolean>(119)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticBooleanMethod0(&self, obj: jobject, methodID: jmethodID) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticBooleanMethod");
            self.check_return_type_object("CallStaticBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jboolean>(117)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticBooleanMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticBooleanMethod");
            self.check_return_type_object("CallStaticBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jboolean>(117)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticBooleanMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticBooleanMethod");
            self.check_return_type_object("CallStaticBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jboolean>(117)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticBooleanMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticBooleanMethod");
            self.check_return_type_object("CallStaticBooleanMethod", obj, methodID, "boolean");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jboolean>(117)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticByteMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticByteMethodA");
            self.check_return_type_static("CallStaticByteMethodA", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jbyte>(122)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticByteMethod0(&self, obj: jobject, methodID: jmethodID) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticByteMethod");
            self.check_return_type_object("CallStaticByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jbyte>(120)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticByteMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticByteMethod");
            self.check_return_type_object("CallStaticByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jbyte>(120)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticByteMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticByteMethod");
            self.check_return_type_object("CallStaticByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jbyte>(120)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticByteMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticByteMethod");
            self.check_return_type_object("CallStaticByteMethod", obj, methodID, "byte");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jbyte>(120)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticCharMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticCharMethodA");
            self.check_return_type_static("CallStaticCharMethodA", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jchar>(125)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticCharMethod0(&self, obj: jobject, methodID: jmethodID) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticCharMethod");
            self.check_return_type_object("CallStaticCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jchar>(123)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticCharMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticCharMethod");
            self.check_return_type_object("CallStaticCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jchar>(123)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticCharMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticCharMethod");
            self.check_return_type_object("CallStaticCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jchar>(123)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticCharMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticCharMethod");
            self.check_return_type_object("CallStaticCharMethod", obj, methodID, "char");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jchar>(123)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticShortMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticShortMethodA");
            self.check_return_type_static("CallStaticShortMethodA", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jshort>(128)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticShortMethod0(&self, obj: jobject, methodID: jmethodID) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticShortMethod");
            self.check_return_type_object("CallStaticShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jshort>(126)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticShortMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticShortMethod");
            self.check_return_type_object("CallStaticShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jshort>(126)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticShortMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticShortMethod");
            self.check_return_type_object("CallStaticShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jshort>(126)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticShortMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticShortMethod");
            self.check_return_type_object("CallStaticShortMethod", obj, methodID, "short");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jshort>(126)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticIntMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticIntMethodA");
            self.check_return_type_static("CallStaticIntMethodA", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jint>(131)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticIntMethod0(&self, obj: jobject, methodID: jmethodID) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticIntMethod");
            self.check_return_type_object("CallStaticIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jint>(129)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticIntMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticIntMethod");
            self.check_return_type_object("CallStaticIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jint>(129)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticIntMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticIntMethod");
            self.check_return_type_object("CallStaticIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jint>(129)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticIntMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticIntMethod");
            self.check_return_type_object("CallStaticIntMethod", obj, methodID, "int");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jint>(129)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticLongMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticLongMethodA");
            self.check_return_type_static("CallStaticLongMethodA", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jlong>(134)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticLongMethod0(&self, obj: jobject, methodID: jmethodID) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticLongMethod");
            self.check_return_type_object("CallStaticLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jlong>(132)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticLongMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticLongMethod");
            self.check_return_type_object("CallStaticLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jlong>(132)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticLongMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticLongMethod");
            self.check_return_type_object("CallStaticLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jlong>(132)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticLongMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticLongMethod");
            self.check_return_type_object("CallStaticLongMethod", obj, methodID, "long");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jlong>(132)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticFloatMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticFloatMethodA");
            self.check_return_type_static("CallStaticFloatMethodA", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jfloat>(137)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticFloatMethod0(&self, obj: jobject, methodID: jmethodID) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticFloatMethod");
            self.check_return_type_object("CallStaticFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jfloat>(135)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticFloatMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticFloatMethod");
            self.check_return_type_object("CallStaticFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jfloat>(135)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticFloatMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticFloatMethod");
            self.check_return_type_object("CallStaticFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jfloat>(135)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticFloatMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticFloatMethod");
            self.check_return_type_object("CallStaticFloatMethod", obj, methodID, "float");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jfloat>(135)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn CallStaticDoubleMethodA(&self, obj: jclass, methodID: jmethodID, args: *const jtype) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticDoubleMethodA");
            self.check_return_type_static("CallStaticDoubleMethodA", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jdouble>(140)(self.functions, obj, methodID, args)
    }

    pub unsafe fn CallStaticDoubleMethod0(&self, obj: jobject, methodID: jmethodID) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticDoubleMethod");
            self.check_return_type_object("CallStaticDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID) -> jdouble>(138)(self.functions, obj, methodID)
    }

    pub unsafe fn CallStaticDoubleMethod1<A: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticDoubleMethod");
            self.check_return_type_object("CallStaticDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A) -> jdouble>(138)(self.functions, obj, methodID, arg1)
    }

    pub unsafe fn CallStaticDoubleMethod2<A: Into<jtype>, B: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticDoubleMethod");
            self.check_return_type_object("CallStaticDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B) -> jdouble>(138)(self.functions, obj, methodID, arg1, arg2)
    }
    pub unsafe fn CallStaticDoubleMethod3<A: Into<jtype>, B: Into<jtype>, C: Into<jtype>>(&self, obj: jobject, methodID: jmethodID, arg1: A, arg2: B, arg3: C) -> jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("CallStaticDoubleMethod");
            self.check_return_type_object("CallStaticDoubleMethod", obj, methodID, "double");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, A, B, C) -> jdouble>(138)(self.functions, obj, methodID, arg1, arg2, arg3)
    }

    pub unsafe fn NewString(&self, unicodeChars: *const jchar, len: jsize) -> jstring {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewString");
            assert!(!unicodeChars.is_null(), "NewString string must not be null");
            assert!(len >= 0, "NewString len must not be negative");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, *const jchar, jsize) -> jstring>(163)(self.functions, unicodeChars, len)
    }

    pub unsafe fn GetStringLength(&self, string: jstring) -> jsize {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStringLength");
            assert!(!string.is_null(), "GetStringLength string must not be null");
            self.check_if_arg_is_string("GetStringLength", string);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jstring) -> jsize>(164)(self.functions, string)
    }

    pub unsafe fn GetStringChars(&self, string: jstring, isCopy: *mut jboolean) -> *const jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStringChars");
            assert!(!string.is_null(), "GetStringChars string must not be null");
            self.check_if_arg_is_string("GetStringChars", string);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jstring, *mut jboolean) -> *const jchar>(165)(self.functions, string, isCopy)
    }

    pub unsafe fn ReleaseStringChars(&self, string: jstring, chars: *const jchar) {
        #[cfg(feature = "asserts")]
        {
            assert!(!string.is_null(), "ReleaseStringChars string must not be null");
            assert!(!chars.is_null(), "ReleaseStringChars chars must not be null");
            self.check_if_arg_is_string("ReleaseStringChars", string);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jstring, *const jchar)>(166)(self.functions, string, chars)
    }

    pub unsafe fn NewStringUTF(&self, bytes: *const c_char) -> jstring {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewStringUTF");
            assert!(!bytes.is_null(), "NewStringUTF string must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, *const c_char) -> jstring>(167)(self.functions, bytes)
    }

    pub unsafe fn NewStringUTF_str(&self, str: &str) -> jstring {
        let raw = CString::new(str).unwrap();
        let x =  self.NewStringUTF(raw.as_ptr());
        x
    }

    pub unsafe fn GetStringUTFLength(&self, string: jstring) -> jsize {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStringUTFLength");
            assert!(!string.is_null(), "GetStringUTFLength string must not be null");
            self.check_if_arg_is_string("GetStringUTFLength", string);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jstring) -> jsize>(168)(self.functions, string)
    }

    pub unsafe fn GetStringUTFChars(&self, string: jstring, isCopy: *mut jboolean) -> *const c_char {
        #[cfg(feature = "asserts")]
        {
            assert!(!string.is_null(), "GetStringUTFChars string must not be null");
            self.check_if_arg_is_string("GetStringUTFChars", string);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jstring, *mut jboolean) -> *const c_char>(169)(self.functions, string, isCopy)
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
            assert!(!string.is_null(), "ReleaseStringUTFChars string must not be null");
            assert!(!utf.is_null(), "ReleaseStringUTFChars utf must not be null");
            self.check_if_arg_is_string("ReleaseStringUTFChars", string);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jstring, *const c_char)>(170)(self.functions, string, utf)
    }

    pub unsafe fn GetStringRegion(&self, string: jstring, start: jsize, len: jsize, buffer: *mut jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStringRegion");
            assert!(!string.is_null(), "GetStringRegion string must not be null");
            assert!(!buffer.is_null(), "GetStringRegion buffer must not be null");
            self.check_if_arg_is_string("GetStringRegion", string);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jstring, jsize, jsize, *mut jchar)>(220)(self.functions, string, start, len, buffer)
    }

    pub unsafe fn GetStringUTFRegion(&self, string: jstring, start: jsize, len: jsize, buffer: *mut c_char) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStringUTFRegion");
            assert!(!string.is_null(), "GetStringUTFRegion string must not be null");
            self.check_if_arg_is_string("GetStringUTFRegion", string);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jstring, jsize, jsize, *mut c_char)>(221)(self.functions, string, start, len, buffer)
    }

    pub unsafe fn GetStringCritical(&self, string: jstring, isCopy: *mut jboolean) -> *const jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetStringCritical");
            assert!(!string.is_null(), "GetStringCritical string must not be null");
            self.check_if_arg_is_string("GetStringCritical", string);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jstring, *mut jboolean) -> *const jchar>(224)(self.functions, string, isCopy)
    }



    pub unsafe fn ReleaseStringCritical(&self, string: jstring, cstring: *const jchar) {
        #[cfg(feature = "asserts")]
        {
            assert!(!string.is_null(), "ReleaseStringCritical string must not be null");
            assert!(!cstring.is_null(), "ReleaseStringCritical cstring must not be null");
            self.check_if_arg_is_string("GetStringCritical", string);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jstring, *const jchar)>(225)(self.functions, string, cstring)
    }


    pub unsafe fn GetArrayLength(&self, array: jarray) -> jsize {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetArrayLength");
            assert!(!array.is_null(), "GetArrayLength array must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jarray) -> jsize>(171)(self.functions, array)
    }

    pub unsafe fn NewObjectArray(&self, len: jsize, elementClass: jclass, initialElement: jobject) -> jobjectArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewObjectArray");
            assert!(!elementClass.is_null(), "NewObjectArray elementClass must not be null");
            assert!(len >= 0, "NewObjectArray len mot not be negative {}", len);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize, jclass, jobject) -> jobjectArray>(172)(self.functions, len, elementClass, initialElement)
    }

    pub unsafe fn GetObjectArrayElement(&self, array: jobjectArray, index: jsize) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetObjectArrayElement");
            assert!(!array.is_null(), "GetObjectArrayElement array must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jobjectArray, jsize) -> jobject>(173)(self.functions, array, index)
    }

    pub unsafe fn SetObjectArrayElement(&self, array: jobjectArray, index: jsize, value: jobject) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetObjectArrayElement");
            assert!(!array.is_null(), "SetObjectArrayElement array must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jobjectArray, jsize, jobject)>(174)(self.functions, array, index, value);
    }

    pub unsafe fn NewBooleanArray(&self, size: jsize) -> jbooleanArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewBooleanArray");
            assert!(size >= 0, "NewBooleanArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize) -> jobject>(175)(self.functions, size)
    }

    pub unsafe fn NewByteArray(&self, size: jsize) -> jbyteArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewByteArray");
            assert!(size >= 0, "NewByteArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize) -> jbyteArray>(176)(self.functions, size)
    }

    pub unsafe fn NewCharArray(&self, size: jsize) -> jcharArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewCharArray");
            assert!(size >= 0, "NewCharArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize) -> jcharArray>(177)(self.functions, size)
    }

    pub unsafe fn NewShortArray(&self, size: jsize) -> jshortArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewShortArray");
            assert!(size >= 0, "NewShortArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize) -> jshortArray>(178)(self.functions, size)
    }

    pub unsafe fn NewIntArray(&self, size: jsize) -> jintArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewIntArray");
            assert!(size >= 0, "NewIntArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize) -> jintArray>(179)(self.functions, size)
    }

    pub unsafe fn NewLongArray(&self, size: jsize) -> jlongArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewLongArray");
            assert!(size >= 0, "NewLongArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize) -> jlongArray>(180)(self.functions, size)
    }

    pub unsafe fn NewFloatArray(&self, size: jsize) -> jfloatArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewFloatArray");
            assert!(size >= 0, "NewFloatArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize) -> jfloatArray>(181)(self.functions, size)
    }

    pub unsafe fn NewDoubleArray(&self, size: jsize) -> jdoubleArray {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewDoubleArray");
            assert!(size >= 0, "NewDoubleArray size must not be negative {}", size);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jsize) -> jdoubleArray>(182)(self.functions, size)
    }

    pub unsafe fn GetBooleanArrayElements(&self, array: jbooleanArray) -> *mut jboolean {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetBooleanArrayElements");
            assert!(!array.is_null(), "GetBooleanArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray) -> *mut jboolean>(183)(self.functions, array)
    }

    pub unsafe fn GetByteArrayElements(&self, array: jbyteArray) -> *mut jbyte {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetByteArrayElements");
            assert!(!array.is_null(), "GetByteArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbyteArray) -> *mut jbyte>(184)(self.functions, array)
    }

    pub unsafe fn GetCharArrayElements(&self, array: jcharArray) -> *mut jchar {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetCharArrayElements");
            assert!(!array.is_null(), "GetCharArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jcharArray) -> *mut jchar>(185)(self.functions, array)
    }

    pub unsafe fn GetShortArrayElements(&self, array: jshortArray) -> *mut jshort {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetShortArrayElements");
            assert!(!array.is_null(), "GetShortArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jshortArray) -> *mut jshort>(186)(self.functions, array)
    }

    pub unsafe fn GetIntArrayElements(&self, array: jintArray) -> *mut jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetIntArrayElements");
            assert!(!array.is_null(), "GetIntArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jintArray) -> *mut jint>(187)(self.functions, array)
    }

    pub unsafe fn GetLongArrayElements(&self, array: jlongArray) -> *mut jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetLongArrayElements");
            assert!(!array.is_null(), "GetLongArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jlongArray) -> *mut jlong>(188)(self.functions, array)
    }

    pub unsafe fn GetFloatArrayElements(&self, array: jfloatArray) -> *mut jfloat {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetFloatArrayElements");
            assert!(!array.is_null(), "GetFloatArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jfloatArray) -> *mut jfloat>(189)(self.functions, array)
    }

    pub unsafe fn GetDoubleArrayElements(&self, array: jdoubleArray) -> *mut jdouble {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetDoubleArrayElements");
            assert!(!array.is_null(), "GetDoubleArrayElements jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jdoubleArray) -> *mut jdouble>(190)(self.functions, array)
    }

    pub unsafe fn ReleaseBooleanArrayElements(&self, array: jbooleanArray, elems: *mut jboolean, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleaseBooleanArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseBooleanArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseBooleanArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, *mut jboolean, jint)>(191)(self.functions, array, elems, mode);
    }

    pub unsafe fn ReleaseByteArrayElements(&self, array: jbyteArray, elems: *mut jbyte, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleaseByteArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseByteArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseByteArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbyteArray, *mut jbyte, jint)>(192)(self.functions, array, elems, mode);
    }

    pub unsafe fn ReleaseCharArrayElements(&self, array: jcharArray, elems: *mut jchar, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleaseCharArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseCharArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseCharArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jcharArray, *mut jchar, jint)>(193)(self.functions, array, elems, mode);
    }

    pub unsafe fn ReleaseShortArrayElements(&self, array: jshortArray, elems: *mut jshort, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleaseShortArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseShortArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseShortArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jshortArray, *mut jshort, jint)>(194)(self.functions, array, elems, mode);
    }

    pub unsafe fn ReleaseIntArrayElements(&self, array: jintArray, elems: *mut jint, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleaseIntArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseIntArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseIntArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jintArray, *mut jint, jint)>(195)(self.functions, array, elems, mode);
    }

    pub unsafe fn ReleaseLongArrayElements(&self, array: jlongArray, elems: *mut jlong, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleaseLongArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseLongArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseLongArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jlongArray, *mut jlong, jint)>(196)(self.functions, array, elems, mode);
    }

    pub unsafe fn ReleaseFloatArrayElements(&self, array: jfloatArray, elems: *mut jfloat, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleaseFloatArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseFloatArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseFloatArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jfloatArray, *mut jfloat, jint)>(197)(self.functions, array, elems, mode);
    }

    pub unsafe fn ReleaseDoubleArrayElements(&self, array: jdoubleArray, elems: *mut jdouble, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleaseDoubleArrayElements jarray must not be null");
            assert!(!elems.is_null(), "ReleaseDoubleArrayElements elems must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleaseDoubleArrayElements mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jdoubleArray, *mut jdouble, jint)>(198)(self.functions, array, elems, mode);
    }

    pub unsafe fn GetBooleanArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *mut jboolean) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetBooleanArrayRegion");
            assert!(!array.is_null(), "GetBooleanArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetBooleanArrayRegion buf must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *mut jboolean)>(199)(self.functions, array, start, len, buf);
    }

    pub unsafe fn GetByteArrayRegion(&self, array: jbyteArray, start: jsize, len: jsize, buf: *mut jbyte) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetByteArrayRegion");
            assert!(!array.is_null(), "GetByteArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetByteArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *mut jbyte)>(200)(self.functions, array, start, len, buf);
    }

    pub unsafe fn GetCharArrayRegion(&self, array: jcharArray, start: jsize, len: jsize, buf: *mut jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetCharArrayRegion");
            assert!(!array.is_null(), "GetCharArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetCharArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *mut jchar)>(201)(self.functions, array, start, len, buf);
    }

    pub unsafe fn GetShortArrayRegion(&self, array: jshortArray, start: jsize, len: jsize, buf: *mut jshort) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetShortArrayRegion");
            assert!(!array.is_null(), "GetShortArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetShortArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *mut jshort)>(202)(self.functions, array, start, len, buf);
    }

    pub unsafe fn GetIntArrayRegion(&self, array: jintArray, start: jsize, len: jsize, buf: *mut jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetIntArrayRegion");
            assert!(!array.is_null(), "GetIntArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetIntArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *mut jint)>(203)(self.functions, array, start, len, buf);
    }

    pub unsafe fn GetLongArrayRegion(&self, array: jlongArray, start: jsize, len: jsize, buf: *mut jlong) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetLongArrayRegion");
            assert!(!array.is_null(), "GetLongArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetLongArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *mut jlong)>(204)(self.functions, array, start, len, buf);
    }

    pub unsafe fn GetFloatArrayRegion(&self, array: jfloatArray, start: jsize, len: jsize, buf: *mut jfloat) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetFloatArrayRegion");
            assert!(!array.is_null(), "GetFloatArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetFloatArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *mut jfloat)>(205)(self.functions, array, start, len, buf);
    }

    pub unsafe fn GetDoubleArrayRegion(&self, array: jdoubleArray, start: jsize, len: jsize, buf: *mut jdouble) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetDoubleArrayRegion");
            assert!(!array.is_null(), "GetDoubleArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "GetDoubleArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *mut jdouble)>(206)(self.functions, array, start, len, buf);
    }

    pub unsafe fn SetBooleanArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jboolean) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetBooleanArrayRegion");
            assert!(!array.is_null(), "SetBooleanArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetBooleanArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *const jboolean)>(207)(self.functions, array, start, len, buf);
    }

    pub unsafe fn SetByteArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jbyte) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetByteArrayRegion");
            assert!(!array.is_null(), "SetByteArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetByteArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *const jbyte)>(208)(self.functions, array, start, len, buf);
    }

    pub unsafe fn SetCharArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jchar) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetCharArrayRegion");
            assert!(!array.is_null(), "SetCharArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetCharArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *const jchar)>(209)(self.functions, array, start, len, buf);
    }

    pub unsafe fn SetShortArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jshort) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetShortArrayRegion");
            assert!(!array.is_null(), "SetShortArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetShortArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *const jshort)>(210)(self.functions, array, start, len, buf);
    }

    pub unsafe fn SetIntArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jint) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetIntArrayRegion");
            assert!(!array.is_null(), "SetIntArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetIntArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *const jint)>(211)(self.functions, array, start, len, buf);
    }

    pub unsafe fn SetLongArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jlong) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetLongArrayRegion");
            assert!(!array.is_null(), "SetLongArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetLongArrayRegion buf must not be null");
        }


        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *const jlong)>(212)(self.functions, array, start, len, buf);
    }

    pub unsafe fn SetFloatArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jfloat) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetFloatArrayRegion");
            assert!(!array.is_null(), "SetFloatArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetFloatArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *const jfloat)>(213)(self.functions, array, start, len, buf);
    }

    pub unsafe fn SetDoubleArrayRegion(&self, array: jbooleanArray, start: jsize, len: jsize, buf: *const jdouble) {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("SetDoubleArrayRegion");
            assert!(!array.is_null(), "SetDoubleArrayRegion jarray must not be null");
            assert!(!buf.is_null(), "SetDoubleArrayRegion buf must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jbooleanArray, jsize, jsize, *const jdouble)>(214)(self.functions, array, start, len, buf);
    }

    pub unsafe fn GetPrimitiveArrayCritical(&self, array: jarray, isCopy: *mut jboolean) -> *mut c_void {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetPrimitiveArrayCritical");
            assert!(!array.is_null(), "GetPrimitiveArrayCritical jarray must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jarray, *mut jboolean) -> *mut c_void>(222)(self.functions, array, isCopy)
    }

    pub unsafe fn ReleasePrimitiveArrayCritical(&self, array: jarray, carray: *mut c_void, mode: jint) {
        #[cfg(feature = "asserts")]
        {
            assert!(!array.is_null(), "ReleasePrimitiveArrayCritical jarray must not be null");
            assert!(!carray.is_null(), "ReleasePrimitiveArrayCritical carray must not be null");
            assert!(mode == JNI_OK || mode == JNI_COMMIT || mode == JNI_ABORT, "ReleasePrimitiveArrayCritical mode is invalid {}", mode);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jarray, *mut c_void, jint)>(223)(self.functions, array, carray, mode);
    }

    pub unsafe fn RegisterNatives(&self, clazz: jclass, methods : *const JNINativeMethod, size: jint) -> jint {
        #[cfg(feature = "asserts")]
        {
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

        self.jni::<extern "system" fn(JNIEnvPtr, jclass, *const JNINativeMethod, jint) -> jint>(215)(self.functions, clazz, methods, size)
    }

    pub unsafe fn UnregisterNatives(&self, clazz: jclass) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("UnregisterNatives");
            assert!(!clazz.is_null(), "UnregisterNatives class must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jclass) -> jint>(216)(self.functions, clazz)
    }

    pub unsafe fn MonitorEnter(&self, obj: jobject) -> jint {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("MonitorEnter");
            assert!(!obj.is_null(), "MonitorEnter object must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jint>(217)(self.functions, obj)
    }

    pub unsafe fn MonitorExit(&self, obj: jobject) -> jint {
        #[cfg(feature = "asserts")]
        {
            assert!(!obj.is_null(), "MonitorExit object must not be null");
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jint>(218)(self.functions, obj)
    }

    pub unsafe fn NewDirectByteBuffer(&self, address: *mut c_void, capacity: jlong) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("NewDirectByteBuffer");
            assert!(!address.is_null(), "NewDirectByteBuffer address must not be null");
            assert!(capacity >= 0, "NewDirectByteBuffer capacity must not be negative {}", capacity);
            assert!(capacity <= jint::MAX as jlong, "NewDirectByteBuffer capacity is too big, its larger than Integer.MAX_VALUE {}", capacity);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, *mut c_void, jlong) -> jobject>(229)(self.functions, address, capacity)
    }

    pub unsafe fn GetDirectBufferAddress(&self, buf: jobject) -> *mut c_void {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetDirectBufferAddress");
            assert!(!buf.is_null(), "GetDirectBufferAddress buffer must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> *mut c_void>(230)(self.functions, buf)
    }

    pub unsafe fn GetDirectBufferCapacity(&self, buf: jobject) -> jlong {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetDirectBufferCapacity");
            assert!(!buf.is_null(), "GetDirectBufferCapacity buffer must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jlong>(231)(self.functions, buf)
    }

    pub unsafe fn FromReflectedMethod(&self, method: jobject) -> jmethodID {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("FromReflectedMethod");
            assert!(!method.is_null(), "FromReflectedMethod method must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jmethodID>(7)(self.functions, method)
    }

    pub unsafe fn ToReflectedMethod(&self, cls: jclass, jmethodID: jmethodID, isStatic: jboolean) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("ToReflectedMethod");
            assert!(!cls.is_null(), "ToReflectedMethod class must not be null");
            assert!(!jmethodID.is_null(), "ToReflectedMethod method must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass, jmethodID, jboolean) -> jobject>(9)(self.functions, cls, jmethodID, isStatic)
    }

    pub unsafe fn FromReflectedField(&self, field: jobject) -> jfieldID {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("FromReflectedField");
            assert!(!field.is_null(), "FromReflectedField field must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jfieldID>(8)(self.functions, field)
    }

    pub unsafe fn ToReflectedField(&self, cls: jclass, jfieldID: jfieldID, isStatic: jboolean) -> jobject {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("ToReflectedField");
            assert!(!cls.is_null(), "ToReflectedField class must not be null");
            assert!(!jfieldID.is_null(), "ToReflectedField field must not be null");
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jclass, jfieldID, jboolean) -> jobject>(12)(self.functions, cls, jfieldID, isStatic)
    }

    pub unsafe fn GetJavaVM(&self) -> Result<JavaVM, jint> {
        #[cfg(feature = "asserts")]
        {
            self.check_no_exception("GetJavaVM");
        }
        let mut r : JNIInvPtr = null_mut();
        let res = self.jni::<extern "system" fn(JNIEnvPtr, *mut JNIInvPtr) -> jint>(219)(self.functions, &mut r);
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
            self.check_no_exception("GetModule");
            assert!(self.GetVersion() >= JNI_VERSION_9);
        }

        self.jni::<extern "system" fn(JNIEnvPtr, jclass) -> jobject>(233)(self.functions, cls)
    }

    pub unsafe fn IsVirtualThread(&self, thread: jobject) -> jboolean {
        #[cfg(feature = "asserts")]
        {
            assert!(self.GetVersion() >= JNI_VERSION_21);
        }
        self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jboolean>(234)(self.functions, thread)
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
    unsafe fn check_is_class(&self, context: &str, obj: jclass) {
        assert!(!obj.is_null(), "{} class is null", context);
        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        //GET OBJECT CLASS
        let tcl = self.jni::<extern "system" fn(JNIEnvPtr, jobject) -> jobject>(31)(self.functions, obj);
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
        let rtc = self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, f, field_rtyp, null());
        assert!(!rtc.is_null(), "{} java/lang/reflect/Field#getType returned null???", context);
        self.DeleteLocalRef(field_cl);
        self.DeleteLocalRef(f);
        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, rtc, class_name,  null());
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
        let rtc = self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, m, meth_rtyp,  null());
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
        let name_str = self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, rtc, class_name,  null());
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
    unsafe fn check_return_type_object(&self, context: &str, obj: jobject, methodID: jmethodID, ty: &str) {
        assert!(!obj.is_null(), "{} obj is null", context);
        let clazz = self.GetObjectClass(obj);
        assert!(!clazz.is_null(), "{} obj.class is null??", context);
        assert!(!methodID.is_null(), "{} methodID is null", context);
        let m = self.ToReflectedMethod(clazz, methodID, false);
        assert!(!m.is_null(), "{} -> ToReflectedMethod returned null", context);
        let meth_cl = self.FindClass_str("java/lang/reflect/Method");
        assert!(!m.is_null(), "{} java/lang/reflect/Method not found???", context);
        let meth_rtyp = self.GetMethodID_str(meth_cl, "getReturnType", "()Ljava/lang/Class;");
        assert!(!meth_rtyp.is_null(), "{} java/lang/reflect/Method#getReturnType not found???", context);
        //CallObjectMethodA
        let rtc = self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, m, meth_rtyp,  null());
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
        let name_str = self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, rtc, class_name,  null());
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
        let rtc = self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, f, field_rtyp, null());
        assert!(!rtc.is_null(), "{} java/lang/reflect/Field#getType returned null???", context);
        self.DeleteLocalRef(field_cl);
        self.DeleteLocalRef(f);
        let class_cl = self.FindClass_str("java/lang/Class");
        assert!(!class_cl.is_null(), "{} java/lang/Class not found???", context);
        let class_name = self.GetMethodID_str(class_cl, "getName", "()Ljava/lang/String;");
        assert!(!class_name.is_null(), "{} java/lang/Class#getName not found???", context);
        //CallObjectMethodA
        let name_str = self.jni::<extern "system" fn(JNIEnvPtr, jobject, jmethodID, *const jtype) -> jobject>(36)(self.functions, rtc, class_name,  null());
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
    JNI_CreateJavaVM: JNI_CreateJavaVM,
    JNI_GetCreatedJavaVMs: JNI_GetCreatedJavaVMs,
}

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
                JNI_CreateJavaVM: mem::transmute_copy(&JNI_CreateJavaVM),
                JNI_GetCreatedJavaVMs: mem::transmute_copy(&JNI_GetCreatedJavaVMs)
            }
        }
    }

    pub fn JNI_CreateJavaVM(&self) -> JNI_CreateJavaVM {
        self.JNI_CreateJavaVM
    }
    pub fn JNI_GetCreatedJavaVMs(&self) -> JNI_GetCreatedJavaVMs {
        self.JNI_GetCreatedJavaVMs
    }
}

unsafe impl Sync for JNIDynamicLink {

}

unsafe impl Send for JNIDynamicLink {

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
    let mut buf : [JNIInvPtr; 64] = [null_mut(); 64];
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

    let mut jvm : JNIInvPtr = null_mut();
    let mut env : JNIEnv = JNIEnv {
        functions: null_mut(),
    };

    let res = link.JNI_CreateJavaVM()(&mut jvm, &mut env, arguments);
    if res != JNI_OK {
        return Err(res);
    }

    if jvm.is_null() {
        panic!("JNI_CreateJavaVM returned JNI_OK but the JavaVM pointer is null");
    }

    if env.functions.is_null() {
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
    let link = get_link();

    let mut vm_args: Vec<JavaVMOption> = Vec::with_capacity(arguments.len());
    let mut dealloc_list = Vec::with_capacity(arguments.len());
    for arg in arguments {
        let jvm_arg = CString::new(arg.as_str()).unwrap().into_raw();

        vm_args.push(JavaVMOption{
            optionString: jvm_arg,
            extraInfo: null_mut(),
        });

        dealloc_list.push(jvm_arg);
    }

    let mut args = JavaVMInitArgs {
        version,
        nOptions: vm_args.len() as i32,
        options: vm_args.as_mut_ptr(),
        ignoreUnrecognized: 1,
    };

    let mut jvm : JNIInvPtr = null_mut();
    let mut env : JNIEnv = JNIEnv {
        functions: null_mut(),
    };


    let err = link.JNI_CreateJavaVM()(&mut jvm, &mut env, &mut args);

    for x in dealloc_list {
        _=CString::from_raw(x);
    }

    if err != JNI_OK {
        return Err(err);
    }

    if jvm.is_null() {
        panic!("JNI_CreateJavaVM returned JNI_OK but the JavaVM pointer is null");
    }

    if env.functions.is_null() {
        panic!("JNI_CreateJavaVM returned JNI_OK but the JNIEnv pointer is null");
    }

    Ok((JavaVM{ functions: jvm }, env))
}



impl JavaVM {

    #[inline]
    unsafe fn jnx<X>(&self, index: usize) -> X {
        unsafe {mem::transmute_copy(&(**self.functions)[index])}
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
        let mut envptr : JNIEnvPtr = null_mut();

        let result = self.jnx::<extern "system" fn(JNIInvPtr, *mut JNIEnvPtr, *mut JavaVMAttachArgs) -> jint>(4)
            (self.functions, &mut envptr, args);
        if result != JNI_OK {
            return Err(result);
        }

        if envptr.is_null() {
            panic!("AttachCurrentThread returned JNI_OK but did not set the JNIEnv pointer!");
        }

        Ok(JNIEnv{functions: envptr})
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
        let mut envptr : JNIEnvPtr = null_mut();

        let result = self.jnx::<extern "system" fn(JNIInvPtr, *mut JNIEnvPtr, *mut JavaVMAttachArgs) -> jint>(7)
            (self.functions, &mut envptr, args);

        if result != JNI_OK {
            return Err(result);
        }

        if envptr.is_null() {
            panic!("AttachCurrentThreadAsDaemon returned JNI_OK but did not set the JNIEnv pointer!");
        }

        Ok(JNIEnv{functions: envptr})
    }

    ///
    /// Gets the JNIEnv for the current thread.
    ///
    pub unsafe fn GetEnv(&self, jni_version: jint) -> Result<JNIEnv, jint> {
        let mut envptr : JNIEnvPtr = null_mut();

        let result = self.jnx::<extern "system" fn(JNIInvPtr, *mut JNIEnvPtr, jint) -> jint>(6)
            (self.functions, &mut envptr, jni_version);

        if result != JNI_OK {
            return Err(result);
        }

        if envptr.is_null() {
            panic!("GetEnv returned JNI_OK but did not set the JNIEnv pointer!");
        }

        Ok(JNIEnv{functions: envptr})
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