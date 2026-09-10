# Space 0.1 compatibility fixture

This directory is frozen compatibility evidence for the first stable Space
generation. The bootstrap metadata and settings are the historical input; the
test opens that input through the current Space reader, exercises the Entry
Form and append-only history paths, closes and reopens the Space, and verifies
the same Knowledge meaning.

`space_version: "0.1"` is the only Space compatibility identity. Product
versions, subsystem-local schema versions, and physical Iceberg encoding are
independent concepts. A missing, malformed, or unsupported Space Version must
fail closed with `UNSUPPORTED_SPACE_VERSION`; the old `schema_version` field is
never inferred as an alias and open never performs migration.
