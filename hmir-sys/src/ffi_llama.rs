//! Raw bindings capturing the `llama.cpp` structures and parameters.
//!
//! Because `llama_context` is an opaque pointer in C, we treat it purely as a generic pointer 
//! handle in Rust. We use strict `repr(C)` forcing the compiler to output standard ABIs.

// C-types bridging over libc
use core::ffi::{c_void, c_int, c_float, c_char, c_uint};

/// Opaque pointer to the llama_model instantiation 
pub type LlamaModelPtr = *mut c_void;

/// Opaque pointer to the execution context (holds the KV cache mapped for a specific layer)
pub type LlamaContextPtr = *mut c_void;

/// Struct matching `llama_context_params` logically. Size must equal 64 bytes on a 64-bit OS.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LlamaContextParams {
    pub n_ctx: c_uint,             // 4 bytes
    pub n_batch: c_uint,           // 4 bytes
    pub n_threads: c_uint,         // 4 bytes
    pub n_threads_batch: c_uint,   // 4 bytes
    pub rope_freq_base: c_float,   // 4 bytes
    pub rope_freq_scale: c_float,  // 4 bytes
    pub type_k: c_int,             // 4 bytes
    pub type_v: c_int,             // 4 bytes
    pub offload_kqv: bool,         // 1 byte
    pub flash_attn: bool,          // 1 byte
    
    // Padding logic to push exactly back to 64 alignment.
    _padding: [u8; 30],
}

// Ensure the struct default matches what `llama.cpp` natively sets up
impl Default for LlamaContextParams {
    fn default() -> Self {
        LlamaContextParams {
            n_ctx: 512,
            n_batch: 512,
            n_threads: 4,
            n_threads_batch: 4,
            rope_freq_base: 10000.0,
            rope_freq_scale: 1.0,
            type_k: 0, // f16 default
            type_v: 0, // f16 default
            offload_kqv: true,
            flash_attn: false,
            _padding: [0; 30],
        }
    }
}

// -----------------------------------------------------------------------------
// EXTERNAL BINDINGS
// -----------------------------------------------------------------------------

extern "C" {
    /// Loads a model from an MMAP file instead of pushing raw bytes over buses.
    pub fn llama_load_model_from_file(
        path_model: *const c_char,
        params: *mut c_void // Placeholder for llama_model_params
    ) -> LlamaModelPtr;

    /// Generates a new context tied directly to the PagedAttention memory manager bounds
    pub fn llama_new_context_with_model(
        model: LlamaModelPtr,
        params: LlamaContextParams
    ) -> LlamaContextPtr;

    /// Direct C call triggering continuous batch evaluation 
    pub fn llama_decode(
        ctx: LlamaContextPtr,
        batch: *mut c_void // Placeholder struct pointer to the sequences
    ) -> c_int;
}
