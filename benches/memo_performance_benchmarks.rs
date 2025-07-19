//! Performance Benchmarks for Memoranda Core Components
//!
//! These benchmarks test the performance characteristics of core components
//! including large collection handling, search performance, and memory usage
//! to ensure the system can handle production workloads efficiently.

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use memoranda::memo::{Memo, MemoId, MemoSearcher, MemoStore, SearchQuery, SearchResult};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

// Configuration constants for benchmarks
const SMALL_COLLECTION_SIZE: usize = 100;
const MEDIUM_COLLECTION_SIZE: usize = 1_000;
const LARGE_COLLECTION_SIZE: usize = 10_000;
const XLARGE_COLLECTION_SIZE: usize = 50_000;

// Content size constants (in characters)
const SMALL_CONTENT_SIZE: usize = 100;
const MEDIUM_CONTENT_SIZE: usize = 1_000;
const LARGE_CONTENT_SIZE: usize = 10_000;

/// Create a test memo with specified title and content size
fn create_test_memo(id: usize, content_size: usize) -> Memo {
    let title = format!("Test Memo #{id}");
    let content = format!(
        "This is test memo content {} - {}",
        id,
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(content_size / 50)
    );

    Memo::new(title, content).unwrap()
}

/// Create a collection of memos for benchmarking
fn create_memo_collection(count: usize, content_size: usize) -> Vec<Memo> {
    (0..count)
        .map(|i| create_test_memo(i, content_size))
        .collect()
}

/// Create a test MemoStore with temporary directory
fn create_test_store() -> (MemoStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a .memoranda directory
    let memoranda_dir = temp_path.join(".memoranda");
    fs::create_dir(&memoranda_dir).unwrap();

    // Create a git directory to satisfy the git_root requirement
    let git_dir = temp_path.join(".git");
    fs::create_dir(&git_dir).unwrap();

    let store = MemoStore::new(temp_path.to_path_buf());
    (store, temp_dir)
}

/// Benchmark memo creation performance
fn bench_memo_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memo_creation");

    for &content_size in &[SMALL_CONTENT_SIZE, MEDIUM_CONTENT_SIZE, LARGE_CONTENT_SIZE] {
        group.throughput(Throughput::Bytes(content_size as u64));
        group.bench_with_input(
            BenchmarkId::new("create_memo", content_size),
            &content_size,
            |b, &content_size| {
                b.iter(|| {
                    let title = "Benchmark Memo".to_string();
                    let content = "x".repeat(content_size);
                    black_box(Memo::new(title, content).unwrap())
                });
            },
        );
    }

    group.finish();
}

/// Benchmark large collection handling
fn bench_large_collection_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_collection_handling");

    for &size in &[
        SMALL_COLLECTION_SIZE,
        MEDIUM_COLLECTION_SIZE,
        LARGE_COLLECTION_SIZE,
    ] {
        let memos = create_memo_collection(size, MEDIUM_CONTENT_SIZE);

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("create_collection", size),
            &size,
            |b, &size| {
                b.iter(|| black_box(create_memo_collection(size, MEDIUM_CONTENT_SIZE)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("iterate_collection", size),
            &memos,
            |b, memos| {
                b.iter(|| {
                    let mut count = 0;
                    for memo in memos {
                        count += black_box(memo.content.len());
                    }
                    black_box(count)
                });
            },
        );

        // Benchmark HashMap operations for large collections
        let mut memo_map: HashMap<MemoId, Memo> = HashMap::new();
        for memo in &memos {
            memo_map.insert(memo.id, memo.clone());
        }

        group.bench_with_input(
            BenchmarkId::new("hashmap_lookup", size),
            &(&memo_map, &memos[0].id),
            |b, (map, id)| {
                b.iter(|| black_box(map.get(id)));
            },
        );
    }

    group.finish();
}

/// Benchmark search performance
fn bench_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_performance");

    for &size in &[
        SMALL_COLLECTION_SIZE,
        MEDIUM_COLLECTION_SIZE,
        LARGE_COLLECTION_SIZE,
    ] {
        let memos = create_memo_collection(size, MEDIUM_CONTENT_SIZE);
        let mut searcher = MemoSearcher::new();

        // Index all memos
        for memo in &memos {
            searcher.index_memo(memo);
        }

        group.throughput(Throughput::Elements(size as u64));

        // Benchmark simple term search
        group.bench_with_input(
            BenchmarkId::new("simple_term_search", size),
            &(&searcher, &memos),
            |b, (searcher, memos)| {
                let query = SearchQuery::with_terms(vec!["test".to_string()]);
                b.iter(|| black_box(searcher.search(&query, memos)));
            },
        );

        // Benchmark phrase search
        group.bench_with_input(
            BenchmarkId::new("phrase_search", size),
            &(&searcher, &memos),
            |b, (searcher, memos)| {
                let query = SearchQuery::with_phrase("test memo".to_string());
                b.iter(|| black_box(searcher.search(&query, memos)));
            },
        );

        // Benchmark boolean search
        group.bench_with_input(
            BenchmarkId::new("boolean_search", size),
            &(&searcher, &memos),
            |b, (searcher, memos)| {
                let query = SearchQuery::parse_query("test AND memo");
                b.iter(|| black_box(searcher.search(&query, memos)));
            },
        );

        // Benchmark wildcard search
        group.bench_with_input(
            BenchmarkId::new("wildcard_search", size),
            &(&searcher, &memos),
            |b, (searcher, memos)| {
                let query = SearchQuery::parse_query("test*");
                b.iter(|| black_box(searcher.search(&query, memos)));
            },
        );

        // Benchmark indexing performance
        group.bench_with_input(BenchmarkId::new("indexing", size), &memos, |b, memos| {
            b.iter(|| {
                let mut searcher = MemoSearcher::new();
                for memo in memos {
                    searcher.index_memo(memo);
                }
                black_box(searcher)
            });
        });
    }

    group.finish();
}

/// Benchmark file system operations
fn bench_filesystem_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("filesystem_operations");

    for &size in &[100, 500, 1000] {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("create_and_save_memos", size),
            &size,
            |b, &size| {
                b.iter_batched(
                    create_test_store,
                    |(store, _temp_dir)| {
                        for i in 0..size {
                            let title = format!("Benchmark Memo {i}");
                            let content = format!("Content for memo {i}");
                            black_box(store.create_memo(title, content).unwrap());
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("load_all_memos", size),
            &size,
            |b, &size| {
                b.iter_batched(
                    || {
                        let (store, temp_dir) = create_test_store();
                        for i in 0..size {
                            let title = format!("Benchmark Memo {i}");
                            let content = format!("Content for memo {i}");
                            store.create_memo(title, content).unwrap();
                        }
                        (store, temp_dir)
                    },
                    |(store, _temp_dir)| black_box(store.list_memos().unwrap()),
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark MemoStore operations
fn bench_memo_store_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memo_store_operations");

    // Prepare test data
    let (store, _temp_dir) = create_test_store();
    let mut memo_ids = Vec::new();

    // Create test memos
    for i in 0..1000 {
        let title = format!("Store Benchmark Memo {i}");
        let content = format!("Content for store benchmark memo {i}");
        let memo = store.create_memo(title, content).unwrap();
        memo_ids.push(memo.id);
    }

    group.bench_function("search_memos_string", |b| {
        b.iter(|| black_box(store.search_memos("benchmark").unwrap()));
    });

    group.bench_function("get_memo_by_id", |b| {
        let id = &memo_ids[memo_ids.len() / 2]; // Get middle memo
        b.iter(|| black_box(store.get_memo(id).unwrap()));
    });

    group.bench_function("update_memo", |b| {
        let id = &memo_ids[0];
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let new_content = format!("Updated content {counter}");
            black_box(store.update_memo(id, new_content).unwrap())
        });
    });

    group.bench_function("get_all_context", |b| {
        b.iter(|| black_box(store.get_all_context().unwrap()));
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    for &size in &[MEDIUM_COLLECTION_SIZE, LARGE_COLLECTION_SIZE] {
        group.throughput(Throughput::Elements(size as u64));

        // Benchmark memory usage for large collections
        group.bench_with_input(
            BenchmarkId::new("clone_large_collection", size),
            &size,
            |b, &size| {
                let memos = create_memo_collection(size, MEDIUM_CONTENT_SIZE);
                b.iter(|| black_box(memos.clone()));
            },
        );

        // Benchmark serialization memory usage
        group.bench_with_input(
            BenchmarkId::new("serialize_collection", size),
            &size,
            |b, &size| {
                let memos = create_memo_collection(size, MEDIUM_CONTENT_SIZE);
                b.iter(|| {
                    let serialized: Vec<String> = memos
                        .iter()
                        .map(|memo| serde_json::to_string(memo).unwrap())
                        .collect();
                    black_box(serialized)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent operations (simulated)
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");

    let memos = create_memo_collection(MEDIUM_COLLECTION_SIZE, MEDIUM_CONTENT_SIZE);
    let mut searcher = MemoSearcher::new();
    for memo in &memos {
        searcher.index_memo(memo);
    }

    // Simulate concurrent searches
    group.bench_function("multiple_concurrent_searches", |b| {
        let queries = vec![
            SearchQuery::with_terms(vec!["test".to_string()]),
            SearchQuery::with_phrase("lorem ipsum".to_string()),
            SearchQuery::parse_query("test AND memo"),
            SearchQuery::parse_query("lorem OR ipsum"),
            SearchQuery::with_terms(vec!["benchmark".to_string()]),
        ];

        b.iter(|| {
            let results: Vec<Vec<SearchResult>> = queries
                .iter()
                .map(|query| searcher.search(query, &memos))
                .collect();
            black_box(results)
        });
    });

    group.finish();
}

/// Benchmark edge cases and stress scenarios
fn bench_stress_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_scenarios");

    // Very large memo content
    group.bench_function("very_large_memo_content", |b| {
        let huge_content = "x".repeat(1_000_000); // 1MB content
        b.iter(|| {
            let memo = Memo::new("Huge Memo".to_string(), huge_content.clone()).unwrap();
            black_box(memo)
        });
    });

    // Search in very large collection
    if cfg!(feature = "stress_tests") {
        let xlarge_memos = create_memo_collection(XLARGE_COLLECTION_SIZE, MEDIUM_CONTENT_SIZE);
        let mut xlarge_searcher = MemoSearcher::new();
        for memo in &xlarge_memos {
            xlarge_searcher.index_memo(memo);
        }

        group.bench_function("search_xlarge_collection", |b| {
            let query = SearchQuery::with_terms(vec!["test".to_string()]);
            b.iter(|| black_box(xlarge_searcher.search(&query, &xlarge_memos)));
        });
    }

    // Many small searches
    let small_memos = create_memo_collection(SMALL_COLLECTION_SIZE, SMALL_CONTENT_SIZE);
    let mut small_searcher = MemoSearcher::new();
    for memo in &small_memos {
        small_searcher.index_memo(memo);
    }

    group.bench_function("many_small_searches", |b| {
        let queries: Vec<SearchQuery> = (0..100)
            .map(|i| SearchQuery::with_terms(vec![format!("term{}", i)]))
            .collect();

        b.iter(|| {
            let results: Vec<Vec<SearchResult>> = queries
                .iter()
                .map(|query| small_searcher.search(query, &small_memos))
                .collect();
            black_box(results)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_memo_creation,
    bench_large_collection_handling,
    bench_search_performance,
    bench_filesystem_operations,
    bench_memo_store_operations,
    bench_memory_usage,
    bench_concurrent_operations,
    bench_stress_scenarios
);

criterion_main!(benches);
