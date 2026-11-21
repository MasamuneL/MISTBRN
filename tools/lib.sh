# Shared helpers for MISTBRN tools. Portable bash: macOS, Linux, Git Bash on Windows.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HORIZON_COMMIT="0a875da91dbb0fdcb7ba8c42c11341aa229af51a"
HORIZON_VERSION="nightly-2026-11-20-0a875da"
# Pinned independently of the download source (from the release's sdk.json): integrity of the
# API source does not depend on SHA256SUMS fetched alongside it.
SDK_API_SOURCE_SHA256="5d57c0b41f0e7edc487b4e07785bfaee1bbfe792c554446e2658deb46205b82d"
DEV_INSTANCE="${MISTBRN_INSTANCE:-mistbrn-dev}"

die() { echo "error: $*" >&2; exit 1; }

os_kind() {
  case "$(uname -s)" in
    Darwin) echo macos ;;
    Linux) echo linux ;;
    MINGW*|MSYS*|CYGWIN*) echo windows ;;
    *) die "unsupported OS $(uname -s)" ;;
  esac
}

# A Windows folder variable as a Git Bash path (C:\x -> /c/x); fails clearly when unset.
win_path() {
  local value="${!1:-}"
  [[ -n "$value" ]] || die "environment variable $1 is not set"
  if command -v cygpath >/dev/null; then cygpath -u "$value"; else echo "$value"; fi
}

# Launcher binary: $HORIZON_LAUNCHER, else the platform's default install path.
launcher() {
  if [[ -n "${HORIZON_LAUNCHER:-}" ]]; then echo "$HORIZON_LAUNCHER"; return; fi
  local candidate
  case "$(os_kind)" in
    macos) candidate="/Applications/Horizon.app/Contents/MacOS/horizon-launcher" ;;
    linux) candidate="$(command -v horizon-launcher || echo "$HOME/.local/bin/horizon-launcher")" ;;
    windows) candidate="$(win_path LOCALAPPDATA)/Programs/Horizon Launcher/horizon-launcher.exe" ;;
  esac
  [[ -x "$candidate" ]] || die "Horizon launcher not found at $candidate (set HORIZON_LAUNCHER)"
  echo "$candidate"
}

# Launcher data folder (versions, rust, instances).
launcher_home() {
  if [[ -n "${HORIZON_LAUNCHER_HOME:-}" ]]; then echo "$HORIZON_LAUNCHER_HOME"; return; fi
  case "$(os_kind)" in
    macos) echo "$HOME/Library/Application Support/horizon-launcher" ;;
    linux) echo "${XDG_DATA_HOME:-$HOME/.local/share}/horizon-launcher" ;;
    windows) echo "$(win_path APPDATA)/horizon-launcher" ;;
  esac
}

# Runtime root for staged builds, logs and test homes: always outside the repo.
runtime_root() {
  local root
  if [[ -n "${MISTBRN_RUNTIME:-}" ]]; then root="$MISTBRN_RUNTIME"
  else
    case "$(os_kind)" in
      macos) root="$HOME/Library/Application Support/MISTBRN-dev" ;;
      linux) root="${XDG_DATA_HOME:-$HOME/.local/share}/MISTBRN-dev" ;;
      windows) root="$(win_path LOCALAPPDATA)/MISTBRN-dev" ;;
    esac
  fi
  case "$root" in "$REPO_ROOT"|"$REPO_ROOT"/*) die "runtime root must be outside the repo: $root" ;; esac
  mkdir -p "$root"; echo "$root"
}

sha256() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1
  else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

require() {
  local tool
  for tool in "$@"; do command -v "$tool" >/dev/null || die "required tool missing: $tool"; done
}

# Process helpers that also work for native Windows PIDs under Git Bash.
proc_alive() {
  case "$(os_kind)" in
    windows) tasklist //FI "PID eq $1" //NH 2>/dev/null | grep -q " $1 " ;;
    *) kill -0 "$1" 2>/dev/null ;;
  esac
}
proc_kill_hard() {
  case "$(os_kind)" in
    windows) taskkill //F //PID "$1" >/dev/null ;;
    *) kill -9 "$1" ;;
  esac
}
