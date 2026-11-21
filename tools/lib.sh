# Shared helpers for MISTBRN tools. Portable bash: macOS, Linux, Git Bash on Windows.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HORIZON_COMMIT="0a875da91dbb0fdcb7ba8c42c11341aa229af51a"
HORIZON_VERSION="nightly-2026-11-20-0a875da"
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

# Launcher binary: $HORIZON_LAUNCHER, else the platform's default install path.
launcher() {
  if [[ -n "${HORIZON_LAUNCHER:-}" ]]; then echo "$HORIZON_LAUNCHER"; return; fi
  local candidate
  case "$(os_kind)" in
    macos) candidate="/Applications/Horizon.app/Contents/MacOS/horizon-launcher" ;;
    linux) candidate="$(command -v horizon-launcher || echo "$HOME/.local/bin/horizon-launcher")" ;;
    windows) candidate="$LOCALAPPDATA/Programs/Horizon Launcher/horizon-launcher.exe" ;;
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
    windows) echo "$APPDATA/horizon-launcher" ;;
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
      windows) root="$LOCALAPPDATA/MISTBRN-dev" ;;
    esac
  fi
  case "$root" in "$REPO_ROOT"|"$REPO_ROOT"/*) die "runtime root must be outside the repo: $root" ;; esac
  mkdir -p "$root"; echo "$root"
}

sha256() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1
  else shasum -a 256 "$1" | cut -d' ' -f1; fi
}
