//! AgentVM Core — Image parsing, validation, diff, merge
//!
//! This crate provides the core data structures and operations for Agent Images.
//!
//! # Quick Start
//!
//! ```rust
//! use agentvm_core::{AgentImage, ImageValidator};
//!
//! let yaml = r#"
//! apiVersion: agentvm/v1
//! kind: AgentImage
//! metadata:
//!   name: "my-agent"
//!   version: "1.0.0"
//! identity:
//!   persona: "A helpful assistant"
//! "#;
//!
//! let image = AgentImage::from_yaml(yaml).unwrap();
//! let validator = ImageValidator::new();
//! assert!(validator.validate(&image).is_ok());
//! ```

mod checksum;
mod diff;
mod error;
mod image;
mod memory;
mod merge;
mod validator;

pub use checksum::{checksum, sha256_hex};
pub use diff::{diff, ImageDiff};
pub use error::{AgentVmError, Result};
pub use image::{
    AgentImage, Identity, MemoryConfig, Metadata, ModelPreference, RegistrySkill, RuntimeConfig,
    SkillEntry, SkillsConfig, ToolsConfig,
};
pub use memory::{
    EpisodicEntry, MemoryState, ProceduralSkill, SemanticCollection, SemanticEntry, SocialContact,
};
pub use merge::merge;
pub use validator::{ImageValidator, ValidationError, ValidationReport, ValidationWarning};

/// Supported spec versions
pub const SPEC_VERSION: &str = "agentvm/v1";

/// Image format version
pub const FORMAT_VERSION: &str = "1.0.0";
