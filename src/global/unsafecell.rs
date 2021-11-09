use crate::Palloc;
use core::{
    alloc::GlobalAlloc,
    cell::UnsafeCell,
    ptr::{null_mut, NonNull},
};

/// GlobalAlloc implementation using an unsafe cell.
/// 
/// This GlobalAlloc implementation is NOT inteded for
/// multi-threaded concurrent applications: it makes use
/// of an unsafe cell which allows immutable references
/// to be transmuted to mutable references. It exists because
/// not all systems support spin-locking, like the raspberry
/// pi 1 when not using the MMU.
/// 
/// For Safety and usage concerns, refer to [`Palloc`](../../palloc/struct.Palloc.html) or
/// the crate root documentation
pub struct UnsafeCellPalloc {
    allocator: UnsafeCell<Palloc>,
}

impl UnsafeCellPalloc {
    /// See [`empty`](../../palloc/struct.Palloc.html#method.empty)
    pub const fn empty() -> UnsafeCellPalloc {
        UnsafeCellPalloc {
            allocator: UnsafeCell::new(Palloc::empty()),
        }
    }

    /// See [`init`](../../palloc/struct.Palloc.html#method.init)
    /// ### Safety
    pub unsafe fn init(&mut self, bottom: NonNull<u8>, size: usize) {
        self.allocator.get_mut().init(bottom, size);
    }

    /// See [`init_from_slice`](../../palloc/struct.Palloc.html#method.init_from_slice)
    /// ### Safety
    pub unsafe fn init_from_slice(&mut self, heap: &mut [u8]) {
        self.allocator.get_mut().init_from_slice(heap)
    }
}

unsafe impl GlobalAlloc for UnsafeCellPalloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        (*self.allocator.get())
            .alloc(layout.size())
            .unwrap_or(null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        (*self.allocator.get()).free(ptr).unwrap();
    }
}
