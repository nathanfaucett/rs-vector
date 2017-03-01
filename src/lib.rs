#![feature(alloc)]
#![no_std]


extern crate alloc;

extern crate collection_traits;


mod vector;


pub use self::vector::Vector;
