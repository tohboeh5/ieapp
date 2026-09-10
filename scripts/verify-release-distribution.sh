#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${UGOITE_GITHUB_REPO:-${UGOITE_RELEASE_REPOSITORY:-ugoite/ugoite}}"
VERSION_INPUT="${UGOITE_VERSION:-}"
RELEASE_TAG_INPUT="${UGOITE_RELEASE_TAG:-v${VERSION_INPUT}}"
RELEASE_SHA_INPUT="${UGOITE_RELEASE_SHA:-}"
CANDIDATE_ID_INPUT="${UGOITE_CANDIDATE_ID:-}"
RELEASE_TOKEN_INPUT="${UGOITE_RELEASE_TOKEN:-}"
ASSET_BASE_URL_INPUT="${UGOITE_RELEASE_ASSET_BASE_URL:-https://github.com/${REPO}/releases/download/${RELEASE_TAG_INPUT}}"
IMAGE_PORT="${UGOITE_DISTRIBUTION_IMAGE_PORT:-18001}"

if [ -n "$RELEASE_TOKEN_INPUT" ] && [ -z "${GH_TOKEN:-}" ]; then
  export GH_TOKEN="$RELEASE_TOKEN_INPUT"
fi

log() {
  printf '%s\n' "$*" >&2
}

fail() {
  log "$*"
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "Required command not found: $1"
}

if [ -z "$VERSION_INPUT" ] || [ -z "$RELEASE_SHA_INPUT" ] || [ -z "$CANDIDATE_ID_INPUT" ]; then
  fail "UGOITE_VERSION, UGOITE_RELEASE_SHA, and UGOITE_CANDIDATE_ID are required"
fi

require_command curl
require_command deno
require_command docker
require_command gh
require_command helm
require_command npm
require_command sha256sum

work_root="$(mktemp -d)"
container_name=""
cleanup() {
  status=$?
  trap - EXIT HUP INT TERM
  if [ -n "$container_name" ] && docker container inspect "$container_name" >/dev/null 2>&1; then
    docker logs "$container_name" >&2 || true
    docker rm -f "$container_name" >/dev/null 2>&1 || true
  fi
  rm -rf "$work_root"
  exit "$status"
}
trap cleanup EXIT HUP INT TERM

download_asset() {
  local asset_name="$1"
  local output_path="$2"
  local -a curl_args=(-fsSL)
  if [ -n "$RELEASE_TOKEN_INPUT" ]; then
    curl_args+=(
      -H "Authorization: Bearer ${RELEASE_TOKEN_INPUT}"
      -H "Accept: application/octet-stream"
    )
  fi
  curl "${curl_args[@]}" "${ASSET_BASE_URL_INPUT%/}/${asset_name}" -o "$output_path"
}

manifest_path="$work_root/release-manifest.json"
download_asset release-manifest.json "$manifest_path"

MANIFEST_PATH="$manifest_path" \
EXPECTED_RELEASE_TAG="$RELEASE_TAG_INPUT" \
EXPECTED_VERSION="$VERSION_INPUT" \
EXPECTED_SOURCE_SHA="$RELEASE_SHA_INPUT" \
EXPECTED_CANDIDATE_ID="$CANDIDATE_ID_INPUT" \
deno eval '
const manifest = JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!));
const expected = {
  release_tag: Deno.env.get("EXPECTED_RELEASE_TAG"),
  version: Deno.env.get("EXPECTED_VERSION"),
  source_sha: Deno.env.get("EXPECTED_SOURCE_SHA"),
  candidate_id: Deno.env.get("EXPECTED_CANDIDATE_ID"),
};
for (const [key, value] of Object.entries(expected)) {
  if (manifest[key] !== value) throw new Error(`${key} does not match the promoted candidate`);
}
if (!manifest.image?.repository || !/^sha256:[0-9a-f]{64}$/.test(manifest.image?.digest ?? "")) {
  throw new Error("release manifest is missing the published image digest");
}
'

immutable="$(gh release view "$RELEASE_TAG_INPUT" --repo "$REPO" --json isImmutable --jq .isImmutable)"
[ "$immutable" = "true" ] || fail "GitHub Release ${RELEASE_TAG_INPUT} is not immutable"

candidate_manifest_path="$work_root/candidate-manifest.json"
candidate_id_path="$work_root/candidate-id.txt"
download_asset candidate-manifest.json "$candidate_manifest_path"
download_asset candidate-id.txt "$candidate_id_path"
candidate_manifest_digest="$(sha256sum "$candidate_manifest_path" | awk '{print $1}')"
[ "sha256:${candidate_manifest_digest}" = "$CANDIDATE_ID_INPUT" ] || fail "Published candidate manifest differs from candidate ID"
[ "$(tr -d '\r\n' <"$candidate_id_path")" = "$CANDIDATE_ID_INPUT" ] || fail "Published candidate ID asset differs from promotion"

receipt_asset_name="$(gh release view "$RELEASE_TAG_INPUT" --repo "$REPO" --json assets --jq '.assets[].name' | awk '/^verification-receipt-[^/]+\.json$/ { print; exit }')"
[ -n "$receipt_asset_name" ] || fail "published release has no verification receipt asset"
receipt_path="$work_root/$receipt_asset_name"
download_asset "$receipt_asset_name" "$receipt_path"
MANIFEST_PATH="$manifest_path" \
  CANDIDATE_MANIFEST_PATH="$candidate_manifest_path" \
  VERIFICATION_RECEIPT_PATH="$receipt_path" \
  EXPECTED_CANDIDATE_ID="$CANDIDATE_ID_INPUT" \
  deno eval '
const {
  candidateIdFromManifestBytes,
  parseCandidateManifest,
  parsePublishedReleaseManifest,
  parseVerificationReceipt,
  RELEASE_SMOKE_POLICY,
  validateVerificationReceipt,
} = await import("./tools/release_verify.ts");
const release = parsePublishedReleaseManifest(
  JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!)),
);
const candidateBytes = await Deno.readFile(Deno.env.get("CANDIDATE_MANIFEST_PATH")!);
const candidate = parseCandidateManifest(candidateBytes);
const candidateId = await candidateIdFromManifestBytes(candidateBytes);
if (candidateId !== Deno.env.get("EXPECTED_CANDIDATE_ID")) {
  throw new Error("candidate manifest digest differs from the promoted candidate");
}
if (release.candidate_id !== candidateId) {
  throw new Error("release manifest candidate ID differs from the candidate manifest");
}
const receipt = parseVerificationReceipt(
  JSON.parse(await Deno.readTextFile(Deno.env.get("VERIFICATION_RECEIPT_PATH")!)),
);
validateVerificationReceipt(receipt, {
  candidateId,
  candidateRunId: candidate.ci_run_id,
  policy: RELEASE_SMOKE_POLICY,
});
'

asset_names=()
while IFS= read -r asset_name; do
  [ -n "$asset_name" ] && asset_names+=("$asset_name")
done < <(gh release view "$RELEASE_TAG_INPUT" --repo "$REPO" --json assets --jq '.assets[].name')
[ "${#asset_names[@]}" -gt 0 ] || fail "GitHub Release has no assets"

for asset_name in "${asset_names[@]}"; do
  case "$asset_name" in
    candidate-manifest.json|candidate-id.txt|release-manifest.json|verification-receipt-*.json)
      continue
      ;;
  esac
  expected_sha="$(ASSET_NAME="$asset_name" MANIFEST_PATH="$manifest_path" deno eval '
const manifest = JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!));
const name = Deno.env.get("ASSET_NAME")!;
const file = manifest.files?.find((entry: { name?: string }) => entry.name === name);
if (!file?.sha256) Deno.exit(1);
console.log(file.sha256);
')" || fail "Release asset ${asset_name} is absent from release manifest"
  asset_path="$work_root/$asset_name"
  download_asset "$asset_name" "$asset_path"
  actual_sha="$(sha256sum "$asset_path" | awk '{print $1}')"
  [ "$actual_sha" = "$expected_sha" ] || fail "Release asset ${asset_name} differs from its manifest digest"
done
log "Verified GitHub Release assets"

expected_npm_sha="$(MANIFEST_PATH="$manifest_path" deno eval '
const manifest = JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!));
const file = manifest.files?.find((entry: { name?: string }) => entry.name?.startsWith("ugoite-ugoite-") && entry.name.endsWith(".tgz"));
if (!file?.sha256) Deno.exit(1);
console.log(file.sha256);
')" || fail "Release manifest is missing npm digest"
npm_url="$(npm view "@ugoite/ugoite@${VERSION_INPUT}" dist.tarball --json | tr -d '"')"
npm_path="$work_root/npm.tgz"
declare -a npm_curl_args=(-fsSL)
if [ -n "${NODE_AUTH_TOKEN:-}" ]; then
  npm_curl_args+=(
    -H "Authorization: Bearer ${NODE_AUTH_TOKEN}"
    -H "Accept: application/octet-stream"
  )
fi
curl "${npm_curl_args[@]}" "$npm_url" -o "$npm_path"
[ "$(sha256sum "$npm_path" | awk '{print $1}')" = "$expected_npm_sha" ] || fail "Published npm package differs from candidate"

helm_digest="$(MANIFEST_PATH="$manifest_path" deno eval '
const manifest = JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!));
if (!manifest.helm_chart?.digest) Deno.exit(1);
console.log(manifest.helm_chart.digest);
')" || fail "Release manifest is missing Helm digest"
helm_dir="$work_root/helm"
mkdir -p "$helm_dir"
helm pull oci://ghcr.io/ugoite/charts/ugoite --version "$VERSION_INPUT" --destination "$helm_dir" >/dev/null
helm_path="$helm_dir/ugoite-${VERSION_INPUT}.tgz"
[ "$(sha256sum "$helm_path" | awk '{print $1}')" = "$helm_digest" ] || fail "Published Helm chart differs from candidate"
log "Verified published npm and Helm artifacts"

image_repository="$(MANIFEST_PATH="$manifest_path" deno eval '
const manifest = JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!));
console.log(manifest.image.repository);
')"
image_digest="$(MANIFEST_PATH="$manifest_path" deno eval '
const manifest = JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!));
console.log(manifest.image.digest);
')"
published_digest="$(docker buildx imagetools inspect "${image_repository}:${VERSION_INPUT}" --format '{{json .Manifest.Digest}}' | tr -d '"')"
[ "$published_digest" = "$image_digest" ] || fail "Published container digest differs from candidate"

container_name="ugoite-distribution-smoke-$$"
node_secret_key="$(head -c 32 /dev/urandom | base64 | tr -d '\n')"
docker run -d \
  --name "$container_name" \
  --tmpfs /data \
  -p "127.0.0.1:${IMAGE_PORT}:8000" \
  -e UGOITE_ROOT=/data \
  -e UGOITE_SERVER_ADDRESS=0.0.0.0:8000 \
  -e UGOITE_STATIC_DIR=/app/static \
  -e UGOITE_PUBLIC_ORIGIN="http://localhost:${IMAGE_PORT}" \
  -e UGOITE_API_BASE_URL="http://localhost:${IMAGE_PORT}/api" \
  -e UGOITE_WEBAUTHN_RP_ID=localhost \
  -e UGOITE_NODE_SECRET_KEY="$node_secret_key" \
  "${image_repository}:${VERSION_INPUT}" >/dev/null
bash "$SCRIPT_DIR/wait-for-http.sh" "http://localhost:${IMAGE_PORT}/health" 120
log "Verified published container availability and health"

install_home="$work_root/cli-home"
install_dir="$install_home/.local/bin"
mkdir -p "$install_home"
HOME="$install_home" \
  UGOITE_VERSION="$VERSION_INPUT" \
  UGOITE_INSTALL_DIR="$install_dir" \
  UGOITE_DOWNLOAD_BASE_URL="${ASSET_BASE_URL_INPUT%/}" \
  UGOITE_RELEASE_TOKEN="$RELEASE_TOKEN_INPUT" \
  UGOITE_TARGET_OVERRIDE=x86_64-unknown-linux-gnu \
  bash "$SCRIPT_DIR/install-ugoite-cli.sh"
installed_version="$("$install_dir/ugoite" --version 2>&1)"
printf '%s' "$installed_version" | grep -Fq "$VERSION_INPUT" || fail "CLI installer did not install ${VERSION_INPUT}"
log "Verified CLI installer distribution"
