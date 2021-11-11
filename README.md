# Palloc

<a href="https://docs.rs/palloc"><img alt="docs.rs" src="https://img.shields.io/docsrs/palloc"></a>
<a href="https://crates.io/crates/palloc"><img alt="crates.io" src="https://img.shields.io/crates/v/palloc"></a>

Portable linked-list allocator for embedded / baremetal systems.

### Using the crate

Include this in the `[dependencies]` section of `Cargo.toml`

```
spin = "0.1.0"
```

This crate uses **unstable features** of Rust, so it requires the `nightly` update channel. Here's how you can
update your Rust toolchain just for the project where you intend to use this:

```
rustup override set nightly
```

### Crate features

- `spin` (default): provides a GlobalAllocator implementation using a spin-lock provided by the [spin](https://crates.io/crate/spin) crate.
- `allocator_api` (default): enables implements the Allocator trait on all global allocators.

### Example

```rust
#![no_std]

use core::ptr::NonNull;
use palloc::{GlobalPalloc, SpinPalloc};

#[global_allocator]
// the allocator is initialized using a const empty function, but it is
// not ready yet, we must initialize it first in main.
static mut ALLOCATOR: SpinPalloc = SpinPalloc::empty();

fn main() {
    // First of all we must define the bounds of our heap. Check
    // Palloc or GlobalPalloc documentation for informations.

    // Heap starting address
    let heap_start = 0x8000 as *mut u8;
    // Heap size
    let heap_size = 0xF000;

    // accessing statics is an unsafe operation
    // so it must be sorrounded by an unsafe block
    unsafe { ALLOCATOR.init(NonNull::new(heap_start).unwrap(), heap_size) };
}
```

### Documentation

Everything you need to know in order to use this crate is already written in the rustdocs.
Click on the badge under this readme's title or [click here](https://docs.rs/palloc) to read the full
documentation.
