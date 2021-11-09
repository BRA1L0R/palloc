/// spinlock based global allocator
pub mod spin;
pub use self::spin::SpinPalloc;

/// unsafecell based global allocator for
/// generic mmu-less single core systems
pub mod unsafecell;
pub use self::unsafecell::UnsafeCellPalloc;
