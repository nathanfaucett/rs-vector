#![feature(alloc)]
#![feature(dropck_eyepatch)]
#![feature(inclusive_range)]
#![no_std]


extern crate alloc;

extern crate collection_traits;


mod vector;


pub use self::vector::Vector;
