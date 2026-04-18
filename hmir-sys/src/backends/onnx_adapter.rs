use crate::backends::{BackendAdapter, BackendError, TensorShape};
use crate::ffi_onnx::{OrtSessionPtr, OrtStatus};
use tokio::task;

pub struct OnnxRuntimeAdapter {
    pub max_draft_size: usize,
    _session: OrtSessionPtr,
}

impl OnnxRuntimeAdapter {
    pub fn new(max_draft_size: usize) -> Self {
        Self {
            max_draft_size,
            _session: std::ptr::null_mut(),
        }
    }
}

impl BackendAdapter for OnnxRuntimeAdapter {
    fn validate_shape(&self, shape: &TensorShape) -> Result<(), BackendError> {
        if shape.dim_x > self.max_draft_size {
            return Err(BackendError::ShapeValidationFailed(format!(
                "NPU draft length [{}] exceeds statically allocated threshold [{}]",
                shape.dim_x, self.max_draft_size
            )));
        }
        Ok(())
    }

    async fn evaluate_batch(&self) -> Result<usize, BackendError> {
        let exec_result = task::spawn_blocking(move || {
            // Unsafe FFI C bindings targeting the NPU triggers here
            // unsafe { ffi_onnx::OrtRun( ... ) }
            OrtStatus::Success // mock
        })
        .await;

        match exec_result {
            Ok(OrtStatus::Success) => Ok(10), // drafted K=10 tokens
            Ok(err_code) => Err(BackendError::OnnxExecutionFailed(err_code as i32)),
            Err(_) => Err(BackendError::HardwareTimeout),
        }
    }
}

// -----------------------------------------------------------------------------
// TESTS
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::llama_adapter::LlamaCppAdapter;

    #[test]
    fn test_shape_guardrails() {
        // Attempting to push a massive 500 element batch tensor through a
        // 128 bounded context should be blocked by the adapter's safe pre-flight logic
        // BEFORE it hits the `unsafe` block which would crash the program.

        let adapter = LlamaCppAdapter::new(128);

        let invalid_tensor = TensorShape {
            dim_x: 500,
            dim_y: 1,
            dim_z: 1,
            byte_size: 4000,
        };

        let result = adapter.validate_shape(&invalid_tensor);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            BackendError::ShapeValidationFailed(
                "Context token batch size [500] exceeds initialized max limit [128]".to_string()
            )
        );
    }

    #[tokio::test]
    async fn test_async_polling_execution() {
        let npu_adapter = OnnxRuntimeAdapter::new(32);

        // This validates the asynchronous blocking pool properly compiles and yields
        let result = npu_adapter.evaluate_batch().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);
    }
}
