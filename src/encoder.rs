pub trait RenderCommandEncoderExt {
    fn set_vertex_value<T>(&self, index: u64, value: &T);
    fn set_fragment_value<T>(&self, index: u64, value: &T);
}

impl RenderCommandEncoderExt for metal::RenderCommandEncoderRef {
    fn set_vertex_value<T>(&self, index: u64, value: &T) {
        let ptr = value as *const T;
        self.set_vertex_bytes(index, std::mem::size_of::<T>() as u64, ptr as *const _)
    }

    fn set_fragment_value<T>(&self, index: u64, value: &T) {
        let ptr = value as *const T;
        self.set_fragment_bytes(index, std::mem::size_of::<T>() as u64, ptr as *const _)
    }
}

pub trait ComputeCommandEncoderExt {
    fn set_value<T>(&self, index: u64, value: &T);
}

impl ComputeCommandEncoderExt for metal::ComputeCommandEncoderRef {
    fn set_value<T>(&self, index: u64, value: &T) {
        let ptr = value as *const T;
        self.set_bytes(index, std::mem::size_of::<T>() as u64, ptr as *const _)
    }
}
