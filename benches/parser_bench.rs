use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use postgrest_parser::{parse_query_string, query_string_to_sql};

/// Benchmark simple query parsing
fn bench_simple_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_parsing");

    group.bench_function("select_fields", |b| {
        b.iter(|| parse_query_string(black_box("select=id,name,email")))
    });

    group.bench_function("single_filter", |b| {
        b.iter(|| parse_query_string(black_box("age=gte.18")))
    });

    group.bench_function("order_by", |b| {
        b.iter(|| parse_query_string(black_box("order=created_at.desc")))
    });

    group.bench_function("limit_offset", |b| {
        b.iter(|| parse_query_string(black_box("limit=10&offset=20")))
    });

    group.finish();
}

/// Benchmark complex query parsing
fn bench_complex_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_parsing");

    group.bench_function("multiple_filters", |b| {
        b.iter(|| {
            parse_query_string(black_box(
                "age=gte.18&status=eq.active&name=like.*Smith*&created_at=gte.2024-01-01"
            ))
        })
    });

    group.bench_function("nested_logic", |b| {
        b.iter(|| {
            parse_query_string(black_box(
                "and=(age.gte.18,status.eq.active,or(role.eq.admin,role.eq.moderator))"
            ))
        })
    });

    group.bench_function("with_relations", |b| {
        b.iter(|| {
            parse_query_string(black_box(
                "select=id,name,orders(id,total,items(name,price))"
            ))
        })
    });

    group.bench_function("full_query", |b| {
        b.iter(|| {
            parse_query_string(black_box(
                "select=id,name,orders(total)&age=gte.18&status=in.(active,pending)&order=created_at.desc&limit=10"
            ))
        })
    });

    group.finish();
}

/// Benchmark individual operator types
fn bench_operators(c: &mut Criterion) {
    let mut group = c.benchmark_group("operators");

    // Comparison operators
    group.bench_function("comparison_eq", |b| {
        b.iter(|| parse_query_string(black_box("id=eq.1")))
    });

    group.bench_function("comparison_gte", |b| {
        b.iter(|| parse_query_string(black_box("age=gte.18")))
    });

    // Pattern matching
    group.bench_function("pattern_like", |b| {
        b.iter(|| parse_query_string(black_box("name=like.*Smith*")))
    });

    group.bench_function("pattern_ilike", |b| {
        b.iter(|| parse_query_string(black_box("name=ilike.*smith*")))
    });

    group.bench_function("pattern_match", |b| {
        b.iter(|| parse_query_string(black_box("name=match.^John")))
    });

    // List operators
    group.bench_function("list_in", |b| {
        b.iter(|| parse_query_string(black_box("status=in.(active,pending,processing)")))
    });

    // Full-text search
    group.bench_function("fts_basic", |b| {
        b.iter(|| parse_query_string(black_box("content=fts.search term")))
    });

    group.bench_function("fts_with_language", |b| {
        b.iter(|| parse_query_string(black_box("content=fts(french).terme recherche")))
    });

    // Array operators
    group.bench_function("array_contains", |b| {
        b.iter(|| parse_query_string(black_box("tags=cs.{rust,elixir}")))
    });

    group.bench_function("array_overlap", |b| {
        b.iter(|| parse_query_string(black_box("tags=ov.(rust,elixir,go)")))
    });

    // Range operators
    group.bench_function("range_sl", |b| {
        b.iter(|| parse_query_string(black_box("range=sl.[1,10)")))
    });

    // JSON path
    group.bench_function("json_path_arrow", |b| {
        b.iter(|| parse_query_string(black_box("data->name=eq.John")))
    });

    group.bench_function("json_path_double_arrow", |b| {
        b.iter(|| parse_query_string(black_box("data->>email=like.*@example.com")))
    });

    // Type casting
    group.bench_function("type_cast", |b| {
        b.iter(|| parse_query_string(black_box("price::numeric=gt.100")))
    });

    // Quantifiers
    group.bench_function("quantifier_any", |b| {
        b.iter(|| parse_query_string(black_box("tags=eq(any).{rust,elixir}")))
    });

    group.bench_function("quantifier_all", |b| {
        b.iter(|| parse_query_string(black_box("score=gte(all).{80,90,95}")))
    });

    group.finish();
}

/// Benchmark SQL generation
fn bench_sql_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sql_generation");

    group.bench_function("simple_select", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("users"),
                black_box("select=id,name,email")
            )
        })
    });

    group.bench_function("with_filters", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("users"),
                black_box("age=gte.18&status=eq.active")
            )
        })
    });

    group.bench_function("with_order", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("users"),
                black_box("select=id,name&order=created_at.desc&limit=10")
            )
        })
    });

    group.bench_function("complex_query", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("users"),
                black_box("select=id,name,email&age=gte.18&status=in.(active,pending)&order=created_at.desc&limit=10&offset=20")
            )
        })
    });

    group.finish();
}

/// Benchmark end-to-end performance across varying query sizes
fn bench_query_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_size_scaling");

    for num_filters in [1, 3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_filters),
            num_filters,
            |b, &num_filters| {
                let mut query_parts = Vec::new();
                for i in 0..num_filters {
                    query_parts.push(format!("field{}=eq.value{}", i, i));
                }
                let query = query_parts.join("&");

                b.iter(|| {
                    query_string_to_sql(black_box("table"), black_box(&query))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark deep nesting performance
fn bench_nesting_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("nesting_depth");

    // Test logic tree nesting
    for depth in [1, 2, 3, 5].iter() {
        group.bench_with_input(
            BenchmarkId::new("logic_tree", depth),
            depth,
            |b, &depth| {
                let mut query = String::from("field=eq.value");
                for _ in 0..depth {
                    query = format!("and=({},other=eq.val)", query);
                }

                b.iter(|| parse_query_string(black_box(&query)))
            },
        );
    }

    group.finish();
}

/// Benchmark realistic workload scenarios
fn bench_realistic_workloads(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workloads");

    group.bench_function("user_search", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("users"),
                black_box("select=id,name,email,avatar&name=ilike.*john*&status=eq.active&order=created_at.desc&limit=20")
            )
        })
    });

    group.bench_function("paginated_list", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("posts"),
                black_box("select=id,title,author(name),created_at&published=is.true&order=created_at.desc&limit=50&offset=100")
            )
        })
    });

    group.bench_function("filtered_report", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("orders"),
                black_box("select=id,total,status,items(name,quantity,price)&status=in.(completed,shipped)&total=gte.100&created_at=gte.2024-01-01&order=total.desc")
            )
        })
    });

    group.bench_function("complex_search", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("articles"),
                black_box("select=id,title,author(name),tags&content=fts(english).machine learning&tags=cs.{ai,ml}&published=is.true&created_at=gte.2024-01-01&order=created_at.desc&limit=25")
            )
        })
    });

    group.bench_function("dashboard_aggregation", |b| {
        b.iter(|| {
            query_string_to_sql(
                black_box("analytics"),
                black_box("and=(date=gte.2024-01-01,date=lte.2024-12-31,or(status.eq.active,status.eq.pending))&order=date.desc&limit=365")
            )
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_parsing,
    bench_complex_parsing,
    bench_operators,
    bench_sql_generation,
    bench_query_size_scaling,
    bench_nesting_depth,
    bench_realistic_workloads
);
criterion_main!(benches);
