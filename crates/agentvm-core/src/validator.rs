use crate::error::Result;
use crate::image::AgentImage;

/// Validation report for an Agent Image
#[derive(Debug)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

/// Validates Agent Images against the spec
pub struct ImageValidator {
    strict: bool,
}

impl ImageValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Return whether strict validation mode is enabled.
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// Validate an Agent Image
    pub fn validate(&self, image: &AgentImage) -> Result<ValidationReport> {
        let mut errors = Vec::new();
        let warnings = Vec::new();

        // Required: metadata.name
        if image.metadata.name.is_empty() {
            errors.push(ValidationError {
                field: "metadata.name".to_string(),
                message: "Agent name is required".to_string(),
            });
        }

        // Required: metadata.version (valid semver)
        if image.metadata.version.is_empty() {
            errors.push(ValidationError {
                field: "metadata.version".to_string(),
                message: "Version is required".to_string(),
            });
        } else if !is_valid_semver(&image.metadata.version) {
            errors.push(ValidationError {
                field: "metadata.version".to_string(),
                message: format!("'{}' is not valid semver", image.metadata.version),
            });
        }

        // Warn if no persona
        let mut warnings = warnings;
        if image.identity.persona.is_none() {
            warnings.push(ValidationWarning {
                field: "identity.persona".to_string(),
                message: "No persona defined — agent will use defaults".to_string(),
            });
        }

        // Warn if no models configured
        if image.runtime.preferred_models.is_empty() {
            warnings.push(ValidationWarning {
                field: "runtime.preferredModels".to_string(),
                message: "No preferred models configured".to_string(),
            });
        }

        // Validate skill entries
        for skill in &image.skills.builtin {
            if skill.id.is_empty() {
                errors.push(ValidationError {
                    field: "skills.builtin".to_string(),
                    message: "Skill ID cannot be empty".to_string(),
                });
            }
        }

        // Validate name format (lowercase, alphanumeric, hyphens)
        if !image.metadata.name.is_empty() && !is_valid_name(&image.metadata.name) {
            errors.push(ValidationError {
                field: "metadata.name".to_string(),
                message: "Name must be lowercase alphanumeric with hyphens (e.g., 'my-agent')"
                    .to_string(),
            });
        }

        let valid = errors.is_empty();

        Ok(ValidationReport {
            valid,
            errors,
            warnings,
        })
    }
}

impl Default for ImageValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a string is valid semver (basic check)
fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

/// Check if a name is valid (lowercase alphanumeric + hyphens)
fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_image() {
        let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "test-agent"
  version: "1.0.0"
identity:
  persona: "A test assistant"
"#;
        let image = AgentImage::from_yaml(yaml).unwrap();
        let validator = ImageValidator::new();
        let report = validator.validate(&image).unwrap();
        assert!(report.valid);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_missing_name() {
        let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: ""
  version: "1.0.0"
"#;
        let image = AgentImage::from_yaml(yaml).unwrap();
        let validator = ImageValidator::new();
        let report = validator.validate(&image).unwrap();
        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.field == "metadata.name"));
    }

    #[test]
    fn test_invalid_name_format() {
        let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "My Agent!"
  version: "1.0.0"
"#;
        let image = AgentImage::from_yaml(yaml).unwrap();
        let validator = ImageValidator::new();
        let report = validator.validate(&image).unwrap();
        assert!(!report.valid);
    }

    #[test]
    fn test_warn_no_persona() {
        let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "test-agent"
  version: "1.0.0"
"#;
        let image = AgentImage::from_yaml(yaml).unwrap();
        let validator = ImageValidator::new();
        let report = validator.validate(&image).unwrap();
        assert!(report.valid); // Valid but with warnings
        assert!(!report.warnings.is_empty());
    }
}
