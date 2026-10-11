//! Borrowed Arc payloads, without casting raw storage to an Arc/Head/Args object.
use crate::{Args, Head};
use std::{marker::PhantomData, ops::Deref, ptr::NonNull, sync::Arc};
pub struct HeadRef<'a, N> {
    pointer: NonNull<N>,
    life: PhantomData<&'a N>,
}
pub struct ArgsRef<'a, N> {
    pointer: NonNull<Vec<N>>,
    life: PhantomData<&'a Vec<N>>,
}
// SAFETY: the private constructor requires an immutable Arc payload alive for
// all of 'a. These bounds match Arc's shared cross-thread ownership contract.
unsafe impl<N: Send + Sync> Send for HeadRef<'_, N> {}
unsafe impl<N: Send + Sync> Sync for HeadRef<'_, N> {}
unsafe impl<N: Send + Sync> Send for ArgsRef<'_, N> {}
unsafe impl<N: Send + Sync> Sync for ArgsRef<'_, N> {}
impl<'a, N> HeadRef<'a, N> {
    /// SAFETY: exact Arc::into_raw/as_ptr payload; an owner lives for all of 'a.
    pub(crate) unsafe fn from_raw(pointer: *const N) -> Self {
        Self {
            pointer: NonNull::new(pointer.cast_mut()).expect("live Arc pointer"),
            life: PhantomData,
        }
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }
    pub fn node(&self) -> &'a N {
        // SAFETY: the source Arc owner is required to live for all of 'a.
        unsafe { self.pointer.as_ref() }
    }
    pub(crate) fn key(&self) -> usize {
        self.pointer.as_ptr().addr()
    }
    pub fn clone_handle(&self) -> Head<N> {
        // SAFETY: source owner remains live for 'a; acquire one count then
        // transfer exactly that count to Arc without a fallible callback.
        unsafe {
            Arc::increment_strong_count(self.pointer.as_ptr());
            Head(Arc::from_raw(self.pointer.as_ptr()))
        }
    }
}
impl<N> Deref for HeadRef<'_, N> {
    type Target = N;
    fn deref(&self) -> &N {
        // SAFETY: private constructor requires a live Arc payload for 'a.
        unsafe { self.pointer.as_ref() }
    }
}
impl<'a, N> ArgsRef<'a, N> {
    /// SAFETY: exact Arc::into_raw/as_ptr Vec payload; owner lives for all of 'a.
    pub(crate) unsafe fn from_raw(pointer: *const Vec<N>) -> Self {
        Self {
            pointer: NonNull::new(pointer.cast_mut()).expect("live Arc pointer"),
            life: PhantomData,
        }
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }
    pub fn slice(&self) -> &'a [N] {
        // SAFETY: the source immutable Vec is required to live for all of 'a.
        unsafe { self.pointer.as_ref().as_slice() }
    }
    pub(crate) fn key(&self) -> usize {
        self.pointer.as_ptr().addr()
    }
    pub fn clone_handle(&self) -> Args<N> {
        // SAFETY: source owner remains live; acquire exactly the returned count.
        unsafe {
            Arc::increment_strong_count(self.pointer.as_ptr());
            Args(Arc::from_raw(self.pointer.as_ptr()))
        }
    }
}
impl<N> Deref for ArgsRef<'_, N> {
    type Target = [N];
    fn deref(&self) -> &[N] {
        // SAFETY: private constructor requires a live immutable Vec for 'a.
        unsafe { self.pointer.as_ref().as_slice() }
    }
}
