// use crate::GPUVec;
use cocoa_foundation::foundation::NSUInteger;
pub trait RenderCommandEncoderExt {
    fn set_vertex_value<T>(&self, index: NSUInteger, value: &T);
    fn set_fragment_value<T>(&self, index: NSUInteger, value: &T);

    // fn set_vertex_vec<T: Copy>(&self, index: NSUInteger, vec: &GPUVec<T>, offset: NSUInteger);
    // fn set_fragment_vec<T: Copy>(&self, index: NSUInteger, vec: &GPUVec<T>, offset: NSUInteger);

    // fn set_vertex_var<T: Copy>(&self, index: NSUInteger, vec: &GPUVec<T>, offset: NSUInteger);
}

impl RenderCommandEncoderExt for metal::RenderCommandEncoderRef {
    fn set_vertex_value<T>(&self, index: NSUInteger, value: &T) {
        let ptr = value as *const T;
        self.set_vertex_bytes(index, std::mem::size_of::<T>() as u64, ptr as *const _)
    }

    fn set_fragment_value<T>(&self, index: NSUInteger, value: &T) {
        let ptr = value as *const T;
        self.set_fragment_bytes(index, std::mem::size_of::<T>() as u64, ptr as *const _)
    }

    // fn set_vertex_vec<T: Copy>(&self, index: NSUInteger, vec: &GPUVec<T>, offset: NSUInteger) {
    //     self.set_vertex_buffer(index, Some(vec.inner()), offset)
    // }

    // fn set_fragment_vec<T: Copy>(&self, index: NSUInteger, vec: &GPUVec<T>, offset: NSUInteger) {
    //     self.set_fragment_buffer(index, Some(vec.inner()), offset)
    // }
}

pub trait ComputeCommandEncoderExt {
    fn set_value<T>(&self, index: NSUInteger, value: &T);
}

impl ComputeCommandEncoderExt for metal::ComputeCommandEncoderRef {
    fn set_value<T>(&self, index: NSUInteger, value: &T) {
        let ptr = value as *const T;
        self.set_bytes(index, std::mem::size_of::<T>() as u64, ptr as *const _)
    }
}
