# 🤝 Contributing to HMIR

Thank you for your interest in contributing to **HMIR: Heterogeneous Memory-First Inference Runtime**! This guide covers architecture overview, development setup, testing methodology, and how to add new features.

---

## 🏗️ Architecture Overview

```text
HMIR Workspace (Rust 2021)
├── crates/
│   ├── hmir-core/          # Core orchestration: scheduler, memory, topology
│   ├── hmir-sys/           # FFI bindings: llama.cpp, ONNX Runtime (bundled)
│   ├── hmir-api/           # OpenAI-compatible Axum server + Prometheus metrics
│   ├── hmir-cli/           # CLI interface: suggest, start, logs, update
│   ├── hmir-dashboard/     # Native egui TaskManager UI (60 FPS telemetry)
│   ├── hmir-hardware-prober/ # Cross-platform hardware detection (cfg-gated)
│   ├── hmir-bench/         # Benchmark harness: TTFT, ITL, tokens/watt
│   └── hmir-e2e/           # End-to-end validation: daemon lifecycle + UI smoke
│
├── deploy/
│   ├── docker-compose.yml  # Multi-service setup (daemon + optional WebUI)
│   ├── Dockerfile          # Multi-stage build with GPU/NPU profiles
│   └── packaging/          # .deb, .pkg, .msi specs for native installers
│
├── scripts/
│   ├── install.sh          # Linux/macOS one-click installer
│   ├── install.ps1         # Windows PowerShell installer
│   └── migrate-repo-name.sh # Safe global find/replace for repo migrations
│
└── examples/
    ├── langchain_hmir.py   # LangChain custom LLM integration
    ├── openwebui-docker.yml # Docker Compose for Open WebUI + HMIR
    └── llamaindex_rag.rs   # Rust LlamaIndex example with HMIR API
```

**Key Design Principles**:
- **Memory-first**: KV cache is paged, non-contiguous, and swap-aware. Never copy unless necessary.
- **Zero-hot-path blocking**: Telemetry, logging, and UI updates use async channels. Inference kernels never wait.
- **Graceful degradation**: If NPU driver missing → fallback to GPU → CPU. Never panic on hardware probe failure.
- **Feature-gated**: Advanced features (`dashboard`, `openai-api`, `lora-adapters`) are optional. Core daemon builds identically without them.

---

## 🛠️ Development Setup

### Prerequisites
| Component | Version | Install Command |
|-----------|---------|----------------|
| Rust toolchain | 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| CMake | 3.20+ | `winget install Kitware.CMake` (Win) / `brew install cmake` (macOS) / `apt install cmake` (Linux) |
| Git | 2.30+ | Package manager of your OS |
| (Optional) CUDA | 12.0+ | [NVIDIA CUDA Toolkit](https://developer.nvidia.com/cuda-downloads) |
| (Optional) Metal | macOS 13+ | Built-in via Xcode Command Line Tools |

### Clone & Build
```bash
git clone https://github.com/bhattkunalb/HMIR.git
cd HMIR

# Build debug version (fast iteration)
cargo build --workspace

# Build release version (optimized)
cargo build --release --workspace --features dashboard,openai-api,hardware-prober

# Run with dashboard
cargo run --release --bin hmir-cli -- start --dashboard
```

### Feature Flags
```bash
# Minimal build (core only)
cargo build --workspace --no-default-features

# Enable specific features
cargo build --features dashboard          # Native egui UI
cargo build --features openai-api         # Axum server + /v1 endpoints
cargo build --features lora-adapters      # Dynamic LoRA loading
cargo build --features v1-1-preview       # Future performance features (adaptive draft, dynamic quant)
```

### IDE Setup
- **VS Code**: Install `rust-analyzer` extension. Add `.vscode/settings.json`:
  ```json
  {
    "rust-analyzer.cargo.features": ["dashboard", "openai-api", "hardware-prober"],
    "rust-analyzer.checkOnSave.command": "clippy"
  }
  ```
- **IntelliJ Rust**: Enable "Fetch external documentation" and "Run cargo check on the fly".

---

## 🧪 Testing Methodology

HMIR uses a multi-layered testing strategy. All PRs must pass:

### 1. Unit Tests (`cargo test --lib`)
- Test individual components in isolation
- Use `#[cfg(test)]` modules with `assert_eq!`, `proptest` for property-based testing
- Example: `paged_gather_preserves_logical_order` validates block table correctness

### 2. Integration Tests (`cargo test --test '*'`)
- Test component interactions (e.g., `DraftVerifier` + `SwapManager`)
- Use `tokio::test` for async tests
- Mock FFI calls with `mockall` or stubbed C functions

### 3. End-to-End Tests (`cargo test -p hmir-e2e`)
- Launch daemon, send API requests, verify telemetry + UI updates
- Run headless in CI, support `--gui` flag for local visual smoke tests
- Output JUnit + HTML reports for release validation

### 4. Benchmark Tests (`cargo bench -p hmir-bench`)
- Measure TTFT, ITL, tokens/sec, VRAM usage, power draw
- Compare against baseline; fail CI if regression >10%
- Output JSON for Grafana/Prometheus ingestion

### Running Tests
```bash
# Quick test suite (unit + integration)
cargo test --workspace --lib --bins

# Full test suite (includes E2E)
cargo test --workspace --all-features

# Benchmarks (release mode only)
cargo bench -p hmir-bench -- --save-baseline main

# Compare benchmarks
cargo bench -p hmir-bench -- --baseline main
```

---

## 🎨 Code Style & Quality Gates

### Formatting & Linting
```bash
# Auto-format all code
cargo fmt --all

# Run clippy with strict warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Pre-commit hook (add to .git/hooks/pre-commit)
#!/bin/bash
cargo fmt --all -- --check || exit 1
cargo clippy --workspace --all-targets -- -D warnings || exit 1
```

### Documentation Standards
- All `pub` functions must have `///` doc comments with `# Example` where applicable
- Use `tracing::instrument` for async functions to auto-log entry/exit
- Return `Result<T, CustomError>` with `thiserror`-derived variants
- Add `// SAFETY:` comments for every `unsafe` block

### Telemetry & Observability
- Emit metrics via `metrics::histogram!`, `counter!`, `gauge!`
- Log structured events with `tracing::info!`, `debug!`, `error!`
- Never block inference hot path: use `tokio::sync::broadcast` for telemetry

---

## 🚀 Adding New Features

### Adding a New Backend Adapter
1. Create `crates/hmir-sys/src/backends/new_backend_adapter.rs`
2. Implement `PagedBackendAdapter` trait:
   ```rust
   impl PagedBackendAdapter for NewBackendAdapter {
       fn register_kv_block(...) -> Result<...> { /* zero-copy mapping */ }
       fn execute_paged_attention(...) -> Result<...> { /* respect block_table */ }
       fn release_block(...) -> Result<()> { /* update ref counts */ }
   }
   ```
3. Add conditional compilation in `hmir-sys/Cargo.toml`:
   ```toml
   [features]
   new-backend = ["dep:new-backend-sys"]
   
   [dependencies]
   new-backend-sys = { version = "x.y", optional = true }
   ```
4. Update `HardwareCompatibilityMatrix` to probe for new backend
5. Add tests: correctness, performance, fallback behavior

### Adding a New CLI Command
1. Create `crates/hmir-cli/src/commands/new_command.rs`
2. Register in `crates/hmir-cli/src/main.rs`:
   ```rust
   #[derive(Subcommand)]
   enum Commands {
       // ... existing
       #[command(about = "New feature description")]
       NewCommand(new_command::Args),
   }
   ```
3. Implement logic using existing `hmir-core` APIs
4. Add `--help` documentation and examples
5. Write integration test in `crates/hmir-e2e/tests/cli_new_command.rs`

### Adding Dashboard Widget
1. Create `crates/hmir-dashboard/src/widgets/new_widget.rs`
2. Use `egui` immediate mode:
   ```rust
   pub fn show(ui: &mut egui::Ui, telemetry: &TelemetryStream) -> egui::Response {
       ui.horizontal(|ui| {
           ui.label("New Metric:");
           ui.add(egui::ProgressBar::new(telemetry.new_metric).show_percentage());
       })
   }
   ```
3. Subscribe to `TelemetrySink` via async channel
4. Ensure 60 FPS: use `ui.ctx().request_repaint()` sparingly, batch updates

---

## 📊 Contributing Benchmarks

When adding performance-sensitive code:

1. **Add benchmark case** in `crates/hmir-bench/src/scenarios/`:
   ```rust
   pub fn new_scenario() -> Scenario {
       Scenario {
           name: "new-feature-latency".into(),
           prompt_file: "prompts/short_chat.txt".into(),
           strategy: Strategy::Latency,
           expected_ttft_max_ms: 500,
           expected_itl_max_ms: 50,
           // ... other constraints
       }
   }
   ```

2. **Run baseline comparison**:
   ```bash
   cargo bench -p hmir-bench -- --save-baseline before-change
   # Apply your changes
   cargo bench -p hmir-bench -- --baseline before-change
   ```

3. **Report results** in PR description:
   ```markdown
   ## Benchmark Results
   | Metric | Before | After | Delta |
   |--------|--------|-------|-------|
   | TTFT (ms) | 420 | 380 | ✅ -9.5% |
   | ITL (ms) | 48 | 45 | ✅ -6.3% |
   | VRAM Peak (GB) | 5.2 | 5.1 | ✅ -1.9% |
   | Tokens/Watt | 4.2 | 4.8 | ✅ +14.3% |
   ```

4. **Fail CI if regression >10%** on any key metric (configurable in `hmir-bench/Cargo.toml`)

---

## 🔄 Pull Request Workflow

1. **Fork & branch**: `git checkout -b feature/your-feature-name`
2. **Implement + test**: Ensure all tests pass locally
3. **Format + lint**: `cargo fmt && cargo clippy -- -D warnings`
4. **Update docs**: Add `# Example` to new public APIs, update `README.md` if user-facing
5. **Open PR**: Use template `.github/PULL_REQUEST_TEMPLATE.md`
6. **CI checks**: Wait for GitHub Actions (Linux/macOS/Windows builds + benchmarks)
7. **Review**: Address feedback, squash commits if needed
8. **Merge**: Maintainer merges to `main`; auto-tag if version bump detected

### PR Checklist
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Benchmarks show no regression >10% (or justify if intentional)
- [ ] Code formatted (`cargo fmt`) and clippy-clean
- [ ] New public APIs documented with examples
- [ ] README/docs updated if user-facing change
- [ ] Feature flagged if experimental (`--features your-feature`)

---

## 🌐 Adding Support for New Hardware

HMIR supports heterogeneous routing via `HardwareCompatibilityMatrix`. To add new hardware:

1. **Add OS-specific probe** in `crates/hmir-hardware-prober/src/`:
   ```rust
   #[cfg(target_os = "your-os")]
   mod your_os;
   
   // In your_os.rs:
   pub fn probe_npu() -> Option<NpuVendor> {
       // Query OS APIs: sysctl, WMI, sysfs, etc.
       // Return vendor enum or None
   }
   ```

2. **Update `NpuVendor` enum** in `hmir-core/src/platform/mod.rs`:
   ```rust
   pub enum NpuVendor {
       // ... existing
       YourVendor,  // New vendor
   }
   ```

3. **Add routing logic** in `HardwareAwareDraftSelector`:
   ```rust
   impl HardwareAwareDraftSelector {
       fn should_use_npu(&self, vendor: NpuVendor) -> bool {
           match vendor {
               NpuVendor::YourVendor => self.your_vendor_policy(),
               // ... existing
           }
       }
   }
   ```

4. **Document in `docs/hardware_compatibility.md`**:
   ```markdown
   ### Your Vendor NPU
   - Driver: [Link to driver download]
   - Minimum version: X.Y
   - Known issues: [List any]
   - Fallback: GPU/CPU if driver missing
   ```

5. **Add CI test matrix entry** (if hardware available in CI):
   ```yaml
   # .github/workflows/ci.yml
   strategy:
     matrix:
       include:
         - os: ubuntu-latest
           hardware: your-npu-runner  # Self-hosted runner label
   ```

---

## 📦 Release Process

HMIR follows semantic versioning (`MAJOR.MINOR.PATCH`).

### Pre-Release Checklist
- [ ] All CI checks green on `main`
- [ ] `cargo test --workspace --all-features` passes
- [ ] Benchmarks show no regression vs. previous release
- [ ] `README.md` and `CONTRIBUTING.md` up to date
- [ ] `CHANGELOG.md` updated with user-facing changes
- [ ] Version bumped in root `Cargo.toml` and all crates

### Tag & Publish
```bash
# Tag release
git tag -a v1.1.0 -m "HMIR v1.1.0: Adaptive speculative depth + dynamic quant selector"
git push origin v1.1.0

# GitHub Actions will:
# 1. Build cross-platform binaries
# 2. Upload to Releases with checksums
# 3. (Optional) Publish to crates.io if --features publish enabled

# Verify release
# - https://github.com/bhattkunalb/HMIR/releases/tag/v1.1.0 has assets
# - Install script works on clean VMs
# - Dashboard + API function as documented
```

### Post-Release
- Update `crates.io` badge in `README.md` (uncomment if first publish)
- Announce on community channels (Discord, Matrix, Reddit)
- Monitor issues for 48 hours for regression reports

---

## 🆘 Getting Help

- **Bug reports**: Use GitHub Issues with `bug` label + `hmir logs --level debug` output
- **Feature requests**: Use GitHub Issues with `enhancement` label + use case description
- **Questions**: Start a GitHub Discussion or join our community chat
- **Security issues**: Email security@hmirlabs.dev (do not publicize until patched)

---

## 🎯 First-Time Contributor Ideas

Look for issues labeled [`good first issue`](https://github.com/bhattkunalb/HMIR/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22):

- ✨ Add telemetry metric for speculative acceptance rate
- 🐛 Fix dashboard rendering on ultra-wide monitors
- 📝 Improve error messages for NPU driver detection failures
- 🧪 Add property-based test for prefix cache ref counting
- 🌐 Add hardware probe for new Linux NPU driver

---

> 💡 **Remember**: HMIR's core philosophy is **memory-first, zero-hot-path-blocking, graceful-degradation**. Any contribution should align with these principles.

Thank you for helping make local heterogeneous inference accessible to everyone! 🚀
