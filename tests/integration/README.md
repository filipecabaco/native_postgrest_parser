# WASM Integration Tests

This directory contains integration tests for the PostgREST Parser WASM bindings using Deno.

## Prerequisites

Install Deno:

```bash
# macOS/Linux
curl -fsSL https://deno.land/install.sh | sh

# Windows (PowerShell)
irm https://deno.land/install.ps1 | iex

# Or via package managers
brew install deno   # macOS
cargo install deno  # Rust users
```

## Running Tests

### Build WASM Package First

Before running tests, build the WASM package:

```bash
# From project root
wasm-pack build --target web --features wasm
```

### Run All Tests

```bash
# From project root
deno test --allow-read tests/integration/wasm_test.ts
```

### Run with Verbose Output

```bash
deno test --allow-read tests/integration/wasm_test.ts -- --nocapture
```

### Run Specific Test

```bash
deno test --allow-read --filter "simple select" tests/integration/wasm_test.ts
```

## Test Coverage

The integration tests cover:

### Basic Operations
- ✅ Simple SELECT queries
- ✅ Field selection
- ✅ Table references

### Filter Operators
- ✅ Comparison operators (`eq`, `neq`, `gt`, `gte`, `lt`, `lte`)
- ✅ Pattern matching (`like`, `ilike`)
- ✅ List operations (`in`)
- ✅ IS operators (`is.null`, `is.true`, `is.false`)

### Advanced Features
- ✅ Logic operators (AND, OR)
- ✅ Nested logic trees
- ✅ JSON path operators (`->`, `->>`)
- ✅ Type casting (`::<type>`)
- ✅ Full-text search (`fts`)
- ✅ Array operators (`cs`, `cd`, `ov`)
- ✅ Quantifiers (`any`, `all`)
- ✅ Negation (`not`)

### Query Building
- ✅ ORDER BY clauses
- ✅ LIMIT and OFFSET
- ✅ Multi-column ordering
- ✅ Complex nested queries

### API Functions
- ✅ `parseQueryString()` - Full parsing + SQL generation
- ✅ `parseOnly()` - Parse without SQL generation
- ✅ `toJSON()` - Result serialization

### Error Handling
- ✅ Invalid table names
- ✅ Invalid query syntax
- ✅ Proper error messages

### Performance
- ✅ Batch query processing
- ✅ Performance benchmarks

## Example Usage

```typescript
import init, { parseQueryString } from "../../pkg/postgrest_parser.js";

// Initialize WASM
await init();

// Parse a query
const result = parseQueryString(
  "users",
  "select=id,name,email&age=gte.18&status=in.(active,pending)&order=created_at.desc&limit=10"
);

console.log("SQL:", result.query);
console.log("Params:", result.params);
console.log("Tables:", result.tables);
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Setup Deno
  uses: denoland/setup-deno@v1
  with:
    deno-version: v1.x

- name: Build WASM
  run: wasm-pack build --target web --features wasm

- name: Run Integration Tests
  run: deno test --allow-read tests/integration/wasm_test.ts
```

## Troubleshooting

### WASM Not Found

If you get an error about missing WASM files:

```bash
# Make sure to build the WASM package first
wasm-pack build --target web --features wasm
```

### Permission Denied

Deno requires explicit permissions. Make sure to use `--allow-read`:

```bash
deno test --allow-read tests/integration/wasm_test.ts
```

### Test Failures

If tests fail:

1. Ensure WASM package is built: `wasm-pack build --target web --features wasm`
2. Check Deno version: `deno --version` (requires 1.x or higher)
3. Verify pkg directory exists and contains WASM files
4. Check browser console for detailed error messages

## Performance Benchmarks

The test suite includes performance benchmarks:

- Average parse time: ~1-3ms per query
- Batch processing: 100 queries in <300ms
- All operations complete in microseconds to low milliseconds

## Adding New Tests

When adding new tests:

1. Follow the existing test structure
2. Use descriptive test names: `"WASM - category: specific behavior"`
3. Include assertions for both SQL output and parameters
4. Add error case tests where applicable
5. Update this README with new test coverage

## Resources

- [Deno Documentation](https://deno.land/manual)
- [WASM-Pack Guide](https://rustwasm.github.io/wasm-pack/)
- [PostgREST API Reference](https://postgrest.org/en/stable/api.html)
