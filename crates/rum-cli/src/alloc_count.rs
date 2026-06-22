use std::alloc::{GlobalAlloc, Layout};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

use mimalloc::MiMalloc;

const ALLOC_PROFILE_ENV: &str = "RUM_PROFILE_ALLOC";

static ALLOC_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);

pub struct CountingAlloc(pub MiMalloc);

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAlloc = CountingAlloc(MiMalloc);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AllocProfile {
    pub count: u64,
    pub bytes_total: u64,
    pub peak_live_bytes: u64,
}

pub fn init() {
    let enabled =
        std::env::var_os(ALLOC_PROFILE_ENV).is_some_and(|value| !value.is_empty() && value != "0");
    ALLOC_PROFILE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
}

#[inline]
pub fn enabled() -> bool {
    ALLOC_PROFILE_ENABLED.load(Ordering::Relaxed) == 1
}

pub fn snapshot() -> AllocProfile {
    AllocProfile {
        count: ALLOC_COUNT.load(Ordering::Relaxed),
        bytes_total: BYTES_TOTAL.load(Ordering::Relaxed),
        peak_live_bytes: PEAK_LIVE_BYTES.load(Ordering::Relaxed),
    }
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.0.alloc(layout) };
        if !ptr.is_null() && enabled() {
            record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if enabled() {
            record_dealloc(layout.size());
        }
        unsafe { self.0.dealloc(ptr, layout) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.0.alloc_zeroed(layout) };
        if !ptr.is_null() && enabled() {
            record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { self.0.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() && enabled() {
            record_realloc(layout.size(), new_size);
        }
        new_ptr
    }
}

#[cold]
#[inline(never)]
fn record_alloc(size: usize) {
    let size = size as u64;
    ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
    BYTES_TOTAL.fetch_add(size, Ordering::Relaxed);
    add_live(size);
}

#[cold]
#[inline(never)]
fn record_dealloc(size: usize) {
    subtract_live(size as u64);
}

#[cold]
#[inline(never)]
fn record_realloc(old_size: usize, new_size: usize) {
    let old_size = old_size as u64;
    let new_size = new_size as u64;

    if new_size > old_size {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        BYTES_TOTAL.fetch_add(new_size, Ordering::Relaxed);
        add_live(new_size - old_size);
    } else if old_size > new_size {
        subtract_live(old_size - new_size);
    }
}

fn add_live(size: u64) {
    let live = LIVE_BYTES
        .fetch_add(size, Ordering::Relaxed)
        .saturating_add(size);
    update_peak_live(live);
}

fn subtract_live(size: u64) {
    let mut current = LIVE_BYTES.load(Ordering::Relaxed);
    loop {
        let next = current.saturating_sub(size);
        match LIVE_BYTES.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => return,
            Err(observed) => current = observed,
        }
    }
}

fn update_peak_live(live: u64) {
    let mut peak = PEAK_LIVE_BYTES.load(Ordering::Relaxed);
    while live > peak {
        match PEAK_LIVE_BYTES.compare_exchange_weak(
            peak,
            live,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return,
            Err(observed) => peak = observed,
        }
    }
}
