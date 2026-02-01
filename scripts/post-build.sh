#!/bin/bash
set -e

PKG_DIR="pkg"
cd "$PKG_DIR"

# Compile TypeScript files
echo "📦 Compiling TypeScript..."
npm install --silent
npm run build --silent

echo "✅ Post-build complete!"
