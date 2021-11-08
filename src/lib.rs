#![no_std]

#[cfg(test)]
mod test;

pub mod block;

use block::{BlockRef, MemoryBlock, PallocError};
use core::ptr::null_mut;

pub struct Palloc {
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

    /// # Alloc
    /// creates a new allocation in the heap.
    /// 
    /// # Safety
    /// 
    pub unsafe fn alloc(&mut self, size: usize) -> Result<*mut u8, PallocError> {
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

            let is_tail = !block.is_linked();
            if is_tail && block.heap() as usize + size > self.bottom as usize + self.size {
                return Err(PallocError::OutOfMemory);
            }

            let allocation = block.allocate(size)?;
            if is_tail {
                block.link_default();
            } else {
                block.segment()?;
            }

            return Ok(allocation);
        }

        panic!("a valid candidate must be found before the loop ends")
    }

    /// # Safety
    pub unsafe fn free(&self, alloc: *mut u8) -> Result<(), PallocError> {
        let block = MemoryBlock::from_heap_ptr_unchecked(alloc);
        block.dealloc()
    }
}
