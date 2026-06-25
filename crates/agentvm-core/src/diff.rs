use serde::{Deserialize, Serialize};

use crate::image::AgentImage;

/// Result of comparing two Agent Images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDiff {
    pub identical: bool,
    pub changes: Vec<DiffEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    pub category: DiffCategory,
    pub field: String,
    pub diff_type: DiffType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiffCategory {
    Metadata,
    Identity,
    Memory,
    Skills,
    Tools,
    Prompts,
    Runtime,
    Export,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
}

/// Compare two Agent Images
pub fn diff(a: &AgentImage, b: &AgentImage) -> ImageDiff {
    let mut changes = Vec::new();

    // Compare metadata
    if a.metadata.name != b.metadata.name {
        changes.push(DiffEntry {
            category: DiffCategory::Metadata,
            field: "metadata.name".to_string(),
            diff_type: DiffType::Modified,
            old_value: Some(a.metadata.name.clone()),
            new_value: Some(b.metadata.name.clone()),
        });
    }

    if a.metadata.version != b.metadata.version {
        changes.push(DiffEntry {
            category: DiffCategory::Metadata,
            field: "metadata.version".to_string(),
            diff_type: DiffType::Modified,
            old_value: Some(a.metadata.version.clone()),
            new_value: Some(b.metadata.version.clone()),
        });
    }

    // Compare identity
    if a.identity.persona != b.identity.persona {
        changes.push(DiffEntry {
            category: DiffCategory::Identity,
            field: "identity.persona".to_string(),
            diff_type: DiffType::Modified,
            old_value: a.identity.persona.clone(),
            new_value: b.identity.persona.clone(),
        });
    }

    if a.identity.name != b.identity.name {
        changes.push(DiffEntry {
            category: DiffCategory::Identity,
            field: "identity.name".to_string(),
            diff_type: DiffType::Modified,
            old_value: a.identity.name.clone(),
            new_value: b.identity.name.clone(),
        });
    }

    // Compare skills
    let a_skill_ids: Vec<&str> = a.skills.builtin.iter().map(|s| s.id.as_str()).collect();
    let b_skill_ids: Vec<&str> = b.skills.builtin.iter().map(|s| s.id.as_str()).collect();

    for skill_id in &a_skill_ids {
        if !b_skill_ids.contains(skill_id) {
            changes.push(DiffEntry {
                category: DiffCategory::Skills,
                field: format!("skills.builtin.{}", skill_id),
                diff_type: DiffType::Removed,
                old_value: Some(skill_id.to_string()),
                new_value: None,
            });
        }
    }

    for skill_id in &b_skill_ids {
        if !a_skill_ids.contains(skill_id) {
            changes.push(DiffEntry {
                category: DiffCategory::Skills,
                field: format!("skills.builtin.{}", skill_id),
                diff_type: DiffType::Added,
                old_value: None,
                new_value: Some(skill_id.to_string()),
            });
        }
    }

    // Compare tools.denied
    for tool in &a.tools.denied {
        if !b.tools.denied.contains(tool) {
            changes.push(DiffEntry {
                category: DiffCategory::Tools,
                field: format!("tools.denied.{}", tool),
                diff_type: DiffType::Removed,
                old_value: Some(tool.clone()),
                new_value: None,
            });
        }
    }

    for tool in &b.tools.denied {
        if !a.tools.denied.contains(tool) {
            changes.push(DiffEntry {
                category: DiffCategory::Tools,
                field: format!("tools.denied.{}", tool),
                diff_type: DiffType::Added,
                old_value: None,
                new_value: Some(tool.clone()),
            });
        }
    }

    // Compare preferred models
    let a_models: Vec<String> = a
        .runtime
        .preferred_models
        .iter()
        .map(|m| format!("{}/{}", m.provider, m.model))
        .collect();
    let b_models: Vec<String> = b
        .runtime
        .preferred_models
        .iter()
        .map(|m| format!("{}/{}", m.provider, m.model))
        .collect();

    if a_models != b_models {
        changes.push(DiffEntry {
            category: DiffCategory::Runtime,
            field: "runtime.preferredModels".to_string(),
            diff_type: DiffType::Modified,
            old_value: Some(a_models.join(", ")),
            new_value: Some(b_models.join(", ")),
        });
    }

    ImageDiff {
        identical: changes.is_empty(),
        changes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(name: &str, version: &str, persona: &str) -> AgentImage {
        let yaml = format!(
            r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "{}"
  version: "{}"
identity:
  persona: "{}"
"#,
            name, version, persona
        );
        AgentImage::from_yaml(&yaml).unwrap()
    }

    #[test]
    fn test_identical_images() {
        let a = make_image("test", "1.0.0", "A helper");
        let b = make_image("test", "1.0.0", "A helper");
        let diff = diff(&a, &b);
        assert!(diff.identical);
    }

    #[test]
    fn test_name_changed() {
        let a = make_image("test-a", "1.0.0", "A helper");
        let b = make_image("test-b", "1.0.0", "A helper");
        let diff = diff(&a, &b);
        assert!(!diff.identical);
        assert!(diff.changes.iter().any(|c| c.field == "metadata.name"));
    }

    #[test]
    fn test_persona_changed() {
        let a = make_image("test", "1.0.0", "Old persona");
        let b = make_image("test", "1.0.0", "New persona");
        let diff = diff(&a, &b);
        assert!(!diff.identical);
        assert!(diff.changes.iter().any(|c| c.field == "identity.persona"));
    }
}
