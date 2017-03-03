extern crate vector;
extern crate zero;
extern crate collection_traits;


use std::ops::AddAssign;

use vector::Vector;
use zero::Zero;
use collection_traits::*;


const SIZE: usize = 32;


#[test]
fn test_vector() {
    let mut v = Vector::<usize>::new();

    for i in 0..SIZE {
        v.push(i);
    }
    for i in 0..SIZE {
        assert_eq!(v.get(i).unwrap(), &i);
    }
    while !v.is_empty() {
        v.pop();
    }

    assert!(v.is_empty());
}

fn sum<'a, A, B>(array: &'a A) -> B
    where A: 'a + Seq<'a, B>,
          B: 'a + Copy + Zero + AddAssign<B>,
{
    let mut out = B::zero();

    for value in array.iter() {
        out += *value;
    }
    out
}

#[test]
fn test_iter() {
    let mut a = Vector::<usize>::new();
    for i in 0..SIZE {
        a.push(i);
    }
    let out = sum(&a);
    assert_eq!(out, 496);
}
