use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt::{self, Debug, Display, Formatter};
use core::hash::{Hash, Hasher};
use core::ops::{Deref, DerefMut, Not};

use crate::private::{SealedJBooleanInputLayout, SealedJBooleanMutPtr};
use crate::{JBooleanInputLayout, JBooleanMutPtr};
use alloc::vec::Vec;
use std::ffi::c_uchar;
use std::ptr::null_mut;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct jboolean(c_uchar);

#[test]
fn test_jboolean_alignments_and_size() {
    assert_eq!(align_of::<jboolean>(), align_of::<bool>());
    assert_eq!(size_of::<jboolean>(), size_of::<bool>());
}

impl jboolean {
    pub const FALSE: Self = Self(0);

    pub const TRUE: Self = Self(1);

    #[must_use]
    pub const fn from(value: bool) -> Self {
        if value { Self::TRUE } else { Self::FALSE }
    }

    #[must_use]
    pub const fn from_wide(value: c_uchar) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn as_bool(self) -> bool {
        self.0 != 0
    }

    #[must_use]
    pub const fn as_wide(self) -> c_uchar {
        self.0
    }

    pub const fn narrow(&mut self) {
        if self.0 != 0 {
            self.0 = 1;
        }
    }
    #[must_use]
    pub const fn is_wide(&self) -> bool {
        (self.0 & 0x1) != self.0
    }

    #[must_use]
    pub const fn transmute_slice(input: &[Self]) -> &[c_uchar] {
        unsafe { core::slice::from_raw_parts(input.as_ptr().cast(), input.len()) }
    }

    #[must_use]
    pub const fn transmute_slice_mut(input: &mut [Self]) -> &mut [c_uchar] {
        unsafe { core::slice::from_raw_parts_mut(input.as_mut_ptr().cast(), input.len()) }
    }

    #[must_use]
    pub const fn transmute_bool_slice(input: &[bool]) -> &[Self] {
        unsafe { core::slice::from_raw_parts(input.as_ptr().cast(), input.len()) }
    }

    #[must_use]
    pub const fn transmute_wide_slice(input: &[c_uchar]) -> &[Self] {
        unsafe { core::slice::from_raw_parts(input.as_ptr().cast(), input.len()) }
    }

    #[must_use]
    pub const fn transmute_wide_slice_mut(input: &mut [c_uchar]) -> &mut [Self] {
        unsafe { core::slice::from_raw_parts_mut(input.as_mut_ptr().cast(), input.len()) }
    }

    #[must_use]
    pub fn transmute_wide_vec(input: Vec<c_uchar>) -> Vec<Self> {
        let mut input = core::mem::ManuallyDrop::new(input);
        unsafe { Vec::from_raw_parts(input.as_mut_ptr().cast(), input.len(), input.capacity()) }
    }

    #[must_use]
    pub fn narrow_vec(mut input: Vec<Self>) -> Vec<bool> {
        input.iter_mut().for_each(Self::narrow);
        let mut input = core::mem::ManuallyDrop::new(input);
        unsafe { Vec::from_raw_parts(input.as_mut_ptr().cast(), input.len(), input.capacity()) }
    }
}

impl AsRef<bool> for jboolean {
    fn as_ref(&self) -> &bool {
        if self.0 != 0 { &true } else { &false }
    }
}

impl Borrow<bool> for jboolean {
    fn borrow(&self) -> &bool {
        if self.0 != 0 { &true } else { &false }
    }
}

impl Deref for jboolean {
    type Target = core::ffi::c_uchar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for jboolean {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for jboolean {
    fn default() -> Self {
        Self::FALSE
    }
}
impl Debug for jboolean {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&bool::from(*self), f)
    }
}
impl Display for jboolean {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&bool::from(*self), f)
    }
}

impl From<bool> for jboolean {
    fn from(value: bool) -> Self {
        Self(value.into())
    }
}
impl From<jboolean> for bool {
    fn from(value: jboolean) -> Self {
        value.0 != 0
    }
}

impl Not for jboolean {
    type Output = bool;
    fn not(self) -> Self::Output {
        self.0 == 0
    }
}

impl Eq for jboolean {}
impl PartialEq<Self> for jboolean {
    fn eq(&self, other: &Self) -> bool {
        bool::from(*self) == bool::from(*other)
    }
}

impl PartialEq<bool> for jboolean {
    fn eq(&self, other: &bool) -> bool {
        bool::from(*self) == *other
    }
}
impl PartialEq<jboolean> for bool {
    fn eq(&self, other: &jboolean) -> bool {
        Self::from(*other) == *self
    }
}

impl PartialOrd<Self> for jboolean {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<bool> for jboolean {
    fn partial_cmp(&self, other: &bool) -> Option<Ordering> {
        PartialOrd::partial_cmp(&bool::from(*self), other)
    }
}
impl PartialOrd<jboolean> for bool {
    fn partial_cmp(&self, other: &jboolean) -> Option<Ordering> {
        PartialOrd::partial_cmp(self, &Self::from(*other))
    }
}

impl Ord for jboolean {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&bool::from(*self), &bool::from(*other))
    }
}

impl Hash for jboolean {
    fn hash<H: Hasher>(&self, state: &mut H) {
        bool::from(*self).hash(state);
    }
}

impl JBooleanMutPtr for &mut bool {}

impl SealedJBooleanMutPtr for &mut bool {
    fn use_jboolean_mut<R>(self, func: impl FnOnce(*mut jboolean) -> R) -> R {
        let mut value = jboolean::from(*self);
        let res = func(&raw mut value);
        *self = value.as_bool();
        res
    }
}

impl JBooleanMutPtr for *mut jboolean {}
impl SealedJBooleanMutPtr for *mut jboolean {
    fn use_jboolean_mut<R>(self, func: impl FnOnce(*mut jboolean) -> R) -> R {
        func(self)
    }
}

impl JBooleanMutPtr for *mut bool {}
impl SealedJBooleanMutPtr for *mut bool {
    fn use_jboolean_mut<R>(self, func: impl FnOnce(*mut jboolean) -> R) -> R {
        if self.is_null() {
            return func(null_mut());
        }
        //Currently all usages of this are assumed to NOT read the original value.
        //It is valid that the original value is uninitialized memory so we cannot really read it here.
        //So we just always use false as the initial value and only write to the pointer after the call completes.
        let mut bool = jboolean::FALSE;
        let result = func(&raw mut bool);
        //SAFETY: if the pointer is non-null then it better not be dangling.
        unsafe {
            self.write_unaligned(bool.as_bool());
        }
        result
    }
}

impl JBooleanMutPtr for () {}
impl SealedJBooleanMutPtr for () {
    fn use_jboolean_mut<R>(self, func: impl FnOnce(*mut jboolean) -> R) -> R {
        func(null_mut())
    }
}

impl SealedJBooleanInputLayout for c_uchar {}

impl JBooleanInputLayout for c_uchar {}

impl SealedJBooleanInputLayout for jboolean {}

impl JBooleanInputLayout for jboolean {}

impl SealedJBooleanInputLayout for bool {}

impl JBooleanInputLayout for bool {}
