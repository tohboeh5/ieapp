---
title: 'Requirements registry'
---

Requirement YAML files define stable IDs, descriptions, governance links, implementation status, and generated test traceability. Migrated domains are authoritative in `docs/mitase`; this registry remains authoritative for domains that have not yet been migrated.

The migrated Entry, Form, Indexer, Search, API, Asset, Frontend, and E2E requirements retain their external
operator/API contracts in the canonical graph; an exact test claim is added
only for the behavior that the selected test actually exercises. Search
authorization remains governed by the authoritative Security requirement until
that domain is migrated. Any remaining verification gap remains explicit rather
than being inferred from the existence of a binding.

The legacy Operations requirement registry at `requirements/ops.yaml` is
retired and is no longer included in Mitase's declared inventory. The canonical
Operations requirements at `docs/mitase/requirements/ops.yaml` are the only
semantic authority for the Operations domain: `REQ-OPS-001` through
`REQ-OPS-028`, `REQ-OPS-030`, `REQ-OPS-031`, `REQ-OPS-034` through
`REQ-OPS-037`, `REQ-OPS-040` through `REQ-OPS-042`, and the inserted
`REQ-OPS-043` and `REQ-OPS-044` gates. The superseded `REQ-OPS-032` and
`REQ-OPS-033` records are absorbed by the canonical root task surface and
workspace quality gates and carry no separate canonical requirement. Planned
work and unverified completeness remain explicit gaps in the canonical graph.

The legacy API requirement registry at `requirements/api.yaml` is retired and
is no longer included in Mitase's declared inventory. The canonical API
requirements at `docs/mitase/requirements/api.yaml` are the only semantic
authority for the migrated API domain.

The legacy Asset requirement registry at `requirements/asset.yaml` is retired and
is no longer included in Mitase's declared inventory. The canonical Asset
requirements at `docs/mitase/requirements/assets.yaml` are the only semantic
authority for the migrated Asset domain.

The legacy E2E requirement registry at `requirements/e2e.yaml` is retired and
is no longer included in Mitase's declared inventory. The canonical E2E
requirements at `docs/mitase/requirements/e2e.yaml` are the only semantic
authority for the migrated E2E domain.

The legacy Indexer requirement registry at `requirements/index.yaml` is retired
and is no longer included in Mitase's declared inventory. Its derived-index and
structured-query semantics are represented by the canonical Search and Form
requirements at `docs/mitase/requirements/search.yaml` and
`docs/mitase/requirements/forms.yaml`.

The legacy Search requirement registry at `requirements/search.yaml` is retired
and is no longer included in Mitase's declared inventory. The canonical Search
requirements at `docs/mitase/requirements/search.yaml` are the only semantic
authority for keyword search, structured query, frontend search behavior, and
derived relation maintenance.

The legacy Entry requirement registry at `requirements/entry.yaml` is retired
and is no longer included in Mitase's declared inventory. The canonical Entry
requirements at `docs/mitase/requirements/entries.yaml` are the only semantic
authority for Entry creation, revision, mutation, history, Markdown extraction,
and interface behavior.

The legacy Form requirement registry at `requirements/form.yaml` is retired and
is no longer included in Mitase's declared inventory. The canonical Form
requirements at `docs/mitase/requirements/forms.yaml` are the only semantic
authority for Form schema governance, CRUD operations, reserved metadata, row
references, attribution, and typed property conversion.

The legacy Frontend requirement registry at `requirements/frontend.yaml` is
retired and is no longer included in Mitase's declared inventory. The canonical
Frontend requirements at `docs/mitase/requirements/frontend.yaml` are the only
semantic authority for routes, components, interaction surfaces, API clients,
and exact Frontend verification evidence.

The OIDC external identity requirement `REQ-SEC-016` is represented canonically
at `docs/mitase/requirements/security.yaml`, together with the agent-principal
(`REQ-SEC-009`), credentialed CORS allowlist (`REQ-SEC-010`), CLI endpoint
transport (`REQ-SEC-011`), and remote CLI device authentication (`REQ-SEC-015`)
records. The owner-approved Space Access Recovery (`REQ-SEC-012`) and durable
recovery audit delivery (`REQ-SEC-013`) records are likewise canonical.

The legacy Security requirement registry at `requirements/security.yaml` is
retired and is no longer included in Mitase's declared inventory. The canonical
Security requirements at `docs/mitase/requirements/security.yaml` are the only
semantic authority for the Security domain. The owner-approved Space Access
Recovery regression verifies that the old account's OIDC methods remain
unchanged while the recovered Space binding moves to the fresh account.

The legacy Integrity requirement registry at `requirements/integrity.yaml` is
retired and is no longer included in Mitase's declared inventory. The canonical
Integrity requirements at `docs/mitase/requirements/integrity.yaml` are the only
semantic authority for the Integrity domain.

The legacy Storage requirement registry at `requirements/storage.yaml` is
retired and is no longer included in Mitase's declared inventory. The canonical
Storage requirements at `docs/mitase/requirements/storage.yaml` are the only
semantic authority for the Storage domain: `REQ-STO-001` through `REQ-STO-014`
with the same account-bound retry (`200`), creation (`201`), and duplicate-slug
(`409` with `SPACE_ALREADY_EXISTS`) contract. The canonical Storage connector
record preserves the connector-update and pre-commit validation contract; its
available API/UI surface and core probe evidence are traced, while mandatory
sequencing of an update after successful validation remains an explicit evidence
gap. The canonical accessible-listing record likewise keeps runtime
authorization and storage-error propagation as implementation requirements while
its current verification target covers only the published OpenAPI boundary.
Complete executable parity between every documented layout path and runtime
creation remains an explicit follow-up rather than an inferred guarantee.

A current test mapping has this shape:

```yaml
verification: traced
tests:
  - file: crates/ugoite-iceberg/tests/test_entry.rs
```

When no current source/test file contains the requirement ID:

```yaml
verification: untraced
```

Do not preserve references to deleted tests. Planned requirements may remain untraced. Any requirement that describes a removed architecture should be rewritten to the current behavior or marked superseded.
