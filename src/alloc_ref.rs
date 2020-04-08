
use std::alloc::{
    MemoryBlock,
    AllocRef,
    Layout,
    AllocInit,
    ReallocPlacement,
    AllocErr
};

use std::ptr::NonNull;

pub struct GPUAlloc {
    device: metal::Device,
}


unsafe impl AllocRef for GPUAlloc {
    fn alloc(
        &mut self,
        layout: Layout,
        init: AllocInit
    ) -> Result<MemoryBlock, AllocErr> {
        todo!()
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        todo!()
    }

    unsafe fn grow(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement,
        init: AllocInit
    ) -> Result<MemoryBlock, AllocErr> {
        todo!()
    }

    unsafe fn shrink(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
        placement: ReallocPlacement
    ) -> Result<MemoryBlock, AllocErr> {
        todo!()
    }
}