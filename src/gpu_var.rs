use crate::{
    MemAlign,
    BufferAllocator
};

pub struct GPUVar<T: Copy> {
    device: metal::Device,
    buffer: metal::Buffer,
    mem_align: MemAlign<T>,
    phantom: std::marker::PhantomData<T>
}

impl<T: Copy> GPUVar<T> {
    pub fn new(device: &metal::DeviceRef, value: T) -> Self {
        let mem_align = MemAlign::<T>::new(1);
        // let byte_capacity = page_aligned(Self::element_size()) as u64;
        let buffer = device.new_mem(mem_align, metal::MTLResourceOptions::CPUCacheModeDefaultCache);
        let mut ret = Self {
            device: device.to_owned(),
            buffer,
            mem_align,
            phantom: std::marker::PhantomData
        };
        *ret = value;
        ret
    }

    #[inline]
    pub fn element_size() -> usize {
        std::mem::size_of::<T>()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.buffer.contents() as *const T
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.buffer.contents() as *mut T
    }

    // #[inline]
    // pub fn value(&self) -> T {
    //     unsafe {
    //         *self.as_ptr()
    //     }
    // }

    // #[inline]
    // pub fn set_value(&mut self, value: T) {
    //     unsafe {
    //         std::ptr::copy(
    //             &value,
    //             self.as_mut_ptr(),
    //             Self::element_size()
    //         );
    //     }
    // }
}

impl<T: Copy> AsRef<metal::Buffer> for GPUVar<T> {
    #[inline]
    fn as_ref(&self) -> &metal::Buffer {
        &self.buffer
    }
}

impl<T: Copy> AsMut<metal::Buffer> for GPUVar<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut metal::Buffer {
        &mut self.buffer
    }
}

impl<T: Copy> Clone for GPUVar<T> {
    fn clone(&self) -> Self {
        Self::new(&self.device, **self)
    }
}


impl<T: Copy> std::ops::Deref for GPUVar<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        unsafe {
            self.as_ptr().as_ref().unwrap()
        }
    }
}

impl<T: Copy> std::ops::DerefMut for GPUVar<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            self.as_mut_ptr().as_mut().unwrap()
        }
    }
}