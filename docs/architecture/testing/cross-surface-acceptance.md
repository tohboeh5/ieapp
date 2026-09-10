# Cross-surface acceptance (JOURNEY-KNOWLEDGE-001)

Informational. This document describes the acceptance model, not a new
specification authority. Outcome semantics stay with Mitase Requirement /
Criterion; the operation inventory stays with `UGOITE_API_OPERATIONS` and
its Rust mirror `SUPPORTED_OPERATIONS`.

## Principle: three surfaces, one meaning

Parity means reaching the same durable Knowledge outcome from different
surfaces, not building the same screen or the same command twice. The
Frontend may use a Create dialog while the CLI uses `form update` (an
upsert); parity passes when the persisted Form carries equivalent schema
semantics. Weak discoverability is recorded separately and never blocks a
durability verdict.

## Golden journey

`JOURNEY-KNOWLEDGE-001`: Space create -> Form establish -> Entry create ->
Entry edit -> Search -> History -> Restore.

Each checkpoint proves a durable postcondition, observable through the
canonical read surface after acting through any surface:

| Checkpoint | Postcondition |
| --- | --- |
| Space | A durable Space exists and reopens with identical compatibility semantics. |
| Form | Schema, required fields, and field-type interpretation match. |
| Entry create | An equivalent Knowledge object exists with exactly one new revision. |
| Entry edit | Optimistic concurrency and validation behave identically. |
| Search | The updated durable Entry is found under identical conditions. |
| History | Create and edit are observable as append-only history. |
| Restore | Restore appends a new revision or change; history never shortens. |

Business rules (validation, error classification, concurrency, history)
live in the shared Rust boundary. Fixtures supply values and observe
canonical representations; they never re-implement validation.

## Capability projection (generated, not authoritative)

`tools/capability_report.ts` projects the journey from existing authorities:

- Inventory: `UGOITE_API_OPERATIONS` + `SUPPORTED_OPERATIONS` (must match).
- Surface usage: Frontend `*-api.ts`, CLI remote `http::execute`, CLI core
  `UgoiteService` methods.
- Verification: exact e2e test names plus exact Mitase `verifies` claims.

Run `deno run -A tools/capability_report.ts --markdown` (or `--json`).
The report is informational in 0.1.x: gaps are diagnosed, not build
failures.

Row states:

- `verified`: every surface reaches the outcome with exact evidence.
- `evidence-gap`: reachable, but e2e or Mitase evidence is missing.
- `surface-gap`: at least one surface cannot reach the outcome.
- `semantic-drift`: adapters disagree on inventory or shared encoding.
- `implemented-undiscoverable`: reachable everywhere, but hidden behind
  an alias (for example CLI `form update` fulfilling `form.upsert`).
- `intentionally-not-required`: reserved for obligations scoped to fewer
  surfaces (for example Frontend-only UX polish). No journey row uses it.

Classification priority is
semantic-drift > surface-gap > implemented-undiscoverable >
evidence-gap > verified, so an aliased capability reads as a
discoverability finding rather than a missing capability or a missing test.

## Status and next steps (planned, not yet implemented)

- C0 (this change): informational projection only.
- C1-C3 (planned): Golden journey evidence for Frontend, CLI core
  (`surface=cli, transport=core/local`), and CLI remote
  (`surface=cli, transport=remote`) as separate evidence identities.
- C4 (planned): semantic failure corpus (validation category, stale
  revision conflict, authorization, history semantics). Presentation may
  differ per surface; codes, classification, and durable state must not.
- C5-C7 (planned): capability coverage expansion, then promotion of the
  stable corpus into the release-grade gate. Mitase never executes tests;
  it declares which exact implementation and verification targets prove a
  criterion, and the runner proves they pass.
