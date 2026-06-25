use std::collections::{HashMap, HashSet};

use crate::image::{AgentImage, ModelPreference, RegistrySkill};

/// Merge two Agent Images, using the overlay image for scalar conflicts and
/// unioning list-like sections.
pub fn merge(base: &AgentImage, overlay: &AgentImage) -> AgentImage {
    let mut merged = base.clone();

    merged.metadata.version = overlay.metadata.version.clone();
    merged.metadata.display_name = overlay
        .metadata
        .display_name
        .clone()
        .or(merged.metadata.display_name);
    merged.metadata.description = overlay
        .metadata
        .description
        .clone()
        .or(merged.metadata.description);
    merged.metadata.tags = union_strings(&merged.metadata.tags, &overlay.metadata.tags);

    merged.identity.name = overlay.identity.name.clone().or(merged.identity.name);
    merged.identity.emoji = overlay.identity.emoji.clone().or(merged.identity.emoji);
    merged.identity.avatar = overlay.identity.avatar.clone().or(merged.identity.avatar);
    merged.identity.persona = overlay.identity.persona.clone().or(merged.identity.persona);
    merged.identity.languages = union_by_key(
        &merged.identity.languages,
        &overlay.identity.languages,
        |language| language.code.clone(),
    );
    merge_string_maps(&mut merged.identity.behaviors, &overlay.identity.behaviors);

    merged.skills.builtin =
        union_by_key(&merged.skills.builtin, &overlay.skills.builtin, |skill| {
            skill.id.clone()
        });
    merged.skills.registry = union_by_key(
        &merged.skills.registry,
        &overlay.skills.registry,
        registry_skill_key,
    );
    merged.skills.preferences = overlay
        .skills
        .preferences
        .clone()
        .or(merged.skills.preferences);

    merge_string_maps(&mut merged.tools.preferred, &overlay.tools.preferred);
    merged.tools.denied = union_strings(&merged.tools.denied, &overlay.tools.denied);
    merged.tools.behavior = overlay.tools.behavior.clone().or(merged.tools.behavior);
    merged.tools.security = overlay.tools.security.clone().or(merged.tools.security);

    merged.prompts.system = overlay.prompts.system.clone().or(merged.prompts.system);
    merged.prompts.examples = overlay.prompts.examples.clone().or(merged.prompts.examples);
    merged.prompts.constraints = overlay
        .prompts
        .constraints
        .clone()
        .or(merged.prompts.constraints);
    merged.prompts.preferences = overlay
        .prompts
        .preferences
        .clone()
        .or(merged.prompts.preferences);

    merged.runtime.preferred_models = union_by_key(
        &merged.runtime.preferred_models,
        &overlay.runtime.preferred_models,
        model_key,
    );
    merged.runtime.context = overlay.runtime.context.clone().or(merged.runtime.context);
    merged.runtime.sessions = overlay.runtime.sessions.clone().or(merged.runtime.sessions);
    merged.runtime.cost = overlay.runtime.cost.clone().or(merged.runtime.cost);
    merged.runtime.performance = overlay
        .runtime
        .performance
        .clone()
        .or(merged.runtime.performance);

    for (key, value) in &overlay.export {
        merged.export.insert(key.clone(), value.clone());
    }

    merged
}

fn union_strings(left: &[String], right: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    left.iter()
        .chain(right.iter())
        .filter(|value| seen.insert((*value).clone()))
        .cloned()
        .collect()
}

fn union_by_key<T, K, F>(left: &[T], right: &[T], key: F) -> Vec<T>
where
    T: Clone,
    K: Eq + std::hash::Hash,
    F: Fn(&T) -> K,
{
    let mut output = left.to_vec();
    let mut indexes = HashMap::new();
    for (index, value) in output.iter().enumerate() {
        indexes.insert(key(value), index);
    }

    for value in right {
        let item_key = key(value);
        if let Some(index) = indexes.get(&item_key).copied() {
            output[index] = value.clone();
        } else {
            indexes.insert(item_key, output.len());
            output.push(value.clone());
        }
    }
    output
}

fn merge_string_maps<T: Clone>(left: &mut HashMap<String, T>, right: &HashMap<String, T>) {
    for (key, value) in right {
        left.insert(key.clone(), value.clone());
    }
}

fn registry_skill_key(skill: &RegistrySkill) -> String {
    format!("{}@{}", skill.id, skill.version)
}

fn model_key(model: &ModelPreference) -> String {
    format!("{}/{}", model.provider, model.model)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(yaml: &str) -> AgentImage {
        AgentImage::from_yaml(yaml).unwrap()
    }

    #[test]
    fn merge_overrides_scalars_and_unions_skills() {
        let base = image(
            r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "base"
  version: "1.0.0"
  tags: ["coding"]
identity:
  persona: "Base persona"
skills:
  builtin:
    - id: "code-review"
"#,
        );
        let overlay = image(
            r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "overlay"
  version: "1.1.0"
  tags: ["devops"]
identity:
  persona: "Overlay persona"
skills:
  builtin:
    - id: "debugging"
"#,
        );

        let merged = merge(&base, &overlay);

        assert_eq!(merged.metadata.name, "base");
        assert_eq!(merged.metadata.version, "1.1.0");
        assert_eq!(merged.identity.persona.as_deref(), Some("Overlay persona"));
        assert_eq!(merged.metadata.tags, vec!["coding", "devops"]);
        assert_eq!(merged.skills.builtin.len(), 2);
    }

    #[test]
    fn overlay_replaces_matching_skill() {
        let base = image(
            r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "base"
  version: "1.0.0"
skills:
  builtin:
    - id: "code-review"
      enabled: true
"#,
        );
        let overlay = image(
            r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "overlay"
  version: "1.0.1"
skills:
  builtin:
    - id: "code-review"
      enabled: false
"#,
        );

        let merged = merge(&base, &overlay);

        assert_eq!(merged.skills.builtin.len(), 1);
        assert_eq!(merged.skills.builtin[0].enabled, Some(false));
    }
}
