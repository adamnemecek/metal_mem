#![feature(core_intrinsics)]
#![feature(ptr_offset_from)]
#![feature(trusted_len)]
#![feature(exact_size_is_empty)]
#![feature(allocator_api)]

mod gpu_resource;
mod gpu_var;
mod gpu_vec;
mod gpu_vec2;
mod util;
mod raw_vec;

pub use {
    gpu_resource::*,
    gpu_var::*,
    gpu_vec::*,
    gpu_vec2::*,
    util::*,
    raw_vec::*,
};