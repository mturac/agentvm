use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An episodic memory entry (what happened)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEntry {
    pub date: String,
    pub summary: String,
    pub importance: f64, // 0.0 - 1.0
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub context: Option<String>,
}

/// A semantic memory entry (what the agent knows)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticEntry {
    pub key: String,
    pub value: String,
    pub confidence: f64, // 0.0 - 1.0
    pub learned: DateTime<Utc>,
    #[serde(default)]
    pub source: Option<String>, // "explicit" | "observed" | "inferred"
    #[serde(default)]
    pub times_reinforced: u32,
}

/// A semantic memory collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCollection {
    pub description: Option<String>,
    pub entries: Vec<SemanticEntry>,
}

/// A procedural skill (what the agent can do)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProceduralSkill {
    pub name: String,
    pub description: Option<String>,
    pub learned: DateTime<Utc>,
    #[serde(default)]
    pub last_used: Option<DateTime<Utc>>,
    pub confidence: f64,
    #[serde(default)]
    pub times_used: u32,
    #[serde(default)]
    pub times_failed: u32,
    #[serde(default)]
    pub trigger: Option<SkillTrigger>,
    #[serde(default)]
    pub steps: Vec<SkillStep>,
    #[serde(default)]
    pub known_issues: Vec<String>,
    #[serde(default)]
    pub lessons: Vec<Lesson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrigger {
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    pub step: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub date: String,
    pub lesson: String,
}

/// A social contact (who the agent knows)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialContact {
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub team: Option<String>,
    #[serde(default)]
    pub relationship: Option<String>,
    #[serde(default)]
    pub communication: Option<CommunicationStyle>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub working_hours: Option<String>,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub sensitivity: Option<String>, // "normal" | "high"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    #[serde(default)]
    pub style: Option<String>, // "technical" | "executive" | "casual"
    #[serde(default)]
    pub detail: Option<String>, // "high" | "medium" | "low"
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
}

/// Full memory state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryState {
    pub episodic: Vec<EpisodicEntry>,
    pub semantic: HashMap<String, SemanticCollection>,
    pub procedural: Vec<ProceduralSkill>,
    pub social: Vec<SocialContact>,
}

impl MemoryState {
    pub fn new() -> Self {
        Self {
            episodic: Vec::new(),
            semantic: HashMap::new(),
            procedural: Vec::new(),
            social: Vec::new(),
        }
    }

    pub fn total_entries(&self) -> usize {
        self.episodic.len()
            + self
                .semantic
                .values()
                .map(|c| c.entries.len())
                .sum::<usize>()
            + self.procedural.len()
            + self.social.len()
    }

    pub fn episodic_size_bytes(&self) -> usize {
        serde_json::to_string(&self.episodic)
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

impl Default for MemoryState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_episodic_entry() {
        let entry = EpisodicEntry {
            date: "2026-06-25".to_string(),
            summary: "Set up Redis cluster".to_string(),
            importance: 0.8,
            tags: vec!["redis".to_string(), "infrastructure".to_string()],
            context: None,
        };
        assert_eq!(entry.importance, 0.8);
        assert_eq!(entry.tags.len(), 2);
    }

    #[test]
    fn test_memory_state() {
        let mut state = MemoryState::new();
        state.episodic.push(EpisodicEntry {
            date: "2026-06-25".to_string(),
            summary: "Test".to_string(),
            importance: 0.5,
            tags: vec![],
            context: None,
        });
        assert_eq!(state.total_entries(), 1);
    }
}
