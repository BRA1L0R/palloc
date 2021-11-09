use crate::Palloc;
use core::{
    alloc::GlobalAlloc,
    ptr::{null_mut, NonNull},
};
use spin::{mutex::Mutex, relax::Loop};

/// Spinlock based GlobalAlloc implementation for Palloc.
///
/// Palloc on its own won't be enough to be used as a global allocator,
/// as it does not implement the GlobalAlloc trait. There are a number
/// of reasons for this. Different platforms may use different types of
/// Mutex locks, and Palloc requires one.
///
/// SpinPalloc is a generic implementation using the spinlock mutex
/// technique. It is the only GlobalALloc implemented because it's
/// the most generic too.
pub struct SpinPalloc {
    allocator: Mutex<Palloc, Loop>,
}

impl SpinPalloc {
    /// Creates an empty const SpinPalloc
    ///
    /// See [`empty`](../../palloc/struct.Palloc.html#method.empty)
    pub const fn empty() -> SpinPalloc {
        let allocator = Mutex::new(Palloc::empty());
        SpinPalloc { allocator }
    }

    /// See [`init`](../../palloc/struct.Palloc.html#method.init)
    /// ### Safety
    pub unsafe fn init(&self, bottom: NonNull<u8>, size: usize) {
        self.allocator
            .try_lock()
            .expect("initialization should never be blocked by a mutex")
            .init(bottom, size);
    }

    /// See [`init_from_slice`](../../palloc/struct.Palloc.html#method.init_from_slice)
    /// ### Safety
    pub unsafe fn init_from_slice(&self, heap: &mut [u8]) {
        self.allocator.lock().init_from_slice(heap)
    }
}

unsafe impl GlobalAlloc for SpinPalloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator
            .lock()
            .alloc(layout.size())
            .unwrap_or(null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        self.allocator.lock().free(ptr).unwrap();
    }
}
