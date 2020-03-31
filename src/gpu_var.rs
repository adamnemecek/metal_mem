use crate::{
    page_aligned,
};

pub struct GPUVar<T: Copy> {
    device: metal::Device,
    buffer: metal::Buffer,
    phantom: std::marker::PhantomData<T>
}

impl<T: Copy> GPUVar<T> {
    pub fn new(device: &metal::DeviceRef, value: T) -> Self {
        let byte_capacity = page_aligned(std::mem::size_of::<T>()) as u64;
        let buffer = device.new_buffer(byte_capacity, metal::MTLResourceOptions::CPUCacheModeDefaultCache);
        let mut ret = Self {
            device: device.to_owned(),
            buffer,
            phantom: std::marker::PhantomData
        };
        ret.set_value(value);
        ret
    }

    pub fn value(&self) -> T {
        unsafe {
            *(self.buffer.contents() as *const T)
        }
    }

    pub fn set_value(&mut self, value: T) {
        unsafe {
            std::ptr::copy(
                &value,
                self.buffer.contents() as *mut T,
                std::mem::size_of::<T>()
            );
        }
    }
}

impl<T: Copy> AsRef<metal::Buffer> for GPUVar<T> {
    #[inline]
    fn as_ref(&self) -> &metal::Buffer {
        &self.buffer
    }
}


mod tests {
    use crate::GPUVar;
    #[test]
    fn test_var() {
        let dev = metal::Device::system_default().unwrap();
        let mut var = GPUVar::new(&dev, 10);
        assert!(var.value() == 10);

        var.set_value(20);
        assert!(var.value() == 20);
    }
}