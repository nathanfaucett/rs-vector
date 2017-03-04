#![feature(test)]


extern crate test;

extern crate vector;
extern crate collection_traits;


use test::Bencher;

use collection_traits::*;


const SIZE: usize = 32;


#[bench]
fn bench_vector(b: &mut Bencher) {
    use vector::Vector;

    b.iter(|| {
        let mut v = Vector::<usize>::new();
        for i in 0..SIZE {
            v.push(i);
        }
        while !v.is_empty() {
            v.pop();
        }
        v
    });
}
#[bench]
fn bench_std_vector(b: &mut Bencher) {
    use std::vec::Vec;

    b.iter(|| {
        let mut v = Vec::<usize>::new();
        for i in 0..SIZE {
            v.push(i);
        }
        while !v.is_empty() {
            v.pop();
        }
        v
    });
}

#[bench]
fn bench_vector_clone(b: &mut Bencher) {
    use vector::Vector;

    b.iter(|| {
        let mut v = Vector::<usize>::new();
        for i in 0..SIZE {
            v.push(i);
        }
        v.clone()
    });
}
#[bench]
fn bench_std_vector_clone(b: &mut Bencher) {
    use std::vec::Vec;

    b.iter(|| {
        let mut v = Vec::<usize>::new();
        for i in 0..SIZE {
            v.push(i);
        }
        v.clone()
    });
}
