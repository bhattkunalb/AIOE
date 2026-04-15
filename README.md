# 🚀 HMIR: Heterogeneous Memory-First Inference Runtime

**Run local LLMs that automatically orchestrate NPU + GPU + CPU for maximum performance per watt.**

[![GitHub Release](https://img.shields.io/github/v/release/bhattkunalb/HMIR?label=release)](https://github.com/bhattkunalb/HMIR/releases)
[![License](https://img.shields.io/github/license/bhattkunalb/HMIR)](LICENSE)
[![CI](https://github.com/bhattkunalb/HMIR/actions/workflows/ci.yml/badge.svg)](https://github.com/bhattkunalb/HMIR/actions)
<!-- [![Crates.io](https://img.shields.io/crates/v/hmir.svg)](https://crates.io/crates/hmir) --> <!-- Uncomment after first crates.io publish -->

---

## 📦 Installation

### One-Command Install

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/bhattkunalb/HMIR/main/scripts/install.sh | sh

# Windows (PowerShell — Run as Administrator for full NPU detection)
irm https://raw.githubusercontent.com/bhattkunalb/HMIR/main/scripts/install.ps1 | iex

# Docker (No install required)
docker run --gpus all -p 8080:8080 -p 3001:3001 ghcr.io/bhattkunalb/hmir:latest
```

The installer automatically downloads prebuilt binaries from GitHub Releases when available. If no release exists yet, it falls back to building from source using the Rust toolchain.

### Build from Source

```bash
git clone https://github.com/bhattkunalb/HMIR.git
cd HMIR
cargo build --release --workspace
```

> **Note:** `hmir-pyo3` (Python bindings) is excluded from the default workspace build due to Python version constraints. Build it separately with `cargo build -p hmir-pyo3` if you have Python ≤ 3.12.

---

## ⚙️ Requirements

| Component | Requirement |
| --- | --- |
| **OS** | macOS 13+, Windows 10/11 22H2+, Ubuntu 20.04+ / Fedora 36+ |
| **RAM** | 8GB minimum (16GB+ recommended for 7B models) |
| **Rust** | 1.75+ (only for source builds) |
| **Storage** | 5GB for runtime + model cache |
| **GPU Drivers** | NVIDIA CUDA 12+, Apple Metal (built-in), AMD ROCm/Vulkan (optional) |
| **NPU Drivers** | Apple Neural Engine (built-in), Intel Core Ultra AI Boost, Qualcomm Snapdragon NPU (optional, auto-fallback) |

🔍 **Do you need to install `llama.cpp` separately?**
**No.** HMIR bundles `llama.cpp` as a statically linked dependency via `hmir-sys`. Zero external setup required.

---

## 🏗️ Project Structure

```text
HMIR/
├── hmir-core/              # Core orchestration: scheduler, memory, topology
├── hmir-sys/               # FFI bindings: llama.cpp, ONNX Runtime (bundled)
├── hmir-api/               # OpenAI-compatible Axum server + Prometheus metrics
├── hmir-dashboard/         # Native egui TaskManager UI (60 FPS telemetry)
├── hmir-hardware-prober/   # Cross-platform hardware detection (cfg-gated)
├── hmir-bench/             # Benchmark harness: TTFT, ITL, tokens/watt
├── hmir-pyo3/              # Python bindings (excluded from default workspace build)
├── crates/hmir-e2e/        # End-to-end validation: daemon lifecycle + UI smoke
├── deploy/
│   ├── packaging/hmir-cli/ # CLI binary: suggest, start, logs, update
│   ├── docker-compose.yml  # Multi-service setup (daemon + optional WebUI)
│   └── Dockerfile.multi    # Multi-stage build with GPU/NPU profiles
├── scripts/
│   ├── install.sh          # Linux/macOS installer (release download + source fallback)
│   └── install.ps1         # Windows installer (NPU detection via ComputeAccelerator class)
└── examples/
    ├── langchain_hmir.py   # LangChain custom LLM integration
    ├── openwebui_docker.yml # Docker Compose for Open WebUI + HMIR
    └── llamaindex_rag.rs   # Rust LlamaIndex example with HMIR API
```

---

## 🎯 Auto-Model Recommendation

Run `hmir suggest` to get hardware-optimized model recommendations:

```bash
$ hmir suggest --strategy latency
🔍 Probing hardware...
✅ Detected: Intel Core Ultra (AI Boost NPU), Intel Arc iGPU, 32GB RAM
📊 Routing Strategy: Latency-Optimized

RECOMMENDED MODELS:
1. Meta-Llama-3-8B-Instruct-Q4_K_M.gguf
   • VRAM: ~5.2 GB | RAM: 0 GB | Expected TTFT: <450ms
   • Routing: GPU (Arc) + NPU draft → CPU fallback
   • Command: hmir load models/Meta-Llama-3-8B-Instruct-Q4_K_M.gguf

2. Phi-3-mini-4k-instruct-Q5_K_S.gguf
   • VRAM: ~3.1 GB | RAM: 0 GB | Expected TTFT: <280ms
   • Routing: NPU draft + GPU verify
   • Command: hmir load models/Phi-3-mini-4k-instruct-Q5_K_S.gguf
```

---

## 🖥️ Dashboard & API Access

### Live TaskManager UI

Launch with: `hmir start --dashboard`

- Real-time CPU/GPU/NPU utilization bars
- Active task registry with color-coded routing (🔵 GPU, 🟣 NPU, 🟠 CPU)
- Speculative acceptance rate, swap throughput, memory pressure graphs
- Controls: Pause/Resume/Kill, Strategy toggle, Hot-swap models, Force fallback

### OpenAI-Compatible API

```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "meta-llama-3-8b",
    "messages": [{"role": "user", "content": "Explain quantum entanglement simply"}],
    "stream": true
  }'
```

Metrics: `http://localhost:8080/metrics` (Prometheus-compatible)

---

## 🌐 Hardware Compatibility Matrix

| Platform | NPU | GPU | CPU | Speculative Decoding | Notes |
| --- | --- | --- | --- | --- | --- |
| Apple Silicon (M1/M2/M3) | ✅ ANE | ✅ Metal | ✅ ARM | ✅ Unified Memory optimized | Best tokens/watt |
| Intel Core Ultra | ✅ AI Boost | ✅ Arc/iGPU | ✅ x86 | ✅ NPU draft + GPU verify | Balanced hybrid |
| Windows + Snapdragon X Elite | ✅ Qualcomm QNN | ❌ | ✅ ARM | ✅ NPU draft + CPU verify | Battery champion |
| Linux + RTX 30/40 series | ❌ | ✅ CUDA | ✅ x86 | ⚠️ CPU draft + GPU verify | Max raw throughput |

> **NPU detection on Windows** uses the `ComputeAccelerator` device class via PnP/WMI. Run `Get-PnpDevice -Class ComputeAccelerator` to verify your NPU is visible.

---

## 🔧 Troubleshooting

| Issue | Fix |
| --- | --- |
| NPU not detected | Check driver installation. HMIR safely routes to GPU/CPU. Run `hmir logs --level debug` |
| VRAM OOM during long context | HMIR auto-swaps KV cache to RAM. Reduce batch size: `hmir config set batch_max_tokens 2048` |
| Dashboard blank on first run | Wait 2-3s for telemetry stream. Verify port 3001 isn't blocked. Run `hmir status` |
| Build fails on `hmir-pyo3` | Requires Python ≤ 3.12. Excluded from workspace by default — build separately if needed. |
| Install script builds from source | No prebuilt release yet. Tag a release: `git tag -a v1.x.x -m "..." && git push origin v1.x.x` |

Collect logs: `hmir logs --since 5m > hmir_debug.log`

---

## 🤝 Contributing & Releases

See [CONTRIBUTING.md](CONTRIBUTING.md) for architecture guides, testing methodology, and PR workflow.

Releases follow semantic versioning. Prebuilt binaries are generated by CI on tag push and include SHA-256 checksums. Auto-update: `hmir-cli update`

**License**: [MIT](LICENSE) | **Built with**: Rust, llama.cpp, ONNX Runtime, egui, axum, tokio
