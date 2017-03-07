#![feature(alloc)]
#![feature(collections)]
#![feature(inclusive_range)]
#![feature(specialization)]
#![feature(heap_api)]
#![feature(shared)]
#![feature(trusted_len)]
#![feature(fused)]
#![feature(core_intrinsics)]
#![feature(exact_size_is_empty)]
#![feature(collections_range)]
#![feature(collections_bound)]
#![no_std]


extern crate alloc;
extern crate collections;

extern crate collection_traits;


mod vector;


pub use self::vector::Vector;
