//! Two demonstrations that `jboolean` may not be Rust's `bool`.
//!
//! **[1] A Java `boolean` holding the byte `0x02`.** Always reproduces. JNI
//! defines only "zero is false, non-zero is true", so `0x02` is a legal value.
//! Read into a `bool` and widened it yields 2, which a `bool` may never do.
//!
//! **[2] A Java method that returns `false`.** Reproduces sometimes.
//!
//! ```ignore
//! if env.CallBooleanMethod0(obj, return_false) {
//!     // Java returned false. On the published crate this block runs anyway.
//! }
//! ```
//!
//! Compiled twice from this one source, selected by feature:
//!
//! * `--features published` links jni-simple `=0.3.2` from crates.io, where
//!   `jboolean = bool`.
//! * `--features local` links this working tree, where `jboolean = u8`.
//!
//! Nothing else differs, so any difference in the output is down to that type.
//!
//! `jni.h` declares `typedef unsigned char jboolean;`, and AAPCS64 defines only
//! bits 0..7 of `x0` for a size-1 return -- the rest are unspecified, and ART
//! does leave junk there. `bool` is lowered as `zeroext i1`, promising the whole
//! register is 0 or 1, so LLVM compiles `if b { .. }` on arm64 to nothing at all
//! and the junk decides the branch.

#[cfg(all(feature = "published", feature = "local"))]
compile_error!("enable exactly one of `published` or `local`, not both");

#[cfg(not(any(feature = "published", feature = "local")))]
compile_error!("enable exactly one of `published` or `local` (e.g. --features published)");

#[cfg(feature = "local")]
use jni_local as jni_simple;
#[cfg(feature = "published")]
use jni_published as jni_simple;

use core::hint::black_box;
use core::ptr::{addr_of, null};
use jni_simple::*;

/// The two things user code does with a `jboolean`.
///
/// Implemented for both `bool` and `u8` so this file compiles either way. The
/// asymmetry is the point: for `bool`, `truthy()` is the identity, which is
/// exactly what `if b { .. }` compiles to; for `u8` it must be written `!= 0`,
/// which is what JNI actually specifies.
trait JBoolExt: Copy {
    const IS_RUST_BOOL: bool;
    fn truthy(self) -> bool;
    fn widen(self) -> u32;
}

impl JBoolExt for bool {
    const IS_RUST_BOOL: bool = true;
    fn truthy(self) -> bool {
        self
    }
    fn widen(self) -> u32 {
        self as u32
    }
}

impl JBoolExt for u8 {
    const IS_RUST_BOOL: bool = false;
    fn truthy(self) -> bool {
        self != 0
    }
    fn widen(self) -> u32 {
        u32::from(self)
    }
}

const IS_RUST_BOOL: bool = <jboolean as JBoolExt>::IS_RUST_BOOL;

// Each measured call is #[inline(never)] and returns an integer, never a
// `bool`: a `bool` return here would itself be lowered as `zeroext i1` and
// would launder the thing being measured. `black_box` goes on the inputs only
// -- on the output it would force LLVM to re-derive a valid `i1`, inserting the
// very mask we are showing is absent.

/// `b as u32`. A `bool` may only ever widen to 0 or 1.
#[inline(never)]
unsafe fn widen_of(env: JNIEnv, obj: jobject, mid: jmethodID) -> u32 {
    env.CallBooleanMethodA(black_box(obj), black_box(mid), null()).widen()
}

/// `if b { 1 } else { 0 }` via `CallBooleanMethod` (vtable slot 37).
#[inline(never)]
unsafe fn branch_call0(env: JNIEnv, obj: jobject, mid: jmethodID) -> u32 {
    u32::from(env.CallBooleanMethod0(black_box(obj), black_box(mid)).truthy())
}

/// `if b { 1 } else { 0 }` via `CallBooleanMethodA` (vtable slot 39).
#[inline(never)]
unsafe fn branch_call_a(env: JNIEnv, obj: jobject, mid: jmethodID) -> u32 {
    u32::from(env.CallBooleanMethodA(black_box(obj), black_box(mid), null()).truthy())
}

/// `if b { 1 } else { 0 }` via `GetBooleanField`.
#[inline(never)]
unsafe fn branch_field(env: JNIEnv, obj: jobject, fid: jfieldID) -> u32 {
    u32::from(env.GetBooleanField(black_box(obj), black_box(fid)).truthy())
}

/// Emits `<label>|<value>|<ok|BROKEN>`.
fn line(out: &mut String, label: &str, value: u32, broken: bool) {
    out.push_str(&format!("{label}|{value}|{}\n", if broken { "BROKEN" } else { "ok" }));
}

/// Runs both checks and returns the report.
///
/// # Safety
/// Called by the JVM as the static native method `JBoolMre.rustProbe()`.
#[no_mangle]
pub unsafe extern "system" fn Java_JBoolMre_rustProbe(env: JNIEnv, class: jclass) -> jstring {
    let ctor = env.GetMethodID(class, "<init>", "()V");
    let obj = env.NewObject0(class, ctor);
    let mut out = String::new();

    // --- [1] a Java boolean holding 0x02 --------------------------------
    let two: u8 = 2;
    let arr = env.GetObjectField(obj, env.GetFieldID(class, "arr", "[Z"));
    env.SetBooleanArrayRegion(arr, 0, 1, addr_of!(two).cast::<jboolean>());

    // black_box before the test is load-bearing, and instructive: LLVM annotates
    // this result as range(i32 0, 2), because for a `bool` it must be 0 or 1.
    // Without the barrier it folds `> 1` to false and the check silently passes,
    // inheriting the very poison it is trying to report. Same below for [2].
    let widened = black_box(widen_of(env, obj, env.GetMethodID(class, "readArr", "()Z")));
    line(&mut out, "[1] boolean[0]=0x02  b as u32", widened, IS_RUST_BOOL && widened > 1);

    // --- [2] Java values that are genuinely false -----------------------
    // `returnFalse()` is `return false;` and `flagFalse` is never assigned, so
    // every one of these must leave the `if` untaken.
    let mid_false = env.GetMethodID(class, "returnFalse", "()Z");
    let fid_false = env.GetFieldID(class, "flagFalse", "Z");

    for (label, taken) in [
        ("[2] returnFalse()    CallBooleanMethod", branch_call0(env, obj, mid_false)),
        ("[2] returnFalse()    CallBooleanMethodA", branch_call_a(env, obj, mid_false)),
        ("[2] flagFalse        GetBooleanField", branch_field(env, obj, fid_false)),
    ] {
        let taken = black_box(taken);
        line(&mut out, label, taken, taken != 0);
    }

    out.pop();
    env.NewStringUTF(out.as_str())
}
