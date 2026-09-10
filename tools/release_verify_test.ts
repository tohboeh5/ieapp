import { assertEquals } from "@std/assert/equals";
import {
  CANDIDATE_CONTRACT_VERSION,
  CANDIDATE_MANIFEST_SCHEMA_VERSION,
  candidateIdFromManifestBytes,
  createVerificationReceipt,
  findPublishedReleaseFile,
  parseCandidateManifest,
  parsePublishedReleaseManifest,
  parseVerificationReceipt,
  RELEASE_SMOKE_POLICY,
  validatePublishedReleaseManifest,
  validateVerificationReceipt,
} from "./release_verify.ts";

const sourceSha = "a".repeat(40);

function manifestBytes(extra: Record<string, unknown> = {}): Uint8Array {
  return new TextEncoder().encode(
    `${
      JSON.stringify(
        {
          schema_version: CANDIDATE_MANIFEST_SCHEMA_VERSION,
          contract_version: CANDIDATE_CONTRACT_VERSION,
          version: "0.1.0",
          source_sha: sourceSha,
          ci_run_id: "candidate-123",
          source_ci_required_check_run_id: "check-456",
          artifacts: [{
            kind: "image",
            files: [],
            config: {
              repository: "ghcr.io/ugoite/ugoite",
              digest: `sha256:${"c".repeat(64)}`,
            },
          }],
          ...extra,
        },
        null,
        2,
      )
    }\n`,
  );
}

async function assertFails(
  action: () => unknown,
  message: string,
): Promise<void> {
  try {
    await action();
    throw new Error(`expected failure: ${message}`);
  } catch (error) {
    if (!(error instanceof Error) || !error.message.includes(message)) {
      throw error;
    }
  }
}

Deno.test("release verifier core derives candidate identity from exact manifest bytes", async () => {
  const first = manifestBytes();
  const second = manifestBytes({ generated_at: "different" });
  const firstId = await candidateIdFromManifestBytes(first);
  const secondId = await candidateIdFromManifestBytes(second);
  assertEquals(firstId.startsWith("sha256:"), true);
  assertEquals(firstId === secondId, false);
  assertEquals(parseCandidateManifest(first).ci_run_id, "candidate-123");
});

Deno.test("release verifier core rejects verification state in candidate manifests", async () => {
  await assertFails(
    () =>
      parseCandidateManifest(
        manifestBytes({ verification: { result: "passed" } }),
      ),
    "must not contain verification state",
  );
});

Deno.test("release verifier core rejects unsafe candidate artifact paths", async () => {
  await assertFails(
    () =>
      parseCandidateManifest(
        manifestBytes({
          artifacts: [{
            kind: "cli",
            files: [{ path: "../candidate", sha256: "e".repeat(64), size: 1 }],
          }],
        }),
      ),
    "safe relative path",
  );
});

Deno.test("release verifier core creates and validates a separate receipt", async () => {
  const bytes = manifestBytes();
  const candidateId = await candidateIdFromManifestBytes(bytes);
  const receipt = createVerificationReceipt({
    candidateId,
    candidateRunId: "candidate-123",
    verifierWorkflowSha: "d".repeat(40),
    verificationRunId: "verification-789",
  });
  assertEquals(receipt.policy, RELEASE_SMOKE_POLICY);
  assertEquals(parseVerificationReceipt(receipt).result, "passed");
  validateVerificationReceipt(receipt, {
    candidateId,
    candidateRunId: "candidate-123",
    verifierWorkflowSha: "d".repeat(40),
    verificationRunId: "verification-789",
    policy: RELEASE_SMOKE_POLICY,
  });
  await assertFails(
    () =>
      validateVerificationReceipt(receipt, {
        candidateId,
        candidateRunId: "different-candidate",
      }),
    "candidate run",
  );
});

Deno.test("release verifier core rejects malformed receipt evidence", async () => {
  await assertFails(
    () =>
      parseVerificationReceipt({
        schema_version: 1,
        candidate_id: "not-a-digest",
        candidate_run_id: "candidate-123",
        verifier_workflow_sha: "d".repeat(40),
        verification_run_id: "verification-789",
        policy: RELEASE_SMOKE_POLICY,
        result: "passed",
      }),
    "candidate_id must be a sha256",
  );
});

Deno.test("release verifier core validates the published distribution projection", async () => {
  const candidateBytes = manifestBytes();
  const candidateId = await candidateIdFromManifestBytes(candidateBytes);
  const published = parsePublishedReleaseManifest({
    schema_version: 2,
    release_tag: "v0.1.0",
    version: "0.1.0",
    source_sha: sourceSha,
    candidate_id: candidateId,
    files: [{ name: "ugoite.tar.gz", sha256: "e".repeat(64), size: 3 }],
    image: {
      repository: "ghcr.io/ugoite/ugoite",
      digest: `sha256:${"c".repeat(64)}`,
    },
    helm_chart: {
      repository: "oci://ghcr.io/ugoite/charts/ugoite",
      digest: "f".repeat(64),
    },
  });
  validatePublishedReleaseManifest(published, {
    releaseTag: "v0.1.0",
    version: "0.1.0",
    sourceSha,
    imageRepository: "ghcr.io/ugoite/ugoite",
    candidateId,
  });
  assertEquals(findPublishedReleaseFile(published, "ugoite.tar.gz").size, 3);
  assertEquals(published.helm_chart?.digest, "f".repeat(64));
  await assertFails(
    () =>
      validatePublishedReleaseManifest(published, {
        releaseTag: "v0.2.0",
        version: "0.1.0",
        sourceSha,
        imageRepository: "ghcr.io/ugoite/ugoite",
        candidateId,
      }),
    "release tag does not match",
  );
  await assertFails(
    () =>
      parsePublishedReleaseManifest({
        ...published,
        image: { repository: "ghcr.io/ugoite/ugoite", digest: "invalid" },
      }),
    "image.digest must be a sha256 digest",
  );
  await assertFails(
    () =>
      parsePublishedReleaseManifest({
        ...published,
        files: [{ name: "../release.tgz", sha256: "e".repeat(64), size: 3 }],
      }),
    "name must be a file name",
  );
});
