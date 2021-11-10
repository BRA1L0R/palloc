use crate::Palloc;
use core::{
    alloc::GlobalAlloc,
    cell::UnsafeCell,
    ptr::{null_mut, NonNull},
};

#[cfg(feature = "allocator_api")]
use core::alloc::Allocator;

/// GlobalAlloc implementation using an unsafe cell.
///
/// This GlobalAlloc implementation is NOT inteded for
/// multi-threaded concurrent applications: it makes use
/// of an unsafe cell which allows immutable references
/// to be transmuted to mutable references. It exists because
/// not all systems support spin-locking, like the raspberry
/// pi 1 when not using the MMU.
///
/// For Safety and usage concerns, refer to [`Palloc`](crate::Palloc) or
/// the crate root documentation
pub struct UnsafeCellPalloc {
    allocator: UnsafeCell<Palloc>,
}

impl UnsafeCellPalloc {
    /// See [`empty`](crate::Palloc::empty)
    pub const fn empty() -> UnsafeCellPalloc {
        UnsafeCellPalloc {
            allocator: UnsafeCell::new(Palloc::empty()),
        }
    }
}

impl super::GlobalPalloc for UnsafeCellPalloc {
    fn new() -> UnsafeCellPalloc {
        Self::empty()
    }

    unsafe fn init(&mut self, bottom: NonNull<u8>, size: usize) {
        self.allocator.get_mut().init(bottom, size);
    }

    unsafe fn init_from_slice(&mut self, heap: &mut [u8]) {
        self.allocator.get_mut().init_from_slice(heap)
    }
}

unsafe impl GlobalAlloc for UnsafeCellPalloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        (*self.allocator.get())
            .alloc(layout.size())
            .map(|ptr| ptr.as_ptr())
            .unwrap_or(null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        (*self.allocator.get())
            .free(NonNull::new(ptr).unwrap())
            .unwrap();
    }
}

#[cfg(feature = "allocator_api")]
unsafe impl Allocator for UnsafeCellPalloc {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        unsafe { (*self.allocator.get()).alloc(layout.size()) }
            .map(|ptr| NonNull::slice_from_raw_parts(ptr, layout.size()))
            .or(Err(core::alloc::AllocError))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: core::alloc::Layout) {
        (*self.allocator.get()).free(ptr).unwrap();
    }
}
