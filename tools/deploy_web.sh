#!/usr/bin/env bash
# Publish the staged web build to its dedicated Pages repository.
#
# Usage: tools/deploy_web.sh [--dry-run]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEB_REPO="${WEB_REPO:-$ROOT/../spy-hunter-web}"
REMOTE="${REMOTE:-https://github.com/orbitmechanic/spy-hunter-web.git}"

if [[ ! -d "$ROOT/web/dist" ]]; then
  echo "run tools/build_web.sh first" >&2
  exit 1
fi

rm -rf "$WEB_REPO"
mkdir -p "$WEB_REPO"
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
git init -b main
git add .
git commit -m "SPY HUNTER: INTERCEPTOR — browser build"
git remote add origin "$REMOTE"
if [[ "${1:-}" == "--dry-run" ]]; then
  echo "staged $WEB_REPO (dry run; not pushed)"
  exit 0
fi
git push -u origin main
