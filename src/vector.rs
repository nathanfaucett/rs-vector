use alloc::boxed::Box;
use alloc::raw_vec::RawVec;
use alloc::heap::EMPTY;

use collections::Bound;
use collections::range::RangeArgument;

use core::{fmt, ptr, slice, mem};
use core::ops::*;
use core::ptr::Shared;
use core::hash::{self, Hash};
use core::cmp::Ordering;
use core::iter::{TrustedLen, FusedIterator, FromIterator};
use core::intrinsics::{assume, arith_offset};

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
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<T>
        where R: RangeArgument<usize>
    {
        let len = self.len();
        let start = match range.start() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => 0,
        };
        let end = match range.end() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => len,
        };
        assert!(start <= end);
        assert!(end <= len);

        unsafe {
            self.set_len(start);
            let range_slice = slice::from_raw_parts_mut(self.as_mut_ptr().offset(start as isize),
                                                        end - start);
            Drain {
                tail_start: end,
                tail_len: len - end,
                iter: range_slice.iter(),
                vec: Shared::new(self as *mut _),
            }
        }
    }
    #[inline]
    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at <= self.len(), "`at` out of bounds");

        let other_len = self.len - at;
        let mut other = Vector::with_capacity(other_len);

        unsafe {
            self.set_len(at);
            other.set_len(other_len);

            ptr::copy_nonoverlapping(
                self.as_ptr().offset(at as isize),
                other.as_mut_ptr(),
                other.len()
            );
        }
        other
    }
}

impl<T> Default for Vector<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vector<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(&mut self[..]);
        }
    }
}

macro_rules! __impl_slice_eq1 {
    ($Lhs: ty, $Rhs: ty) => {
        __impl_slice_eq1! { $Lhs, $Rhs, Sized }
    };
    ($Lhs: ty, $Rhs: ty, $Bound: ident) => {
        impl<'a, 'b, A: $Bound, B> PartialEq<$Rhs> for $Lhs where A: PartialEq<B> {
            #[inline(always)]
            fn eq(&self, other: &$Rhs) -> bool { self[..] == other[..] }
            #[inline(always)]
            fn ne(&self, other: &$Rhs) -> bool { self[..] != other[..] }
        }
    }
}

__impl_slice_eq1! { Vector<A>, Vector<B> }
__impl_slice_eq1! { Vector<A>, &'b [B] }
__impl_slice_eq1! { Vector<A>, &'b mut [B] }

macro_rules! array_impls {
    ($($N: expr)+) => {
        $(
            __impl_slice_eq1! { Vector<A>, [B; $N] }
            __impl_slice_eq1! { Vector<A>, &'b [B; $N] }
            __impl_slice_eq1! { Vector<A>, &'b mut [B; $N] }
        )+
    }
}

array_impls! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}

impl<T: PartialOrd> PartialOrd for Vector<T> {
    #[inline]
    fn partial_cmp(&self, other: &Vector<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: Eq> Eq for Vector<T> {}

impl<T: Ord> Ord for Vector<T> {
    #[inline]
    fn cmp(&self, other: &Vector<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Hash> Hash for Vector<T> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
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

impl<T: fmt::Debug> fmt::Debug for Vector<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: Clone> Clone for Vector<T> {
    #[inline]
    fn clone(&self) -> Self {
        let cap = self.raw.cap();
        let new_raw = RawVec::with_capacity(cap);

        unsafe {
            ptr::copy_nonoverlapping(self.raw.ptr(), new_raw.ptr(), cap);
        }

        Vector {
            raw: new_raw,
            len: self.len,
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

impl<T> Index<Range<usize>> for Vector<T> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: Range<usize>) -> &[T] {
        Index::index(&**self, index)
    }
}
impl<T> Index<RangeTo<usize>> for Vector<T> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: RangeTo<usize>) -> &[T] {
        Index::index(&**self, index)
    }
}
impl<T> Index<RangeFrom<usize>> for Vector<T> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: RangeFrom<usize>) -> &[T] {
        Index::index(&**self, index)
    }
}
impl<T> Index<RangeFull> for Vector<T> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, _index: RangeFull) -> &[T] {
        self
    }
}
impl<T> Index<RangeInclusive<usize>> for Vector<T> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: RangeInclusive<usize>) -> &[T] {
        Index::index(&**self, index)
    }
}
impl<T> Index<RangeToInclusive<usize>> for Vector<T> {
    type Output = [T];

    #[inline(always)]
    fn index(&self, index: RangeToInclusive<usize>) -> &[T] {
        Index::index(&**self, index)
    }
}

impl<T> IndexMut<Range<usize>> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: Range<usize>) -> &mut [T] {
        IndexMut::index_mut(&mut **self, index)
    }
}
impl<T> IndexMut<RangeTo<usize>> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut [T] {
        IndexMut::index_mut(&mut **self, index)
    }
}
impl<T> IndexMut<RangeFrom<usize>> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut [T] {
        IndexMut::index_mut(&mut **self, index)
    }
}
impl<T> IndexMut<RangeFull> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, _index: RangeFull) -> &mut [T] {
        self
    }
}
impl<T> IndexMut<RangeInclusive<usize>> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut [T] {
        IndexMut::index_mut(&mut **self, index)
    }
}
impl<T> IndexMut<RangeToInclusive<usize>> for Vector<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: RangeToInclusive<usize>) -> &mut [T] {
        IndexMut::index_mut(&mut **self, index)
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
    #[inline(always)]
    fn push(&mut self, element: T) { self.push_front(element) }
    #[inline(always)]
    fn pop(&mut self) -> Option<T> { self.pop_front() }
    #[inline(always)]
    fn top(&self) -> Option<&T> { self.front() }
    #[inline(always)]
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


impl<T> FromIterator<T> for Vector<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Vector<T> {
        <Self as SpecExtend<_, _>>::from_iter(iter.into_iter())
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(mut self) -> IntoIter<T> {
        unsafe {
            let begin = self.as_mut_ptr();
            assume(!begin.is_null());
            let end = if mem::size_of::<T>() == 0 {
                arith_offset(begin as *const i8, self.len() as isize) as *const T
            } else {
                begin.offset(self.len() as isize) as *const T
            };
            let cap = self.raw.cap();
            mem::forget(self);
            IntoIter {
                raw: Shared::new(begin),
                cap: cap,
                ptr: begin,
                end: end,
            }
        }
    }
}

impl<'a, T> IntoIterator for &'a Vector<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vector<T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(mut self) -> slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<T> Extend<T> for Vector<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.spec_extend(iter.into_iter())
    }
}

trait SpecExtend<T, I> {
    fn from_iter(iter: I) -> Self;
    fn spec_extend(&mut self, iter: I);
}

impl<T, I> SpecExtend<T, I> for Vector<T>
    where I: Iterator<Item=T>,
{
    default fn from_iter(mut iterator: I) -> Self {
        let mut vector = match iterator.next() {
            None => return Vector::new(),
            Some(element) => {
                let (lower, _) = iterator.size_hint();
                let mut vector = Vector::with_capacity(lower.saturating_add(1));
                unsafe {
                    ptr::write(vector.get_unchecked_mut(0), element);
                    vector.set_len(1);
                }
                vector
            }
        };
        vector.spec_extend(iterator);
        vector
    }

    default fn spec_extend(&mut self, iter: I) {
        self.extend_desugared(iter)
    }
}

struct SetLenOnDrop<'a> {
    len: &'a mut usize,
    local_len: usize,
}

impl<'a> SetLenOnDrop<'a> {
    #[inline]
    fn new(len: &'a mut usize) -> Self {
        SetLenOnDrop { local_len: *len, len: len }
    }
    #[inline]
    fn increment_len(&mut self, increment: usize) {
        self.local_len += increment;
    }
}

impl<'a> Drop for SetLenOnDrop<'a> {
    #[inline]
    fn drop(&mut self) {
        *self.len = self.local_len;
    }
}

impl<T, I> SpecExtend<T, I> for Vector<T>
    where I: TrustedLen<Item=T>,
{
    fn from_iter(iterator: I) -> Self {
        let mut vector = Vector::new();
        vector.spec_extend(iterator);
        vector
    }

    fn spec_extend(&mut self, iterator: I) {
        let (low, high) = iterator.size_hint();
        if let Some(high_value) = high {
            debug_assert_eq!(low, high_value,
                             "TrustedLen iterator's size hint is not exact: {:?}",
                             (low, high));
        }
        if let Some(additional) = high {
            self.reserve(additional);
            unsafe {
                let mut ptr = self.as_mut_ptr().offset(self.len() as isize);
                let mut local_len = SetLenOnDrop::new(&mut self.len);
                for element in iterator {
                    ptr::write(ptr, element);
                    ptr = ptr.offset(1);
                    local_len.increment_len(1);
                }
            }
        } else {
            self.extend_desugared(iterator)
        }
    }
}

impl<'a, T: 'a, I> SpecExtend<&'a T, I> for Vector<T>
    where I: Iterator<Item=&'a T>,
          T: Clone,
{
    default fn from_iter(iterator: I) -> Self {
        SpecExtend::from_iter(iterator.cloned())
    }

    default fn spec_extend(&mut self, iterator: I) {
        self.spec_extend(iterator.cloned())
    }
}

impl<'a, T: 'a> SpecExtend<&'a T, slice::Iter<'a, T>> for Vector<T>
    where T: Copy,
{
    fn spec_extend(&mut self, iterator: slice::Iter<'a, T>) {
        let slice = iterator.as_slice();
        self.reserve(slice.len());
        unsafe {
            let len = self.len();
            self.set_len(len + slice.len());
            self.get_unchecked_mut(len..).copy_from_slice(slice);
        }
    }
}

impl<T> Vector<T> {
    fn extend_desugared<I: Iterator<Item = T>>(&mut self, mut iterator: I) {
        while let Some(element) = iterator.next() {
            let len = self.len();
            if len == self.capacity() {
                let (lower, _) = iterator.size_hint();
                self.reserve(lower.saturating_add(1));
            }
            unsafe {
                ptr::write(self.get_unchecked_mut(len), element);
                self.set_len(len + 1);
            }
        }
    }
}

impl<'a, T: 'a + Copy> Extend<&'a T> for Vector<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.spec_extend(iter.into_iter())
    }
}

impl<T> AsRef<Vector<T>> for Vector<T> {
    fn as_ref(&self) -> &Vector<T> {
        self
    }
}

impl<T> AsMut<Vector<T>> for Vector<T> {
    fn as_mut(&mut self) -> &mut Vector<T> {
        self
    }
}

impl<T> AsRef<[T]> for Vector<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for Vector<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'a, T: Clone> From<&'a [T]> for Vector<T> {
    fn from(s: &'a [T]) -> Vector<T> {
        unsafe {
            let len = s.len();
            let raw = RawVec::with_capacity(len.next_power_of_two());
            ptr::copy_nonoverlapping(s.as_ptr(), raw.ptr(), len);

            Vector {
                raw: raw,
                len: len,
            }
        }
    }
}

impl<'a> From<&'a str> for Vector<u8> {
    fn from(s: &'a str) -> Vector<u8> {
        From::from(s.as_bytes())
    }
}

pub struct IntoIter<T> {
    raw: Shared<T>,
    cap: usize,
    ptr: *const T,
    end: *const T,
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IntoIter")
            .field(&self.as_slice())
            .finish()
    }
}

impl<T> IntoIter<T> {
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len())
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.ptr as *mut T, self.len())
        }
    }
}

unsafe impl<T: Send> Send for IntoIter<T> {}
unsafe impl<T: Sync> Sync for IntoIter<T> {}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        unsafe {
            if self.ptr as *const _ == self.end {
                None
            } else {
                if mem::size_of::<T>() == 0 {
                    self.ptr = arith_offset(self.ptr as *const i8, 1) as *mut T;
                    Some(ptr::read(EMPTY as *mut T))
                } else {
                    let old = self.ptr;
                    self.ptr = self.ptr.offset(1);

                    Some(ptr::read(old))
                }
            }
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let diff = (self.end as usize) - (self.ptr as usize);
        let size = mem::size_of::<T>();
        let exact = diff /
                    (if size == 0 {
                         1
                     } else {
                         size
                     });
        (exact, Some(exact))
    }
    #[inline(always)]
    fn count(self) -> usize {
        self.len()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        unsafe {
            if self.end == self.ptr {
                None
            } else {
                if mem::size_of::<T>() == 0 {
                    self.end = arith_offset(self.end as *const i8, -1) as *mut T;
                    Some(ptr::read(EMPTY as *mut T))
                } else {
                    self.end = self.end.offset(-1);

                    Some(ptr::read(self.end))
                }
            }
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn is_empty(&self) -> bool {
        self.ptr == self.end
    }
}

impl<T> FusedIterator for IntoIter<T> {}

unsafe impl<T> TrustedLen for IntoIter<T> {}

impl<T: Clone> Clone for IntoIter<T> {
    fn clone(&self) -> IntoIter<T> {
        IntoIter {
            raw: self.raw,
            cap: self.cap,
            ptr: self.ptr,
            end: self.end,
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        for _x in self.by_ref() {}
        let _ = unsafe { RawVec::from_raw_parts(*self.raw, self.cap) };
    }
}

pub struct Drain<'a, T: 'a> {
    tail_start: usize,
    tail_len: usize,
    iter: slice::Iter<'a, T>,
    vec: Shared<Vector<T>>,
}

unsafe impl<'a, T: Sync> Sync for Drain<'a, T> {}
unsafe impl<'a, T: Send> Send for Drain<'a, T> {}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.iter.next().map(|elt| unsafe { ptr::read(elt as *const _) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back().map(|elt| unsafe { ptr::read(elt as *const _) })
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        while let Some(_) = self.next() {}

        if self.tail_len > 0 {
            unsafe {
                let source_vec = &mut **self.vec;
                let start = source_vec.len();
                let tail = self.tail_start;
                let src = source_vec.as_ptr().offset(tail as isize);
                let dst = source_vec.as_mut_ptr().offset(start as isize);
                ptr::copy(src, dst, self.tail_len);
                source_vec.set_len(start + self.tail_len);
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for Drain<'a, T> {
    fn is_empty(&self) -> bool {
        self.iter.is_empty()
    }
}

impl<'a, T> FusedIterator for Drain<'a, T> {}
