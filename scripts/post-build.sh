#!/bin/bash
set -e

PKG_DIR="pkg"
cd "$PKG_DIR"

echo "Compiling TypeScript..."
npm install --save-dev typescript --silent
npx tsc -p tsconfig.json

echo "Patching package.json..."
node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));

pkg.files = [
  'postgrest_parser_bg.wasm',
  'postgrest_parser.js',
  'postgrest_parser.d.ts',
  'postgrest_parser_bg.wasm.d.ts',
  'client.js',
  'client.d.ts',
  'types.js',
  'types.d.ts'
];

pkg.exports = {
  '.': {
    types: './postgrest_parser.d.ts',
    import: './postgrest_parser.js',
    default: './postgrest_parser.js'
  },
  './client.js': {
    types: './client.d.ts',
    import: './client.js',
    default: './client.js'
  },
  './types.js': {
    types: './types.d.ts',
    import: './types.js',
    default: './types.js'
  },
  './*.wasm': './*.wasm'
};

pkg.sideEffects = ['./postgrest_parser.js', './snippets/*'];

delete pkg.devDependencies;

fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
"

echo "Post-build complete!"
