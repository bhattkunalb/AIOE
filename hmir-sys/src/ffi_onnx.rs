use core::ffi::c_void;

pub type OrtSessionPtr = *mut c_void;
pub type OrtEnvPtr = *mut c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrtStatus {
    Success = 0,
    Fail = 1,
    InvalidArgument = 2,
    NoSuchFile = 3,
    NoModel = 4,
    EngineError = 5,
    RuntimeException = 6,
    InvalidProtobuf = 7,
    ModelLoaded = 8,
    NotImplemented = 9,
    InvalidGraph = 10,
    EpFail = 11,
}

extern "C" {
    /// Creates a generic execution pipeline handle
    pub fn OrtCreateSession(
        env: OrtEnvPtr,
        model_path: *const core::ffi::c_char,
        options: *const c_void,
    ) -> OrtSessionPtr;

    /// Specific function triggering highly efficient static inference loop (for NPU drafting)
    pub fn OrtRun(
        session: OrtSessionPtr,
        run_options: *const c_void,
        input_names: *const *const core::ffi::c_char,
        input_values: *const c_void, // Tensors
        input_count: core::ffi::c_ulong,
        output_names: *const *const core::ffi::c_char,
        output_count: core::ffi::c_ulong,
        output_values: *mut *mut c_void,
    ) -> OrtStatus;
}
