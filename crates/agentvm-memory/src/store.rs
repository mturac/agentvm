use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Memory category represented by the AgentVM image format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryKind {
    Episodic,
    Semantic,
    Procedural,
    Social,
}

impl MemoryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
            Self::Procedural => "procedural",
            Self::Social => "social",
        }
    }
}

/// A searchable memory document loaded from an AgentVM image.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryDocument {
    pub kind: MemoryKind,
    pub id: String,
    pub source: PathBuf,
    pub text: String,
}

impl MemoryDocument {
    pub fn tokens(&self) -> Vec<String> {
        tokenize(&self.text)
    }
}

/// In-memory representation of all memory files in an image directory.
#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    documents: Vec<MemoryDocument>,
}

impl MemoryStore {
    pub fn load(image_dir: impl AsRef<Path>) -> Result<Self> {
        let image_dir = image_dir.as_ref();
        let memory_dir = image_dir.join("memory");
        let mut documents = Vec::new();

        load_text_file(
            &mut documents,
            MemoryKind::Episodic,
            memory_dir.join("episodic.md"),
        )?;
        load_json_file(
            &mut documents,
            MemoryKind::Semantic,
            memory_dir.join("semantic.json"),
        )?;
        load_yaml_file(
            &mut documents,
            MemoryKind::Procedural,
            memory_dir.join("procedural.yaml"),
        )?;
        load_yaml_file(
            &mut documents,
            MemoryKind::Social,
            memory_dir.join("social.yaml"),
        )?;

        Ok(Self { documents })
    }

    pub fn documents(&self) -> &[MemoryDocument] {
        &self.documents
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    pub fn len(&self) -> usize {
        self.documents.len()
    }

    #[cfg(test)]
    pub(crate) fn from_documents_for_test(documents: Vec<MemoryDocument>) -> Self {
        Self { documents }
    }
}

fn load_text_file(
    documents: &mut Vec<MemoryDocument>,
    kind: MemoryKind,
    path: PathBuf,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    for (index, block) in split_markdown_blocks(&text).into_iter().enumerate() {
        documents.push(MemoryDocument {
            kind,
            id: format!("{}:{}", kind.as_str(), index + 1),
            source: path.clone(),
            text: block,
        });
    }

    Ok(())
}

fn load_json_file(
    documents: &mut Vec<MemoryDocument>,
    kind: MemoryKind,
    path: PathBuf,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    flatten_json(kind, &path, "$", &value, documents);
    Ok(())
}

fn load_yaml_file(
    documents: &mut Vec<MemoryDocument>,
    kind: MemoryKind,
    path: PathBuf,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    flatten_yaml(kind, &path, "$", &value, documents);
    Ok(())
}

fn split_markdown_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    for line in text.lines() {
        if line.starts_with("## ") && !current.is_empty() {
            blocks.push(current.join("\n").trim().to_string());
            current.clear();
        }
        if !line.trim().is_empty() {
            current.push(line.to_string());
        }
    }

    if !current.is_empty() {
        blocks.push(current.join("\n").trim().to_string());
    }

    blocks
        .into_iter()
        .filter(|block| block.chars().any(|character| character.is_alphanumeric()))
        .collect()
}

fn flatten_json(
    kind: MemoryKind,
    source: &Path,
    path: &str,
    value: &serde_json::Value,
    documents: &mut Vec<MemoryDocument>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                flatten_json(kind, source, &format!("{path}.{key}"), child, documents);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                flatten_json(kind, source, &format!("{path}[{index}]"), child, documents);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
        serde_json::Value::String(text) => push_scalar(kind, source, path, text, documents),
    }
}

fn flatten_yaml(
    kind: MemoryKind,
    source: &Path,
    path: &str,
    value: &serde_yaml::Value,
    documents: &mut Vec<MemoryDocument>,
) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, child) in map {
                let key = key.as_str().unwrap_or("value");
                flatten_yaml(kind, source, &format!("{path}.{key}"), child, documents);
            }
        }
        serde_yaml::Value::Sequence(items) => {
            for (index, child) in items.iter().enumerate() {
                flatten_yaml(kind, source, &format!("{path}[{index}]"), child, documents);
            }
        }
        serde_yaml::Value::String(text) => push_scalar(kind, source, path, text, documents),
        serde_yaml::Value::Null
        | serde_yaml::Value::Bool(_)
        | serde_yaml::Value::Number(_)
        | serde_yaml::Value::Tagged(_) => {}
    }
}

fn push_scalar(
    kind: MemoryKind,
    source: &Path,
    path: &str,
    text: &str,
    documents: &mut Vec<MemoryDocument>,
) {
    if text.trim().is_empty() {
        return;
    }

    documents.push(MemoryDocument {
        kind,
        id: format!("{}:{path}", kind.as_str()),
        source: source.to_path_buf(),
        text: text.trim().to_string(),
    });
}

pub(crate) fn tokenize(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|token| token.len() >= 2)
        .map(str::to_ascii_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenization_normalizes_text() {
        assert_eq!(
            tokenize("Redis cluster, redis!"),
            vec!["redis", "cluster", "redis"]
        );
    }
}
