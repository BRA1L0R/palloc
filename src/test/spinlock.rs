use core::alloc::{GlobalAlloc, Layout};

use crate::SpinPalloc;

fn empty_allocator(heap: &mut [u8]) -> SpinPalloc {
    let spalloc = SpinPalloc::empty();
    unsafe { spalloc.init_from_slice(heap) };

    spalloc
}

#[test]
fn allocation_deallocation() {
    let mut heap = [0; 50];
    let palloc = empty_allocator(&mut heap);

    let layout = Layout::from_size_align(20, 1).unwrap();
    let allocation = unsafe { palloc.alloc(layout) };
    assert!(!allocation.is_null());

    unsafe { palloc.dealloc(allocation, layout) }
}
