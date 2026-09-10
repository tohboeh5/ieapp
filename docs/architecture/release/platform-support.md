---
title: "Release platform support"
sidebar:
  order: 4
---

Ugoite separates the platforms it promises for every release from platforms that
may be explored without making the release gate depend on them. This policy
describes support tiers; it does not create a second artifact authority or
change the portable Knowledge contract.

## Current tiers

| Tier        | Current assignment                                                                                                                                                                                | Release expectation                                                                                                                                                      |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Tier 1      | CLI: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, and `aarch64-apple-darwin`; container: Linux `amd64` and `arm64`; npm and Helm packages are platform-neutral | Build, verify, and include the exact artifact in every Release Candidate and verify its published distribution before announcement                                       |
| Tier 2      | None currently                                                                                                                                                                                    | A future target may be built and exercised in nightly or explicitly requested validation, but its result does not block a release until the target is promoted to Tier 1 |
| Unsupported | Every target not explicitly listed in Tier 1 or Tier 2, including Windows targets                                                                                                                 | Do not describe the target as supported or require it in candidate generation, promotion, or distribution verification                                                   |

The current four CLI targets remain Tier 1 even though they use different hosted
runners. The container's two Linux architectures are one versioned
multi-architecture image identity, not separate product versions. Registry
packages without a platform-specific payload remain covered by the versioned
package checks rather than by a platform matrix.

## Tier changes

A target can move from unsupported to Tier 2 after a maintainer records a
working build and minimum runtime smoke in a nightly or manual validation
context. A target can move from Tier 2 to Tier 1 only after its build, candidate
verification, and published-distribution checks are automated and reliable on
the supported runner. A target leaves Tier 1 only through an explicit
release-policy change that records the compatibility and support impact;
artifact reuse from an earlier release is not a substitute for that decision.

Support claims must use these tiers. A successful build on an unlisted platform
is useful evidence for a future tier, but it is not a release promise.
