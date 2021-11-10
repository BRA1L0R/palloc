extern crate std;

use crate::{Palloc, PallocError};
use core::ptr::{slice_from_raw_parts_mut, NonNull};

fn empty_allocator(heap: &mut [u8]) -> Palloc {
    let mut palloc = Palloc::empty();
    unsafe { palloc.init_from_slice(heap) };
    palloc
}

fn memtest_allocation(start: NonNull<u8>, size: usize) -> bool {
    let memory = unsafe { &mut *slice_from_raw_parts_mut(start.as_ptr(), size) };
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
    let mut heap = [0u8; 50];
    let mut palloc = empty_allocator(&mut heap);

    let ptr = unsafe { palloc.alloc(30)? };
    assert!(memtest_allocation(ptr, 30), "should pass memtest");

    Ok(())
}

#[test]
fn test_realloc() -> Result<(), PallocError> {
    let mut heap = [0u8; 100];
    let mut palloc = empty_allocator(&mut heap);

    let allocation = unsafe { palloc.alloc(50)? };
    unsafe { palloc.free(allocation)? };

    let new_allocation = unsafe { palloc.alloc(20)? };
    assert_eq!(allocation, new_allocation);

    Ok(())
}

#[test]
fn test_merge() -> Result<(), PallocError> {
    let mut heap = [0; 100];
    let mut palloc = empty_allocator(&mut heap);

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
    let mut heap = [0; 100];
    let mut palloc = empty_allocator(&mut heap);

    let alloc = unsafe { palloc.alloc(50)? };
    unsafe { palloc.free(alloc)? };

    let new_alloc = unsafe { palloc.alloc(5)? };

    assert!((new_alloc.as_ptr() as usize) < alloc.as_ptr() as usize + 50);
    Ok(())
}

#[test]
fn test_oom() {
    let mut heap = [0; 30];
    let mut palloc = empty_allocator(&mut heap);

    assert_eq!(
        unsafe { palloc.alloc(15) }.unwrap_err(),
        PallocError::OutOfMemory
    );
}
