use crate::backends::{BackendAdapter, BackendError, TensorShape};
use crate::ffi_llama::{LlamaContextPtr, LlamaModelPtr};
use tokio::task;

pub struct LlamaCppAdapter {
    pub max_batch_size: usize,
    _model: LlamaModelPtr,
    _ctx: LlamaContextPtr,
}

impl LlamaCppAdapter {
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            max_batch_size,
            _model: std::ptr::null_mut(),
            _ctx: std::ptr::null_mut(),
        }
    }
}

impl BackendAdapter for LlamaCppAdapter {
    fn validate_shape(&self, shape: &TensorShape) -> Result<(), BackendError> {
        if shape.dim_x > self.max_batch_size {
            return Err(BackendError::ShapeValidationFailed(format!(
                "Context token batch size [{}] exceeds initialized max limit [{}]",
                shape.dim_x, self.max_batch_size
            )));
        }
        Ok(())
    }

    async fn evaluate_batch(&self) -> Result<usize, BackendError> {
        // Here we throw the synchronous blocking C-FFI request off the main async loop.
        // This prevents the orchestrator from deadlocking during continuous batching!
        
        let exec_result = task::spawn_blocking(move || {
            // Unsafe FFI C bindings trigger here:
            // unsafe { ffi_llama::llama_decode(...) }
            1 // mock token return
        }).await;

        match exec_result {
            Ok(tokens_processed) => Ok(tokens_processed),
            Err(_) => Err(BackendError::HardwareTimeout),
        }
    }
}
