#!/usr/bin/env sh
# GitCredit CLI installer — https://gitcredit.dev
# Usage: curl -fsSL https://gitcredit.dev/install | sh
set -eu

REPO="kataqatsi/GitCredit"
BIN_NAME="gitcredit"
DEFAULT_INSTALL_DIR="${HOME}/.local/bin"

err() {
  printf 'install: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || err "missing required command: $1"
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64 | amd64) printf '%s\n' "x86_64-unknown-linux-gnu" ;;
        aarch64 | arm64) printf '%s\n' "aarch64-unknown-linux-gnu" ;;
        *) err "unsupported Linux architecture: $arch" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64) printf '%s\n' "x86_64-apple-darwin" ;;
        arm64 | aarch64) printf '%s\n' "aarch64-apple-darwin" ;;
        *) err "unsupported macOS architecture: $arch" ;;
      esac
      ;;
    *)
      err "unsupported OS: $os (install manually from https://github.com/${REPO}/releases)"
      ;;
  esac
}

release_base_url() {
  if [ -n "${GITCREDIT_VERSION:-}" ]; then
    version="$GITCREDIT_VERSION"
    case "$version" in
      v*) ;;
      *) version="v${version}" ;;
    esac
    printf '%s\n' "https://github.com/${REPO}/releases/download/${version}"
  else
    printf '%s\n' "https://github.com/${REPO}/releases/latest/download"
  fi
}

download() {
  url="$1"
  dest="$2"
  need_cmd curl
  curl -fsSL "$url" -o "$dest"
}

verify_checksum() {
  archive="$1"
  sums_url="$2"

  if [ "${GITCREDIT_NO_VERIFY:-}" = "1" ]; then
    return 0
  fi

  need_cmd curl
  sums="$(mktemp)"
  trap 'rm -f "$sums"' EXIT HUP INT TERM

  download "$sums_url" "$sums"
  expected="$(grep "  $(basename "$archive")$" "$sums" | awk '{print $1}')"
  [ -n "$expected" ] || err "checksum not found for $(basename "$archive")"

  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$archive" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$archive" | awk '{print $1}')"
  else
    err "need sha256sum or shasum to verify download"
  fi

  [ "$actual" = "$expected" ] || err "checksum mismatch"
  rm -f "$sums"
  trap - EXIT HUP INT TERM
}

install_binary() {
  archive="$1"
  install_dir="$2"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT HUP INT TERM

  tar xzf "$archive" -C "$tmp"
  mkdir -p "$install_dir"
  install -m 755 "$tmp/$BIN_NAME" "$install_dir/$BIN_NAME"
  rm -rf "$tmp"
  trap - EXIT HUP INT TERM
}

main() {
  target="$(detect_target)"
  archive="gitcredit-${target}.tar.gz"
  base="$(release_base_url)"
  url="${base}/${archive}"

  install_dir="${GITCREDIT_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT HUP INT TERM
  archive_path="${tmp}/${archive}"

  printf 'Installing %s (%s) to %s\n' "$BIN_NAME" "$target" "$install_dir"
  download "$url" "$archive_path"
  verify_checksum "$archive_path" "${base}/SHA256SUMS"
  install_binary "$archive_path" "$install_dir"
  rm -rf "$tmp"
  trap - EXIT HUP INT TERM

  case ":${PATH}:" in
    *":${install_dir}:"*) ;;
    *)
      printf '\nAdd %s to your PATH:\n' "$install_dir"
      printf '  export PATH="%s:$PATH"\n' "$install_dir"
      ;;
  esac

  printf '\nInstalled %s %s\n' "$BIN_NAME" "$("$install_dir/$BIN_NAME" --version 2>/dev/null || true)"
  printf 'Run `gitcredit configure api-key` to finish setup.\n'
}

main "$@"
