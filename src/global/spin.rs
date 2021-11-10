use crate::Palloc;
use core::{
    alloc::{AllocError, GlobalAlloc},
    ptr::{null_mut, NonNull},
};
use spin::{mutex::Mutex, relax::Loop};

#[cfg(feature = "allocator_api")]
use core::alloc::Allocator;

/// Spinlock based GlobalsAlloc implementation for Palloc.
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
    /// Creates an empty const SpinPalloc uninitialized instance.
    ///
    /// See [`empty`](crate::Palloc::empty)
    pub const fn empty() -> SpinPalloc {
        let allocator = Mutex::new(Palloc::empty());
        SpinPalloc { allocator }
    }
}

impl super::GlobalPalloc for SpinPalloc {
    fn new() -> Self {
        Self::empty()
    }

    unsafe fn init(&mut self, bottom: NonNull<u8>, size: usize) {
        self.allocator
            .try_lock()
            .expect("initialization should never be blocked by a mutex")
            .init(bottom, size);
    }

    unsafe fn init_from_slice(&mut self, heap: &mut [u8]) {
        self.allocator.lock().init_from_slice(heap)
    }
}

unsafe impl GlobalAlloc for SpinPalloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator
            .lock()
            .alloc(layout.size())
            .map(NonNull::as_ptr)
            .unwrap_or(null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        self.allocator
            .lock()
            .free(NonNull::new(ptr).expect("pointer for deallocation cannot be null"))
            .unwrap();
    }
}

#[cfg(feature = "allocator_api")]
unsafe impl Allocator for SpinPalloc {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        match unsafe { self.allocator.lock().alloc(layout.size()) } {
            Err(_) => Err(AllocError),
            Ok(ptr) => Ok(NonNull::slice_from_raw_parts(ptr, layout.size())),
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: core::alloc::Layout) {
        self.allocator.lock().free(ptr).unwrap();
    }
}
