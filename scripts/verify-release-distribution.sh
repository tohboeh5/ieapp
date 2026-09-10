#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION_INPUT="${UGOITE_VERSION:-}"
RELEASE_TAG_INPUT="${UGOITE_RELEASE_TAG:-v${VERSION_INPUT}}"
RELEASE_SHA_INPUT="${UGOITE_RELEASE_SHA:-}"
IMAGE_REPOSITORY="${UGOITE_IMAGE_REPOSITORY:-ghcr.io/ugoite/ugoite}"
RELEASE_TOKEN_INPUT="${UGOITE_RELEASE_TOKEN:-}"
ASSET_BASE_URL_INPUT="${UGOITE_RELEASE_ASSET_BASE_URL:-}"
INSTALL_DIR_INPUT="${UGOITE_INSTALL_DIR:-}"

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

download_asset() {
  local asset_name="$1"
  local output_path="$2"
  local attempt
  local -a curl_args=(-fsSL)

  if [ -n "$RELEASE_TOKEN_INPUT" ]; then
    curl_args+=(
      -H "Authorization: Bearer ${RELEASE_TOKEN_INPUT}"
      -H "Accept: application/octet-stream"
    )
  fi

  for attempt in $(seq 1 10); do
    if curl "${curl_args[@]}" -o "$output_path" "${ASSET_BASE_URL}/${asset_name}"; then
      return 0
    fi
    if [ "$attempt" -eq 10 ]; then
      fail "Failed to download ${asset_name} after ${attempt} attempts"
    fi
    sleep 3
  done
}

detect_target() {
  case "$(uname -s):$(uname -m)" in
    Linux:x86_64) printf '%s' 'x86_64-unknown-linux-gnu' ;;
    Linux:arm64 | Linux:aarch64) printf '%s' 'aarch64-unknown-linux-gnu' ;;
    Darwin:x86_64) printf '%s' 'x86_64-apple-darwin' ;;
    Darwin:arm64 | Darwin:aarch64) printf '%s' 'aarch64-apple-darwin' ;;
    *) fail "Unsupported release CLI target: $(uname -s) $(uname -m)" ;;
  esac
}

verify_checksum() {
  local archive_path="$1"
  local checksum_path="$2"
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$(dirname "$archive_path")" && sha256sum -c "$(basename "$checksum_path")")
    return
  fi
  local expected actual
  expected="$(awk '{print $1}' <"$checksum_path")"
  actual="$(shasum -a 256 "$archive_path" | awk '{print $1}')"
  [ "$expected" = "$actual" ] || fail "Checksum verification failed for $(basename "$archive_path")"
}

if [ -z "$VERSION_INPUT" ]; then
  fail "UGOITE_VERSION must be set to the exact release version"
fi
if [ -z "$RELEASE_SHA_INPUT" ]; then
  fail "UGOITE_RELEASE_SHA must be set to the prepared release commit"
fi

require_command curl
require_command deno
require_command docker
require_command tar

ASSET_BASE_URL="${ASSET_BASE_URL_INPUT:-https://github.com/ugoite/ugoite/releases/download/${RELEASE_TAG_INPUT}}"
WORK_ROOT="$(mktemp -d)"
INSTALL_DIR="${INSTALL_DIR_INPUT:-$WORK_ROOT/bin}"
CLI_TARGET="$(detect_target)"
CLI_ARCHIVE="ugoite-${RELEASE_TAG_INPUT}-${CLI_TARGET}.tar.gz"
CLI_CHECKSUM="${CLI_ARCHIVE}.sha256"
CONTAINER_NAME="ugoite-distribution-${RANDOM}-${RANDOM}"
CONTAINER_STARTED=0

cleanup() {
  status=$?
  trap - EXIT HUP INT TERM
  if [ "$CONTAINER_STARTED" -eq 1 ]; then
    docker rm --force "$CONTAINER_NAME" >/dev/null 2>&1 || true
  fi
  rm -rf "$WORK_ROOT"
  exit "$status"
}
trap cleanup EXIT HUP INT TERM

mkdir -p "$WORK_ROOT/assets"
download_asset candidate-manifest.json "$WORK_ROOT/assets/candidate-manifest.json"
download_asset release-manifest.json "$WORK_ROOT/assets/release-manifest.json"
download_asset docker-compose.release.yaml "$WORK_ROOT/assets/docker-compose.release.yaml"
download_asset docker-compose.release.yaml.sha256 "$WORK_ROOT/assets/docker-compose.release.yaml.sha256"
download_asset "$CLI_ARCHIVE" "$WORK_ROOT/assets/$CLI_ARCHIVE"
download_asset "$CLI_CHECKSUM" "$WORK_ROOT/assets/$CLI_CHECKSUM"
verify_checksum "$WORK_ROOT/assets/docker-compose.release.yaml" \
  "$WORK_ROOT/assets/docker-compose.release.yaml.sha256"
verify_checksum "$WORK_ROOT/assets/$CLI_ARCHIVE" "$WORK_ROOT/assets/$CLI_CHECKSUM"

export MANIFEST_PATH="$WORK_ROOT/assets/release-manifest.json"
export CANDIDATE_MANIFEST_PATH="$WORK_ROOT/assets/candidate-manifest.json"
export COMPOSE_PATH="$WORK_ROOT/assets/docker-compose.release.yaml"
export COMPOSE_CHECKSUM_PATH="$WORK_ROOT/assets/docker-compose.release.yaml.sha256"
export CLI_ARCHIVE_PATH="$WORK_ROOT/assets/$CLI_ARCHIVE"
export CLI_ARCHIVE_NAME="$CLI_ARCHIVE"
export RELEASE_TAG_INPUT VERSION_INPUT RELEASE_SHA_INPUT IMAGE_REPOSITORY
deno eval '
const fail = (message: string): never => {
  console.error(`distribution validation failed: ${message}`);
  Deno.exit(1);
};
const manifest = JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!));
const candidate = JSON.parse(await Deno.readTextFile(Deno.env.get("CANDIDATE_MANIFEST_PATH")!));
if (manifest.release_tag !== Deno.env.get("RELEASE_TAG_INPUT")) fail("release tag mismatch");
if (manifest.version !== Deno.env.get("VERSION_INPUT")) fail("version mismatch");
if (manifest.source_sha !== Deno.env.get("RELEASE_SHA_INPUT")) fail("source SHA mismatch");
if (manifest.image?.repository !== Deno.env.get("IMAGE_REPOSITORY")) fail("image repository mismatch");
const digest = async (path: string): Promise<string> => {
  const hash = await crypto.subtle.digest("SHA-256", await Deno.readFile(path));
  return [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
};
const candidateId = `sha256:${await digest(Deno.env.get("CANDIDATE_MANIFEST_PATH")!)}`;
if (manifest.candidate_id !== candidateId) fail("candidate ID does not match candidate manifest bytes");
for (const [name, path] of [
  ["docker-compose.release.yaml", Deno.env.get("COMPOSE_PATH")!],
  ["docker-compose.release.yaml.sha256", Deno.env.get("COMPOSE_CHECKSUM_PATH")!],
  [Deno.env.get("CLI_ARCHIVE_NAME")!, Deno.env.get("CLI_ARCHIVE_PATH")!],
] as const) {
  const record = manifest.files?.find((file: { name?: string }) => file.name === name);
  if (!record) fail(`${name} is absent from release manifest`);
  const bytes = await Deno.readFile(path);
  if (record.size !== bytes.byteLength || record.sha256 !== await digest(path)) {
    fail(`${name} differs from release manifest`);
  }
}
if (!manifest.image?.digest?.startsWith("sha256:")) fail("image digest is missing");
'

log "Verifying published image digest and health"
EXPECTED_IMAGE_DIGEST="$(deno eval 'console.log(JSON.parse(await Deno.readTextFile(Deno.env.get("MANIFEST_PATH")!)).image.digest)')"
actual_image_digest="$(docker buildx imagetools inspect "${IMAGE_REPOSITORY}:${VERSION_INPUT}" --format '{{json .Manifest.Digest}}' | tr -d '"')"
[ "$actual_image_digest" = "$EXPECTED_IMAGE_DIGEST" ] || fail "version tag points to ${actual_image_digest}, expected ${EXPECTED_IMAGE_DIGEST}"

node_secret="$(head -c 32 /dev/urandom | base64 | tr -d '\n')"
docker run --detach --rm --name "$CONTAINER_NAME" \
  --publish 127.0.0.1::8000 \
  --env UGOITE_ROOT=/data \
  --env UGOITE_SERVER_ADDRESS=0.0.0.0:8000 \
  --env UGOITE_PUBLIC_ORIGIN=http://localhost \
  --env UGOITE_API_BASE_URL=http://localhost/api \
  --env UGOITE_WEBAUTHN_RP_ID=localhost \
  --env "UGOITE_NODE_SECRET_KEY=${node_secret}" \
  "${IMAGE_REPOSITORY}@${EXPECTED_IMAGE_DIGEST}" >/dev/null
CONTAINER_STARTED=1
container_port="$(docker port "$CONTAINER_NAME" 8000/tcp | sed -n 's/.*:\([0-9][0-9]*\)$/\1/p')"
[ -n "$container_port" ] || fail "published container did not expose port 8000"
for attempt in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:${container_port}/health" >/dev/null; then
    break
  fi
  if [ "$attempt" -eq 60 ]; then
    fail "published container did not become healthy"
  fi
  sleep 1
done

log "Verifying published CLI installer and version"
mkdir -p "$INSTALL_DIR"
HOME="$WORK_ROOT/home" PATH="$INSTALL_DIR:$PATH" UGOITE_VERSION="$VERSION_INPUT" \
  UGOITE_INSTALL_DIR="$INSTALL_DIR" UGOITE_DOWNLOAD_BASE_URL="$ASSET_BASE_URL" \
  UGOITE_RELEASE_TOKEN="$RELEASE_TOKEN_INPUT" UGOITE_TARGET_OVERRIDE="$CLI_TARGET" \
  /bin/bash "$SCRIPT_DIR/install-ugoite-cli.sh"
version_output="$($INSTALL_DIR/ugoite --version 2>&1)"
[ "$version_output" = "ugoite ${VERSION_INPUT#v}" ] || \
  fail "published CLI reported ${version_output}, expected ugoite ${VERSION_INPUT#v}"

log "Published distribution verification passed for ${VERSION_INPUT}"
