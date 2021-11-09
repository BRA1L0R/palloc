mod palloc;
mod spinlock;

static mut _HEAP: [u8; 150] = [0; 150];
unsafe fn empty_heap() -> &'static mut [u8] {
    _HEAP.iter_mut().for_each(|v| *v = 0);
    _HEAP.as_mut()
}
