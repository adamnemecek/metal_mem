#![feature(allocator_api)]
#![feature(core_intrinsics)]
#![feature(exact_size_is_empty)]
#![feature(ptr_offset_from)]
#![feature(slice_partition_dedup)]
#![feature(trusted_len)]
#![feature(box_syntax)]

mod alloc_ref;
mod gpu_resource;
mod gpu_var;
mod gpu_vec;
mod gpu_vec2;
mod mem;
mod raw_vec;

mod macros;

pub use {
    alloc_ref::*, gpu_resource::*, gpu_var::*, gpu_vec::*, gpu_vec2::*, macros::*, mem::*,
    raw_vec::*,
};

#[macro_use]
extern crate lazy_static;

mod prelude {}
