# Plan: Replace README.md with English version and ensure Cantonese README-ZH.md exists

## TL;DR
- Replace the repository's top-level README.md with the English README content (decision provided by user).
- Ensure README-ZH.md contains the existing Cantonese content (we have already created README-ZH.md). If README-ZH.md does not exist, move the original README.md to README-ZH.md before overwriting.
- Commit changes on branch `docs/scheduler-readme`, open PR against `dev/scheduler` with exact title/body.

Effort: Trivial — single-file documentation update. Parallel: NO

## Work Objective
- Overwrite README.md with the English content in .sisyphus/drafts/README-ENG.md and ensure README-ZH.md contains the Cantonese content (already created at README-ZH.md). Produce a single commit and open a PR.

## Decision-Complete Steps (run from repo root)

Preflight checks (copy-paste exactly):

1) Ensure working tree is clean (no uncommitted changes):

```bash
git status --porcelain
```

If output is non-empty, STOP and stash or commit changes before proceeding.

2) Confirm README-ZH.md exists; if not, create it from current README.md backup:

```bash
if [ ! -f README-ZH.md ]; then
  echo "README-ZH.md not found — creating from current README.md backup"
  cp README.md README-ZH.md
  git add README-ZH.md
  git commit -m "docs: add README-ZH.md backup (cantonese)" || true
fi
```

Main steps (exact commands):

3) Create branch and overwrite README.md with the English content from the draft file:

```bash
git fetch origin --prune
git checkout dev/scheduler
git pull --ff-only origin dev/scheduler
git checkout -b docs/scheduler-readme

# Overwrite README.md with the English draft (this assumes .sisyphus/drafts/README-ENG.md exists)
cp .sisyphus/drafts/README-ENG.md README.md

git add README.md
git commit -m "docs: replace README.md with English README (add scheduling docs)" 
git push -u origin docs/scheduler-readme

# Create PR (uses gh cli). If gh not installed, open PR manually on GitHub with title/body below.
gh pr create --base dev/scheduler --head docs/scheduler-readme --title "docs: README (eng) — add scheduling docs" --body "Replace top-level README.md with English README and add scheduling documentation. See README-ZH.md for Cantonese version."
```

Post-PR checks (automatable):

4) Verify files in remote branch (copy-paste to run locally after push):

```bash
git fetch origin docs/scheduler-readme:refs/remotes/origin/docs/scheduler-readme
git show origin/docs/scheduler-readme:README.md | head -n 5
test -f README-ZH.md && echo "README-ZH.md exists" || echo "WARNING: README-ZH.md missing"
```

Acceptance criteria (all must pass):
- README.md contains header line starting with "# 🚀 OpenCB — Open CLI Broker/Bridge" (automatable: grep -q '^# 🚀 OpenCB' README.md).
- README-ZH.md exists and contains the Chinese header (automatable: grep -q 'OpenCB（Open CLI Broker/Bridge）' README-ZH.md).
- PR created against dev/scheduler with title exactly: "docs: README (eng) — add scheduling docs".

Rollback (exact):

If you must revert the change after merge, run:

```bash
git checkout dev/scheduler
git pull origin dev/scheduler
git revert <merge-commit-sha> -m 1
git push origin dev/scheduler
```

Notes & Caveats
- This plan does ONLY documentation changes; it does NOT modify source code or tests.
- The English README content is taken from `.sisyphus/drafts/README-ENG.md`. Ensure that draft file contains the final desired text before running the copy command.
- If the `gh` CLI is not available or authenticated, create the PR via the GitHub web UI using the branch `docs/scheduler-readme`.

Plan saved to `.sisyphus/plans/replace-readme-with-english.md`.
