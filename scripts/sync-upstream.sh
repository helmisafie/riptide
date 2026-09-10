#!/usr/bin/env bash
# Sync local master with upstream, push master to origin,
# rebase the current branch onto master, and verify tests.
#
# Usage:
#   ./scripts/sync-upstream.sh [OPTIONS]
#
# Options:
#   --no-push    Do not push branches to origin
#   --no-test    Skip running cargo test after rebase
#   -h, --help   Show this help message

set -euo pipefail

cd "$(dirname "$0")/.."

UPSTREAM_URL="https://github.com/fezzik-the-giant/riptide.git"
DO_PUSH=true
DO_TEST=true

for arg in "$@"; do
    case "$arg" in
        --no-push)
            DO_PUSH=false
            ;;
        --no-test)
            DO_TEST=false
            ;;
        -h|--help)
            sed -n '2,11p' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo "Unknown option: $arg" >&2
            echo "Use -h or --help for usage." >&2
            exit 1
            ;;
    esac
done

if ! git diff-index --quiet HEAD --; then
    echo "error: working tree has uncommitted changes. Please commit or stash them first." >&2
    exit 1
fi

CURRENT_BRANCH=$(git branch --show-current)
if [ -z "$CURRENT_BRANCH" ]; then
    echo "error: HEAD is detached. Please checkout a branch first." >&2
    exit 1
fi

if ! git remote get-url upstream &>/dev/null; then
    echo "Adding upstream remote: $UPSTREAM_URL"
    git remote add upstream "$UPSTREAM_URL"
fi

echo "==> Fetching latest changes and tags from upstream..."
git fetch upstream master --tags

echo "==> Updating local master..."
git checkout master
git merge --ff-only upstream/master

if [ "$DO_PUSH" = true ]; then
    echo "==> Pushing updated master to origin..."
    git push origin master
fi

if [ "$CURRENT_BRANCH" != "master" ]; then
    echo "==> Switching back to $CURRENT_BRANCH..."
    git checkout "$CURRENT_BRANCH"

    echo "==> Rebasing $CURRENT_BRANCH onto master..."
    if ! git rebase master; then
        echo "" >&2
        echo "⚠️  Rebase conflict detected!" >&2
        echo "    Resolve the conflicts in the affected files, then run:" >&2
        echo "      git rebase --continue" >&2
        echo "    Or to abort:" >&2
        echo "      git rebase --abort" >&2
        exit 1
    fi

    if [ "$DO_TEST" = true ]; then
        echo "==> Running tests to verify build..."
        if ! cargo test; then
            echo "error: tests failed after rebase. Not pushing to origin." >&2
            exit 1
        fi
    fi

    if [ "$DO_PUSH" = true ]; then
        echo "==> Pushing rebased $CURRENT_BRANCH to origin..."
        git push --force-with-lease origin "$CURRENT_BRANCH"
    fi
fi

echo ""
echo "✨ All done! Synced with upstream successfully."
