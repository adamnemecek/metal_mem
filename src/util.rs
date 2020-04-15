
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

/// Paged alloc represents how many elements a page-aligned allocation.
///
#[derive(PartialEq, Eq, Debug)]
pub struct PagedAlloc<T> {
    pub aligned_byte_size: usize,

    pub element_size: usize,
    // how many elements does
    pub capacity: usize,
    pub remainder: usize,
    phantom: std::marker::PhantomData<T>
}

impl<T> PagedAlloc<T> {
    pub fn is_valid(&self) -> bool {
        (self.element_size * self.capacity) + self.remainder == self.byte_size
    }

    pub fn new(capacity: usize) -> Self {
        let element_size = std::mem::size_of::<T>();
        let size = element_size * capacity;

        let aligned_byte_size = page_aligned(size);
        let remainder = aligned_byte_size % element_size;
        assert!((aligned_byte_size - remainder) % element_size == 0);
        let capacity = (aligned_byte_size - remainder) / element_size;


        Self {
            aligned_byte_size,
            element_size,
            capacity,
            remainder,
            phantom: Default::default()
        }
    }
}




