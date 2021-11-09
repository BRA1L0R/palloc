use crate::Palloc;
use core::{
    alloc::GlobalAlloc,
    ptr::{null_mut, NonNull},
};
use spin::Mutex;

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
    allocator: Mutex<Palloc>,
}

impl SpinPalloc {
    /// Creates an empty const SpinPalloc
    pub const fn empty() -> SpinPalloc {
        let allocator = Mutex::new(Palloc::empty());
        SpinPalloc { allocator }
    }

    /// Initializes the underlying Palloc with a non-null bottom
    /// and a given size.
    ///
    /// ### Safety
    /// Bottom must never be zero or else it will lead to **undefined behaviour**.
    /// The whole memory space (bottom <-> bottom+size) must be free for use.
    pub unsafe fn init(&self, bottom: NonNull<u8>, size: usize) {
        self.allocator.lock().init(bottom, size);
    }

    /// Initializes the underlying Palloc from a heap slice.
    ///
    /// ### Safety
    /// Panics if heap start is null. See [`init`](#method.init)
    /// for general safety informations.
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
