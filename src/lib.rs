#![no_std]
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(
    feature = "allocator_api",
    feature(allocator_api, nonnull_slice_from_raw_parts)
)]
#![feature(trait_alias)]

//! Portable allocator designed for baremetal systems
//!
//! This crate provides a linked-list allocator for baremetal systems.
//! This was originally intended to work on the 32-bit raspberry Pi
//! baremetal project, also available on my github.
//!
//! This allocator is not speed-oriented, while still being relatively efficent.
//! Allocations have a 2*usize overhead
//!
//! A practical example of how to use this crate as a global allocator is available
//! in the [README.md](https://github.com/BRA1L0R/palloc) file

/// allocator module
pub mod palloc;
pub use crate::palloc::{Palloc, PallocError};

/// GlobalAlloc implementations
pub mod global;
pub use crate::global::*;

#[cfg(test)]
mod test;
