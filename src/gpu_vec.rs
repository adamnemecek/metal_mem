///
/// TODO:
///     * maybe remove copy? maybe not, because things in the array are values
///     * should iterator return references?
///     * fix interators
///         * right now
///     * fix push
///     * append
///

use crate::{
    round_up,
    page_aligned,
    GPUResource
};

use std::hash::{
    Hash,
    Hasher
};

use std::ops::{
    RangeBounds,
    Bound::{Excluded, Included, Unbounded}
};

use std::ptr::{NonNull};

pub struct GPUVec<T: Copy> {
    device: metal::Device,
    buffer: metal::Buffer,
    len: usize,
    capacity: usize,
    phantom: std::marker::PhantomData<T>
}

impl<T: Copy> GPUResource for GPUVec<T> {
    type Device = metal::Device;
    fn device(&self) -> &Self::Device {
        &self.device
    }

    fn set_device(&mut self, device: &Self::Device) {
        self.device = device.to_owned();
    }
}

impl<T: Copy> GPUVec<T> {
    pub fn with_capacity(device: &metal::DeviceRef, capacity: usize) -> Self {
        let byte_capacity = page_aligned(capacity * Self::element_size()) as u64;
        let buffer = device.new_buffer(
            byte_capacity,
            metal::MTLResourceOptions::CPUCacheModeDefaultCache
        );
        Self {
            device: device.to_owned(),
            buffer,
            len: 0,
            capacity,
            phantom: std::marker::PhantomData
        }
    }

    pub fn from_slice(device: &metal::DeviceRef, data: &[T]) -> Self {
        let len = data.len();
        let mut ret = Self::with_capacity(device, len);

        unsafe {
            std::ptr::copy(
                data.as_ptr(),
                ret.as_mut_ptr(),
                len
            );
        }

        ret.len = len;
        ret
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Reserves space for at least `addtional` more elements;
    pub fn reserve(&mut self, additional: usize) {
        self.resize(self.capacity() + additional)
    }

    // pub fn shrink_to(&mut self, min_capacity: usize) {
    //     self.buf.shrink_to_fit(cmp::max(self.len, min_capacity));
    // }

    #[inline]
    fn element_size() -> usize {
        std::mem::size_of::<T>()
    }

    #[inline]
    pub fn byte_len(&self) -> usize {
        Self::element_size() * self.len()
    }

    #[inline]
    pub fn byte_capacity(&self) -> usize {
        Self::element_size() * self.capacity()
    }

    pub fn resize(&mut self, capacity: usize) {
        if capacity <= self.capacity() {
            return;
        }

        let byte_capacity = page_aligned(capacity * Self::element_size()) as u64;
        let buffer = self.device.new_buffer(byte_capacity, metal::MTLResourceOptions::CPUCacheModeDefaultCache);
        unsafe {
            std::ptr::copy(
                self.as_ptr(),
                buffer.contents() as *mut T,
                self.len()
            );
        }
        self.buffer = buffer;
        self.capacity = capacity;
    }

    pub fn extend_from_slice(&mut self, v: &[T]) {
        let offset = self.len();

        let new_len = self.len() + v.len();

        self.resize(new_len);

        unsafe {
            std::ptr::copy(
                v.as_ptr(),
                self.as_mut_ptr().offset(self.len() as isize),
                v.len()
            );
        }
        self.len = new_len;
    }

    /// untested
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len > self.len {
            return;
        }

        unsafe {
            self.set_len(len)
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.buffer.contents() as *const T
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.buffer.contents() as *mut T
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());
        self.len = new_len;
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        unsafe {
            // We replace self[index] with the last element. Note that if the
            // bounds check on hole succeeds there must be a last element (which
            // can be self[index] itself).
            let hole: *mut T = &mut self[index];
            let last = std::ptr::read(self.as_ptr().offset((self.len - 1) as isize));
            self.len -= 1;
            std::ptr::replace(hole, last)
        }
    }

    pub fn insert(&mut self, index: usize, element: T) {
        let len = self.len();
        assert!(index <= len);

        // space for the new element
        if len == self.capacity() {
            self.reserve(1);
        }

        unsafe {
            // infallible
            // The spot to put the new value
            {
                let p = self.as_mut_ptr().add(index);
                // Shift everything over to make space. (Duplicating the
                // `index`th element into two consecutive places.)
                std::ptr::copy(p, p.offset(1), len - index);
                // Write it in, overwriting the first copy of the `index`th
                // element.
                std::ptr::write(p, element);
            }
            self.set_len(len + 1);
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        ///
        /// Implementation based on Rust's Vec::remove
        ///
        let len = self.len();
        assert!(index < len);
        unsafe {
            // infallible
            let ret;
            {
                // the place we are taking from.
                let ptr = self.as_mut_ptr().add(index);
                // copy it out, unsafely having a copy of the value on
                // the stack and in the vector at the same time.
                ret = std::ptr::read(ptr);

                // Shift everything down to fill in that spot.
                std::ptr::copy(ptr.offset(1), ptr, len - index - 1);
            }
            self.set_len(len - 1);
            ret
        }
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let len = self.len();
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

    // #[inline]
    // pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    // where
    //     F: FnMut(&mut T) -> K,
    //     K: PartialEq,
    // {
    //     self.dedup_by(|a, b| key(a) == key(b))
    // }

    // pub fn dedup_by<F>(&mut self, same_bucket: F)
    // where
    //     F: FnMut(&mut T, &mut T) -> bool,
    // {
    //     let len = {
    //         let (dedup, _) = self.as_mut_slice().partition_dedup_by(same_bucket);
    //         dedup.len()
    //     };
    //     self.truncate(len);
    // }

    /// Appends an element to the back of a collection.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the vector overflows a `usize`.
    ///
    /// # Examples
    ///
    ///
    /// //let mut vec = vec![1, 2];
    /// //vec.push(3);
    /// //assert_eq!(vec, [1, 2, 3]);
    ///
    #[inline]
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity() {
            self.reserve(1);
        }
        unsafe {
            let end = self.as_mut_ptr().add(self.len);
            std::ptr::write(end, value);
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            let last = self[self.len()];
            self.len -= 1;
            Some(last)
        }
    }

    #[inline]
    // #[stable(feature = "append", since = "1.4.0")]
    pub fn append(&mut self, other: &mut Self) {
        todo!()
        // unsafe {
        //     self.append_elements(other.as_slice() as _);
        //     other.set_len(0);
        // }
    }

    /// Appends elements to `Self` from other buffer.
    #[inline]
    unsafe fn append_elements(&mut self, other: *const [T]) {
        todo!()
        // let count = (*other).len();
        // self.reserve(count);
        // let len = self.len();
        // ptr::copy_nonoverlapping(other as *const T, self.as_mut_ptr().add(len), count);
        // self.len += count;
    }

    pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    where
        R: RangeBounds<usize>,
    {

        // Memory safety
        //
        // When the Drain is first created, it shortens the length of
        // the source vector to make sure no uninitialized or moved-from elements
        // are accessible at all if the Drain's destructor never gets to run.
        //
        // Drain will ptr::read out the values to remove.
        // When finished, remaining tail of the vec is copied back to cover
        // the hole, and the vector length is restored to the new length.
        //
        let len = self.len();
        let start = match range.start_bound() {
            Included(&n) => n,
            Excluded(&n) => n + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(&n) => n + 1,
            Excluded(&n) => n,
            Unbounded => len,
        };

        assert!(start <= end);
        assert!(end <= len);

        unsafe {
            // set self.vec length's to start, to be safe in case Drain is leaked
            self.set_len(start);
            // Use the borrow in the IterMut to indicate borrowing behavior of the
            // whole Drain iterator (like &mut T).
            let range_slice = std::slice::from_raw_parts_mut(self.as_mut_ptr().add(start), end - start);
            Drain {
                tail_start: end,
                tail_len: len - end,
                iter: range_slice.iter(),
                vec: NonNull::from(self),
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            self.set_len(0)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at <= self.len(), "`at` out of bounds");

        let other_len = self.len - at;
        let mut other = Self::with_capacity(&self.device, other_len);

        // Unsafely `set_len` and copy items to `other`.
        unsafe {
            self.set_len(at);
            other.set_len(other_len);

            std::ptr::copy_nonoverlapping(self.as_ptr().add(at), other.as_mut_ptr(), other.len());
        }
        other
    }

    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        todo!()
        // let len = self.len();
        // if new_len > len {
        //     self.extend_with(new_len - len, ExtendFunc(f));
        // } else {
        //     self.truncate(new_len);
        // }
    }

    // pub fn iter(&self) -> Iter<T> {
    //     todo!()
    //     // Iter {
    //     //     len: self.len,
    //     //     inner: self.items.iter().enumerate(),
    //     // }
    // }

    // pub fn iter_mut(&mut self) -> IterMut<T> {
    //     todo!()
    //     // IterMut {
    //     //     len: self.len,
    //     //     inner: self.items.iter_mut().enumerate(),
    //     // }
    // }
}

impl<T : Copy> GPUVec<T> {
    /// Removes the first instance of `item` from the vector if the item exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(vec_remove_item)]
    /// let mut vec = vec![1, 2, 3, 1];
    ///
    /// vec.remove_item(&1);
    ///
    /// assert_eq!(vec, vec![2, 3, 1]);
    /// ```
    pub fn remove_item<V>(&mut self, item: &V) -> Option<T>
    where
        T: PartialEq<V>,
    {
        let pos = self.iter().position(|x| *x == *item)?;
        Some(self.remove(pos))
    }
}

impl<T: Copy> std::ops::Index<usize> for GPUVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len());
        unsafe {
            self.as_ptr().offset(index as isize).as_ref().unwrap()
        }
    }
}

impl<T: Copy> std::ops::IndexMut<usize> for GPUVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len());
        unsafe {
            self.as_mut_ptr().offset(index as isize).as_mut().unwrap()
        }
    }
}

impl<T: Copy> Extend<T> for GPUVec<T> {
    fn extend<I: IntoIterator<Item=T>>(&mut self, iter: I) {
        let v: Vec<T> = iter.into_iter().collect();
        self.extend_from_slice(&v);
    }
}

// impl<T: Copy + std::fmt::Display> std::fmt::Display for GPUVec<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         // std::fmt::Display::fmt(&**self, f)
//         // fmt::Debug::fmt(&**self, f)
//         // for i in 0..self.len() {
//         //     // debugtuple
//         //     writeln!(f, "{}", self[i]);
//         // }
//         // Ok(())
//     }
// }

impl<T: Copy> std::ops::Deref for GPUVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr(),
                self.len()
            )
        }
    }
}

impl<T: Copy> std::ops::DerefMut for GPUVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut_ptr(),
                self.len()
            )
        }
    }
}

pub struct Drain<'a, T: Copy > {
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: std::slice::Iter<'a, T>,
    vec: NonNull<GPUVec<T>>,
    // inner: &'a GPUVec<T>
}

impl<T: Copy> Iterator for Drain<'_, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.iter.next().map(|elt| unsafe { std::ptr::read(elt as *const _) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Copy> DoubleEndedIterator for Drain<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back().map(|elt| unsafe { std::ptr::read(elt as *const _) })
    }
}


pub struct Splice<'a, I: Iterator> where I::Item : Copy{
    drain: Drain<'a, I::Item>,
    replace_with: I,
}

impl<I: Iterator> Iterator for Splice<'_, I> where I::Item : Copy {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain.size_hint()
    }
}

impl<I: Iterator> DoubleEndedIterator for Splice<'_, I> where I::Item : Copy {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.next_back()
    }
}

// pub struct DrainFilter<'a, T: Copy, F>
// where
//     F: FnMut(&mut T) -> bool,
// {
//     vec: &'a mut GPUVec<T>,
//     /// The index of the item that will be inspected by the next call to `next`.
//     idx: usize,
//     /// The number of items that have been drained (removed) thus far.
//     del: usize,
//     /// The original length of `vec` prior to draining.
//     old_len: usize,
//     /// The filter test predicate.
//     pred: F,
//     /// A flag that indicates a panic has occurred in the filter test prodicate.
//     /// This is used as a hint in the drop implmentation to prevent consumption
//     /// of the remainder of the `DrainFilter`. Any unprocessed items will be
//     /// backshifted in the `vec`, but no further items will be dropped or
//     /// tested by the filter predicate.
//     panic_flag: bool,
// }

impl<T: Copy> GPUVec<T> {
    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        Splice { drain: self.drain(range), replace_with: replace_with.into_iter() }
    }

    // pub fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<'_, T, F>
    // where
    //     F: FnMut(&mut T) -> bool,
    // {
    //     let old_len = self.len();

    //     // Guard against us getting leaked (leak amplification)
    //     unsafe {
    //         self.set_len(0);
    //     }

    //     DrainFilter { vec: self, idx: 0, del: 0, old_len, pred: filter, panic_flag: false }
    // }
}

impl<T: Copy> Clone for GPUVec<T> {
    fn clone(&self) -> Self {
        // let byte_capacity = self.byte_capacity();
        // let buffer = self.device.new_buffer(
        //     byte_capacity as u64,
        //     metal::MTLResourceOptions::CPUCacheModeDefaultCache
        // );

        // unsafe {
        //     std::ptr::copy(
        //         self.as_ptr(),
        //         buffer.contents() as *mut T,
        //         self.len()
        //     );
        // }
        // Self {
        //     device: self.device.to_owned(),
        //     buffer,
        //     len: self.len(),
        //     capacity: self.capacity(),
        //     phantom: std::marker::PhantomData
        // }
        Self::from_slice(&self.device, self.as_slice())
    }

    // fn clone_from(&mut self, other: &Vec<T>) {
    //     other.as_slice().clone_into(self);
    // }
}

impl<T: Copy> AsRef<metal::Buffer> for GPUVec<T> {
    #[inline]
    fn as_ref(&self) -> &metal::Buffer {
        &self.buffer
    }
}

impl<T: Copy> AsMut<metal::Buffer> for GPUVec<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut metal::Buffer {
        &mut self.buffer
    }
}

/// untested
impl<T: Copy + PartialEq> PartialEq for GPUVec<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            false
        }
        else {
            for i in 0..self.len() {
                if self[i] != other[i] {
                    return false;
                }
            }
            true
        }
    }
}

impl<T: Copy + Eq> Eq for GPUVec<T> { }

impl<T: Hash + Copy> Hash for GPUVec<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T: Copy + PartialOrd> PartialOrd for GPUVec<T> {
    #[inline]
    fn partial_cmp(&self, other: &GPUVec<T>) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T: Copy + Ord> Ord for GPUVec<T> {
    #[inline]
    fn cmp(&self, other: &GPUVec<T>) -> std::cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

pub struct Iter<'a, T: Copy> {
    inner: &'a GPUVec<T>,
    idx: usize,
}

impl<'a, T: Copy> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.inner.len() {
            None
        }
        else {
            let ret = &self.inner[self.idx];
            self.idx += 1;
            Some(ret)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.inner.len();
        (l, Some(l))
    }
}

impl<'a, T: Copy> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
        // loop {
        //     match self.inner.next_back() {
        //         Some((_, &Entry::Free { .. })) => continue,
        //         Some((
        //             index,
        //             &Entry::Occupied {
        //                 generation,
        //                 ref value,
        //             },
        //         )) => {
        //             self.len -= 1;
        //             let idx = Index { index, generation };
        //             return Some((idx, value));
        //         }
        //         None => {
        //             debug_assert_eq!(self.len, 0);
        //             return None;
        //         }
        //     }
        // }
    }
}

impl<'a, T: Copy> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T: Copy + std::fmt::Debug> std::fmt::Debug for GPUVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

// use core::iter::{self, Extend, FromIterator, FusedIterator};
impl<'a, T: Copy> std::iter::FusedIterator for Iter<'a, T> {}


////////////////////////////////////////////////////////////////////////////////
// Iterators
////////////////////////////////////////////////////////////////////////////////

/// An iterator that moves out of a vector.
///
/// This `struct` is created by the `into_iter` method on [`Vec`] (provided
/// by the [`IntoIterator`] trait).
///
/// [`Vec`]: struct.Vec.html
/// [`IntoIterator`]: ../../std/iter/trait.IntoIterator.html

pub struct IntoIter<T: Copy> {
    inner: GPUVec<T>,
    idx: usize
}

impl<T: Copy> IntoIterator for GPUVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self,
            idx: 0
        }
    }
}

impl<T: Copy> IntoIter<T> {
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.inner.as_mut_slice()
    }
}

impl<T: Copy + std::fmt::Debug> std::fmt::Debug for IntoIter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<T: Copy> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.inner.len() {
            None
        }
        else {
            let result = self.inner[self.idx];
            self.idx += 1;
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.len() - self.idx;
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.inner.len()
    }
}

impl<'a, T: Copy> IntoIterator for &'a GPUVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T: Copy> IntoIterator for &'a mut GPUVec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) ->  Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

impl<T: Copy> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        todo!()
        // unsafe {
        //     if self.end == self.ptr {
        //         None
        //     } else {
        //         if mem::size_of::<T>() == 0 {
        //             // See above for why 'ptr.offset' isn't used
        //             self.end = arith_offset(self.end as *const i8, -1) as *mut T;

        //             // Make up a value of this ZST.
        //             Some(mem::zeroed())
        //         } else {
        //             self.end = self.end.offset(-1);

        //             Some(ptr::read(self.end))
        //         }
        //     }
        // }
    }
}

// #[feature(exact_size_is_empty)]
// impl<T: Copy> ExactSizeIterator for IntoIter<T> {
//     fn is_empty(&self) -> bool {
//         // self.ptr == self.end
//         todo!()
//     }
// }

// impl<T> FusedIterator for IntoIter<T> {}

// unsafe impl<T> TrustedLen for IntoIter<T> {}

unsafe impl<T: Copy> Send for GPUVec<T> { }
unsafe impl<T: Copy> Sync for GPUVec<T> { }

impl<T: Copy> Drop for GPUVec<T> {
    fn drop(&mut self) {
        self.buffer.set_purgeable_state(metal::MTLPurgeableState::Empty);
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_from_slice() {
        let dev = metal::Device::system_default().unwrap();
        let vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
        println!("capacity: {}", vec.capacity());

        for e in 0..vec.len() {
            assert!(vec[e] == e);
        }
    }

    #[test]
    fn test_index() {
        let dev = metal::Device::system_default().unwrap();
        let vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        assert!(vec[3] == 3);
        assert!(vec[4] == 4);
        assert!(vec[5] == 5);
        assert!(vec[6] == 6);
    }

    #[test]
    fn test_index_mut() {
        let dev = metal::Device::system_default().unwrap();
        let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);

        vec[0] = 8;
        vec[1] = 8;
        vec[2] = 8;
        vec[3] = 8;
        vec[4] = 8;
        vec[5] = 8;
        vec[6] = 8;

        assert!(vec[0] == 8);
        assert!(vec[1] == 8);
        assert!(vec[2] == 8);
        assert!(vec[3] == 8);
        assert!(vec[4] == 8);
        assert!(vec[5] == 8);
        assert!(vec[6] == 8);
    }

    #[test]
    fn test_extend() {
        let dev = metal::Device::system_default().unwrap();
        let v = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_slice(&dev, &v);
        vec.extend(v.into_iter());

        assert!(vec.len() == 14);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        assert!(vec[3] == 3);
        assert!(vec[4] == 4);
        assert!(vec[5] == 5);
        assert!(vec[6] == 6);

        assert!(vec[7] == 0);
        assert!(vec[8] == 1);
        assert!(vec[9] == 2);
        assert!(vec[10] == 3);
        assert!(vec[11] == 4);
        assert!(vec[12] == 5);
        assert!(vec[13] == 6);
    }

    #[test]
    fn test_extend_from_slice() {
        let dev = metal::Device::system_default().unwrap();
        let v = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_slice(&dev, &v);

        vec.extend_from_slice(&v);

        assert!(vec.len() == 14);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        assert!(vec[3] == 3);
        assert!(vec[4] == 4);
        assert!(vec[5] == 5);
        assert!(vec[6] == 6);

        assert!(vec[7] == 0);
        assert!(vec[8] == 1);
        assert!(vec[9] == 2);
        assert!(vec[10] == 3);
        assert!(vec[11] == 4);
        assert!(vec[12] == 5);
        assert!(vec[13] == 6);
    }

    #[test]
    fn test_push() {
        let dev = metal::Device::system_default().unwrap();

        let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
        assert!(vec.len() == 7);

        vec.push(7);

        assert!(vec.len() == 8);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        assert!(vec[3] == 3);
        assert!(vec[4] == 4);
        assert!(vec[5] == 5);
        assert!(vec[6] == 6);
        assert!(vec[7] == 7);
    }

    #[test]
    fn test_insert() {
        let dev = metal::Device::system_default().unwrap();
        let mut vec = GPUVec::from_slice(&dev, &[0,1,2,4,5,6]);
        vec.insert(3, 3);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        assert!(vec[3] == 3);
        assert!(vec[4] == 4);
        assert!(vec[5] == 5);
        assert!(vec[6] == 6);
    }

    #[test]
    fn test_truncate() {
        let dev = metal::Device::system_default().unwrap();
        let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
        vec.truncate(3);

        assert!(vec.len() == 3);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        assert!(vec.get(3) == None);
    }

    #[test]
    fn test_remove() {
        let dev = metal::Device::system_default().unwrap();
        let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
        vec.remove(3);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        assert!(vec[3] == 4);
        assert!(vec[4] == 5);
        assert!(vec[5] == 6);
    }

    #[test]
    fn test_iter() {
        let dev = metal::Device::system_default().unwrap();
        let mut v1 = GPUVec::from_slice(&dev, &[1,2,3]);
        let mut v1_iter = v1.iter();

        // iter() returns an iterator of slices.
        assert_eq!(v1_iter.next(), Some(&1));
        assert_eq!(v1_iter.next(), Some(&2));
        assert_eq!(v1_iter.next(), Some(&3));
        assert_eq!(v1_iter.next(), None);
    }

    #[test]
    fn test_into_iter() {
        let dev = metal::Device::system_default().unwrap();
        let mut v1 = GPUVec::from_slice(&dev, &[1,2,3]);
        let mut v1_iter = v1.into_iter();

        // into_iter() returns an iterator from a value.
        assert_eq!(v1_iter.next(), Some(1));
        assert_eq!(v1_iter.next(), Some(2));
        assert_eq!(v1_iter.next(), Some(3));
        assert_eq!(v1_iter.next(), None);
    }

    #[test]
    fn test_iter_mut() {
        let dev = metal::Device::system_default().unwrap();
        let mut v1 = GPUVec::from_slice(&dev, &[1,2,3]);

        let mut v1_iter = v1.iter_mut();

        // iter_mut() returns an iterator that allows modifying each value.
        assert_eq!(v1_iter.next(), Some(&mut 1));
        assert_eq!(v1_iter.next(), Some(&mut 2));
        assert_eq!(v1_iter.next(), Some(&mut 3));
        assert_eq!(v1_iter.next(), None);
    }

    #[test]
    fn test_retain() {
        let dev = metal::Device::system_default().unwrap();
        let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
        vec.retain(|x| x % 2 == 0);
        assert!(vec.len() == 4);

        assert!(vec[0] == 0);
        assert!(vec[1] == 2);
        assert!(vec[2] == 4);
        assert!(vec[3] == 6);
    }

    #[test]
    fn test_eq() {
        let dev = metal::Device::system_default().unwrap();

        let mut a = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
        let mut b = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
        let mut c = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,7]);

        assert!(a == b);
        assert!(b != c);
    }

    #[test]
    fn test_swap_remove() {
        let dev = metal::Device::system_default().unwrap();
        let mut vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);

        let expected = GPUVec::from_slice(&dev, &[0,1,2,6,4,5]);

        let res = vec.swap_remove(3);

        assert!(res == 3);
        assert!(expected == vec);
    }

    #[test]
    fn test_clone() {
        let dev = metal::Device::system_default().unwrap();
        let vec = GPUVec::from_slice(&dev, &[0,1,2,3,4,5,6]);
        let copy = vec.clone();
        assert!(vec.len() == copy.len());
        assert!(vec.capacity() == copy.capacity());

        assert!(copy[0] == 0);
        assert!(copy[1] == 1);
        assert!(copy[2] == 2);
        assert!(copy[3] == 3);
        assert!(copy[4] == 4);
        assert!(copy[5] == 5);
        assert!(copy[6] == 6);
    }

    #[test]
    fn test_drain() {
        let dev = metal::Device::system_default().unwrap();
        let mut v: GPUVec<u32> = GPUVec::from_slice(&dev, &[1, 2, 3]);
        let u: Vec<_> = v.drain(1..).collect();
        assert!(v.iter().eq([1].iter()));
        assert!(u.iter().eq([2,3].iter()));
    }

    #[test]
    fn test_splice() {
        let dev = metal::Device::system_default().unwrap();
        let mut v = GPUVec::from_slice(&dev, &[1, 2, 3]);

        let new = [7, 8];
        let u: Vec<_> = v.splice(..2, new.iter().cloned()).collect();

        // println!("len: {}", v.len());
        assert!(v.iter().eq([7,8,3].iter()));
        assert!(u.iter().eq([1,2].iter()));
        // let expected = vec![7, 8, 3];
        // dbg!("{}", &result);
        // assert!(result == expected);


        // assert_eq!(u, &[1, 2]);
    }

    #[test]
    fn test_drain_filter() {
        // let dev = metal::Device::system_default().unwrap();
        // let mut v = GPUVec::from_slice(&dev, &[1, 2, 3]);

        // let new = [7, 8];
        // let u: Vec<_> = v.splice(..2, new.iter().cloned()).collect();

        // assert!(v.iter().eq([7,8,3].iter()));
        // assert!(u.iter().eq([1,2].iter()));
        // // let expected = vec![7, 8, 3];
        // // dbg!("{}", &result);
        // // assert!(result == expected);


        // // assert_eq!(u, &[1, 2]);
    }
}


