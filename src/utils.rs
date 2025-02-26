use std::intrinsics::prefetch_read_data;

pub fn prefetch_index<T>(s: &[T], index: usize){
    let ptr = unsafe { s.as_ptr().add(index) as *const u64 };
    unsafe { prefetch_read_data(ptr, 3) };
}
