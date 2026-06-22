pub mod core;
pub mod cli;

pub use core::detector::detect_agents;
pub use core::types::{AgentDef, DetectedAgent, DetectionResult, ModelOption};
