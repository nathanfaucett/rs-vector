#![feature(test)]


extern crate test;

extern crate vector;
extern crate collection_traits;


use test::Bencher;

use vector::Vector;
use collection_traits::*;


const SIZE: usize = 1024;


#[bench]
fn bench_vector(b: &mut Bencher) {
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
    let mut v = Vector::<usize>::new();
    for i in 0..SIZE {
        v.push(i);
    }

    b.iter(move || {
        v.clone()
    });
}
#[bench]
fn bench_std_vector_clone(b: &mut Bencher) {
    let mut v = Vec::<usize>::new();
    for i in 0..SIZE {
        v.push(i);
    }

    b.iter(move || {
        v.clone()
    });
}
