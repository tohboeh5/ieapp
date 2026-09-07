use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SpaceMeta {
    pub space_uid: Uuid,
    pub slug: String,
    pub id: String,
    pub name: String,
    pub created_at: f64,
    pub space_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub storage_type: String,
    pub root: String,
}

pub fn storage_type_and_root(root_uri: &str) -> (String, String, String) {
    if let Ok(url) = Url::parse(root_uri) {
        let scheme = url.scheme().to_string();
        let root = if scheme == "fs" || scheme == "file" {
            url.path().to_string()
        } else {
            url.path().trim_start_matches('/').to_string()
        };
        let storage_type = if scheme == "fs" || scheme == "file" {
            "local".to_string()
        } else {
            scheme.clone()
        };
        return (storage_type, root, scheme);
    }

    (
        "local".to_string(),
        root_uri.to_string(),
        "file".to_string(),
    )
}

/// Stable Space compatibility generation.
///
/// Product Version and Space Version evolve independently. Space Version
/// changes only when an incompatible durable Space change is introduced.
/// See the Space Compatibility and Migration Contract.
pub const CURRENT_SPACE_VERSION: &str = "0.1";

/// Space Versions directly supported by this Product release.
pub const SUPPORTED_SPACE_VERSIONS: &[&str] = &["0.1"];

/// Parsed Space Version `<major>.<generation>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpaceVersion {
    pub major: u64,
    pub generation: u64,
}

impl std::fmt::Display for SpaceVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.generation)
    }
}

/// Failure to classify a bootstrap `space_version` before version-specific
/// validation. Callers MUST fail closed without authoritative mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceVersionError {
    Missing,
    Malformed { detected: String },
    Unsupported { detected: String },
}

impl SpaceVersionError {
    pub fn detected(&self) -> Option<&str> {
        match self {
            Self::Missing => None,
            Self::Malformed { detected } | Self::Unsupported { detected } => Some(detected),
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Missing => {
                "Space bootstrap metadata is missing required field space_version".to_string()
            }
            Self::Malformed { detected } => {
                format!("Space bootstrap metadata has malformed space_version: {detected:?}")
            }
            Self::Unsupported { detected } => {
                format!(
                    "Unsupported Space version {detected}; this Product supports: {}",
                    SUPPORTED_SPACE_VERSIONS.join(", ")
                )
            }
        }
    }
}

/// Parse a Space Version string without accepting Product version semantics.
///
/// Valid generations are `<major>.<generation>` where both components are
/// ASCII decimal integers without surrounding whitespace or extra segments.
/// Product minor numbers MUST NOT be copied into Space Version.
pub fn parse_space_version(value: &str) -> Option<SpaceVersion> {
    let (major_text, generation_text) = value.split_once('.')?;
    if major_text.is_empty() || generation_text.is_empty() {
        return None;
    }
    if !major_text.bytes().all(|b| b.is_ascii_digit())
        || !generation_text.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    // Reject leading `+`/`-`, whitespace, and empty segments already handled
    // above; `parse` below only accepts canonical decimal digits.
    let major: u64 = major_text.parse().ok()?;
    let generation: u64 = generation_text.parse().ok()?;
    Some(SpaceVersion { major, generation })
}

/// Shared compatibility classification used by server/core and local CLI
/// paths. This is the single source of truth for bootstrap Space Version
/// handling: locate metadata, extract `space_version`, classify, and only
/// then perform version-specific validation.
pub fn classify_space_version(meta: &serde_json::Value) -> Result<SpaceVersion, SpaceVersionError> {
    let detected = match meta.get("space_version") {
        Some(serde_json::Value::String(value)) => value.clone(),
        Some(other) => {
            let rendered = match other {
                serde_json::Value::Null => "null".to_string(),
                serde_json::Value::Bool(value) => value.to_string(),
                serde_json::Value::Number(value) => value.to_string(),
                serde_json::Value::String(value) => value.clone(),
                serde_json::Value::Array(_) => "array".to_string(),
                serde_json::Value::Object(_) => "object".to_string(),
            };
            return Err(SpaceVersionError::Malformed { detected: rendered });
        }
        None => return Err(SpaceVersionError::Missing),
    };
    let parsed = parse_space_version(&detected).ok_or_else(|| SpaceVersionError::Malformed {
        detected: detected.clone(),
    })?;
    let canonical = parsed.to_string();
    if !SUPPORTED_SPACE_VERSIONS.contains(&canonical.as_str()) {
        return Err(SpaceVersionError::Unsupported { detected });
    }
    Ok(parsed)
}

/// Extract the raw `space_version` string for error detail without guessing
/// the meaning of unknown durable data.
pub fn raw_space_version(meta: &serde_json::Value) -> Option<String> {
    meta.get("space_version").and_then(|value| match value {
        serde_json::Value::String(value) => Some(value.clone()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        _ => None,
    })
}
