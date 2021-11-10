/// spinlock based global allocator
pub mod spin;
use core::alloc::GlobalAlloc;
use core::ptr::NonNull;

pub use self::spin::SpinPalloc;

#[cfg(feature = "allocator_api")]
use core::alloc::Allocator;

#[cfg(feature = "allocator_api")]
pub trait GlobalPallocConstraint = GlobalAlloc + Allocator;
#[cfg(not(feature = "allocator_api"))]
pub trait GlobalPallocConstraint = GlobalAlloc;

/// Defines what an allocator implementing GlobalAlloc
/// and Allocator for Palloc should look like.
/// Struct implementing this are guaranteed to implement GlobalAlloc
/// and Allocator (not if allocato_api is disabled).
///
/// Allocator implementations may implement an empty const method
/// for static initialization.
///
/// All safety concerns that apply to [`Palloc`](crate::Palloc)
/// apply to here too.
pub trait GlobalPalloc: GlobalPallocConstraint + Sized {
    /// Creates an empty uninitialized instance of the allocator
    fn new() -> Self;

    /// Initialies the allocator from the base pointer and size
    /// components
    ///
    /// ### Safety
    /// Check out [`Palloc.init`](crate::Palloc::init)
    /// for more informations.
    unsafe fn init(&mut self, bottom: NonNull<u8>, size: usize);

    /// Initializes the allocator from a slice. Panics if pointing to
    /// a null pointer.
    ///
    /// ### Safety
    /// Check out [`Palloc.init_from_slice`](crate::Palloc::init_from_slice)
    /// for more informations.
    unsafe fn init_from_slice(&mut self, heap: &mut [u8]);

    /// Creates a [`new`](#tymethod.new) allocator and calls [`init`]
    ///
    /// ### Safety
    /// Refer to [`init`]
    ///
    /// [`init`]: #tymethod.init
    unsafe fn new_initialized(bottom: NonNull<u8>, size: usize) -> Self {
        let mut allocator = Self::new();
        allocator.init(bottom, size);

        allocator
    }

    /// Creates a [`new`](#tymethod.new) allocator and calls [`init_from_slice`]
    ///
    /// ### Safety
    /// Refer to [`init_from_slice`]
    ///
    /// [`init_from_slice`]: #tymethod.init_from_slice
    unsafe fn new_from_slice(heap: &mut [u8]) -> Self {
        let mut allocator = Self::new();
        allocator.init_from_slice(heap);

        allocator
    }
}

/// unsafecell based global allocator for
/// generic mmu-less single core systems
pub mod unsafecell;
pub use self::unsafecell::UnsafeCellPalloc;
