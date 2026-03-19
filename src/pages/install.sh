#!/usr/bin/env bash
set -euo pipefail

REPO="${PINEL_REPO:-kashsuks/Pinel}"
BIN_NAME="pinel"
INSTALL_DIR="${PINEL_INSTALL_DIR:-${HOME}/.local/bin}"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"
TMP_DIR=""

log() {
  printf '%s\n' "$*" >&2
}

fail() {
  log "error: $*"
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

cleanup() {
  if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64) TARGET="linux-x86_64" ;;
        aarch64|arm64) TARGET="linux-arm64" ;;
        *) fail "unsupported Linux architecture: $arch" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64) TARGET="macos-x86_64" ;;
        arm64) TARGET="macos-arm64" ;;
        *) fail "unsupported macOS architecture: $arch" ;;
      esac
      ;;
    *)
      fail "unsupported operating system: $os"
      ;;
  esac
}

fetch_release_metadata() {
  RELEASE_JSON="$(curl -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "$API_URL")" || fail "failed to fetch latest release metadata from $API_URL"
}

extract_value() {
  local key="$1"
  printf '%s' "$RELEASE_JSON" | grep -m1 "\"${key}\":" | sed -E 's/.*"'${key}'": ?"([^"]+)".*/\1/'
}

download_from_release() {
  local version base_url candidate archive_path binary_path

  version="$(extract_value tag_name)"
  version="${version#v}"
  [[ -n "$version" ]] || fail "could not determine latest release version"

  base_url="https://github.com/${REPO}/releases/download/v${version}"
  TMP_DIR="$(mktemp -d)"

  for candidate in \
    "${BIN_NAME}-${TARGET}.tar.gz" \
    "${BIN_NAME}-${TARGET}.tgz" \
    "${BIN_NAME}-${TARGET}" \
    "${BIN_NAME}-${TARGET}.zip"
  do
    archive_path="${TMP_DIR}/${candidate}"
    if curl -fsSL "$base_url/$candidate" -o "$archive_path"; then
      case "$candidate" in
        *.tar.gz|*.tgz)
          tar -xzf "$archive_path" -C "$TMP_DIR"
          ;;
        *.zip)
          need_cmd unzip
          unzip -oq "$archive_path" -d "$TMP_DIR"
          ;;
        *)
          chmod +x "$archive_path"
          mv "$archive_path" "${TMP_DIR}/${BIN_NAME}"
          ;;
      esac

      binary_path="$(find "$TMP_DIR" -type f -name "$BIN_NAME" -perm -u+x | head -n 1)"
      [[ -n "$binary_path" ]] || fail "downloaded asset did not contain a ${BIN_NAME} binary"

      mkdir -p "$INSTALL_DIR"
      install -m 755 "$binary_path" "${INSTALL_DIR}/${BIN_NAME}"
      log "installed ${BIN_NAME} ${version} to ${INSTALL_DIR}/${BIN_NAME}"
      return 0
    fi
  done

  return 1
}

install_with_cargo() {
  need_cmd cargo
  cargo install "${BIN_NAME}" --locked
  log "installed ${BIN_NAME} with cargo"
}

main() {
  need_cmd curl
  trap cleanup EXIT INT TERM
  detect_target
  fetch_release_metadata

  if download_from_release; then
    :
  else
    log "no release asset found for ${TARGET}; falling back to cargo install"
    install_with_cargo
  fi

  case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
      log "warning: ${INSTALL_DIR} is not on PATH"
      log "add this to your shell config:"
      log "  export PATH=\"${INSTALL_DIR}:\$PATH\""
      ;;
  esac
}

main "$@"
