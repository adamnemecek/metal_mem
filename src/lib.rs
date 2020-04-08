#![feature(allocator_api)]
#![feature(core_intrinsics)]
#![feature(exact_size_is_empty)]
#![feature(ptr_offset_from)]
#![feature(slice_partition_dedup)]
#![feature(trusted_len)]

mod gpu_resource;
mod gpu_var;
mod gpu_vec;
mod gpu_vec2;
mod util;
mod raw_vec;
mod alloc_ref;

pub use {
    gpu_resource::*,
    gpu_var::*,
    gpu_vec::*,
    gpu_vec2::*,
    util::*,
    raw_vec::*,
    alloc_ref::*,
};