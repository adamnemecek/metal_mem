
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

#[derive(PartialEq, Eq)]
pub struct PagedAlloc {
    byte_size: usize,

    element_size: usize,

    count: usize,
    remainder: usize
}

impl PagedAlloc {
    pub fn validate(&self) -> bool {
        todo!()
    }

    pub fn new(element_size: usize, count: usize, page_size: usize) -> Self {
        todo!()
    }
}




