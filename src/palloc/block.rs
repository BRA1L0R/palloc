use crate::PallocError;
use core::{mem::size_of, ptr::NonNull};

pub type BlockRef = &'static mut MemoryBlock;

#[derive(Default)]
#[repr(C)]
pub struct MemoryBlock {
    allocation: usize,
    next: Option<BlockRef>,
}

impl MemoryBlock {
    /// # Safety
    pub unsafe fn from_ptr(mut ptr: NonNull<MemoryBlock>) -> BlockRef {
        ptr.as_mut()
    }

    pub unsafe fn from_ptr_unchecked(ptr: *mut MemoryBlock) -> BlockRef {
        &mut *ptr
    }

    /// # Safety
    pub unsafe fn default_from_ptr(ptr: NonNull<MemoryBlock>) -> BlockRef {
        let block = Self::from_ptr(ptr);
        *block = MemoryBlock::default();

        block
    }

    pub fn allocate(&mut self, size: usize) -> Result<*mut u8, PallocError> {
        match (self.allocation, self.max_size()) {
            (0, Some(maxsize)) if maxsize < size => Err(PallocError::NoBlockSpace),
            (0, _) => {
                self.allocation = size;
                Ok(self.heap())
            }
            (_, _) => Err(PallocError::AlreadyAllocated),
        }
    }

    /// # Safety
    pub unsafe fn insert_default(&mut self, address: NonNull<MemoryBlock>) {
        let inserted = Self::default_from_ptr(address);
        inserted.next = self.next.take();
        self.next = Some(inserted);
    }

    pub fn merge(&mut self, target_size: usize) -> Result<(), PallocError> {
        while let Some(maxsize) = self.max_size() {
            if maxsize >= target_size {
                break;
            }

            let next = self.next.as_mut().unwrap().next.take();
            match self.next.as_ref().unwrap().allocation {
                0 => self.next = next,
                _ => return Err(PallocError::NoBlockSpace),
            }
        }

        Ok(())
    }

    pub unsafe fn segment(&mut self) -> Result<(), PallocError> {
        let maxsize = self.max_size().ok_or(PallocError::SegmentingTail)?;
        let allocated = match self.allocation {
            0 => Err(PallocError::NotAllocated),
            n => Ok(n),
        }?;

        if (allocated + size_of::<Self>()) < maxsize {
            let newblock = NonNull::new_unchecked((self.heap() as usize + allocated) as *mut _);
            self.insert_default(newblock);
        }

        Ok(())
    }

    pub fn dealloc(&mut self) -> Result<(), PallocError> {
        match self.allocation {
            0 => Err(PallocError::NotAllocated),
            _ => {
                self.allocation = 0;
                Ok(())
            }
        }
    }

    pub fn max_size(&self) -> Option<usize> {
        self.next
            .as_ref()
            .map(|next| (*next as *const MemoryBlock as usize) - self.heap() as usize)
    }

    /// # Safety
    pub unsafe fn link_default(&mut self) {
        if self.allocation == 0 {
            panic!("cannot without being allocated")
        }

        let new_link = Self::default_from_ptr(NonNull::new_unchecked(
            (self.heap() as usize + self.allocation) as *mut _,
        ));

        self.next = Some(new_link);
    }

    pub fn heap(&self) -> *mut u8 {
        let self_addr = self as *const MemoryBlock as usize;
        (self_addr + size_of::<Self>()) as _
    }

    pub fn top(&'static mut self) -> *mut MemoryBlock {
        let selfptr = self as *mut MemoryBlock;
        self.next.as_mut().map_or(selfptr, |block| block.top())
    }

    /// # Safety
    pub unsafe fn from_heap_ptr(heap: *mut u8) -> Option<BlockRef> {
        ((heap as usize - size_of::<Self>()) as *mut MemoryBlock).as_mut()
    }

    /// # Safety
    pub unsafe fn from_heap_ptr_unchecked(heap: *mut u8) -> BlockRef {
        &mut *((heap as usize - size_of::<Self>()) as *mut MemoryBlock)
    }

    pub fn iter_mut(&'static mut self) -> BlockIterator {
        BlockIterator::new(self)
    }

    #[inline]
    pub fn is_allocated(&self) -> bool {
        self.allocation != 0
    }

    #[inline]
    pub fn is_linked(&self) -> bool {
        self.next.is_some()
    }
}

pub struct BlockIterator {
    current: Option<*mut MemoryBlock>,
}

impl BlockIterator {
    pub fn new(start: *mut MemoryBlock) -> Self {
        Self {
            current: Some(start),
        }
    }

    pub unsafe fn current_mut(&mut self) -> Option<BlockRef> {
        self.current.map(|ptr| &mut *ptr)
    }
}

impl Iterator for BlockIterator {
    type Item = BlockRef;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { self.current_mut() }.map(|current| {
            self.current = current
                .next
                .as_mut()
                .map(|block| *block as *mut MemoryBlock);
            current
        })
    }
}
