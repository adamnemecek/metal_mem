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
    // GPUAlloc,
    // ViewSize,
    round_up,
    page_aligned,
};


pub struct GPUVec<T: Copy> {
    device: metal::Device,
    pub buffer: metal::Buffer,
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
        let mut ret = Self::new(device, data.len());
        let len = data.len();

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

    fn element_size() -> usize {
        std::mem::size_of::<T>()
    }

    pub fn byte_len(&self) -> usize {
        Self::element_size() * self.len()
    }

    pub fn byte_capacity(&self) -> usize {
        self.buffer.length() as usize
    }

    fn as_ptr(&self) -> *const T {
        self.buffer.contents() as *const T
    }

    fn as_mut_ptr(&self) -> *mut T {
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

    pub fn push(&mut self, x: &T) {
        // let offset = self.len();
        let len = self.len();
        self.resize(len + 1);
        self[len] = *x;
        self.len += 1;
        // offset
    }
    // in elements, not bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.set_len(0)
    }
}

impl<T: Copy> std::ops::Index<usize> for GPUVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            self.as_ptr().offset(index as isize).as_ref().unwrap()
        }
    }
}

impl<T: Copy> std::ops::IndexMut<usize> for GPUVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
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

impl<T: Copy> Clone for GPUVec<T> {
    fn clone(&self) -> Self {
        todo!()
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



pub struct GPUVecIterator<'a, T: Copy> {
    inner: &'a GPUVec<T>,
    pos: usize,
}

impl<'a, T: Copy> Iterator for GPUVecIterator<'a, T> {
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
        vec.push(&7);
        assert!(vec.len() == v.len() + 1);
        assert!(vec[vec.len()-1] == 7);
    }

    #[test]
    fn test_iter() {
        let dev = metal::Device::system_default().unwrap();
        let mut vec: Vec<usize> = vec![0,1,2,3,4,5,6];
        let gpuvec = GPUVec::from_iter(&dev, &vec);

        // let z = gpuvec.iter().
        let sum = gpuvec.into_iter().fold(0, |a, b| a + b );
        assert!(sum == 21);
    }
}


