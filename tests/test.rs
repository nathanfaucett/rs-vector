#[macro_use]
extern crate vector;
extern crate zero;
extern crate collection_traits;


use std::ops::AddAssign;

use vector::Vector;
use zero::Zero;
use collection_traits::*;


const SIZE: usize = 32;


#[test]
fn test_vector_macro() {
    let mut v = vector![
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
        12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
        22, 23, 24, 25, 26, 27, 28, 29, 30, 31
    ];

    for i in 0..SIZE {
        assert_eq!(v.get(i).unwrap(), &i);
    }
    while !v.is_empty() {
        v.pop();
    }

    assert!(v.is_empty());

    let v = vector![1];
    assert!(v.len() == 1);
}

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

#[test]
fn test_clone() {
    let mut a = Vector::new();
    a.push(1);
    a.push(2);
    let mut b = a.clone();
    a.push(3);
    b.push(0);
    assert_ne!(a, b);
    a.pop();
    b.pop();
    assert_eq!(a, b);
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
