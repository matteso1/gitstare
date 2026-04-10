#!/usr/bin/env bash
# ============================================================================
# gitstare test harness -- creates a realistic filesystem of git repos
# covering every edge case gitstare needs to handle correctly.
#
# Usage:
#   ./tests/setup_test_repos.sh [target_dir]
#   Default target: ./test_repos
#
# After running, point gitstare at the directory:
#   cargo run -- -p ./test_repos --fresh
# ============================================================================

set -euo pipefail

TARGET="${1:-./test_repos}"
rm -rf "$TARGET"
mkdir -p "$TARGET"
cd "$TARGET"
ABS_TARGET="$(pwd)"

echo "=== gitstare test harness ==="
echo "Creating test repos in: $ABS_TARGET"
echo ""

# Helper: create a repo with N commits
make_repo() {
    local name="$1"
    local dir="$ABS_TARGET/$name"
    mkdir -p "$dir"
    cd "$dir"
    git init -q -b main
}

commit() {
    local msg="$1"
    local file="${2:-file.txt}"
    echo "$msg - $(date +%s)" >> "$file"
    git add "$file"
    git -c user.name="Test" -c user.email="test@test.com" commit -q -m "$msg"
}

make_bare() {
    local name="$1"
    git init -q --bare -b main "$ABS_TARGET/_remotes/$name.git"
}

# --------------------------------------------------------------------------
# 1. CLEAN REPO -- no changes, on main, simple history
# --------------------------------------------------------------------------
echo "[1/15] Clean repo (hello-world)"
make_repo "hello-world"
commit "Initial commit"
commit "Add README"
commit "Fix typo in README"

# --------------------------------------------------------------------------
# 2. DIRTY REPO -- modified files
# --------------------------------------------------------------------------
echo "[2/15] Dirty repo with modified files (api-server)"
make_repo "api-server"
commit "Initial commit"
commit "Add server code"
commit "Add database layer"
commit "Add auth middleware"
echo "// TODO: fix this" >> server.js
echo "dirty stuff" >> db.js

# --------------------------------------------------------------------------
# 3. UNTRACKED FILES ONLY
# --------------------------------------------------------------------------
echo "[3/15] Repo with untracked files only (mobile-app)"
make_repo "mobile-app"
commit "Initial commit"
commit "Add app scaffold"
echo "new untracked file" > notes.txt
echo "another one" > TODO.md
echo "build output" > app.apk

# --------------------------------------------------------------------------
# 4. MIXED -- modified + untracked + staged
# --------------------------------------------------------------------------
echo "[4/15] Mixed dirty repo (website)"
make_repo "website"
commit "Initial commit"
commit "Add index.html" "index.html"
commit "Add styles"
echo "modified" >> index.html
echo "new file" > draft.md
echo "staged change" > staged.txt
git add staged.txt

# --------------------------------------------------------------------------
# 5. MANY BRANCHES (some stale)
# --------------------------------------------------------------------------
echo "[5/15] Repo with many branches, some stale (monorepo)"
make_repo "monorepo"
commit "Initial commit"
commit "Core setup"

# Create stale branches (backdate commits)
for branch in feature/old-auth feature/dead-code fix/legacy-bug experiment/ml; do
    git checkout -q -b "$branch"
    GIT_COMMITTER_DATE="2024-01-15T12:00:00" \
    GIT_AUTHOR_DATE="2024-01-15T12:00:00" \
    git -c user.name="Test" -c user.email="test@test.com" commit -q --allow-empty -m "Old work on $branch"
    git checkout -q main
done

# Create recent branches
for branch in feature/new-ui feature/api-v2 hotfix/login; do
    git checkout -q -b "$branch"
    commit "Work on $branch"
    git checkout -q main
done

# --------------------------------------------------------------------------
# 6. DETACHED HEAD
# --------------------------------------------------------------------------
echo "[6/15] Detached HEAD state (infra-tools)"
make_repo "infra-tools"
commit "Initial commit"
commit "Add terraform configs"
commit "Add ansible playbooks"
FIRST=$(git rev-list --max-parents=0 HEAD)
git checkout -q "$FIRST"

# --------------------------------------------------------------------------
# 7. EMPTY REPO (no commits at all)
# --------------------------------------------------------------------------
echo "[7/15] Empty repo, no commits (new-project)"
make_repo "new-project"
# intentionally no commits

# --------------------------------------------------------------------------
# 8. REPO WITH UPSTREAM -- ahead
# --------------------------------------------------------------------------
echo "[8/15] Repo ahead of upstream (cli-tool)"
mkdir -p "$ABS_TARGET/_remotes"
make_bare "cli-tool"
make_repo "cli-tool"
git remote add origin "$ABS_TARGET/_remotes/cli-tool.git"
commit "Initial commit"
commit "Add CLI parser"
git push -q -u origin main
commit "Add new subcommand"
commit "Add help text"
# Now 2 commits ahead of origin

# --------------------------------------------------------------------------
# 9. REPO WITH UPSTREAM -- behind
# --------------------------------------------------------------------------
echo "[9/15] Repo behind upstream (shared-lib)"
make_bare "shared-lib"
make_repo "shared-lib"
git remote add origin "$ABS_TARGET/_remotes/shared-lib.git"
commit "Initial commit"
commit "Add core module"
commit "Add utils"
git push -q -u origin main

# Push extra commits to bare repo from a temp clone
TMPCLONE=$(mktemp -d)
git clone -q -b main "$ABS_TARGET/_remotes/shared-lib.git" "$TMPCLONE/shared-lib"
cd "$TMPCLONE/shared-lib"
echo "upstream change 1" >> upstream.txt
git add upstream.txt
git -c user.name="Teammate" -c user.email="team@test.com" commit -q -m "Teammate: add upstream feature"
echo "upstream change 2" >> upstream.txt
git add upstream.txt
git -c user.name="Teammate" -c user.email="team@test.com" commit -q -m "Teammate: fix upstream bug"
git push -q origin main
cd "$ABS_TARGET"
rm -rf "$TMPCLONE"

cd "$ABS_TARGET/shared-lib"
git fetch -q origin
# Now 2 commits behind origin/main

# --------------------------------------------------------------------------
# 10. REPO WITH UPSTREAM -- ahead AND behind (diverged)
# --------------------------------------------------------------------------
echo "[10/15] Diverged repo, ahead and behind (backend-api)"
make_bare "backend-api"
make_repo "backend-api"
git remote add origin "$ABS_TARGET/_remotes/backend-api.git"
commit "Initial commit"
commit "Add routes"
git push -q -u origin main

# Push from clone
TMPCLONE=$(mktemp -d)
git clone -q -b main "$ABS_TARGET/_remotes/backend-api.git" "$TMPCLONE/backend-api"
cd "$TMPCLONE/backend-api"
echo "remote work" >> remote.txt
git add remote.txt
git -c user.name="Teammate" -c user.email="team@test.com" commit -q -m "Teammate: remote changes"
git push -q origin main
cd "$ABS_TARGET"
rm -rf "$TMPCLONE"

cd "$ABS_TARGET/backend-api"
commit "Local divergent work"
git fetch -q origin
# Now 1 ahead, 1 behind

# --------------------------------------------------------------------------
# 11. REPO ON NON-MAIN BRANCH
# --------------------------------------------------------------------------
echo "[11/15] Repo on feature branch (design-system)"
make_repo "design-system"
commit "Initial commit"
commit "Add component library"
git checkout -q -b feature/dark-mode
commit "WIP: dark mode support"
commit "Add theme tokens"
echo "wip" >> theme.css

# --------------------------------------------------------------------------
# 12. REPO WITH MERGE CONFLICT MARKERS
# --------------------------------------------------------------------------
echo "[12/15] Repo with conflict markers in files (data-pipeline)"
make_repo "data-pipeline"
commit "Initial commit"
echo "original content" > config.yaml
git add config.yaml
git -c user.name="Test" -c user.email="test@test.com" commit -q -m "Add config"
git checkout -q -b feature/new-source
echo "feature branch content" > config.yaml
git add config.yaml
git -c user.name="Test" -c user.email="test@test.com" commit -q -m "Update config for new source"
git checkout -q main
echo "main branch content" > config.yaml
git add config.yaml
git -c user.name="Test" -c user.email="test@test.com" commit -q -m "Update config for prod"
# Attempt merge (will conflict)
git merge feature/new-source --no-commit 2>/dev/null || true

# --------------------------------------------------------------------------
# 13. LARGE HISTORY REPO
# --------------------------------------------------------------------------
echo "[13/15] Repo with large commit history (core-engine)"
make_repo "core-engine"
for i in $(seq 1 50); do
    echo "commit $i content" >> "module_$((i % 5)).rs"
    git add .
    git -c user.name="Test" -c user.email="test@test.com" commit -q -m "Commit #$i: update module $((i % 5))"
done

# --------------------------------------------------------------------------
# 14. NESTED REPOS (repo inside repo)
# --------------------------------------------------------------------------
echo "[14/15] Nested repos (outer-project/inner-lib)"
make_repo "outer-project"
commit "Outer initial"
commit "Add outer code"
mkdir -p lib
cd lib
git init -q -b main
echo "inner lib" > lib.rs
git add lib.rs
git -c user.name="Test" -c user.email="test@test.com" commit -q -m "Inner lib init"

# --------------------------------------------------------------------------
# 15. REPO WITH TAGS, LOTS OF BRANCHES
# --------------------------------------------------------------------------
echo "[15/15] Repo with tags and many branches (release-tracker)"
make_repo "release-tracker"
commit "Initial commit"
commit "v0.1.0 prep"
git tag v0.1.0
commit "Post-release work"
commit "v0.2.0 prep"
git tag v0.2.0

for i in $(seq 1 10); do
    git checkout -q -b "feature/feature-$i"
    commit "Work on feature $i"
    git checkout -q main
done

echo ""
echo "=== Done! Created 15+ test repos in: $ABS_TARGET ==="
echo ""
echo "Run gitstare against them:"
echo "  cargo run -- -p \"$ABS_TARGET\" --fresh"
echo ""
echo "Edge cases covered:"
echo "  - Clean repo"
echo "  - Dirty (modified files)"
echo "  - Untracked files only"
echo "  - Mixed (modified + untracked + staged)"
echo "  - Many branches (stale + recent)"
echo "  - Detached HEAD"
echo "  - Empty repo (no commits)"
echo "  - Ahead of upstream"
echo "  - Behind upstream"
echo "  - Diverged (ahead AND behind)"
echo "  - On non-main branch (feature branch)"
echo "  - Merge conflict state"
echo "  - Large commit history (50 commits)"
echo "  - Nested repos"
echo "  - Tags + many branches"
