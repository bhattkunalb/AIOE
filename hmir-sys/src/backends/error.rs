use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum BackendError {
    #[error("Hardware execution timeout while waiting for engine poll")]
    HardwareTimeout,
    
    #[error("Shape mismatch detected during pre-flight validation: {0}")]
    ShapeValidationFailed(String),

    #[error("C-FFI Error: LLaMA Model pointer returned null")]
    LlamaPointerUnallocated,

    #[error("C-FFI Error: ONNX Runtime Engine returned failure status -> {0}")]
    OnnxExecutionFailed(i32),
}
