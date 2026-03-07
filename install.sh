#!/usr/bin/env bash
# ○ HomeOS — One-line install
# curl -fsSL https://raw.githubusercontent.com/goldlotus1810/Origin/main/install.sh | bash
#
# Giống `curl -fsSL https://get.docker.com | sh`
# Tự clone repo → build → run
# Không phụ thuộc gì ngoài git + go

set -euo pipefail

REPO="https://github.com/goldlotus1810/Origin"
DIR="${HOMEOS_DIR:-$HOME/homeos}"
ADDR="${HOMEOS_ADDR:-:8080}"

# ── Colors ─────────────────────────────────────────────────────
R='\033[0;31m'; G='\033[0;32m'; Y='\033[1;33m'
C='\033[0;36m'; B='\033[1;34m'; N='\033[0m'

log()  { echo -e "${C}○${N}  $*"; }
ok()   { echo -e "${G}✓${N}  $*"; }
warn() { echo -e "${Y}⚠${N}  $*"; }
die()  { echo -e "${R}✗${N}  $*" >&2; exit 1; }

echo ""
echo -e "${B}  ○  HomeOS — Silk Web Knowledge System${N}"
echo -e "${B}  ─────────────────────────────────────${N}"
echo ""

# ── 1. Kiểm tra dependencies ───────────────────────────────────
log "Checking dependencies..."

command -v git  >/dev/null 2>&1 || die "git not found. Install: https://git-scm.com"
command -v go   >/dev/null 2>&1 || die "go not found.  Install: https://go.dev/dl"

GO_MIN="1.22"
GO_VER=$(go version | grep -oP '\d+\.\d+' | head -1)
# Compare versions
# Version compare dùng sort -V (portable, không cần python3)
LOWEST=$(printf '%s\n%s' "${GO_MIN}" "${GO_VER}" | sort -V | head -1)
[ "${LOWEST}" = "${GO_MIN}" ] || \
  die "Go ${GO_MIN}+ required. Have ${GO_VER}. Install: https://go.dev/dl"

ok "git $(git --version | awk '{print $3}')"
ok "go  ${GO_VER}"

# ── 2. Clone hoặc update repo ─────────────────────────────────
echo ""
if [ -d "$DIR/.git" ]; then
  log "Updating existing install at $DIR ..."
  git -C "$DIR" pull --ff-only --quiet
  ok "Updated"
else
  log "Cloning $REPO → $DIR ..."
  git clone --depth 1 --quiet "$REPO" "$DIR"
  ok "Cloned"
fi

cd "$DIR"

# ── 3. go mod tidy ────────────────────────────────────────────
echo ""
log "Resolving dependencies..."
go mod tidy 2>&1 | grep -v "^$" || true
ok "go mod tidy done"

# ── 4. Build ──────────────────────────────────────────────────
echo ""
log "Building..."
go build -o "$DIR/homeos" ./cmd/homeos/ 2>&1 || {
  echo ""
  warn "Build failed. Run manually:"
  echo "  cd $DIR && go build ./cmd/homeos/"
  exit 1
}
ok "Build complete → $DIR/homeos"

# ── 5. Launch ─────────────────────────────────────────────────
echo ""
log "Starting HomeOS on $ADDR ..."
echo ""
echo -e "  ${G}http://localhost${ADDR}/${N}           Silk Web Tree"
echo -e "  ${G}http://localhost${ADDR}/health${N}     Status"
echo -e "  ${G}http://localhost${ADDR}/api/tree${N}   Nodes JSON"
echo -e "  ${G}http://localhost${ADDR}/ws/sse${N}     Live stream"
echo ""

HOMEOS_ADDR="$ADDR" \
HOMEOS_STATIC="$DIR/web/static" \
  exec "$DIR/homeos"
