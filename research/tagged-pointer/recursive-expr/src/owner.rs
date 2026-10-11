//! Private Arc owner. Immediate bits are never dereferenced or given to Arc.
use std::{marker::PhantomData, mem::ManuallyDrop, ptr::NonNull, sync::Arc};

pub(crate) const MIN: i64 = -(1_i64 << 60);
pub(crate) const MAX: i64 = (1_i64 << 60) - 1;

#[repr(align(8))]
#[derive(Clone)]
pub(crate) struct Payload<T>(T);

pub(crate) enum View<'a, T> {
    Small(i64),
    Heap(&'a T),
}

pub(crate) enum Owned<T> {
    Small(i64),
    Heap(T),
}

#[repr(transparent)]
pub(crate) struct Packed<T> {
    pointer: NonNull<Payload<T>>,
    ownership: PhantomData<Arc<Payload<T>>>,
}

// SAFETY: heap storage has exactly Arc's immutable sharing policy. Both bounds
// are needed for sending an Arc owner. Integers contain no shared allocation.
unsafe impl<T: Send + Sync> Send for Packed<T> {}
unsafe impl<T: Send + Sync> Sync for Packed<T> {}

impl<T> Packed<T> {
    #[inline]
    pub(crate) fn small(value: i64) -> Self {
        assert!((MIN..=MAX).contains(&value));
        Self {
            pointer: NonNull::new(std::ptr::without_provenance_mut(
                ((value as usize) << 3) | 1,
            ))
            .unwrap(),
            ownership: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn heap(value: T) -> Self {
        Self::from_arc(Arc::new(Payload(value)))
    }

    fn from_arc(arc: Arc<Payload<T>>) -> Self {
        let pointer = NonNull::new(Arc::into_raw(arc).cast_mut()).unwrap();
        debug_assert_eq!(pointer.addr().get() & 7, 0);
        Self {
            pointer,
            ownership: PhantomData,
        }
    }

    #[inline]
    fn is_small(&self) -> bool {
        self.pointer.addr().get() & 1 != 0
    }

    #[inline]
    pub(crate) fn view(&self) -> View<'_, T> {
        if self.is_small() {
            View::Small((self.pointer.addr().get() as i64) >> 3)
        } else {
            // SAFETY: this owner holds one live strong count; this is Arc's
            // unchanged data pointer. The returned reference cannot outlive self.
            View::Heap(&unsafe { &*self.pointer.as_ptr() }.0)
        }
    }

    /// Consume one count, returning the body only if this was the last owner.
    /// Arc::into_inner also works with Weak counts; no payload borrow escapes.
    pub(crate) fn take(self) -> Option<T> {
        if self.is_small() {
            return None;
        }
        let this = ManuallyDrop::new(self);
        // SAFETY: transfer exactly this owner's count; suppress its Drop.
        let arc = unsafe { Arc::from_raw(this.pointer.as_ptr()) };
        Arc::into_inner(arc).map(|body| body.0)
    }

    pub(crate) fn into_owned(self) -> Owned<T>
    where
        T: Clone,
    {
        if let View::Small(value) = self.view() {
            return Owned::Small(value);
        }
        let this = ManuallyDrop::new(self);
        // SAFETY: transfer exactly this owner's count; suppress its Drop.
        let arc = unsafe { Arc::from_raw(this.pointer.as_ptr()) };
        Owned::Heap(match Arc::try_unwrap(arc) {
            Ok(body) => body.0,
            Err(shared) => shared.0.clone(),
        })
    }
}

impl<T> Clone for Packed<T> {
    #[inline]
    fn clone(&self) -> Self {
        if !self.is_small() {
            // SAFETY: &self keeps the allocation alive while acquiring one count.
            // After increment no fallible/user operation occurs before ownership.
            unsafe { Arc::increment_strong_count(self.pointer.as_ptr()) };
        }
        Self {
            pointer: self.pointer,
            ownership: PhantomData,
        }
    }
}

impl<T> Drop for Packed<T> {
    #[inline]
    fn drop(&mut self) {
        if !self.is_small() {
            // SAFETY: release exactly this owner's live count, using Arc's exact
            // original data pointer. Arc handles concurrent counts and destruction.
            drop(unsafe { Arc::from_raw(self.pointer.as_ptr()) });
        }
    }
}

#[derive(Clone)]
pub(crate) enum Safe<T> {
    Small(i64),
    Heap(Arc<Payload<T>>),
}

impl<T> Safe<T> {
    #[inline]
    pub(crate) fn small(value: i64) -> Self {
        assert!((MIN..=MAX).contains(&value));
        Self::Small(value)
    }

    #[inline]
    pub(crate) fn heap(value: T) -> Self {
        Self::Heap(Arc::new(Payload(value)))
    }

    #[inline]
    pub(crate) fn view(&self) -> View<'_, T> {
        match self {
            Self::Small(value) => View::Small(*value),
            Self::Heap(body) => View::Heap(&body.0),
        }
    }

    pub(crate) fn take(self) -> Option<T> {
        match self {
            Self::Small(_) => None,
            Self::Heap(arc) => Arc::into_inner(arc).map(|body| body.0),
        }
    }

    pub(crate) fn into_owned(self) -> Owned<T>
    where
        T: Clone,
    {
        match self {
            Self::Small(value) => Owned::Small(value),
            Self::Heap(arc) => Owned::Heap(match Arc::try_unwrap(arc) {
                Ok(body) => body.0,
                Err(shared) => shared.0.clone(),
            }),
        }
    }
}
