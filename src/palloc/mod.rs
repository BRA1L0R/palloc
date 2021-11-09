mod block;

use block::{BlockRef, MemoryBlock};
use core::ptr::{null_mut, NonNull};

/// defines an error returned from either an allocation
/// or a deallocation
#[derive(Debug, PartialEq)]
pub enum PallocError {
    /// no space is left between the block heap start
    /// the next link of the chain.
    NoBlockSpace,
    /// the block has already been allocated
    /// to something else.
    AlreadyAllocated,
    /// no allocation has been made on this block yet
    /// so the operation cannot proceed.
    NotAllocated,
    /// cannot segment the last block (tail) of the linked
    /// list as it does not have a defined heap size.
    SegmentingTail,
    /// no more blocks can be allocated without going
    /// outside of memory bounds.
    OutOfMemory,
}

/// defines a both uninitialized and initialized allocator.
///
/// An ['empty'](#method.empty) instance may be created for static purposes,
/// but in order to allocate memory [initialization](#method.init) must occur.
pub struct Palloc {
    bottom: *mut MemoryBlock,
    size: usize,
}

impl Palloc {
    /// creates an empty allocator, pointing to 0-sized null memory.
    ///
    /// to make the allocator working, check out [`init`](#method.init)
    pub const fn empty() -> Palloc {
        Palloc {
            bottom: null_mut(),
            size: 0,
        }
    }

    unsafe fn get_origin(&self) -> BlockRef {
        NonNull::new_unchecked(self.bottom).as_mut()
    }

    /// Initializes the allocator with a pointer to a free heap region
    /// and a size which defines the upper bound of the same.
    ///
    /// ### Safety
    /// Memory is not asserted to be zeroed. However the whole region must
    /// be accessible and free to use.
    ///
    /// Initializing using a null pointer will result in a panic.
    pub unsafe fn init(&mut self, bottom: NonNull<u8>, size: usize) {
        let bottom = bottom.cast();

        self.bottom = bottom.as_ptr();
        self.size = size;

        MemoryBlock::default_from_ptr(bottom);
    }

    /// Initializes heap from a memory slice. See [`init`](#method.init) for more informations.
    ///
    /// ### Safety
    /// See [`init`](#method.init)
    pub unsafe fn init_from_slice(&mut self, heap: &mut [u8]) {
        let (bottom, size) = (heap.as_mut_ptr(), heap.len());
        let bottom = NonNull::new(bottom).expect("non nullpointed slice");

        self.init(bottom.cast(), size);
    }

    /// Creates a new allocation of `size` bytes. When Ok, returns a pointer
    /// to a free uninitialized (not to be assumed zero) memory region.
    /// May result in one of the errors defined in
    /// [`PallocError`](enum.PallocError.html).
    ///
    /// Alloc will potentially traverse the entire heap in search of a free segment.
    /// It will also merge all freed adjacent blocks. Once a free block is found, if
    /// it does not fill the entire segment (in case of reallocation) a chunk will be split
    /// and the rest of the memory made available for further allocations.
    ///
    /// This whole process, while not ensuring super fast allocation all of the time, it
    /// assures that every piece of memory is being used as much as possible.
    ///
    /// ### Safety
    /// Null pointer is never returned, in case of OOM a PallocError is returned
    /// instead. As stated before, memory is never to be assumed initialized.
    pub unsafe fn alloc(&mut self, size: usize) -> Result<*mut u8, PallocError> {
        let origin = self.get_origin(); // base memory block starting from bottom
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

    /// Deallocates memory at a given pointer location, giving it back to
    /// the allocator for further allocational purposes.
    ///
    /// Once deallocated, memory cannot be used anymore and
    /// its integrity is not assured.
    ///
    /// ### Safety
    /// `alloc` must point to the bottom of a valid allocation. Not being aligned to
    /// one will lead to **undefined behaviour**, potentially destructive.
    pub unsafe fn free(&self, alloc: NonNull<u8>) -> Result<(), PallocError> {
        let block = MemoryBlock::from_heap_ptr(alloc).unwrap(); // should never result into a null pointer
        if block.is_allocated() {
            block.dealloc()
        } else {
            Err(PallocError::NotAllocated)
        }
    }
}
