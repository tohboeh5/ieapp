export const CANDIDATE_MANIFEST_SCHEMA_VERSION = 4;
export const CANDIDATE_CONTRACT_VERSION = 4;
export const RELEASE_MANIFEST_SCHEMA_VERSION = 3;
export const VERIFICATION_RECEIPT_SCHEMA_VERSION = 1;
export const RELEASE_SMOKE_POLICY = "release-smoke-v1";

export type CandidateFile = {
  path: string;
  sha256: string;
  size: number;
};

export type CandidateArtifact = {
  kind: "cli" | "npm" | "helm" | "image" | "release";
  files: CandidateFile[];
  config?: Record<string, string>;
};

export type CandidateManifest = {
  schema_version: number;
  contract_version: number;
  version: string;
  source_sha: string;
  ci_run_id: string;
  source_ci_required_check_run_id: string;
  artifacts: CandidateArtifact[];
};

export type VerificationReceipt = {
  schema_version: number;
  candidate_id: string;
  candidate_run_id: string;
  verifier_workflow_sha: string;
  verification_run_id: string;
  policy: string;
  result: "passed";
};

export type VerificationReceiptInput = {
  candidateId: string;
  candidateRunId: string;
  verifierWorkflowSha: string;
  verificationRunId: string;
  policy?: string;
};

export type VerificationReceiptExpectation = {
  candidateId: string;
  candidateRunId: string;
  verifierWorkflowSha?: string;
  verificationRunId?: string;
  policy?: string;
};

export type PublishedReleaseFile = {
  name: string;
  sha256: string;
  size: number;
};

export type PublishedReleaseManifest = {
  schema_version: number;
  release_tag: string;
  version: string;
  source_sha: string;
  candidate_id: string;
  files: PublishedReleaseFile[];
  image: {
    repository: string;
    digest: string;
  };
  npm_package: {
    name: string;
    digest: string;
  };
  helm_chart: {
    repository: string;
    digest: string;
  };
};

export type PublishedReleaseExpectation = {
  releaseTag: string;
  version: string;
  sourceSha: string;
  candidateId: string;
  imageRepository: string;
};

const stableVersionPattern = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
const gitShaPattern = /^[0-9a-f]{40}$/;
const digestPattern = /^sha256:[0-9a-f]{64}$/;
const fileDigestPattern = /^[0-9a-f]{64}$/;

export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", bytes as BufferSource);
  return [...new Uint8Array(digest)].map((value) =>
    value.toString(16).padStart(2, "0")
  ).join("");
}

export async function candidateIdFromManifestBytes(
  bytes: Uint8Array,
): Promise<string> {
  return `sha256:${await sha256Hex(bytes)}`;
}

export function parseCandidateManifest(bytes: Uint8Array): CandidateManifest {
  let value: unknown;
  try {
    value = JSON.parse(new TextDecoder().decode(bytes));
  } catch (error) {
    throw new Error(
      `candidate manifest is not valid JSON: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
  if (!isRecord(value)) throw new Error("candidate manifest must be an object");
  if (value.schema_version !== CANDIDATE_MANIFEST_SCHEMA_VERSION) {
    throw new Error(
      `candidate manifest schema_version must be ${CANDIDATE_MANIFEST_SCHEMA_VERSION}`,
    );
  }
  if (value.contract_version !== CANDIDATE_CONTRACT_VERSION) {
    throw new Error(
      `candidate manifest contract_version must be ${CANDIDATE_CONTRACT_VERSION}`,
    );
  }
  if ("verification" in value) {
    throw new Error(
      "candidate manifest must not contain verification state; use a verification receipt",
    );
  }
  if (
    typeof value.version !== "string" ||
    !stableVersionPattern.test(value.version)
  ) {
    throw new Error(
      `candidate manifest version must be stable SemVer x.y.z, got ${
        String(value.version)
      }`,
    );
  }
  requireGitSha(value.source_sha, "candidate source_sha");
  requireNonEmptyString(value.ci_run_id, "candidate ci_run_id");
  requireNonEmptyString(
    value.source_ci_required_check_run_id,
    "candidate source_ci_required_check_run_id",
  );
  if (!Array.isArray(value.artifacts)) {
    throw new Error("candidate manifest artifacts must be an array");
  }
  const artifacts = value.artifacts.map((artifact, index) =>
    parseCandidateArtifact(artifact, index)
  );
  return {
    schema_version: value.schema_version,
    contract_version: value.contract_version,
    version: value.version,
    source_sha: value.source_sha,
    ci_run_id: value.ci_run_id,
    source_ci_required_check_run_id: value.source_ci_required_check_run_id,
    artifacts,
  };
}

export function createVerificationReceipt(
  input: VerificationReceiptInput,
): VerificationReceipt {
  const policy = input.policy ?? RELEASE_SMOKE_POLICY;
  assertCandidateId(input.candidateId);
  requireNonEmptyString(input.candidateRunId, "candidate run ID");
  requireGitSha(input.verifierWorkflowSha, "verifier workflow SHA");
  requireNonEmptyString(input.verificationRunId, "verification run ID");
  requireNonEmptyString(policy, "verification policy");
  return {
    schema_version: VERIFICATION_RECEIPT_SCHEMA_VERSION,
    candidate_id: input.candidateId,
    candidate_run_id: input.candidateRunId,
    verifier_workflow_sha: input.verifierWorkflowSha,
    verification_run_id: input.verificationRunId,
    policy,
    result: "passed",
  };
}

export function parseVerificationReceipt(
  value: unknown,
): VerificationReceipt {
  if (!isRecord(value)) {
    throw new Error("verification receipt must be an object");
  }
  if (value.schema_version !== VERIFICATION_RECEIPT_SCHEMA_VERSION) {
    throw new Error(
      `verification receipt schema_version must be ${VERIFICATION_RECEIPT_SCHEMA_VERSION}`,
    );
  }
  assertCandidateId(value.candidate_id, "verification receipt candidate_id");
  requireNonEmptyString(
    value.candidate_run_id,
    "verification receipt candidate_run_id",
  );
  requireGitSha(
    value.verifier_workflow_sha,
    "verification receipt verifier_workflow_sha",
  );
  requireNonEmptyString(
    value.verification_run_id,
    "verification receipt verification_run_id",
  );
  requireNonEmptyString(value.policy, "verification receipt policy");
  if (value.result !== "passed") {
    throw new Error("verification receipt result must be passed");
  }
  return {
    schema_version: value.schema_version,
    candidate_id: value.candidate_id,
    candidate_run_id: value.candidate_run_id,
    verifier_workflow_sha: value.verifier_workflow_sha,
    verification_run_id: value.verification_run_id,
    policy: value.policy,
    result: "passed",
  };
}

export function validateVerificationReceipt(
  receipt: VerificationReceipt,
  expected: VerificationReceiptExpectation,
): void {
  if (receipt.candidate_id !== expected.candidateId) {
    throw new Error(
      `verification receipt candidate ${receipt.candidate_id} does not match ${expected.candidateId}`,
    );
  }
  if (receipt.candidate_run_id !== expected.candidateRunId) {
    throw new Error(
      `verification receipt candidate run ${receipt.candidate_run_id} does not match ${expected.candidateRunId}`,
    );
  }
  if (
    expected.verifierWorkflowSha &&
    receipt.verifier_workflow_sha !== expected.verifierWorkflowSha
  ) {
    throw new Error(
      "verification receipt verifier workflow SHA does not match",
    );
  }
  if (
    expected.verificationRunId &&
    receipt.verification_run_id !== expected.verificationRunId
  ) {
    throw new Error("verification receipt verification run ID does not match");
  }
  if (expected.policy && receipt.policy !== expected.policy) {
    throw new Error("verification receipt policy does not match");
  }
}

export function parsePublishedReleaseManifest(
  value: unknown,
): PublishedReleaseManifest {
  if (!isRecord(value)) throw new Error("release manifest must be an object");
  if (value.schema_version !== RELEASE_MANIFEST_SCHEMA_VERSION) {
    throw new Error(
      `release manifest schema_version must be ${RELEASE_MANIFEST_SCHEMA_VERSION}`,
    );
  }
  requireNonEmptyString(value.release_tag, "release manifest release_tag");
  requireNonEmptyString(value.version, "release manifest version");
  if (!stableVersionPattern.test(value.version)) {
    throw new Error("release manifest version must be stable SemVer x.y.z");
  }
  requireGitSha(value.source_sha, "release manifest source_sha");
  assertCandidateId(value.candidate_id, "release manifest candidate_id");
  if (!Array.isArray(value.files)) {
    throw new Error("release manifest files must be an array");
  }
  const files = value.files.map((file, index) =>
    parsePublishedReleaseFile(file, `${index}`)
  );
  if (!isRecord(value.image)) {
    throw new Error("release manifest image must be an object");
  }
  requireNonEmptyString(
    value.image.repository,
    "release manifest image.repository",
  );
  if (
    typeof value.image.digest !== "string" ||
    !digestPattern.test(value.image.digest)
  ) {
    throw new Error("release manifest image.digest must be a sha256 digest");
  }
  const npmPackage = parsePublishedNpmPackage(value.npm_package);
  const helmChart = parsePublishedHelmChart(value.helm_chart);
  return {
    schema_version: value.schema_version,
    release_tag: value.release_tag,
    version: value.version,
    source_sha: value.source_sha,
    candidate_id: value.candidate_id,
    files,
    image: {
      repository: value.image.repository,
      digest: value.image.digest,
    },
    npm_package: npmPackage,
    helm_chart: helmChart,
  };
}

export function validatePublishedReleaseManifest(
  manifest: PublishedReleaseManifest,
  expected: PublishedReleaseExpectation,
): void {
  if (manifest.release_tag !== expected.releaseTag) {
    throw new Error("release manifest release tag does not match");
  }
  if (manifest.version !== expected.version) {
    throw new Error("release manifest version does not match");
  }
  if (manifest.source_sha !== expected.sourceSha) {
    throw new Error("release manifest source SHA does not match");
  }
  if (manifest.candidate_id !== expected.candidateId) {
    throw new Error("release manifest candidate ID does not match");
  }
  if (manifest.image.repository !== expected.imageRepository) {
    throw new Error("release manifest image repository does not match");
  }
}

export function findPublishedReleaseFile(
  manifest: PublishedReleaseManifest,
  name: string,
): PublishedReleaseFile {
  const file = manifest.files.find((entry) => entry.name === name);
  if (!file) throw new Error(`${name} is absent from release manifest`);
  return file;
}

function parseCandidateArtifact(
  value: unknown,
  index: number,
): CandidateArtifact {
  if (!isRecord(value)) {
    throw new Error(`candidate artifact ${index} must be an object`);
  }
  const kind = value.kind;
  if (
    kind !== "cli" && kind !== "npm" && kind !== "helm" &&
    kind !== "image" && kind !== "release"
  ) {
    throw new Error(`candidate artifact ${index} has an unsupported kind`);
  }
  if (!Array.isArray(value.files)) {
    throw new Error(`candidate artifact ${index} files must be an array`);
  }
  const files = value.files.map((file, fileIndex) =>
    parseCandidateFile(file, `${index}.${fileIndex}`)
  );
  let config: Record<string, string> | undefined;
  if (value.config !== undefined) {
    if (!isRecord(value.config)) {
      throw new Error(`candidate artifact ${index} config must be an object`);
    }
    config = {};
    for (const [key, item] of Object.entries(value.config)) {
      if (typeof item !== "string") {
        throw new Error(
          `candidate artifact ${index} config.${key} must be a string`,
        );
      }
      config[key] = item;
    }
  }
  return { kind, files, ...(config ? { config } : {}) };
}

function parseCandidateFile(value: unknown, label: string): CandidateFile {
  if (!isRecord(value)) {
    throw new Error(`candidate file ${label} must be an object`);
  }
  requireNonEmptyString(value.path, `candidate file ${label} path`);
  requireSafeRelativePath(value.path, `candidate file ${label} path`);
  if (
    typeof value.sha256 !== "string" || !fileDigestPattern.test(value.sha256)
  ) {
    throw new Error(
      `candidate file ${label} sha256 must be a lowercase SHA-256 digest`,
    );
  }
  if (
    typeof value.size !== "number" || !Number.isSafeInteger(value.size) ||
    value.size < 0
  ) {
    throw new Error(
      `candidate file ${label} size must be a non-negative integer`,
    );
  }
  return { path: value.path, sha256: value.sha256, size: value.size };
}

function parsePublishedReleaseFile(
  value: unknown,
  label: string,
): PublishedReleaseFile {
  if (!isRecord(value)) {
    throw new Error(`release manifest file ${label} must be an object`);
  }
  requireNonEmptyString(value.name, `release manifest file ${label} name`);
  if (value.name.includes("/") || value.name.includes("\\")) {
    throw new Error(`release manifest file ${label} name must be a file name`);
  }
  if (
    typeof value.sha256 !== "string" || !fileDigestPattern.test(value.sha256)
  ) {
    throw new Error(`release manifest file ${label} sha256 is invalid`);
  }
  if (
    typeof value.size !== "number" || !Number.isSafeInteger(value.size) ||
    value.size < 0
  ) {
    throw new Error(`release manifest file ${label} size is invalid`);
  }
  return { name: value.name, sha256: value.sha256, size: value.size };
}

function parsePublishedHelmChart(value: unknown): {
  repository: string;
  digest: string;
} {
  if (!isRecord(value)) {
    throw new Error("release manifest helm_chart must be an object");
  }
  requireNonEmptyString(
    value.repository,
    "release manifest helm_chart.repository",
  );
  if (
    typeof value.digest !== "string" || !fileDigestPattern.test(value.digest)
  ) {
    throw new Error(
      "release manifest helm_chart.digest must be a SHA-256 digest",
    );
  }
  return { repository: value.repository, digest: value.digest };
}

function parsePublishedNpmPackage(value: unknown): {
  name: string;
  digest: string;
} {
  if (!isRecord(value)) {
    throw new Error("release manifest npm_package must be an object");
  }
  requireNonEmptyString(value.name, "release manifest npm_package.name");
  if (
    typeof value.digest !== "string" || !fileDigestPattern.test(value.digest)
  ) {
    throw new Error(
      "release manifest npm_package.digest must be a SHA-256 digest",
    );
  }
  return { name: value.name, digest: value.digest };
}

function assertCandidateId(
  value: unknown,
  label = "candidate ID",
): asserts value is string {
  if (typeof value !== "string" || !digestPattern.test(value)) {
    throw new Error(`${label} must be a sha256: digest`);
  }
}

function requireGitSha(value: unknown, label: string): asserts value is string {
  if (typeof value !== "string" || !gitShaPattern.test(value)) {
    throw new Error(`${label} must be a 40-character Git commit SHA`);
  }
}

function requireNonEmptyString(
  value: unknown,
  label: string,
): asserts value is string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} is required`);
  }
}

function requireSafeRelativePath(value: string, label: string): void {
  if (
    value.startsWith("/") || value.startsWith("\\") ||
    /^[A-Za-z]:[\\/]/.test(value) || value.includes("\\") ||
    value.split("/").some((part) =>
      part === "" || part === "." || part === ".."
    )
  ) {
    throw new Error(`${label} must be a safe relative path`);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
