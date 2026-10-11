//! Off-by-default requested-byte accounting, outside the rum kernel. This is
//! neither RSS nor allocator usable-size accounting. No timing uses this build.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering::SeqCst};
pub struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);
static PEAK: AtomicIsize = AtomicIsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);
fn add(bytes: isize) {
    let current = LIVE.fetch_add(bytes, SeqCst) + bytes;
    PEAK.fetch_max(current, SeqCst);
}
// SAFETY: all requests are delegated unchanged to System; returned pointers,
// layouts and allocator failure behavior follow the GlobalAlloc contract.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarding the caller's allocator request to System.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            CALLS.fetch_add(1, SeqCst);
            add(layout.size() as isize);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: pointer/layout are the caller's original live allocation.
        unsafe { System.dealloc(pointer, layout) };
        add(-(layout.size() as isize));
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: forward a valid live allocation and requested replacement size.
        let replacement = unsafe { System.realloc(pointer, layout, size) };
        if !replacement.is_null() {
            CALLS.fetch_add(1, SeqCst);
            add(size as isize - layout.size() as isize);
        }
        replacement
    }
}
#[derive(Clone, Copy)]
pub struct Snapshot {
    pub live: isize,
    pub peak: isize,
    pub calls: usize,
}
pub fn start() -> Snapshot {
    PEAK.store(LIVE.load(SeqCst), SeqCst);
    snapshot()
}
pub fn snapshot() -> Snapshot {
    Snapshot {
        live: LIVE.load(SeqCst),
        peak: PEAK.load(SeqCst),
        calls: CALLS.load(SeqCst),
    }
}
