use alloc::boxed::Box;
use alloc::raw_vec::RawVec;

use core::{ptr, slice, mem};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use collection_traits::*;


pub struct Vector<T> {
    raw: RawVec<T>,
    len: usize,
}

unsafe impl<T: Send> Send for Vector<T> {}
unsafe impl<T: Sync> Sync for Vector<T> {}

impl<T> Vector<T> {
    #[inline(always)]
    pub fn new() -> Self {
        Vector {
            raw: RawVec::new(),
            len: 0,
        }
    }
    #[inline(always)]
    pub fn with_capacity(cap: usize) -> Self {
        Vector {
            raw: RawVec::with_capacity(cap),
            len: 0,
        }
    }
    #[inline(always)]
    pub unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Vector<T> {
        Vector {
            raw: RawVec::from_raw_parts(ptr, capacity),
            len: length,
        }
    }
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.raw.cap()
    }
    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(self.len, additional);
    }
    #[inline(always)]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.raw.reserve_exact(self.len, additional);
    }
    #[inline(always)]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit(self.len);
    }
    #[inline]
    pub fn into_boxed_slice(mut self) -> Box<[T]> {
        unsafe {
            self.shrink_to_fit();
            let raw = ptr::read(&self.raw);
            mem::forget(self);
            raw.into_box()
        }
    }
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        &**self
    }
    #[inline(always)]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut **self
    }
    #[inline(always)]
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        unsafe {
            while len < self.len {
                self.len -= 1;
                let len = self.len;
                ptr::drop_in_place(self.get_unchecked_mut(len));
            }
        }
    }
    #[inline]
    pub fn retain<F>(&mut self, mut f: F)
        where F: FnMut(&T) -> bool
    {
        let len = self.len;
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        self.reserve(other.len());

        let len = self.len;
        unsafe {
            ptr::copy_nonoverlapping(other.as_ptr(), self.get_unchecked_mut(len), other.len());
        }

        self.len += other.len();
        unsafe {
            other.set_len(0);
        }
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe {
            slice::from_raw_parts(self.raw.ptr(), self.len)
        }
    }
}
impl<T> DerefMut for Vector<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            slice::from_raw_parts_mut(self.raw.ptr(), self.len)
        }
    }
}

impl<T> Index<usize> for Vector<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &(**self)[index]
    }
}
impl<T> IndexMut<usize> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut (**self)[index]
    }
}

impl<T> Collection for Vector<T> {
    #[inline(always)]
    fn len(&self) -> usize { self.len }
    #[inline(always)]
    fn clear(&mut self) {
        self.truncate(0)
    }
}

impl<T> Insert<usize, T> for Vector<T> {
    type Output = ();

    #[inline]
    fn insert(&mut self, index: usize, element: T) -> Self::Output {
        let len = self.len;
        assert!(index <= len);

        if len == self.raw.cap() {
            self.raw.double();
        }

        unsafe {
            {
                let p = self.as_mut_ptr().offset(index as isize);
                ptr::copy(p, p.offset(1), len - index);
                ptr::write(p, element);
            }
            self.len += 1;
        }
    }
}

impl<T> Remove<usize> for Vector<T> {
    type Output = T;

    #[inline]
    fn remove(&mut self, index: usize) -> T {
        let len = self.len;
        assert!(index < len);
        unsafe {
            let ret;
            {
                let ptr = self.as_mut_ptr().offset(index as isize);
                ret = ptr::read(ptr);
                ptr::copy(ptr.offset(1), ptr, len - index - 1);
            }
            self.len -= 1;
            ret
        }
    }
}

impl<T> Deque<T> for Vector<T> {
    #[inline]
    fn push_front(&mut self, element: T) {
        if self.len == self.raw.cap() {
            self.raw.double();
        }
        unsafe {
            let end = self.as_mut_ptr().offset(self.len as isize);
            ptr::write(end, element);
            self.len += 1;
        }
    }
    #[inline(always)]
    fn push_back(&mut self, element: T) {
        self.insert(0, element);
    }
    #[inline]
    fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(ptr::read(self.get_unchecked(self.len)))
            }
        }
    }
    #[inline(always)]
    fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            Some(self.remove(0))
        }
    }
    #[inline]
    fn front(&self) -> Option<&T> {
        let len = self.len;

        if len == 0 {
            None
        } else {
            unsafe {
                Some(self.get_unchecked(len - 1))
            }
        }
    }
    #[inline]
    fn back(&self) -> Option<&T> {
        let len = self.len;

        if len == 0 {
            None
        } else {
            unsafe {
                Some(self.get_unchecked(0))
            }
        }
    }
    #[inline]
    fn front_mut(&mut self) -> Option<&mut T> {
        let len = self.len;

        if len == 0 {
            None
        } else {
            unsafe {
                Some(self.get_unchecked_mut(len - 1))
            }
        }
    }
    #[inline]
    fn back_mut(&mut self) -> Option<&mut T> {
        let len = self.len;

        if len == 0 {
            None
        } else {
            unsafe {
                Some(self.get_unchecked_mut(0))
            }
        }
    }
}

impl<T> Stack<T> for Vector<T> {
    #[inline]
    fn push(&mut self, element: T) { self.push_front(element) }
    #[inline]
    fn pop(&mut self) -> Option<T> { self.pop_front() }
    #[inline]
    fn top(&self) -> Option<&T> { self.front() }
    #[inline]
    fn top_mut(&mut self) -> Option<&mut T> { self.front_mut() }
}

impl<T> Queue<T> for Vector<T> {
    #[inline(always)]
    fn enqueue(&mut self, element: T) { self.push_back(element) }
    #[inline(always)]
    fn dequeue(&mut self) -> Option<T> { self.pop_front() }
    #[inline(always)]
    fn peek(&self) -> Option<&T> { self.front() }
    #[inline(always)]
    fn peek_mut(&mut self) -> Option<&mut T> { self.front_mut() }
}

impl<'a, T: 'a> Iterable<'a, &'a T> for Vector<T> {
    type Iter = slice::Iter<'a, T>;

    #[inline(always)]
    fn iter(&'a self) -> Self::Iter {
        (&**self).iter()
    }
}

impl<'a, T: 'a> IterableMut<'a, &'a mut T> for Vector<T> {
    type IterMut = slice::IterMut<'a, T>;

    #[inline(always)]
    fn iter_mut(&'a mut self) -> Self::IterMut {
        (&mut **self).iter_mut()
    }
}

impl<'a, T: 'a> Seq<'a, T> for Vector<T> {}
impl<'a, T: 'a> SeqMut<'a, T> for Vector<T> {}
