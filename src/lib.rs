#![feature(alloc)]
#![feature(inclusive_range)]
#![feature(specialization)]
#![feature(shared)]
#![feature(trusted_len)]
#![feature(fused)]
#![feature(core_intrinsics)]
#![feature(exact_size_is_empty)]
#![feature(collections_range)]
#![feature(box_syntax)]
#![no_std]


pub extern crate alloc;

extern crate collection_traits;


pub mod vector;


pub use self::vector::Vector;


#[macro_export]
macro_rules! vector {
    ($elem:expr; $n:expr) => (
        $crate::vector::from_elem($elem, $n)
    );
    ($($x:expr),*) => (
        $crate::slice_to_vector($crate::alloc::boxed::Box::new([$($x),*]))
    );
    ($($x:expr,)*) => (vector![$($x),*])
}


#[inline]
pub fn slice_to_vector<T>(mut slice: alloc::boxed::Box<[T]>) -> Vector<T> {
    unsafe {
        let vector = Vector::from_raw_parts(slice.as_mut_ptr(), slice.len(), slice.len());
        core::mem::forget(slice);
        vector
    }
}
