use hmir_sys::backends::BackendType;

pub struct ExecutionPlan {
    pub backend: BackendType,
    pub score: f32,
    pub reason: String,
}

pub struct OrchestratorPlanner {}

impl OrchestratorPlanner {
    pub fn create_plans(forced_backend: Option<String>) -> Vec<ExecutionPlan> {
        let mut plans = Vec::new();

        if let Some(forced) = forced_backend {
            let backend = match forced.to_lowercase().as_str() {
                "ov" | "openvino" => BackendType::OpenVino,
                "mlx" => BackendType::Mlx,
                "trt" | "tensorrt" => BackendType::Trt,
                "rocm" => BackendType::Rocm,
                "llama" | "llamacpp" => BackendType::LlamaCpp,
                _ => BackendType::LlamaCpp,
            };

            plans.push(ExecutionPlan {
                backend,
                score: 1.0,
                reason: format!("User explicitly forced backend: {}", forced),
            });

            return plans;
        }

        // Default heuristic if no forced backend
        plans.push(ExecutionPlan {
            backend: BackendType::OpenVino,
            score: 0.9,
            reason: "NPU Acceleration (Standard)".to_string(),
        });

        plans.push(ExecutionPlan {
            backend: BackendType::Mlx,
            score: 0.8,
            reason: "Apple Silicon Optimized".to_string(),
        });

        plans.push(ExecutionPlan {
            backend: BackendType::Trt,
            score: 0.85,
            reason: "NVIDIA TensorRT Optimized".to_string(),
        });

        plans.push(ExecutionPlan {
            backend: BackendType::LlamaCpp,
            score: 0.1,
            reason: "General Fallback".to_string(),
        });

        plans.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        plans
    }
}
