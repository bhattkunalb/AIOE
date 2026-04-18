//! HMIR Core Runtime Library
//!
//! This crate contains the hardware-agnostic allocation, execution strategy, and orchestration
//! logic for heterogeneous inference.

pub mod adapters;
pub mod memory;
pub mod orchestrator;
pub mod platform;
pub mod security;
pub mod telemetry;
pub mod topology;
