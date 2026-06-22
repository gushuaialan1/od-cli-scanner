pub mod cli;
pub mod core;

pub use core::detector::detect_agents;
pub use core::probe::{NotInvocableCause, ProbeError};
pub use core::types::{
    AgentDef, AgentDiagnostic, AgentEnvConfig, DetectedAgent, DetectionResult, FixAction,
    ModelOption,
};
pub use core::types::{AuthStatus, ModelsSource};
