#!/bin/bash
# VTR CI Health Check
# Run weekly to monitor CI integration health

set -e

echo "=== VTR CI Health Check ==="
echo

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) not installed"
    echo "Install: brew install gh"
    exit 1
fi

# Get recent runs
echo "📊 Recent VTR analysis runs:"
gh run list --workflow=vtr-analysis.yml --limit 10 --json conclusion,databaseId,createdAt,headBranch | \
  jq -r '.[] | "\(.databaseId): \(.conclusion) on \(.headBranch) (\(.createdAt | split("T")[0]))"'

echo
echo "🔍 Checking for failures..."
FAILURES=$(gh run list --workflow=vtr-analysis.yml --limit 100 --json conclusion | \
  jq '[.[] | select(.conclusion == "failure")] | length')

echo "Failures in last 100 runs: $FAILURES"

if [ "$FAILURES" -eq "0" ]; then
  echo "✅ All runs successful"
else
  echo "⚠️  Check failed runs:"
  gh run list --workflow=vtr-analysis.yml --limit 100 --json conclusion,databaseId,createdAt | \
    jq -r '.[] | select(.conclusion == "failure") | "  Run \(.databaseId) failed on \(.createdAt | split("T")[0])"'
fi

echo
echo "📈 Success rate: $(( (100 - FAILURES) ))%"
echo

# Check for recent hash consistency
echo "🔐 Checking hash consistency (manual verification required):"
echo "Download recent artifacts and compare hashes manually"
echo "  gh run download [run-id] -n vtr-analysis-results"
echo
