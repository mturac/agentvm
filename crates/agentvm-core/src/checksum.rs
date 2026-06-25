use sha2::{Digest, Sha256};

use crate::image::AgentImage;
use crate::Result;

/// Compute a stable SHA-256 checksum for an Agent Image manifest.
pub fn checksum(image: &AgentImage) -> Result<String> {
    let yaml = image.to_yaml()?;
    Ok(format!("sha256:{}", sha256_hex(yaml.as_bytes())))
}

/// Compute a SHA-256 checksum for arbitrary bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_is_stable_for_same_image() {
        let image = AgentImage::from_yaml(
            r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "test-agent"
  version: "1.0.0"
"#,
        )
        .unwrap();

        assert_eq!(checksum(&image).unwrap(), checksum(&image).unwrap());
        assert!(checksum(&image).unwrap().starts_with("sha256:"));
    }
}
