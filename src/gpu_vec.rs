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
};

use std::hash::{
    Hash,
    Hasher
};


pub struct GPUVec<T: Copy> {
    device: metal::Device,
    buffer: metal::Buffer,
    len: usize,
    capacity: usize,
    phantom: std::marker::PhantomData<T>
}

impl<T: Copy> GPUVec<T> {
    pub fn new(device: &metal::DeviceRef, capacity: usize) -> Self {
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

    // pub fn new_with_page(device: &'a metal::DeviceRef, page: usize) -> Self {
    //     todo!()
    //     // Self {
    //     //     device,
    //     //     buffer,
    //     //     len: 0,
    //     //     capacity,
    //     //     phantom: std::marker::PhantomData
    //     // }
    // }

    pub fn from_iter(device: &metal::DeviceRef, data: &[T]) -> Self {
        let len = data.len();
        let mut ret = Self::new(device, len);

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

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.buffer.contents() as *const T
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.buffer.contents() as *mut T
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
    //
    // returns an offset into the array that can accomodate n indices
    //
    //
    // pub fn alloc(&mut self, n: usize) -> GPUAlloc<'a, T> {
    //     self.resize(self.len() + n);

    //     let offset = self.len();

    //     let slice = unsafe {
    //         std::slice::from_raw_parts_mut(
    //             self.as_mut_ptr().offset(offset as isize),
    //             n
    //         )
    //     };

    //     self.len += n;
    //     GPUAlloc::new(offset, slice)
    // }

    /// Reserves space for at least `addtional` more elements;
    pub fn reserve(&mut self, additional: usize) {
        self.resize(self.capacity() + additional)
    }

    /// untested
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.set_len(len)
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

    // pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
    // where
    //     R: RangeBounds<usize>,
    // {
    //     // Memory safety
    //     //
    //     // When the Drain is first created, it shortens the length of
    //     // the source vector to make sure no uninitialized or moved-from elements
    //     // are accessible at all if the Drain's destructor never gets to run.
    //     //
    //     // Drain will ptr::read out the values to remove.
    //     // When finished, remaining tail of the vec is copied back to cover
    //     // the hole, and the vector length is restored to the new length.
    //     //
    //     let len = self.len();
    //     let start = match range.start_bound() {
    //         Included(&n) => n,
    //         Excluded(&n) => n + 1,
    //         Unbounded => 0,
    //     };
    //     let end = match range.end_bound() {
    //         Included(&n) => n + 1,
    //         Excluded(&n) => n,
    //         Unbounded => len,
    //     };
    //     assert!(start <= end);
    //     assert!(end <= len);

    //     unsafe {
    //         // set self.vec length's to start, to be safe in case Drain is leaked
    //         self.set_len(start);
    //         // Use the borrow in the IterMut to indicate borrowing behavior of the
    //         // whole Drain iterator (like &mut T).
    //         let range_slice = slice::from_raw_parts_mut(self.as_mut_ptr().add(start), end - start);
    //         Drain {
    //             tail_start: end,
    //             tail_len: len - end,
    //             iter: range_slice.iter(),
    //             vec: NonNull::from(self),
    //         }
    //     }
    // }

    pub fn replace_subrange<I>(
        &mut self,
        subrange: std::ops::Range<usize>,
        replace_with: I
    ) //-> Splice<<I as IntoIterator>::IntoIter>
    where
        I: IntoIterator<Item = T>,
        {

        unsafe {
            let mut new_elements_vec: Vec<T> = replace_with.into_iter().collect();
            let new_len = new_elements_vec.len();
            let new_elements = new_elements_vec.as_mut_ptr();

            let old_len = self.len();
            let erase_count = subrange.len();

            let growth = new_len - erase_count;

            if growth > 0 {
                self.reserve(growth);
            }
            else {
                self.set_len(old_len + growth);
            }

            let elements = self.as_mut_ptr();
            let old_tail_index = subrange.end;
            let mut old_tail_start = &elements.offset(old_tail_index as isize);
            let new_tail_index = old_tail_index + growth;
            let mut new_tail_start = old_tail_start.offset(growth as isize);
            let tail_count = old_len - subrange.end;

            if growth > 0 {
                // Slide the tail part of the buffer forwards, in reverse order
                // so as not to self-clobber.
                // newTailStart.moveInitialize(from: oldTailStart, count: tailCount)
                std::ptr::copy(
                    old_tail_start,
                    &mut new_tail_start,
                    tail_count
                );

                // Assign over the original subrange
                let mut i = 0;
                for j in subrange {
                    *elements.offset(j as isize) = new_elements_vec[i];
                    i += 1;
                }
                // Initialize the hole left by sliding the tail forward
                for j in old_tail_index..new_tail_index {
                    *elements.offset(j as isize) = new_elements_vec[i];
                    i += 1;
                }
            }
            else { // We're not growing the buffer
                // Assign all the new elements into the start of the subrange
                let mut i = subrange.start;
                let j = 0; // todo
                let mut j = 0;
                for _ in 0..new_len {
                    *elements.offset(i as isize) = new_elements_vec[j];
                    i += 1;
                    j += 1;
                }

                // If the size didn't change, we're done.
                if growth == 0 {
                    return;
                }

                // Move the tail backward to cover the shrinkage.
                let shrinkage = -(growth as isize);
                if tail_count as isize > shrinkage {   // If the tail length exceeds the shrinkage
                    // Assign over the rest of the replaced range with the first
                    // part of the tail.
                    // newTailStart.moveAssign(from: oldTailStart, count: shrinkage)
                    std::ptr::copy(
                        old_tail_start,
                        &mut new_tail_start,
                        shrinkage as usize
                    );

                    // Slide the rest of the tail back
                    // oldTailStart.moveInitialize(
                        // from: oldTailStart + shrinkage, count: tailCount - shrinkage)
                    std::ptr::copy(
                        old_tail_start.offset(shrinkage),
                        *old_tail_start,
                        tail_count - shrinkage as usize
                    );
                }
                else {                      // Tail fits within erased elements
                    // Assign over the start of the replaced range with the tail
                    // newTailStart.moveAssign(from: oldTailStart, count: tailCount)

                    std::ptr::copy(
                        old_tail_start,
                        &mut new_tail_start,
                        tail_count
                    );
                    // Destroy elements remaining after the tail in subrange
                    // (newTailStart + tailCount).deinitialize(
                        // count: shrinkage - tailCount)
                }
            }
        }
    }

    // in elements, not bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn set_len(&mut self, new_len: usize) {
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

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn clear(&mut self) {
        self.set_len(0)
    }

    // untested
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr(),
                self.len()
            )
        }
    }

    // untested
    #[inline]
    pub fn as_mut_slice(&self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut_ptr(),
                self.len()
            )
        }
    }

    pub fn iter(&self) -> Iter<T> {
        todo!()
        // Iter {
        //     len: self.len,
        //     inner: self.items.iter().enumerate(),
        // }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        todo!()
        // IterMut {
        //     len: self.len,
        //     inner: self.items.iter_mut().enumerate(),
        // }
    }


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

impl<T: Copy + std::fmt::Display> std::fmt::Display for GPUVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for i in 0..self.len() {
            writeln!(f, "{}", self[i]);
        }
        Ok(())
    }
}

impl<T: Copy> Into<metal::Buffer> for GPUVec<T> {
    fn into(self) -> metal::Buffer {
        self.buffer
    }
}

impl<T: Copy> std::ops::Deref for GPUVec<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Copy> std::ops::DerefMut for GPUVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

// impl<'a, T: Copy> Into<&'a metal::BufferRef> for GPUVec<T> {
//     fn into(self) -> &'a metal::BufferRef {
//         self.buffer.as_ref()
//         // todo!()
//     }
// }

impl<T: Copy> Clone for GPUVec<T> {
    fn clone(&self) -> Self {
        let byte_capacity = self.byte_capacity();
        let buffer = self.device.new_buffer(
            byte_capacity as u64,
            metal::MTLResourceOptions::CPUCacheModeDefaultCache
        );

        unsafe {
            std::ptr::copy(
                self.as_ptr(),
                buffer.contents() as *mut T,
                self.len()
            );
        }
        Self {
            device: self.device.to_owned(),
            buffer,
            len: 0,
            capacity: self.capacity(),
            phantom: std::marker::PhantomData
        }
    }
}

// impl <'a> GPUVec<'a, nvg::renderer::Vertex> {
//     fn extend_from_path()
// }

// impl<'a, T: Copy> IntoIterator for GPUVec<'a, T> {
//     type Item = T;
//     type IntoIter = GPUVecIterator<T>;
//     fn into_iter(self) -> Self::IntoIter {
//         todo!();
//         // unsafe {
//             // GPUVecIterator {
//             //     ptr: self.as_ptr(),
//             //     len: self.len(),
//             //     index: 0,
//             // }
//         // }
//     }
// }

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

impl<T: Hash + Copy> Hash for GPUVec<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
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

// impl<T: Copy + std::fmt::Debug> std::fmt::Debug for GPUVec<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for e in self.iter() {
//             // write!(f, "{}", e)
//         }
//         Ok(())
//     }
// }

// use core::iter::{self, Extend, FromIterator, FusedIterator};
impl<'a, T: Copy> std::iter::FusedIterator for Iter<'a, T> {}

// #[derive(Debug)]
pub struct IterMut<'a, T: Copy> {
    idx: usize,
    inner: &'a mut GPUVec<T>
}

impl<'a, T: Copy> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // if self.idx >= self.inner.len() {
        //     None
        // }
        // else {
        //     // let mut ret = &mut self.inner[self.idx];
        //     // self.idx += 1;
        //     // Some(ret)
        //     Some(&mut self.inner[self.idx])
        // }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.inner.len();
        (l, Some(l))
    }
}

impl<'a, T: Copy> IntoIterator for &'a mut GPUVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// impl<T> IntoIterator for Vec<T>
// impl<'a, T> IntoIterator for &'a Vec<T>
// impl<'a, T> IntoIterator for &'a mut Vec<T>



// // impl<T: Copy> GPUVecIterator<T> {
// //     pub fn new(vec: ) -> Self {
// //         unsafe {
// //             GPUVecIterator {
// //                 ptr: self.as_ptr(),
// //                 len: self.len(),
// //                 index: 0,
// //             }
// //         }
// //     }
// // }

// impl<T: Copy> Iterator for GPUVecIterator<T> {
//     type Item = &T;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.index < self.len {
//             unsafe {
//                 let ptr = self.ptr.offset(self.index as isize);
//                 self.index += 1;
//                 ptr
//             }
//         }
//         else {
//             None
//         }
//     }
// }

// // impl<'a, T: Copy> IntoIterator for GPUVec<'a, T> {
// //     type Item = T;
// //     type IntoIter = GPUVecIterator<T>;
// //     fn into_iter(self) -> Self::IntoIter {
// //         unsafe {
// //             GPUVecIterator {
// //                 ptr: self.as_ptr(),
// //                 len: self.len(),
// //                 index: 0,
// //             }
// //         }
// //     }
// // }

unsafe impl<T: Copy> Send for GPUVec<T> { }
unsafe impl<T: Copy> Sync for GPUVec<T> { }

// impl<T: Copy> Drop for GPUVec<T> {
//     fn drop(&mut self) {
//         // println!("Dropping!");
//     }
// }

// impl<T: Copy> Copy for GPUVec<T> {
//     fn copy(&self) -> Self {
//         todo!()
//     }
// }

mod tests {
    use super::*;

    #[test]
    fn test_roundup() {
        assert!(round_up(1, 4096) == 4096);
        assert!(round_up(4095, 4096) == 4096);
        assert!(round_up(4096, 4096) == 4096);
        assert!(round_up(4097, 4096) == 2 * 4096);
        assert!(round_up(2 * 4096 + 1, 4096) == 3 * 4096);
    }

    #[test]
    fn test_new() {
        let dev = metal::Device::system_default().unwrap();
        let vec: Vec<usize> = vec![0,1,2,3,4,5,6];
        let gpuvec = GPUVec::from_iter(&dev, &vec);
        println!("capacity: {}", gpuvec.capacity());

        for e in 0..gpuvec.len() {
            assert!(gpuvec[e] == e);
        }
    }

    #[test]
    fn test_index() {
        let dev = metal::Device::system_default().unwrap();
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];
        let vec = GPUVec::from_iter(&dev, &v);

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
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_iter(&dev, &v);

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
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_iter(&dev, &v);
        vec.extend(v);

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
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_iter(&dev, &v);
        vec.extend_from_slice(&v);
        // assert!(ret == v.len());

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
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];

        let mut vec = GPUVec::from_iter(&dev, &v);
        vec.push(7);

        assert!(v.len() == 7);
        assert!(vec.len() == 8);
        // assert!(vec.len() == v.len() + 1);

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
        let v: Vec<usize> = vec![0,1,2,4,5,6];
        let mut vec = GPUVec::from_iter(&dev, &v);
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
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_iter(&dev, &v);
        vec.truncate(3);

        assert!(vec.len() == 3);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        // assert!(vec[3] == 3);
        // assert!(vec[3] == 3);
        // assert!(vec[4] == 4);
        // assert!(vec[5] == 5);
        // assert!(vec[6] == 6);
    }

    #[test]
    fn test_remove() {
        let dev = metal::Device::system_default().unwrap();
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_iter(&dev, &v);
        vec.remove(3);

        assert!(vec[0] == 0);
        assert!(vec[1] == 1);
        assert!(vec[2] == 2);
        assert!(vec[3] == 4);
        assert!(vec[4] == 5);
        assert!(vec[5] == 6);
    }

    // #[test]
    // fn test_iter() {
    //     let dev = metal::Device::system_default().unwrap();
    //     let mut vec: Vec<usize> = vec![0,1,2,3,4,5,6];
    //     let gpuvec = GPUVec::from_iter(&dev, &vec);

    //     // let z = gpuvec.iter().
    //     let sum = gpuvec.into_iter().fold(0, |a, b| a + b );
    //     assert!(sum == 21);
    // }

    #[test]
    fn test_retain() {
        let dev = metal::Device::system_default().unwrap();
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_iter(&dev, &v);
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
        let va: Vec<usize> = vec![0,1,2,3,4,5,6];
        let vb: Vec<usize> = vec![0,1,2,3,4,5,6];
        let vc: Vec<usize> = vec![0,1,2,3,4,5,7];

        let mut a = GPUVec::from_iter(&dev, &va);
        let mut b = GPUVec::from_iter(&dev, &vb);
        let mut c = GPUVec::from_iter(&dev, &vc);

        assert!(a == b);
        assert!(b != c);
    }

    #[test]
    fn test_replace_subrange() {
        let dev = metal::Device::system_default().unwrap();
        let a: Vec<usize> = vec![0,1,2,3,4,5,6];

        let mut vec = GPUVec::from_iter(&dev, &a);


        /// same size
        vec.replace_subrange(0..2, vec![10, 11]);

        assert!(vec[0] == 10);
        assert!(vec[1] == 11);
        assert!(vec[2] == 2);

        vec.replace_subrange(0..2, vec![10, 11, 12]);

        println!("{}", vec);

        assert!(vec[0] == 10);
        assert!(vec[1] == 11);
        assert!(vec[2] == 12);
        assert!(vec[3] == 2);



        // assert!(vec[3] == 6);

        // let mut b = GPUVec::from_iter(&dev, &vb);
        // let mut c = GPUVec::from_iter(&dev, &vc);

        // assert!(a == b);
        // assert!(b != c);
    }

    #[test]
    fn test_swap_remove() {
        let dev = metal::Device::system_default().unwrap();
        let v: Vec<usize> = vec![0,1,2,3,4,5,6];
        let mut vec = GPUVec::from_iter(&dev, &v);

        let e: Vec<usize> = vec![0,1,2,6,4,5];
        let expected = GPUVec::from_iter(&dev, &e);

        let res = vec.swap_remove(3);

        assert!(res == 3);
        assert!(expected == vec);
    }
}


