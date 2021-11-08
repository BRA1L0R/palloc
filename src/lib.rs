#![no_std]

mod block;

use block::{BlockRef, MemoryBlock, PallocError};
use core::ptr::null_mut;

struct Palloc {
    bottom: *mut MemoryBlock,
    size: usize,
}

impl Palloc {
    pub fn empty() -> Palloc {
        Palloc {
            bottom: null_mut(),
            size: 0,
        }
    }

    fn get_origin(&self) -> BlockRef {
        unsafe { MemoryBlock::from_ptr_unchecked(self.bottom) }
    }

    pub fn init(&mut self, bottom: *mut u8, size: usize) {
        self.bottom = bottom as *mut MemoryBlock;
        self.size = size;

        unsafe { MemoryBlock::default_from_ptr(self.bottom) }.expect("bottom should not be null");
    }

    pub fn init_from_slice(&mut self, heap: &mut [u8]) {
        let (bottom, size) = (heap.as_mut_ptr(), heap.len());
        self.init(bottom, size);
    }

    pub fn alloc(&mut self, size: usize) -> Result<*mut u8, PallocError> {
        let origin = self.get_origin();
        let list = origin.iter_mut();

        for block in list.filter(|block| !block.is_allocated()) {
            match block.max_size() {
                Some(max) if max < size => {
                    if block.merge(size).is_err() {
                        continue;
                    }
                }
                _ => (),
            }

            let allocation = block.allocate(size);

            if !block.is_linked() {
                block.link_default();
            } else {
                block.segment()?;
            }

            return allocation;
        }

        panic!("a valid candidate must be found before the loop ends")
    }

    pub fn free(&self, alloc: *mut u8) -> Result<(), PallocError> {
        let block = unsafe { MemoryBlock::from_heap_ptr_unchecked(alloc) };
        block.dealloc()
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use crate::{block::PallocError, Palloc};
    use core::ptr::slice_from_raw_parts_mut;
    use std::{boxed::Box, sync::Once, vec, vec::Vec};

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

        let ptr = palloc.alloc(30)?;
        assert!(memtest_allocation(ptr, 30), "should pass memtest");

        Ok(())
    }

    #[test]
    fn test_realloc() -> Result<(), PallocError> {
        init();
        let mut palloc = empty_allocator();

        let allocation = palloc.alloc(50)?;
        palloc.free(allocation)?;

        let new_allocation = palloc.alloc(20)?;
        assert_eq!(allocation, new_allocation);

        Ok(())
    }

    #[test]
    fn test_merge() -> Result<(), PallocError> {
        init();
        let mut palloc = empty_allocator();

        let first = palloc.alloc(20)?;
        let second = palloc.alloc(20)?;

        palloc.free(first)?;
        palloc.free(second)?;

        let realloc = palloc.alloc(40)?;

        assert_eq!(first, realloc);
        Ok(())
    }

    #[test]
    fn test_segment() -> Result<(), PallocError> {
        init();
        let mut palloc = empty_allocator();

        let alloc = palloc.alloc(50)?;
        palloc.free(alloc)?;

        let new_alloc = palloc.alloc(5)?;

        assert!((new_alloc as usize) < alloc as usize + 50);
        Ok(())
    }
}
