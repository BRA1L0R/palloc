use core::{fmt::Display, mem::size_of};

pub type BlockRef = &'static mut MemoryBlock;

#[derive(Debug)]
pub enum PallocError {
    NoBlockSpace,
    AlreadyAllocated,
    NotAllocated,
    SegmentingTail,
}

#[derive(Default)]
#[repr(C)]
pub struct MemoryBlock {
    allocation: usize,
    next: Option<BlockRef>,
}

impl MemoryBlock {
    pub unsafe fn from_ptr(ptr: *mut MemoryBlock) -> Option<BlockRef> {
        ptr.as_mut()
    }

    pub unsafe fn from_ptr_unchecked(ptr: *mut MemoryBlock) -> BlockRef {
        &mut *ptr
    }

    pub unsafe fn default_from_ptr(ptr: *mut MemoryBlock) -> Option<BlockRef> {
        Self::from_ptr(ptr).map(|b| {
            *b = MemoryBlock::default();
            b
        })
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

    pub unsafe fn insert_default(&mut self, address: *mut MemoryBlock) {
        let inserted = Self::default_from_ptr(address).unwrap();
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

    pub fn segment(&mut self) -> Result<(), PallocError> {
        let maxsize = self.max_size().ok_or(PallocError::SegmentingTail)?;
        let allocated = match self.allocation {
            0 => Err(PallocError::NotAllocated),
            n => Ok(n),
        }?;

        if (allocated + size_of::<Self>()) < maxsize {
            let newblock = (self.heap() as usize + allocated) as *mut MemoryBlock;
            unsafe { self.insert_default(newblock) };
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

    pub fn link_default(&mut self) {
        if self.allocation == 0 {
            panic!("cannot without being allocated")
        }

        self.next = unsafe {
            Self::default_from_ptr((self.heap() as usize + self.allocation) as *mut MemoryBlock)
        };
    }

    pub fn heap(&self) -> *mut u8 {
        let self_addr = self as *const MemoryBlock as usize;
        (self_addr + size_of::<Self>()) as _
    }

    pub fn top(&'static mut self) -> *mut MemoryBlock {
        let selfptr = self as *mut MemoryBlock;
        self.next.as_mut().map_or(selfptr, |block| block.top())
    }

    pub unsafe fn from_heap_ptr(heap: *mut u8) -> Option<BlockRef> {
        ((heap as usize - size_of::<Self>()) as *mut MemoryBlock).as_mut()
    }

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

    pub fn current_mut(&mut self) -> Option<BlockRef> {
        unsafe { self.current.map(|ptr| &mut *ptr) }
    }
}

impl Iterator for BlockIterator {
    type Item = BlockRef;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|ptr| unsafe { &mut *ptr }).map(|current| {
            self.current = current
                .next
                .as_mut()
                .map(|block| *block as *mut MemoryBlock);
            current
        })
    }
}
