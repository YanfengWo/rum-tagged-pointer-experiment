//! Acquire, consume and release exactly the count(s) selected by each tag.
use super::*;
use std::mem::ManuallyDrop;
impl Typed16 {
    fn into_call(self) -> (Head<Self>, Args<Self>) {
        debug_assert_eq!(self.tag(), CALL);
        let this = ManuallyDrop::new(self);
        // SAFETY: suppress this owner and transfer its two exact strong counts.
        unsafe {
            (
                Head(Arc::from_raw(this.data::<Self>())),
                Args(Arc::from_raw(this.args_ptr())),
            )
        }
    }
    pub fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom> {
        if self.tag() == CALL {
            return Ok(self.into_call());
        }
        let this = ManuallyDrop::new(self);
        // SAFETY in heap arms: into_raw's exact typed pointer is restored,
        // including str metadata. Each recovered Arc owns this node's sole
        // count. Locals release it on return/unwind; the old owner is suppressed.
        Err(match this.tag() {
            INTEGER => Atom::Integer(IntValue::Small(this.second.addr() as i64)),
            PLAIN | GENERATED => Atom::Symbol(this.symbol()),
            STRING => {
                let s = unsafe { Arc::from_raw(this.str_ptr()) };
                Atom::String(s.to_string())
            }
            BIG => {
                let n = unsafe { Arc::from_raw(this.data::<BigInt>()) };
                Atom::Integer(IntValue::Big(Box::new(
                    Arc::try_unwrap(n).unwrap_or_else(|n| n.as_ref().clone()),
                )))
            }
            RATIONAL => {
                let n = unsafe { Arc::from_raw(this.data::<Rational>()) };
                Atom::Rational(Box::new(
                    Arc::try_unwrap(n).unwrap_or_else(|n| n.as_ref().clone()),
                ))
            }
            OWNED => {
                let n = unsafe { Arc::from_raw(this.data::<Atom>()) };
                Arc::try_unwrap(n).unwrap_or_else(|n| n.as_ref().clone())
            }
            _ => unreachable!(),
        })
    }
}
impl Clone for Typed16 {
    #[inline]
    fn clone(&self) -> Self {
        // SAFETY: &self keeps the exact count live while acquiring one more.
        // Each typed pointer agrees with its constructor's allocation. Arc's
        // overflow aborts; the two-count Call cannot unwind midway through.
        unsafe {
            match self.tag() {
                OWNED => Arc::increment_strong_count(self.data::<Atom>()),
                STRING => Arc::increment_strong_count(self.str_ptr()),
                BIG => Arc::increment_strong_count(self.data::<BigInt>()),
                RATIONAL => Arc::increment_strong_count(self.data::<Rational>()),
                CALL => {
                    Arc::increment_strong_count(self.data::<Self>());
                    Arc::increment_strong_count(self.args_ptr());
                }
                INTEGER | PLAIN | GENERATED => {}
                _ => unreachable!(),
            }
        }
        Self::words(self.first, self.second)
    }
}
impl Drop for Typed16 {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: transfer exactly this owner's count(s) into typed Arc locals.
        // CALL recovers both before releasing either, ensuring unwind cleanup.
        unsafe {
            match self.tag() {
                OWNED => drop(Arc::from_raw(self.data::<Atom>())),
                STRING => drop(Arc::from_raw(self.str_ptr())),
                BIG => drop(Arc::from_raw(self.data::<BigInt>())),
                RATIONAL => drop(Arc::from_raw(self.data::<Rational>())),
                CALL => {
                    let head = Arc::from_raw(self.data::<Self>());
                    let args = Arc::from_raw(self.args_ptr());
                    drop(head);
                    drop(args);
                }
                INTEGER | PLAIN | GENERATED => {}
                _ => unreachable!(),
            }
        }
    }
}
impl Storage for Typed16 {
    fn take_call(self) -> Option<(Head<Self>, Args<Self>)> {
        (self.tag() == CALL).then(|| self.into_call())
    }
}
