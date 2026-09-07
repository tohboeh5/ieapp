# Space 0.1 historical compatibility fixture

This is the frozen historical compatibility evidence for the first stable
Space generation (`space_version: "0.1"`).

- Produced by the Space 0.1 implementation (`space::create_space` bootstrap
  metadata contract): `spaces/<space_uid>/meta.json` plus
  `spaces/<space_uid>/settings.json`.
- The `meta.json` bytes below are the authoritative bootstrap input. The
  compatibility test opens this historical metadata through the shared
  `space_version` classification before version-specific validation, recovers
  the expected Knowledge, performs representative supported mutations on a
  live Space 0.1 cloned from this seed, closes, reopens, and verifies
  Knowledge, history, and authority/integrity invariants.
- `space_version` is the stable Space compatibility identity.
  `schema_version` is not used for this purpose.
- Opening this fixture MUST NOT implicitly migrate it. Unsupported versions
  fail closed with `UNSUPPORTED_SPACE_VERSION`.

Product Version and Space Version evolve independently. Space 0.1 does not
record Product release history.
