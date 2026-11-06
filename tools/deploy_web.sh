#!/usr/bin/env bash
# Publish the staged web build to its dedicated Pages repository.
#
# Usage: tools/deploy_web.sh [--dry-run]
# Safe to re-run: if the local web repo exists it is reset to origin/main and
# the staged files are overlaid before a normal commit/push.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_REPO="${WEB_REPO:-$ROOT/../spy-hunter-web}"
REMOTE="${REMOTE:-https://github.com/orbitmechanic/spy-hunter-web.git}"

if [[ ! -d "$ROOT/web/dist" ]]; then
  echo "run tools/build_web.sh first" >&2
  exit 1
fi

if [[ -d "$WEB_REPO/.git" ]]; then
  git -C "$WEB_REPO" fetch origin
  git -C "$WEB_REPO" reset --hard origin/main
  find "$WEB_REPO" -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
else
  rm -rf "$WEB_REPO"
  mkdir -p "$WEB_REPO"
  git -C "$WEB_REPO" init -b main
  git -C "$WEB_REPO" remote add origin "$REMOTE"
fi

cp -R "$ROOT/web/dist/." "$WEB_REPO/"
touch "$WEB_REPO/.nojekyll"
cat > "$WEB_REPO/README.md" <<'EOF'
# SPY HUNTER: INTERCEPTOR — browser build

Static WebAssembly build of <https://github.com/orbitmechanic/spy-hunter>.
Play at <https://orbitmechanic.github.io/spy-hunter-web/>.

Source, art pipeline, and tests live in the game repository.
EOF
cp "$ROOT/LICENSE" "$WEB_REPO/LICENSE"

cd "$WEB_REPO"
git add -A
if git diff --cached --quiet; then
  echo "web build is already current"
  exit 0
fi
git commit -m "SPY HUNTER: INTERCEPTOR — browser build"
if [[ "${1:-}" == "--dry-run" ]]; then
  echo "staged $WEB_REPO (dry run; not pushed)"
  exit 0
fi
git push origin main
