use core::alloc::{GlobalAlloc, Layout};

use crate::SpinPalloc;

unsafe fn empty_allocator() -> SpinPalloc {
    let spalloc = SpinPalloc::empty();
    spalloc.init_from_slice(super::empty_heap());

    spalloc
}

#[test]
fn allocation_deallocation() {
    let palloc = unsafe { empty_allocator() };

    let layout = Layout::from_size_align(20, 1).unwrap();
    let allocation = unsafe { palloc.alloc(layout) };
    assert!(!allocation.is_null());

    unsafe { palloc.dealloc(allocation, layout) }
}
