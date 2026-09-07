use serde_json::json;
use ugoite_domain::space::{
    classify_space_version, parse_space_version, SpaceVersionError, CURRENT_SPACE_VERSION,
    SUPPORTED_SPACE_VERSIONS,
};

#[test]
fn current_space_version_is_first_stable_generation() {
    assert_eq!(CURRENT_SPACE_VERSION, "0.1");
    assert_eq!(SUPPORTED_SPACE_VERSIONS, &["0.1"]);
}

#[test]
fn space_version_parses_compatibility_generations_not_product_versions() {
    let parsed = parse_space_version("0.1").expect("0.1 parses");
    assert_eq!((parsed.major, parsed.generation), (0, 1));
    assert_eq!(parsed.to_string(), "0.1");

    // Product minor versions MUST NOT be copied into Space Version parsing
    // as valid generations beyond classification; parsing itself stays
    // structural, while support is decided by the supported list.
    assert!(parse_space_version("0.1.0").is_none());
    assert!(parse_space_version("0").is_none());
    assert!(parse_space_version("").is_none());
    assert!(parse_space_version(" 0.1").is_none());
    assert!(parse_space_version("0.1 ").is_none());
    assert!(parse_space_version("v0.1").is_none());
}

#[test]
fn space_version_classification_is_shared_source_of_truth() {
    assert!(classify_space_version(&json!({"space_version": "0.1"})).is_ok());
    assert_eq!(
        classify_space_version(&json!({})),
        Err(SpaceVersionError::Missing)
    );
    assert!(matches!(
        classify_space_version(&json!({"space_version": "0.1.0"})),
        Err(SpaceVersionError::Malformed { .. })
    ));
    assert!(matches!(
        classify_space_version(&json!({"space_version": 3})),
        Err(SpaceVersionError::Malformed { .. })
    ));
    assert_eq!(
        classify_space_version(&json!({"space_version": "0.2"})),
        Err(SpaceVersionError::Unsupported {
            detected: "0.2".to_string()
        })
    );
    // Pre-stable internal identity is not the stable compatibility identity.
    assert!(matches!(
        classify_space_version(&json!({"schema_version": 3})),
        Err(SpaceVersionError::Missing)
    ));
}
