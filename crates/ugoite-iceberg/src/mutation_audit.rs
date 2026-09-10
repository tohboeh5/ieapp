//! Commit-coupled audit evidence for content mutations.
//!
//! Issue #2508: ordinary Entry and saved-SQL mutations committed Knowledge
//! without leaving audit evidence, while authorization mutations were wired
//! to [`crate::audit`]. Simply appending after commit is not enough: a crash
//! between the Knowledge commit and the audit append would leave Knowledge
//! without evidence forever.
//!
//! Contract: **if an authoritative mutation committed, its audit intent is
//! recoverable.** This module implements that without creating a second
//! business authority:
//!
//! - The committed revision (Entry history / saved-SQL row) is the only
//!   source of truth. No outbox table, no second Knowledge record.
//! - Every content-mutation event carries a deterministic UUIDv5 `event_id`
//!   derived from `(space_uid, action, target_id, revision_id)`. Redelivering
//!   the same committed revision converges to one audit event via the
//!   existing idempotent append path; a *differing* payload under the same
//!   event ID fails closed instead of overwriting evidence.
//! - [`UgoiteService::reconcile_entry_audit`] and
//!   [`UgoiteService::reconcile_saved_sql_audit`] re-derive the expected
//!   event from committed truth and deliver it, closing a crash gap.
//! - Payloads are allow-listed: actor/action/target/revision/change IDs
//!   only. Entry bodies, SQL text, variables, credentials, and tokens never
//!   enter audit events (enforced by [`crate::audit`] secret rejection plus
//!   construction that never accepts content).
//!
//! Delivery after commit is best-effort (consistent with the existing
//! authorization wiring): a failed delivery never fails an already-committed
//! mutation. The deterministic IDs plus reconcile keep that gap recoverable.

use anyhow::{Context, Result};
use opendal::Operator;
use serde_json::Value;
use uuid::Uuid;

use crate::audit;
use crate::service::UgoiteService;

/// Fixed namespace for deterministic UUIDv5 content-mutation audit event IDs.
const MUTATION_AUDIT_EVENT_NAMESPACE: Uuid =
    Uuid::from_u128(0x7567_6f69_7465_5f61_7564_6974_5f76_3141);

/// Actions emitted for Entry content mutations.
pub const ENTRY_CREATED_ACTION: &str = "entry.created";
/// Actions emitted for Entry content mutations.
pub const ENTRY_UPDATED_ACTION: &str = "entry.updated";
/// Actions emitted for Entry content mutations.
pub const ENTRY_DELETED_ACTION: &str = "entry.deleted";
/// Actions emitted for saved-SQL content mutations.
pub const SAVED_SQL_CREATED_ACTION: &str = "saved_sql.created";
/// Actions emitted for saved-SQL content mutations.
pub const SAVED_SQL_UPDATED_ACTION: &str = "saved_sql.updated";
/// Actions emitted for saved-SQL content mutations.
pub const SAVED_SQL_DELETED_ACTION: &str = "saved_sql.deleted";

/// Derives the deterministic audit `event_id` for one committed revision.
///
/// The same committed `(space, action, target, revision)` always maps to the
/// same event ID, so retries, crash recovery, and duplicate deliveries are
/// idempotent by construction.
pub fn mutation_audit_event_id(
    space_uid: &Uuid,
    action: &str,
    target_id: &str,
    revision_id: &str,
) -> Uuid {
    let material = format!("{space_uid}:{action}:{target_id}:{revision_id}");
    Uuid::new_v5(&MUTATION_AUDIT_EVENT_NAMESPACE, material.as_bytes())
}

/// Resolves audit attribution without inventing identities.
///
/// Authorized callers pass their principal IDs (first one attributes the
/// event). Operator-local callers have no principal, so the free-form author
/// string is used as subject. As a last resort the Space UID marks the event
/// as unattributed operator-local activity rather than dropping evidence.
pub fn audit_attribution(
    principal_ids: &[Uuid],
    author_fallback: &str,
    space_uid: &Uuid,
) -> (String, Option<String>) {
    if let Some(principal_id) = principal_ids.first() {
        let id = principal_id.to_string();
        return (id.clone(), Some(id));
    }
    if !author_fallback.trim().is_empty() {
        return (author_fallback.trim().to_string(), None);
    }
    (space_uid.to_string(), None)
}

#[allow(clippy::too_many_arguments)]
fn base_mutation_event(
    space_uid: &Uuid,
    action: &str,
    target_type: &str,
    target_id: &str,
    revision_id: &str,
    change_id: Option<&str>,
    subject: &str,
    actor: Option<&str>,
) -> Value {
    let event_id = mutation_audit_event_id(space_uid, action, target_id, revision_id);
    let mut metadata = serde_json::Map::from_iter([(
        "revision_id".to_string(),
        Value::String(revision_id.to_string()),
    )]);
    if let Some(change_id) = change_id.filter(|value| !value.trim().is_empty()) {
        metadata.insert(
            "change_id".to_string(),
            Value::String(change_id.to_string()),
        );
    }
    let mut event = serde_json::Map::from_iter([
        ("action".to_string(), Value::String(action.to_string())),
        (
            "subject_principal_id".to_string(),
            Value::String(subject.to_string()),
        ),
        (
            "space_uid".to_string(),
            Value::String(space_uid.to_string()),
        ),
        (
            "target_type".to_string(),
            Value::String(target_type.to_string()),
        ),
        (
            "target_id".to_string(),
            Value::String(target_id.to_string()),
        ),
        ("outcome".to_string(), Value::String("success".to_string())),
        ("event_id".to_string(), Value::String(event_id.to_string())),
        ("metadata".to_string(), Value::Object(metadata)),
    ]);
    if let Some(actor) = actor {
        event.insert(
            "actor_principal_id".to_string(),
            Value::String(actor.to_string()),
        );
    }
    Value::Object(event)
}

/// Builds the allow-listed audit event for one committed Entry revision.
///
/// Only identity fields are recorded; Entry body content is never accepted,
/// so it can never leak into the audit chain.
#[allow(clippy::too_many_arguments)]
pub fn entry_mutation_event(
    space_uid: &Uuid,
    action: &str,
    entry_id: &str,
    revision_id: &str,
    change_id: Option<&str>,
    subject: &str,
    actor: Option<&str>,
) -> Value {
    base_mutation_event(
        space_uid,
        action,
        "entry",
        entry_id,
        revision_id,
        change_id,
        subject,
        actor,
    )
}

/// Builds the allow-listed audit event for one committed saved-SQL revision.
///
/// SQL text and variable values are never accepted, so query content and
/// parameter secrets can never leak into the audit chain.
#[allow(clippy::too_many_arguments)]
pub fn saved_sql_mutation_event(
    space_uid: &Uuid,
    action: &str,
    sql_id: &str,
    revision_id: &str,
    subject: &str,
    actor: Option<&str>,
) -> Value {
    base_mutation_event(
        space_uid,
        action,
        "saved_sql",
        sql_id,
        revision_id,
        Some(revision_id),
        subject,
        actor,
    )
}

/// Idempotently delivers one content-mutation audit event.
///
/// Redelivering an event for the same committed revision returns the stored
/// event without duplicating the hash chain.
pub(crate) async fn deliver_mutation_audit_event(
    op: &Operator,
    space_id: &str,
    event: &Value,
) -> Result<Value> {
    audit::append_audit_event(op, space_id, event, None).await
}

fn latest_entry_revision(history: &Value) -> Option<(String, Option<String>, String)> {
    let revisions = history.get("revisions")?.as_array()?;
    let last = revisions.last()?.as_object()?;
    let revision_id = last.get("revision_id")?.as_str()?.to_string();
    let change_id = last
        .get("change_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let operation = last.get("operation")?.as_str()?.to_string();
    Some((revision_id, change_id, operation))
}

impl UgoiteService {
    #[allow(clippy::too_many_arguments)]
    async fn deliver_entry_revision_audit(
        &self,
        space_id: &str,
        action: &str,
        entry_id: &str,
        revision_id: &str,
        change_id: Option<&str>,
        principal_ids: &[Uuid],
        author_fallback: &str,
    ) -> Result<()> {
        let space_uid = self.space_uid(space_id).await?;
        let (subject, actor) = audit_attribution(principal_ids, author_fallback, &space_uid);
        let event = entry_mutation_event(
            &space_uid,
            action,
            entry_id,
            revision_id,
            change_id,
            &subject,
            actor.as_deref(),
        );
        deliver_mutation_audit_event(self.operator(), space_id, &event).await?;
        Ok(())
    }

    /// Best-effort audit delivery for a committed Entry revision.
    ///
    /// The revision identity is always re-read from Entry history (the
    /// committed truth, tombstones included) rather than assumed: the entry
    /// layer mints change IDs the caller cannot predict. Failures never fail
    /// the already-committed mutation; the deterministic event ID keeps the
    /// intent recoverable via [`Self::reconcile_entry_audit`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn record_committed_entry_revision(
        &self,
        space_id: &str,
        entry_id: &str,
        action: &str,
        principal_ids: &[Uuid],
        author_fallback: &str,
    ) {
        let delivered = async {
            let history = crate::entry::get_entry_history(
                self.operator(),
                &self.workspace_path(space_id),
                entry_id,
            )
            .await?;
            let Some((revision_id, change_id, _)) = latest_entry_revision(&history) else {
                return Ok::<(), anyhow::Error>(());
            };
            self.deliver_entry_revision_audit(
                space_id,
                action,
                entry_id,
                &revision_id,
                change_id.as_deref(),
                principal_ids,
                author_fallback,
            )
            .await?;
            Ok::<(), anyhow::Error>(())
        }
        .await;
        let _ = delivered;
    }

    /// Best-effort audit delivery for a committed saved-SQL revision.
    ///
    /// Failures never fail the already-committed mutation; the deterministic
    /// event ID keeps the intent recoverable via
    /// [`Self::reconcile_saved_sql_audit`].
    pub(crate) async fn record_saved_sql_audit(
        &self,
        space_id: &str,
        action: &str,
        sql_id: &str,
        revision_id: &str,
        principal_ids: &[Uuid],
        author_fallback: &str,
    ) {
        let delivered = async {
            let space_uid = self.space_uid(space_id).await?;
            let (subject, actor) = audit_attribution(principal_ids, author_fallback, &space_uid);
            let event = saved_sql_mutation_event(
                &space_uid,
                action,
                sql_id,
                revision_id,
                &subject,
                actor.as_deref(),
            );
            deliver_mutation_audit_event(self.operator(), space_id, &event).await?;
            Ok::<(), anyhow::Error>(())
        }
        .await;
        let _ = delivered;
    }

    /// Best-effort audit delivery for a committed Entry tombstone.
    ///
    /// Deletes share the revision re-read path: the tombstone revision only
    /// exists after commit. Failures never fail the already-committed
    /// delete; reconcile closes the gap.
    pub(crate) async fn record_committed_entry_delete(
        &self,
        space_id: &str,
        entry_id: &str,
        principal_ids: &[Uuid],
        actor_fallback: &str,
    ) {
        self.record_committed_entry_revision(
            space_id,
            entry_id,
            ENTRY_DELETED_ACTION,
            principal_ids,
            actor_fallback,
        )
        .await;
    }

    /// Re-derives the expected audit event for `entry_id` from committed
    /// truth (Entry history) and delivers it idempotently.
    ///
    /// Returns `None` when the Entry has no committed revisions, including
    /// when it never existed (consistent with saved-SQL reconcile). Closing a
    /// commit→delivery crash gap is a second call away however long after
    /// the crash the Space is reopened.
    pub async fn reconcile_entry_audit(
        &self,
        space_id: &str,
        entry_id: &str,
        principal_ids: &[Uuid],
        author_fallback: &str,
    ) -> Result<Option<Value>> {
        let history = match crate::entry::get_entry_history(
            self.operator(),
            &self.workspace_path(space_id),
            entry_id,
        )
        .await
        {
            Ok(history) => history,
            Err(error)
                if error
                    .downcast_ref::<ugoite_core::error::AppError>()
                    .is_some_and(|app| {
                        app.code() == ugoite_core::error::ErrorCode::EntryNotFound
                    }) =>
            {
                return Ok(None);
            }
            Err(error) => {
                return Err(error).with_context(|| format!("reconcile audit for Entry {entry_id}"));
            }
        };
        let revisions = history
            .get("revisions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if revisions.is_empty() {
            return Ok(None);
        }
        let Some((revision_id, change_id, operation)) = latest_entry_revision(&history) else {
            return Ok(None);
        };
        let action = if operation == "delete" {
            ENTRY_DELETED_ACTION
        } else if revisions.len() == 1 {
            ENTRY_CREATED_ACTION
        } else {
            ENTRY_UPDATED_ACTION
        };
        let space_uid = self.space_uid(space_id).await?;
        let (subject, actor) = audit_attribution(principal_ids, author_fallback, &space_uid);
        let event = entry_mutation_event(
            &space_uid,
            action,
            entry_id,
            &revision_id,
            change_id.as_deref(),
            &subject,
            actor.as_deref(),
        );
        Ok(Some(
            deliver_mutation_audit_event(self.operator(), space_id, &event).await?,
        ))
    }

    /// Re-derives the expected audit event for `sql_id` from committed truth
    /// (saved-SQL row, including tombstones) and delivers it idempotently.
    pub async fn reconcile_saved_sql_audit(
        &self,
        space_id: &str,
        sql_id: &str,
        principal_ids: &[Uuid],
        author_fallback: &str,
    ) -> Result<Option<Value>> {
        let Some((revision_id, parent_revision_id, deleted)) =
            crate::saved_sql::read_sql_row_for_audit(
                self.operator(),
                &self.workspace_path(space_id),
                sql_id,
            )
            .await?
        else {
            return Ok(None);
        };
        let action = if deleted {
            SAVED_SQL_DELETED_ACTION
        } else if parent_revision_id.is_none() {
            SAVED_SQL_CREATED_ACTION
        } else {
            SAVED_SQL_UPDATED_ACTION
        };
        let space_uid = self.space_uid(space_id).await?;
        let (subject, actor) = audit_attribution(principal_ids, author_fallback, &space_uid);
        let event = saved_sql_mutation_event(
            &space_uid,
            action,
            sql_id,
            &revision_id,
            &subject,
            actor.as_deref(),
        );
        Ok(Some(
            deliver_mutation_audit_event(self.operator(), space_id, &event).await?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_uid() -> Uuid {
        Uuid::parse_str("0198a1b2-c3d4-7e5f-8901-23456789abcd").expect("fixed test UIDv7")
    }

    #[test]
    fn event_ids_are_deterministic_per_committed_revision() {
        let space_uid = test_uid();
        let first = mutation_audit_event_id(&space_uid, ENTRY_CREATED_ACTION, "entry-1", "rev-1");
        let retry = mutation_audit_event_id(&space_uid, ENTRY_CREATED_ACTION, "entry-1", "rev-1");
        assert_eq!(first, retry);
        let next_revision =
            mutation_audit_event_id(&space_uid, ENTRY_UPDATED_ACTION, "entry-1", "rev-2");
        assert_ne!(first, next_revision);
        let other_entry =
            mutation_audit_event_id(&space_uid, ENTRY_CREATED_ACTION, "entry-2", "rev-1");
        assert_ne!(first, other_entry);
    }

    #[test]
    fn entry_events_carry_no_content() {
        let space_uid = test_uid();
        let event = entry_mutation_event(
            &space_uid,
            ENTRY_CREATED_ACTION,
            "entry-1",
            "rev-1",
            Some("change-1"),
            "author",
            None,
        );
        let raw = serde_json::to_string(&event).expect("event serializes");
        for forbidden in ["markdown", "content", "body", "secret", "token"] {
            assert!(
                !raw.contains(forbidden),
                "event must not contain {forbidden}"
            );
        }
        assert_eq!(event["action"], json!(ENTRY_CREATED_ACTION));
        assert_eq!(event["target_type"], json!("entry"));
        assert_eq!(event["metadata"]["revision_id"], json!("rev-1"));
        assert_eq!(event["metadata"]["change_id"], json!("change-1"));
    }

    #[test]
    fn saved_sql_events_carry_no_query_text() {
        let space_uid = test_uid();
        let event = saved_sql_mutation_event(
            &space_uid,
            SAVED_SQL_CREATED_ACTION,
            "sql-1",
            "rev-1",
            "author",
            Some("author"),
        );
        let raw = serde_json::to_string(&event).expect("event serializes");
        for forbidden in ["SELECT", "sql_text", "variables", "secret", "token"] {
            assert!(
                !raw.contains(forbidden),
                "event must not contain {forbidden}"
            );
        }
        assert_eq!(event["target_type"], json!("saved_sql"));
    }

    #[test]
    fn attribution_prefers_principals_then_author_then_space() {
        let space_uid = test_uid();
        let principal = Uuid::now_v7();
        let (subject, actor) = audit_attribution(&[principal], "author", &space_uid);
        assert_eq!(subject, principal.to_string());
        assert_eq!(actor, Some(principal.to_string()));
        let (subject, actor) = audit_attribution(&[], "author", &space_uid);
        assert_eq!(subject, "author");
        assert_eq!(actor, None);
        let (subject, _) = audit_attribution(&[], "   ", &space_uid);
        assert_eq!(subject, space_uid.to_string());
    }

    const ENTRY_MARKDOWN: &str = "---\nform: Entry\n---\n# hello\n\n## Body\ncontent";
    const ENTRY_MARKDOWN_V2: &str = "---\nform: Entry\n---\n# hello\n\n## Body\nupdated";

    async fn audit_test_space(slug_suffix: &str) -> anyhow::Result<(UgoiteService, String)> {
        let service = UgoiteService::new(format!("memory://mutation-audit-{slug_suffix}"))?;
        let space_uid = service
            .create_operator_space(&format!("audit-space-{slug_suffix}"))
            .await?;
        Ok((service, space_uid.to_string()))
    }

    async fn audit_total(service: &UgoiteService, space_id: &str) -> anyhow::Result<usize> {
        let listed = crate::audit::list_audit_events(
            service.operator(),
            space_id,
            crate::audit::AuditListOptions::default(),
        )
        .await?;
        Ok(listed.get("total").and_then(Value::as_u64).unwrap_or(0) as usize)
    }

    #[tokio::test]
    async fn committed_entry_mutations_leave_evidence() -> anyhow::Result<()> {
        let (service, space_id) = audit_test_space("evidence").await?;
        let created = service
            .create_entry(&space_id, "entry-1", ENTRY_MARKDOWN, "author")
            .await?;
        let revision_id = created
            .get("revision_id")
            .and_then(Value::as_str)
            .expect("revision id")
            .to_string();
        let updated = service
            .update_entry(&space_id, "entry-1", ENTRY_MARKDOWN_V2, None, "author")
            .await?;
        let updated_revision_id = updated
            .get("revision_id")
            .and_then(Value::as_str)
            .expect("revision id")
            .to_string();
        assert_ne!(revision_id, updated_revision_id);
        service
            .delete_entry(&space_id, "entry-1", false, "author")
            .await?;

        // list_audit_events verifies the hash chain on every read.
        let listed = crate::audit::list_audit_events(
            service.operator(),
            &space_id,
            crate::audit::AuditListOptions::default(),
        )
        .await?;
        assert_eq!(listed.get("total").and_then(Value::as_u64), Some(3));
        let actions: Vec<&str> = listed
            .get("items")
            .and_then(Value::as_array)
            .expect("items")
            .iter()
            .filter_map(|item| item.get("action").and_then(Value::as_str))
            .collect();
        for expected in [
            ENTRY_CREATED_ACTION,
            ENTRY_UPDATED_ACTION,
            ENTRY_DELETED_ACTION,
        ] {
            assert!(
                actions.contains(&expected),
                "missing {expected}: {actions:?}"
            );
        }
        // Allow-list: no Entry body anywhere in the evidence.
        let raw = serde_json::to_string(&listed).expect("serializes");
        assert!(
            !raw.contains("## Body"),
            "audit must not contain Entry content"
        );
        Ok(())
    }

    #[tokio::test]
    async fn redelivering_the_same_revision_never_duplicates() -> anyhow::Result<()> {
        let (service, space_id) = audit_test_space("dedupe").await?;
        let created = service
            .create_entry(&space_id, "entry-1", ENTRY_MARKDOWN, "author")
            .await?;
        let revision_id = created
            .get("revision_id")
            .and_then(Value::as_str)
            .expect("revision id")
            .to_string();
        assert_eq!(audit_total(&service, &space_id).await?, 1);
        // Rebuild the event from committed truth exactly as delivery and
        // reconcile do: converges, never duplicates. A differing payload
        // under the same event_id would fail closed.
        let history = crate::entry::get_entry_history(
            service.operator(),
            &service.workspace_path(&space_id),
            "entry-1",
        )
        .await?;
        let last = history
            .get("revisions")
            .and_then(Value::as_array)
            .and_then(|revisions| revisions.last())
            .expect("committed revision");
        let committed_change_id = last
            .get("change_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let space_uid = service.space_uid(&space_id).await?;
        let event = entry_mutation_event(
            &space_uid,
            ENTRY_CREATED_ACTION,
            "entry-1",
            &revision_id,
            committed_change_id.as_deref(),
            "author",
            None,
        );
        deliver_mutation_audit_event(service.operator(), &space_id, &event).await?;
        deliver_mutation_audit_event(service.operator(), &space_id, &event).await?;
        assert_eq!(audit_total(&service, &space_id).await?, 1);
        Ok(())
    }

    #[tokio::test]
    async fn reconcile_after_successful_delivery_stays_singular() -> anyhow::Result<()> {
        // Guards the live-delivery/reconcile payload contract: both derive
        // from committed truth, so reconcile after success converges instead
        // of failing closed on a payload conflict.
        let (service, space_id) = audit_test_space("reconcile-ok").await?;
        service
            .create_entry(&space_id, "entry-1", ENTRY_MARKDOWN, "author")
            .await?;
        assert_eq!(audit_total(&service, &space_id).await?, 1);
        let delivered = service
            .reconcile_entry_audit(&space_id, "entry-1", &[], "author")
            .await?
            .expect("reconcile converges after success");
        assert_eq!(delivered["action"], json!(ENTRY_CREATED_ACTION));
        assert_eq!(audit_total(&service, &space_id).await?, 1);
        // Unknown targets reconcile to no evidence, not an error.
        assert!(service
            .reconcile_entry_audit(&space_id, "missing-entry", &[], "author")
            .await?
            .is_none());
        assert!(service
            .reconcile_saved_sql_audit(&space_id, "missing-sql", &[], "author")
            .await?
            .is_none());
        Ok(())
    }

    #[tokio::test]
    async fn crash_gap_is_closed_by_reconcile_after_reopen() -> anyhow::Result<()> {
        let (service, space_id) = audit_test_space("crash").await?;
        // Simulate commit-without-delivery by writing through the low-level
        // entry layer, bypassing the audited service method.
        let integrity =
            crate::integrity::RealIntegrityProvider::from_space(service.operator(), &space_id)
                .await?;
        crate::entry::create_entry(
            service.operator(),
            &service.workspace_path(&space_id),
            "entry-1",
            ENTRY_MARKDOWN,
            "author",
            &integrity,
        )
        .await?;
        assert_eq!(audit_total(&service, &space_id).await?, 0);

        // Reopen the Space (fresh service over the same storage) and
        // reconcile: the committed revision becomes audit evidence.
        let root_uri = service.root_uri().to_string();
        let service2 = UgoiteService::from_operator(service.operator().clone(), root_uri);
        let delivered = service2
            .reconcile_entry_audit(&space_id, "entry-1", &[], "author")
            .await?
            .expect("committed revision reconciles");
        assert_eq!(delivered["action"], json!(ENTRY_CREATED_ACTION));
        assert_eq!(audit_total(&service2, &space_id).await?, 1);
        // Reconcile is idempotent too.
        service2
            .reconcile_entry_audit(&space_id, "entry-1", &[], "author")
            .await?;
        assert_eq!(audit_total(&service2, &space_id).await?, 1);
        Ok(())
    }

    #[tokio::test]
    async fn committed_saved_sql_mutations_leave_evidence() -> anyhow::Result<()> {
        let (service, space_id) = audit_test_space("sql").await?;
        let payload = crate::saved_sql::SqlPayload {
            name: Some("q".to_string()),
            kind: crate::saved_sql::SqlKind::UserQuery,
            metadata: None,
            sql: "SELECT * FROM t WHERE secret = 's3cr3t'".to_string(),
            variables: serde_json::json!([]),
        };
        let created = service
            .create_saved_sql(&space_id, "sql-1", &payload, "author")
            .await?;
        let revision_id = created
            .get("revision_id")
            .and_then(Value::as_str)
            .expect("revision id")
            .to_string();
        let update_payload = crate::saved_sql::SqlUpdatePayload {
            name: Some("q".to_string()),
            kind: crate::saved_sql::SqlKind::UserQuery,
            metadata: None,
            sql: "SELECT 1".to_string(),
            variables: serde_json::json!([]),
            parent_revision_id: revision_id,
        };
        service
            .update_saved_sql(
                &space_id,
                "sql-1",
                &update_payload.clone().into_sql_payload(),
                &update_payload.parent_revision_id,
                "author",
            )
            .await?;
        service
            .delete_saved_sql(&space_id, "sql-1", "author")
            .await?;

        let listed = crate::audit::list_audit_events(
            service.operator(),
            &space_id,
            crate::audit::AuditListOptions::default(),
        )
        .await?;
        assert_eq!(listed.get("total").and_then(Value::as_u64), Some(3));
        // Query content never enters the evidence.
        let raw = serde_json::to_string(&listed).expect("serializes");
        assert!(!raw.contains("s3cr3t"), "audit must not contain SQL text");
        Ok(())
    }

    #[tokio::test]
    async fn secret_metadata_is_rejected_fail_closed() -> anyhow::Result<()> {
        let (service, space_id) = audit_test_space("redact").await?;
        let space_uid = service.space_uid(&space_id).await?;
        let mut event = entry_mutation_event(
            &space_uid,
            ENTRY_CREATED_ACTION,
            "entry-1",
            "rev-1",
            None,
            "author",
            None,
        );
        event["metadata"]["token"] = json!("bearer-secret");
        let error = deliver_mutation_audit_event(service.operator(), &space_id, &event)
            .await
            .expect_err("secret material must be rejected");
        assert!(error.to_string().contains("secret"));
        assert_eq!(audit_total(&service, &space_id).await?, 0);
        Ok(())
    }
}
