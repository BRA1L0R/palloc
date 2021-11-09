use core::{
    alloc::GlobalAlloc,
    ptr::{null_mut, NonNull},
};

use spin::Mutex;

use crate::Palloc;

pub struct SpinPalloc {
    allocator: Mutex<Palloc>,
}

impl SpinPalloc {
    pub const fn empty() -> SpinPalloc {
        let allocator = Mutex::new(Palloc::empty());
        SpinPalloc { allocator }
    }

    pub unsafe fn init(&self, bottom: NonNull<u8>, size: usize) {
        self.allocator.lock().init(bottom, size);
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
        self.allocator
            .lock()
            .free(NonNull::new(ptr).unwrap())
            .unwrap();
    }
}
