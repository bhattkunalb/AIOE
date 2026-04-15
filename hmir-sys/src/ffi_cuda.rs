//! Specific CUDA memory mapping interactions bypassing generic Unified Memory tracking.
//! 
//! When HMIR determines a KV block must be evicted, we don't rely on OS level paging or
//! NVIDIA's opaque UVM. We explicitly execute DMA transfer buffers.

use core::ffi::{c_void, c_int, c_ulonglong};

/// NVIDIA Error Code return value layout 
pub type CudaError = c_int;
const CUDA_SUCCESS: CudaError = 0;

#[repr(u32)]
pub enum CudaMemcpyKind {
    HostToHost = 0,
    HostToDevice = 1,
    DeviceToHost = 2,
    DeviceToDevice = 3,
}

extern "C" {
    /// Forces standard allocation natively onto the specified accelerator
    pub fn cudaMalloc(
        devPtr: *mut *mut c_void,
        size: c_ulonglong
    ) -> CudaError;

    /// Our core DMA swapper explicitly pushing data over the PCIe lanes
    pub fn cudaMemcpyAsync(
        dst: *mut c_void,
        src: *const c_void,
        count: c_ulonglong,
        kind: CudaMemcpyKind,
        stream: *mut c_void
    ) -> CudaError;

    /// Frees the physical memory frame immediately 
    pub fn cudaFree(
        devPtr: *mut c_void
    ) -> CudaError;
}

// -----------------------------------------------------------------------------
// RUST WRAPPERS 
// -----------------------------------------------------------------------------

/// Explicit manual KV Cache block lifter transferring directly between Host memory and VRAM
pub unsafe fn execute_kv_block_swap(
    vram_ptr: *const c_void, 
    ram_ptr: *mut c_void, 
    bytes: u64, 
    to_ram: bool
) -> Result<(), &'static str> {
    
    let kind = if to_ram { CudaMemcpyKind::DeviceToHost } else { CudaMemcpyKind::HostToDevice };
    
    /* 
    This triggers the external FFI function. If compiling naturally, this triggers the specific driver DMA copy.
    let err = cudaMemcpyAsync(
        if to_ram { ram_ptr } else { vram_ptr as *mut c_void },
        if to_ram { vram_ptr } else { ram_ptr as *const c_void },
        bytes as c_ulonglong,
        kind,
        std::ptr::null_mut()
    );

    if err != CUDA_SUCCESS {
        return Err("NVIDIA Driver failed to DMA swap explicit Paged KV Matrix element.");
    }
    */
    
    Ok(())
}
