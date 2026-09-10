//! JOURNEY-KNOWLEDGE-001 through the local/core CLI.
//!
//! Evidence identity: surface=cli, transport=core/local. This runs the same
//! scenario as e2e/knowledge-journey.test.ts (Space create -> Form establish
//! -> Entry create -> Entry edit -> Search -> History -> Restore -> Reopen)
//! and asserts the same durable postconditions through canonical reads.
//! CLI stdout wording is never compared; only exit status and the returned
//! durable state matter. Form establish intentionally drives `form update`,
//! which is the upsert path behind a weaker name.

use std::process::Command;
use std::process::Output;

fn ugoite_bin() -> std::path::PathBuf {
    if let Some(path) = option_env!("CARGO_BIN_EXE_ugoite") {
        return std::path::PathBuf::from(path);
    }

    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("ugoite");
    path
}

fn run_cli(config: &std::path::Path, args: &[&str]) -> Output {
    Command::new(ugoite_bin())
        .args(args)
        .env("UGOITE_CLI_CONFIG_PATH", config)
        .output()
        .expect("run ugoite")
}

fn stdout_json(output: &Output, what: &str) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).unwrap_or_else(|_| panic!("{what} stdout is not JSON: {stdout}"))
}

fn contains_string(value: &serde_json::Value, needle: &str) -> bool {
    match value {
        serde_json::Value::String(text) => text == needle,
        serde_json::Value::Array(items) => items.iter().any(|item| contains_string(item, needle)),
        serde_json::Value::Object(fields) => {
            fields.values().any(|item| contains_string(item, needle))
        }
        _ => false,
    }
}

fn contains_substring(value: &serde_json::Value, needle: &str) -> bool {
    match value {
        serde_json::Value::String(text) => text.contains(needle),
        serde_json::Value::Array(items) => {
            items.iter().any(|item| contains_substring(item, needle))
        }
        serde_json::Value::Object(fields) => {
            fields.values().any(|item| contains_substring(item, needle))
        }
        _ => false,
    }
}

fn revision_ids(history: &serde_json::Value) -> Vec<String> {
    history
        .get("revisions")
        .and_then(|revisions| revisions.as_array())
        .unwrap_or_else(|| panic!("history has no revisions array: {history}"))
        .iter()
        .map(|revision| {
            revision
                .get("revision_id")
                .and_then(|id| id.as_str())
                .unwrap_or_else(|| panic!("revision has no revision_id: {revision}"))
                .to_string()
        })
        .collect()
}

/// JOURNEY-KNOWLEDGE-001 on surface=cli transport=core/local reaches the
/// same durable Knowledge outcome as the Frontend evidence.
#[test]
fn test_journey_cli_core_local_durable_outcome() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_string_lossy().to_string();
    let config_path = dir.path().join("cli-config.json");
    let space_id = "journey-core-space";
    let space_path = format!("{root}/spaces/{space_id}");
    let form_name = "JourneyCoreForm";
    let needle = "journey-core-needle";
    let entry_id = "journey-core-entry";

    // Space create: a durable Space comes into existence.
    let output = run_cli(&config_path, &["create-space", "--root", &root, space_id]);
    assert!(
        output.status.success(),
        "space create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Form establish via `form update`: the upsert path behind a weaker name.
    let form_file = dir.path().join("journey-core-form.json");
    std::fs::write(
        &form_file,
        format!(
            "{{\"name\":\"{form_name}\",\"version\":1,\"template\":\"# {form_name}\\n\\n## Status\\n\\n## Body\\n\",\"fields\":{{\"Status\":{{\"type\":\"string\",\"required\":true}},\"Body\":{{\"type\":\"markdown\"}}}}}}"
        ),
    )
    .unwrap();
    let output = run_cli(
        &config_path,
        &["form", "update", &space_path, form_file.to_str().unwrap()],
    );
    assert!(
        output.status.success(),
        "form establish failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let form = stdout_json(
        &run_cli(&config_path, &["form", "get", &space_path, form_name]),
        "form get",
    );
    assert_eq!(
        form.get("name").and_then(|name| name.as_str()),
        Some(form_name)
    );
    assert_eq!(
        form.pointer("/fields/Status/type").and_then(|t| t.as_str()),
        Some("string")
    );
    assert_eq!(
        form.pointer("/fields/Status/required")
            .and_then(|r| r.as_bool()),
        Some(true)
    );

    // Entry create appends exactly one revision.
    let v1 = format!(
        "---\nform: {form_name}\n---\n# Journey core v1\n\n## Status\n{needle}\n\n## Body\njourney core v1\n"
    );
    let created = stdout_json(
        &run_cli(
            &config_path,
            &["entry", "create", "--content", &v1, &space_path, entry_id],
        ),
        "entry create",
    );
    assert!(contains_string(&created, entry_id));
    let history = stdout_json(
        &run_cli(&config_path, &["entry", "history", &space_path, entry_id]),
        "entry history after create",
    );
    let ids = revision_ids(&history);
    assert_eq!(ids.len(), 1);
    let rev1 = ids[0].clone();

    // Entry edit appends a revision; a stale parent conflicts.
    let v2 = format!(
        "---\nform: {form_name}\n---\n# Journey core v2\n\n## Status\n{needle}\n\n## Body\njourney core v2\n"
    );
    // NOTE: `--markdown=<value>` keeps frontmatter (leading `---`) from
    // parsing as a flag; the update flag lacks allow_hyphen_values.
    let markdown_arg = format!("--markdown={v2}");
    let output = run_cli(
        &config_path,
        &[
            "entry",
            "update",
            &space_path,
            entry_id,
            markdown_arg.as_str(),
            "--parent-revision-id",
            &rev1,
        ],
    );
    assert!(
        output.status.success(),
        "entry update failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let history = stdout_json(
        &run_cli(&config_path, &["entry", "history", &space_path, entry_id]),
        "entry history after edit",
    );
    let ids = revision_ids(&history);
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&rev1));
    let rev2 = ids.into_iter().find(|id| id != &rev1).expect("rev2");
    let stale = run_cli(
        &config_path,
        &[
            "entry",
            "update",
            &space_path,
            entry_id,
            markdown_arg.as_str(),
            "--parent-revision-id",
            &rev1,
        ],
    );
    assert!(
        !stale.status.success(),
        "stale parent revision must conflict instead of overwriting"
    );

    // Search finds the updated durable Entry.
    let results = stdout_json(
        &run_cli(&config_path, &["search", "keyword", &space_path, needle]),
        "search keyword",
    );
    assert!(
        contains_string(&results, entry_id),
        "search must find the updated entry: {results}"
    );

    // Restore appends a new revision replaying rev1; history never shortens.
    let output = run_cli(
        &config_path,
        &["entry", "restore", &space_path, entry_id, &rev1],
    );
    assert!(
        output.status.success(),
        "entry restore failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let history = stdout_json(
        &run_cli(&config_path, &["entry", "history", &space_path, entry_id]),
        "entry history after restore",
    );
    let ids = revision_ids(&history);
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&rev1));
    assert!(ids.contains(&rev2));
    let rev3 = ids
        .into_iter()
        .find(|id| id != &rev1 && id != &rev2)
        .expect("rev3");
    let revision = stdout_json(
        &run_cli(
            &config_path,
            &["entry", "revision", &space_path, entry_id, &rev3],
        ),
        "entry revision after restore",
    );
    assert_eq!(
        revision.get("revision_id").and_then(|id| id.as_str()),
        Some(rev3.as_str())
    );
    assert!(
        contains_substring(&revision, "journey core v1"),
        "restored revision must replay rev1 content: {revision}"
    );

    // Reopen: fresh processes read identical durable state.
    let space = stdout_json(
        &run_cli(&config_path, &["space", "get", &space_path]),
        "space get on reopen",
    );
    assert!(contains_string(&space, space_id));
    let history = stdout_json(
        &run_cli(&config_path, &["entry", "history", &space_path, entry_id]),
        "entry history on reopen",
    );
    assert_eq!(revision_ids(&history).len(), 3);
    let results = stdout_json(
        &run_cli(&config_path, &["search", "keyword", &space_path, needle]),
        "search keyword on reopen",
    );
    assert!(contains_string(&results, entry_id));
}
