use crate::telemetry::TelemetryEvent;

pub struct Recommendation {
    pub tier: String,
    pub model_name: String,
    pub model_id: String,
    pub reason: String,
    pub stats: String,
    pub command: String,
}

pub struct ModelRecommender {}

impl ModelRecommender {
    pub fn suggest_for_hardware(event: &TelemetryEvent) -> Vec<Recommendation> {
        if let TelemetryEvent::HardwareState {
            cpu_name: _,
            gpu_name,
            npu_name,
            cpu_temp,
            ram_total: _,
            ..
        } = event
        {
            let mut recs = Vec::new();

            if *cpu_temp > 80.0 {
                recs.push(Recommendation {
                    tier: "EFFICIENCY TIER".to_string(),
                    model_name: "Phi-3 Mini (4K Instruct)".to_string(),
                    model_id: "phi3-mini".to_string(),
                    reason: format!("LOW-POWER mode active due to high thermals ({:.1}°C)", cpu_temp),
                    stats: "GGUF on CPU or light GPU fallback".to_string(),
                    command: "hmir start --model phi3-mini".to_string(),
                });
            } else if npu_name != "None" {
                recs.push(Recommendation {
                    tier: "ELITE TIER".to_string(),
                    model_name: "Qwen 2.5 1.5B (INT4 OpenVINO)".to_string(),
                    model_id: "qwen2.5-1.5b-ov".to_string(),
                    reason: "NATIVE NPU ACCELERATION available via Intel/Qualcomm".to_string(),
                    stats: "~120 T/s | Ultra-low Power | 0% CPU Overhead".to_string(),
                    command: "hmir start --model qwen2.5-1.5b-ov".to_string(),
                });

                recs.push(Recommendation {
                    tier: "BALANCED TIER".to_string(),
                    model_name: "Phi-3 Mini (INT4 OpenVINO)".to_string(),
                    model_id: "phi3-mini-ov".to_string(),
                    reason: "Smaller NPU-friendly pack for highly interactive workloads".to_string(),
                    stats: "Lower memory footprint | Fast startup".to_string(),
                    command: "hmir start --model phi3-mini-ov".to_string(),
                });
            } else if gpu_name.to_uppercase().contains("APPLE") || npu_name.contains("ANE") {
                recs.push(Recommendation {
                    tier: "ELITE TIER (UNIFIED)".to_string(),
                    model_name: "Llama 3.2 3B (MLX)".to_string(),
                    model_id: "llama3.2-3b-mlx".to_string(),
                    reason: "Apple Silicon detected. Utilizing Unified Memory and ANE via MLX Bridge.".to_string(),
                    stats: "Direct metal access | Zero-copy VRAM | Ultra-efficient".to_string(),
                    command: "hmir start --model llama3.2-3b-mlx".to_string(),
                });
            } else if gpu_name.to_uppercase().contains("NVIDIA") {
                recs.push(Recommendation {
                    tier: "PERFORMANCE TIER".to_string(),
                    model_name: "Llama 3 8B (TRT/CUDA)".to_string(),
                    model_id: "llama3-8b-trt".to_string(),
                    reason: "NVIDIA GPU detected. High-throughput path via TensorRT Bridge.".to_string(),
                    stats: "Maximum T/s | FP16 Precision | Dedicated VRAM".to_string(),
                    command: "hmir start --model llama3-8b-trt".to_string(),
                });
            } else if gpu_name.to_uppercase().contains("AMD") || gpu_name.to_uppercase().contains("RADEON") {
                recs.push(Recommendation {
                    tier: "PERFORMANCE TIER".to_string(),
                    model_name: "Llama 3.2 3B (GGUF/ROCm)".to_string(),
                    model_id: "llama3.2-3b".to_string(),
                    reason: "AMD GPU detected. Optimized execution via ROCm/MIGraphX Bridge.".to_string(),
                    stats: "Hardware-accelerated GGUF | Low latency".to_string(),
                    command: "hmir start --model llama3.2-3b".to_string(),
                });
            } else {
                recs.push(Recommendation {
                    tier: "STANDARD TIER".to_string(),
                    model_name: "Phi-3 Mini (GGUF)".to_string(),
                    model_id: "phi3-mini".to_string(),
                    reason: "CPU-dominant execution path with minimal memory pressure.".to_string(),
                    stats: "Baseline performance on any x64/ARM system".to_string(),
                    command: "hmir start --model phi3-mini".to_string(),
                });
            }

            recs
        } else {
            Vec::new()
        }
    }
}
