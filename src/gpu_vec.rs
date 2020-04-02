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
        self.buffer.length() as usize
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
        self.resize(self.capacity() + 1)
    }

    /// untested
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

    // #[inline]
    // pub fn pop(&mut self) -> Option<T> {
    //     if self.len == 0 {
    //         None
    //     } else {
    //         unsafe {
    //             self.len -= 1;
    //             Some(std::ptr::read(self.get_unchecked(self.len())))
    //         }
    //     }
    // }

    // in elements, not bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());
        self.len = new_len;
    }

    // pub fn swap_remove(&mut self, index: usize) -> T {
    //     unsafe {
    //         // We replace self[index] with the last element. Note that if the
    //         // bounds check on hole succeeds there must be a last element (which
    //         // can be self[index] itself).
    //         let hole: *mut T = &mut self[index];
    //         let last = std::ptr::read(self.get_unchecked(self.len - 1));
    //         self.len -= 1;
    //         ptr::replace(hole, last)
    //     }
    // }

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
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr(),
                self.len()
            )
        }
    }

    // untested
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

impl<T: Copy> Into<metal::Buffer> for GPUVec<T> {
    fn into(self) -> metal::Buffer {
        self.buffer
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

pub struct Iter<'a, T: Copy> {
    inner: &'a GPUVec<T>,
    pos: usize,
}

impl<'a, T: Copy> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.inner.len() {
            None
        }
        else {
            let ret = &self.inner[self.pos];
            self.pos += 1;
            Some(ret)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.len();
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
        for e in self.iter() {
            // write!(f, "{}", e)
        }
        Ok(())
    }
}

// use core::iter::{self, Extend, FromIterator, FusedIterator};
impl<'a, T: Copy> std::iter::FusedIterator for Iter<'a, T> {}

#[derive(Debug)]
pub struct IterMut<'a, T: Copy> {
    len: usize,
    inner: &'a GPUVec<T>
}

impl<'a, T: Copy> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // if self.pos >= self.inner.len() {
        //     None
        // }
        // else {
        //     let ret = &self.inner[self.pos];
        //     self.pos += 1;
        //     Some(ret)
        // }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }
}

impl<'a, T: Copy> IntoIterator for &'a mut GPUVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        // self.iter_mut()
        todo!()
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
}


