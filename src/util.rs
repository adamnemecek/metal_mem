
///
/// This is needed since some structs need a different alignment
/// e.g. fraguniform
///
// pub trait ByteSized {
//     fn size_of() -> usize;
// }

// impl ByteSized for u32 {
//     fn size_of() -> usize {
//         std::mem::size_of::<Self>()
//     }
// }

// impl ByteSized for usize {
//     fn size_of() -> usize {
//         std::mem::size_of::<Self>()
//     }
// }

// impl ByteSized for nvg::renderer::Vertex {
//     fn size_of() -> usize {
//         std::mem::size_of::<Self>()
//     }
// }

// impl ByteSized for crate::FragUniforms {
//     fn size_of() -> usize {
//         256
//     }
// }

// impl ByteSized for ViewSize {
//     fn size_of() -> usize {
//         2 * std::mem::size_of::<f32>()
//     }
// }

// pub(crate)
pub fn round_up(x: usize, to: usize) -> usize {
    let m = x % to;
    if m == 0 {
        x
    }
    else {
        x - m + to
    }
}

// pub(crate)
pub fn page_aligned(size: usize) -> usize {
    round_up(size, 4096)
}

/// `MemAlign` represents metadata for a page alligned allocation.
///
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct MemAlign<T> {
    pub byte_size: usize,
    pub capacity: usize,
    pub remainder: usize,
    phantom: std::marker::PhantomData<T>
}

impl<T> MemAlign<T> {
    pub fn element_size() -> usize {
        std::mem::size_of::<T>()
    }

    pub fn is_valid(&self) -> bool {
        (Self::element_size() * self.capacity) + self.remainder == self.byte_size
    }

    pub fn new(capacity: usize) -> Self {
        let element_size = Self::element_size();
        let size = element_size * capacity;

        let byte_size = page_aligned(size);
        let remainder = byte_size % element_size;
        assert!((byte_size - remainder) % element_size == 0);
        let capacity = (byte_size - remainder) / element_size;
        assert!(byte_size != 0);

        Self {
            byte_size,
            capacity,
            remainder,
            phantom: Default::default()
        }
    }
}

pub trait BufferAllocator<T> {
    type Output;
    type Opts;
    fn new_mem(&self, mem_align: MemAlign<T>, opts: Self::Opts) -> Self::Output;
}

impl<T> BufferAllocator<T> for metal::DeviceRef {
    type Output = metal::Buffer;
    type Opts = metal::MTLResourceOptions;
    fn new_mem(&self, mem_align: MemAlign<T>, opts: Self::Opts) -> Self::Output {
        self.new_buffer(mem_align.byte_size as u64, opts)
    }
}

pub trait AsPtr<T> {
    fn as_ptr(&self) -> *const T;
}

pub trait AsMutPtr<T> {
    fn as_mut_ptr(&self) -> *mut T;
}

impl<'a, T> AsPtr<T> for metal::Buffer {
    fn as_ptr(&self) -> *const T {
        self.contents() as *const T
    }
}

impl<T> AsMutPtr<T> for metal::Buffer{
    fn as_mut_ptr(&self) -> *mut T {
        self.contents() as *mut T
    }
}
