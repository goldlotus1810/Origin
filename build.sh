#!/bin/bash
# build.sh — Build olang binary
set -e

echo "○ Building olang..."

# Build olang CLI (zero dependencies — stdlib only)
go build -o olang ./cmd/olang/
echo "✅ olang binary created"

# Verify
./olang verify homeos.olang

echo ""
echo "Usage:"
echo "  ./olang run    homeos.olang"
echo "  ./olang info   homeos.olang"
echo "  ./olang get    homeos.olang A"
echo "  ./olang get    homeos.olang 山"
