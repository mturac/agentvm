use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{AgentVmError, Result};
use crate::SPEC_VERSION;

/// Top-level Agent Image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentImage {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    #[serde(default)]
    pub identity: Identity,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub skills: SkillsConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub prompts: PromptsConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub export: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub created: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub lineage: Option<Lineage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lineage {
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub forked_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Identity {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub persona: Option<String>,
    #[serde(default)]
    pub tone: Option<Tone>,
    #[serde(default)]
    pub languages: Vec<Language>,
    #[serde(default)]
    pub behaviors: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tone {
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub humor: Option<String>,
    #[serde(default)]
    pub formality: Option<String>,
    #[serde(default)]
    pub verbosity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    #[serde(default)]
    pub proficiency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryConfig {
    #[serde(default)]
    pub strategy: Option<MemoryStrategy>,
    #[serde(default)]
    pub episodic: Option<MemorySource>,
    #[serde(default)]
    pub semantic: Option<MemorySource>,
    #[serde(default)]
    pub procedural: Option<MemorySource>,
    #[serde(default)]
    pub social: Option<MemorySource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStrategy {
    #[serde(default)]
    pub consolidation_frequency: Option<String>,
    #[serde(default)]
    pub forgetting_policy: Option<String>,
    #[serde(default)]
    pub max_episodic_entries: Option<usize>,
    #[serde(default)]
    pub retrieval_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySource {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub collections: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsConfig {
    #[serde(default)]
    pub builtin: Vec<SkillEntry>,
    #[serde(default)]
    pub registry: Vec<RegistrySkill>,
    #[serde(default)]
    pub preferences: Option<SkillPreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEntry {
    pub id: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub config: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySkill {
    pub id: String,
    pub version: String,
    pub source: String,
    #[serde(default)]
    pub installed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPreferences {
    #[serde(default)]
    pub auto_activate: Option<bool>,
    #[serde(default)]
    pub conflict_resolution: Option<String>,
    #[serde(default)]
    pub max_active_skills: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    #[serde(default)]
    pub preferred: HashMap<String, ToolMapping>,
    #[serde(default)]
    pub denied: Vec<String>,
    #[serde(default)]
    pub behavior: Option<ToolBehavior>,
    #[serde(default)]
    pub security: Option<ToolSecurity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMapping {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub fallback: Option<String>,
    #[serde(default)]
    pub config: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolBehavior {
    #[serde(default)]
    pub confirm_before_exec: Option<bool>,
    #[serde(default)]
    pub max_concurrent_tools: Option<usize>,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    #[serde(default)]
    pub retry_on_failure: Option<bool>,
    #[serde(default)]
    pub max_retries: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSecurity {
    #[serde(default)]
    pub exec_policy: Option<String>,
    #[serde(default)]
    pub network_policy: Option<String>,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub denied_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptsConfig {
    #[serde(default)]
    pub system: Option<PromptSource>,
    #[serde(default)]
    pub examples: Option<PromptSource>,
    #[serde(default)]
    pub constraints: Option<PromptSource>,
    #[serde(default)]
    pub preferences: Option<PromptPreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptSource {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub max_examples: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreferences {
    #[serde(default)]
    pub chain_of_thought: Option<bool>,
    #[serde(default)]
    pub self_reflection: Option<bool>,
    #[serde(default)]
    pub confidence_reporting: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConfig {
    #[serde(default)]
    pub preferred_models: Vec<ModelPreference>,
    #[serde(default)]
    pub context: Option<ContextConfig>,
    #[serde(default)]
    pub sessions: Option<SessionConfig>,
    #[serde(default)]
    pub cost: Option<CostConfig>,
    #[serde(default)]
    pub performance: Option<PerformanceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPreference {
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub priority: Option<u8>,
    #[serde(default)]
    pub max_cost_per_turn: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextConfig {
    #[serde(default)]
    pub min_window: Option<usize>,
    #[serde(default)]
    pub preferred_window: Option<usize>,
    #[serde(default)]
    pub compaction_strategy: Option<String>,
    #[serde(default)]
    pub compaction_trigger: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    #[serde(default)]
    pub isolation_level: Option<String>,
    #[serde(default)]
    pub history_limit: Option<usize>,
    #[serde(default)]
    pub idle_timeout_minutes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostConfig {
    #[serde(default)]
    pub daily_budget_usd: Option<f64>,
    #[serde(default)]
    pub monthly_budget_usd: Option<f64>,
    #[serde(default)]
    pub alert_threshold_percent: Option<f64>,
    #[serde(default)]
    pub fallback_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceConfig {
    #[serde(default)]
    pub max_tokens_per_turn: Option<usize>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
}

impl AgentImage {
    /// Parse an Agent Image from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let image: AgentImage = serde_yaml::from_str(yaml)?;

        // Validate apiVersion
        if image.api_version != SPEC_VERSION {
            return Err(AgentVmError::UnsupportedVersion {
                version: image.api_version.clone(),
            });
        }

        Ok(image)
    }

    /// Serialize to YAML string
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(AgentVmError::from)
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(AgentVmError::from)
    }

    /// Get the agent's display name (falls back to metadata.name)
    pub fn display_name(&self) -> &str {
        self.metadata
            .display_name
            .as_deref()
            .unwrap_or(&self.metadata.name)
    }

    /// Get the agent's persona text
    pub fn persona(&self) -> &str {
        self.identity
            .persona
            .as_deref()
            .unwrap_or("A helpful AI assistant")
    }

    /// Check if a tool is denied
    pub fn is_tool_denied(&self, tool: &str) -> bool {
        self.tools.denied.iter().any(|t| t == tool)
    }

    /// Get preferred model (first by priority)
    pub fn preferred_model(&self) -> Option<&ModelPreference> {
        self.runtime
            .preferred_models
            .iter()
            .min_by_key(|m| m.priority.unwrap_or(99))
    }
}

impl Default for AgentImage {
    fn default() -> Self {
        Self {
            api_version: SPEC_VERSION.to_string(),
            kind: "AgentImage".to_string(),
            metadata: Metadata {
                name: String::new(),
                version: "0.1.0".to_string(),
                display_name: None,
                author: None,
                created: None,
                updated: None,
                description: None,
                tags: vec![],
                license: None,
                lineage: None,
            },
            identity: Identity::default(),
            memory: MemoryConfig::default(),
            skills: SkillsConfig::default(),
            tools: ToolsConfig::default(),
            prompts: PromptsConfig::default(),
            runtime: RuntimeConfig::default(),
            export: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_image() {
        let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "test-agent"
  version: "1.0.0"
"#;
        let image = AgentImage::from_yaml(yaml).unwrap();
        assert_eq!(image.metadata.name, "test-agent");
        assert_eq!(image.metadata.version, "1.0.0");
    }

    #[test]
    fn test_parse_full_image() {
        let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "my-agent"
  version: "2.0.0"
  displayName: "My Agent"
  author: "user123"
  description: "A test agent"
  tags: ["test", "demo"]

identity:
  name: "TestBot"
  emoji: "🤖"
  persona: "A helpful test assistant"
  tone:
    style: "friendly"
    verbosity: "concise"

memory:
  strategy:
    consolidationFrequency: "weekly"

skills:
  builtin:
    - id: "code-review"
      version: "1.0.0"
      enabled: true

tools:
  denied:
    - "tts"
    - "camera_snap"

runtime:
  preferredModels:
    - provider: "anthropic"
      model: "claude-sonnet-4"
      priority: 1
"#;
        let image = AgentImage::from_yaml(yaml).unwrap();
        assert_eq!(image.display_name(), "My Agent");
        assert_eq!(image.persona(), "A helpful test assistant");
        assert!(image.is_tool_denied("tts"));
        assert!(!image.is_tool_denied("exec"));
        assert_eq!(image.preferred_model().unwrap().model, "claude-sonnet-4");
    }

    #[test]
    fn test_invalid_version() {
        let yaml = r#"
apiVersion: agentvm/v2
kind: AgentImage
metadata:
  name: "test"
  version: "1.0.0"
"#;
        let result = AgentImage::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip() {
        let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "roundtrip-test"
  version: "1.0.0"
identity:
  persona: "Test agent"
"#;
        let image = AgentImage::from_yaml(yaml).unwrap();
        let serialized = image.to_yaml().unwrap();
        let deserialized = AgentImage::from_yaml(&serialized).unwrap();
        assert_eq!(deserialized.metadata.name, "roundtrip-test");
    }
}
