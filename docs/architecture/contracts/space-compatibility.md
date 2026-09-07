---
title: "Space compatibility and migration contract"
---

Ugoite Product versions and Space versions evolve independently.

A Space version changes only when Ugoite must introduce an incompatible
change to durable Space data. Compatible Product releases continue using
the same Space version.

Space versions begin at `0.1` and advance by compatibility generation,
such as `0.1`, `0.2`, and `0.3`, regardless of Product minor version.

When a future incompatible Space generation is introduced under a new
Product major, the Space major moves to that Product major, beginning at
`.0`, for example `1.0`.

Product major releases do not themselves force Space migrations.

## Current generation

The current stable Space generation is `0.1`.

Every stable Space exposes it through bootstrap metadata before
version-specific durable structures are interpreted:

```json
{
  "space_version": "0.1"
}
```

`schema_version` is not the stable Space compatibility identity.

Opening a Space occurs in this order:

1. locate bootstrap metadata;
2. parse enough metadata to obtain `space_version`;
3. classify the Space Version;
4. reject unsupported versions before authoritative mutation;
5. only then perform version-specific metadata validation and normal operations.

Missing, malformed, or unsupported Space Versions fail closed with the
typed `UNSUPPORTED_SPACE_VERSION` error. The error includes the detected
version and the Product's supported versions so the caller can explain
that another Product version or an explicit migration is required.

Opening a Space never implicitly migrates it.

## Compatibility promise

Space compatibility is a forward compatibility promise. If a later stable
Ugoite release claims support for Space Version `V`, it MUST be able to
directly open and safely operate on historical stable Spaces of `V`.

There is no downgrade promise: an older Product MAY reject a
representation first written by a newer Product, provided it detects the
incompatibility before authoritative mutation, fails closed, does not
guess unknown durable data, and does not overwrite or silently normalize
it.

Individual files, tables, manifests, or metadata documents MAY have their
own local format versions independent from Space Version. A compatible
local format evolution does not require a Space Version change.

A dependency upgrade changing Space bytes does not itself require a new
Space Version. The frozen historical compatibility tests determine whether
the durable contract remains satisfied.

## Migration rules

An incompatible durable Space change requires a new Space Version, an
explicit migration from the older supported generation, and compatibility
evidence for both the historical input and migrated result.

Migration is explicit: normal open, read, or write operations MUST NOT
silently migrate. Migration preserves Knowledge semantics, stable
identities, authoritative history, append-only semantics, authentication
and integrity guarantees, and user ownership/portability — not physical
byte equivalence.

Supported migration paths are explicit and tested. Ugoite does not ship a
generic migration graph, registry, trait hierarchy, downgrade framework,
or generalized migration engine before a real migration requires one.

## Evidence

Every published stable Space generation has a frozen historical physical
fixture under `fixtures/spaces/<version>/`. Compatibility tests open the
historical Space, recover expected Knowledge, perform representative
mutations, close, reopen, verify Knowledge/history, and verify authority
and integrity invariants. These tests run in ordinary required CI.

Introducing a new Space generation is intentional and CI-enforced: the
version differs from the previous generation, its major matches the
current Product major, generations advance sequentially within a Product
major, the first generation under a new Product major is `<major>.0`,
previous historical evidence remains present, and migration evidence
exists when previously stable Spaces require migration.
