extern crate std;

use crate::{block::PallocError, Palloc};
use core::ptr::slice_from_raw_parts_mut;
use std::{sync::Once, vec, vec::Vec};

static INIT: Once = Once::new();
static mut _FAKE_HEAP: Option<Vec<u8>> = None;

fn init() {
    INIT.call_once(|| unsafe { _FAKE_HEAP = Some(vec![0u8; 150]) });
}

unsafe fn empty_heap() -> &'static mut [u8] {
    let heap = _FAKE_HEAP.as_mut().unwrap();
    heap.iter_mut().for_each(|v| *v = 0);

    heap
}

fn empty_allocator() -> Palloc {
    let mut palloc = Palloc::empty();
    palloc.init_from_slice(unsafe { empty_heap() });

    palloc
}

fn memtest_allocation(start: *mut u8, size: usize) -> bool {
    let memory = unsafe { &mut *slice_from_raw_parts_mut(start, size) };
    !memory
        .iter_mut()
        .enumerate()
        .map(|(index, addr)| (index as u8, addr))
        .any(|(index, addr)| {
            *addr = index;
            *addr != index
        })
}

#[test]
fn test_single_alloc() -> Result<(), PallocError> {
    init();
    let mut palloc = empty_allocator();

    let ptr = unsafe { palloc.alloc(30)? };
    assert!(memtest_allocation(ptr, 30), "should pass memtest");

    Ok(())
}

#[test]
fn test_realloc() -> Result<(), PallocError> {
    init();
    let mut palloc = empty_allocator();

    let allocation = unsafe { palloc.alloc(50)? };
    unsafe { palloc.free(allocation)? };

    let new_allocation = unsafe { palloc.alloc(20)? };
    assert_eq!(allocation, new_allocation);

    Ok(())
}

#[test]
fn test_merge() -> Result<(), PallocError> {
    init();
    let mut palloc = empty_allocator();

    let first = unsafe { palloc.alloc(20)? };
    let second = unsafe { palloc.alloc(20)? };

    unsafe {
        palloc.free(first)?;
        palloc.free(second)?;
    }

    let realloc = unsafe { palloc.alloc(40)? };

    assert_eq!(first, realloc);
    Ok(())
}

#[test]
fn test_segment() -> Result<(), PallocError> {
    init();
    let mut palloc = empty_allocator();

    let alloc = unsafe { palloc.alloc(50)? };
    unsafe { palloc.free(alloc)? };

    let new_alloc = unsafe { palloc.alloc(5)? };

    assert!((new_alloc as usize) < alloc as usize + 50);
    Ok(())
}

#[test]
fn test_oom() {
    init();
    let mut palloc = empty_allocator();

    assert_eq!(
        unsafe { palloc.alloc(135) }.unwrap_err(),
        PallocError::OutOfMemory
    );
}
