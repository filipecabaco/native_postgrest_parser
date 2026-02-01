# Integration Tests with PostgreSQL

This directory contains integration tests that verify relation resolution with a real PostgreSQL database.

## Prerequisites

1. **Docker** - Install from [docker.com](https://www.docker.com/)
2. **Docker Compose** - Usually included with Docker Desktop

## Running Tests

### 1. Start PostgreSQL

From the project root:

```bash
docker-compose up -d
```

This starts PostgreSQL with the test schema loaded from `tests/fixtures/init.sql`.

### 2. Run Integration Tests

```bash
# Run all tests with postgres feature
cargo test --features postgres

# Run only integration tests
cargo test --test integration_relations --features postgres

# Run with output
cargo test --test integration_relations --features postgres -- --nocapture
```

### 3. Stop PostgreSQL

```bash
docker-compose down
```

## Test Database

- **Host**: localhost
- **Port**: 5433 (not 5432 to avoid conflicts)
- **Database**: postgrest_parser_test
- **User**: postgres
- **Password**: postgres

## Test Schema

The test database includes:

### Tables
- `customers` - Main customer table
- `orders` - Orders (FK to customers)
- `products` - Product catalog
- `order_items` - Junction table (orders ↔ products)
- `posts` - Blog posts (FK to customers as author)
- `tags` - Post tags
- `post_tags` - Junction table (posts ↔ tags)
- `customer_profiles` - One-to-one with customers

### Relationships Tested
- ✅ Many-to-One: orders → customers
- ✅ One-to-Many: customers → orders
- ⏳ Many-to-Many: posts ↔ tags (via post_tags) - Coming soon
- ⏳ Nested relations: customers → orders → order_items

## What Gets Tested

### Schema Introspection
- Loading foreign keys from `information_schema`
- Building relationship graph
- Detecting relationship types (M2O, O2M, M2M)

### SQL Generation
- Proper JOIN conditions using foreign keys
- COALESCE for null safety
- json_agg for one-to-many arrays
- Parameterized queries for filters

### Error Handling
- Relations without foreign keys
- Missing schema cache
- Invalid table names

## Troubleshooting

### "Connection refused"
```bash
# Check if PostgreSQL is running
docker-compose ps

# Check logs
docker-compose logs postgres

# Restart
docker-compose restart
```

### "Database does not exist"
```bash
# Recreate database
docker-compose down -v
docker-compose up -d
```

### Tests fail with "relation not found"
The schema might not have loaded. Check:
```bash
docker-compose exec postgres psql -U postgres -d postgrest_parser_test -c "\dt"
```

You should see tables like `customers`, `orders`, `products`, etc.

## CI/CD

For CI pipelines, use:

```yaml
# .github/workflows/test.yml
services:
  postgres:
    image: supabase/postgres:15.8.1.040
    env:
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: postgrest_parser_test
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
```

Then run:
```bash
cargo test --features postgres
```

## Performance

Integration tests are slower than unit tests due to database I/O:
- Schema loading: ~50-100ms
- Each query test: ~10-50ms

Run unit tests separately for faster feedback:
```bash
cargo test --lib  # Fast unit tests only
```
