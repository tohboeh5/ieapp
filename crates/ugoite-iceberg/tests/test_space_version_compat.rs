mod common;
use common::setup_operator;
use serde_json::Value;
use ugoite_core::error::{AppError, ErrorCode};
use ugoite_domain::space::{
    classify_space_version, CURRENT_SPACE_VERSION, SUPPORTED_SPACE_VERSIONS,
};
use ugoite_iceberg::{form, space};

const FIXTURE_META: &str = include_str!(
    "../../../fixtures/spaces/0.1/spaces/019c1234-5678-7abc-8def-0123456789ab/meta.json"
);
const FIXTURE_SETTINGS: &str = include_str!(
    "../../../fixtures/spaces/0.1/spaces/019c1234-5678-7abc-8def-0123456789ab/settings.json"
);
const FIXTURE_EXPECTED: &str = include_str!("../../../fixtures/spaces/0.1/expected.json");

fn fixture_meta() -> Value {
    serde_json::from_str(FIXTURE_META).expect("parse frozen Space 0.1 meta.json")
}

/// Historical Space 0.1 compatibility: open the frozen historical Space,
/// recover expected Knowledge, perform representative mutations, close,
/// reopen, and verify Knowledge, history, authority, and integrity.
#[tokio::test]
async fn space_01_historical_fixture_opens_and_preserves_knowledge() -> anyhow::Result<()> {
    let meta = fixture_meta();
    let expected: Value =
        serde_json::from_str(FIXTURE_EXPECTED).expect("parse frozen Space 0.1 expected.json");
    let settings: Value =
        serde_json::from_str(FIXTURE_SETTINGS).expect("parse frozen Space 0.1 settings.json");

    // 1. Open the historical Space: classify before version-specific validation.
    let version = classify_space_version(&meta).expect("frozen Space 0.1 must classify");
    assert_eq!(version.to_string(), "0.1");
    assert_eq!(CURRENT_SPACE_VERSION, "0.1");
    assert_eq!(SUPPORTED_SPACE_VERSIONS, &["0.1"]);

    // 2. Recover expected Knowledge from frozen bootstrap metadata.
    assert_eq!(meta["space_version"], expected["space_version"]);
    assert_eq!(meta["space_id"], expected["space_id"]);
    assert_eq!(meta["space_uid"], expected["space_uid"]);
    assert_eq!(meta["slug"], expected["slug"]);
    assert_eq!(meta["name"], expected["name"]);
    assert!(meta.get("schema_version").is_none());
    assert_eq!(settings["default_form"], "Entry");
    let uid = uuid::Uuid::parse_str(meta["space_uid"].as_str().expect("space_uid"))?;
    assert_eq!(uid.get_version(), Some(uuid::Version::SortRand));

    // 3. Perform representative supported mutations on a live Space 0.1.
    let op = setup_operator()?;
    space::create_space(&op, "compat-live-01", "/tmp").await?;
    let opened = space::get_space(&op, "compat-live-01").await?;
    assert_eq!(opened.space_version, "0.1");
    let raw = space::get_space_raw(&op, "compat-live-01").await?;
    assert_eq!(raw["space_version"], "0.1");
    let ws_path = "spaces/compat-live-01".to_string();
    let forms = form::list_forms(&op, &ws_path).await?;
    assert!(forms
        .iter()
        .any(|form| form.get("name").and_then(|name| name.as_str()) == Some("Entry")));

    let patched = space::patch_space(
        &op,
        "compat-live-01",
        &serde_json::json!({"name": "Compat Renamed"}),
    )
    .await?;
    assert_eq!(patched["name"], "Compat Renamed");
    // Stable identity is preserved across mutation, not rewritten.
    assert_eq!(patched["space_version"], "0.1");
    assert_eq!(patched["space_uid"], raw["space_uid"]);
    assert_eq!(patched["slug"], raw["slug"]);

    // 4-6. Close, reopen, and verify expected Knowledge and history.
    let reopened = space::get_space_raw(&op, "compat-live-01").await?;
    assert_eq!(reopened["name"], "Compat Renamed");
    assert_eq!(reopened["space_version"], "0.1");
    assert_eq!(reopened["space_uid"], raw["space_uid"]);
    let listed = space::list_spaces(&op).await?;
    assert!(listed.contains(&"compat-live-01".to_string()));

    // 7. Authority and integrity invariants: UUIDv7 identity, directory
    // binding, and typed fail-closed compatibility for unknown versions.
    let reopened_uid = uuid::Uuid::parse_str(reopened["space_uid"].as_str().unwrap())?;
    assert_eq!(reopened_uid.get_version(), Some(uuid::Version::SortRand));

    let mut unsupported = fixture_meta();
    unsupported["space_version"] = Value::String("0.2".to_string());
    let op2 = setup_operator()?;
    space::create_space(&op2, "compat-guard", "/tmp").await?;
    let guard_path = "spaces/compat-guard/meta.json";
    op2.write(guard_path, serde_json::to_vec(&unsupported)?)
        .await?;
    let error = space::get_space(&op2, "compat-guard").await.unwrap_err();
    let app_error = error
        .downcast_ref::<AppError>()
        .expect("unsupported Space Version must be typed");
    assert_eq!(app_error.code(), ErrorCode::UnsupportedSpaceVersion);
    assert_eq!(app_error.code_str(), "UNSUPPORTED_SPACE_VERSION");
    Ok(())
}
