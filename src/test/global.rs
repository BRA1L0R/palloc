extern crate std;

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    vec::Vec,
};

use crate::GlobalPalloc;

macro_rules! test_global_palloc {
    ($sec:tt, $alloc:ty, $($testfun:tt),+) => {
        mod $sec {
            $(
                #[test]
                fn $testfun() {
                    super::$testfun::<$alloc>()
                }
            )*
        }
    };
}

test_global_palloc!(
    spin,
    crate::SpinPalloc,
    test_vector_allocation,
    test_concurrence
);
test_global_palloc!(unsafecell, crate::UnsafeCellPalloc, test_vector_allocation);

fn test_vector_allocation<T: GlobalPalloc>() {
    let mut heap = [0; 36];
    let allocator = unsafe { T::new_from_slice(&mut heap) };

    let mut allocated = Vec::<u8, &T>::with_capacity_in(20, &allocator);
    (0..20).for_each(|val| allocated.push(val));
}

fn test_concurrence<T: 'static + GlobalPalloc + Send + Sync>() {
    let mut heap = std::vec![0u8; 500];
    let allocator = unsafe { T::new_from_slice(&mut heap) };

    // vector must not go out of scope before every thread has finished
    let palloc = Arc::new((heap, allocator));

    #[allow(clippy::needless_collect)]
    let threads: Vec<JoinHandle<_>> = (0..10)
        .map(|n| {
            let palloc = palloc.clone();
            thread::spawn(move || {
                let mut vec = Vec::<u8, &T>::with_capacity_in(20, &palloc.1);
                vec.push(n);
            })
        })
        .collect();

    // the two are not joined because collection of threads must occur
    // before joining them

    threads.into_iter().try_for_each(JoinHandle::join).unwrap();
}
