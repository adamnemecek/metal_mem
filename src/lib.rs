#![feature(allocator_api)]
#![feature(core_intrinsics)]
#![feature(exact_size_is_empty)]
#![feature(ptr_offset_from)]
#![feature(slice_partition_dedup)]
#![feature(trusted_len)]
#![feature(box_syntax)]

// mod alloc_ref;
// pub use alloc_ref::*;

mod encoder;
pub use encoder::*;

mod gpu_resource;
pub use gpu_resource::*;

mod gpu_var;
pub use gpu_var::*;

mod gpu_vec;
pub use gpu_vec::*;

// mod gpu_vec2;
// pub use gpu_vec2::*;

mod mem;
pub use mem::*;

// mod raw_vec;
// pub use raw_vec::*;

mod macros;

#[macro_use]
extern crate lazy_static;

mod prelude {}
