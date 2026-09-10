use serde_json::json;
use ugoite_domain::space::{
    classify_space_version, parse_space_version, SpaceVersionError, CURRENT_SPACE_VERSION,
    SUPPORTED_SPACE_VERSIONS,
};

#[test]
fn current_space_version_is_the_only_supported_identity() {
    assert_eq!(CURRENT_SPACE_VERSION, "0.1");
    assert_eq!(SUPPORTED_SPACE_VERSIONS, &["0.1"]);
}

#[test]
fn parser_rejects_product_versions_and_non_canonical_values() {
    let version = parse_space_version("0.1").expect("0.1 parses");
    assert_eq!((version.major, version.generation), (0, 1));
    assert_eq!(version.to_string(), "0.1");
    for value in ["0", "0.1.0", "00.01", " 0.1", "0.1 ", "v0.1", ""] {
        assert!(parse_space_version(value).is_none(), "accepted {value:?}");
    }
}

#[test]
fn classifier_fails_closed_without_schema_aliases() {
    assert!(classify_space_version(&json!({"space_version": "0.1"})).is_ok());
    assert_eq!(
        classify_space_version(&json!({})),
        Err(SpaceVersionError::Missing)
    );
    assert!(matches!(
        classify_space_version(&json!({"space_version": 1})),
        Err(SpaceVersionError::Malformed { .. })
    ));
    assert!(matches!(
        classify_space_version(&json!({"space_version": "0.1.0"})),
        Err(SpaceVersionError::Malformed { .. })
    ));
    assert_eq!(
        classify_space_version(&json!({"space_version": "0.2"})),
        Err(SpaceVersionError::Unsupported {
            detected: "0.2".to_string()
        })
    );
    assert_eq!(
        classify_space_version(&json!({"schema_version": 3})),
        Err(SpaceVersionError::Missing)
    );
}
