//! Criterion benchmarks for the cache store, mirroring kevin's bench suite
//! (set/get/keys) plus the typed operations.

use std::hint::black_box;

use blazers_core::db;
use blazers_server::cache::CacheStore;
use criterion::{Criterion, criterion_group, criterion_main};
use rusqlite::Connection;
use serde_json::json;

fn bench_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::migrate_db(&conn).unwrap();
    conn
}

fn bench_set(c: &mut Criterion) {
    let cache = CacheStore::new();
    let conn = bench_conn();
    c.bench_function("cache_set", |b| {
        b.iter(|| {
            cache
                .set(
                    black_box(&conn),
                    black_box("key"),
                    black_box("value"),
                    black_box(0),
                )
                .unwrap();
        });
    });
}

fn bench_get(c: &mut Criterion) {
    let cache = CacheStore::new();
    let conn = bench_conn();
    cache.set(&conn, "key", "value", 0).unwrap();
    c.bench_function("cache_get", |b| {
        b.iter(|| {
            black_box(cache.get(black_box(&conn), black_box("key")).unwrap());
        });
    });
}

fn bench_keys(c: &mut Criterion) {
    let cache = CacheStore::new();
    let conn = bench_conn();
    for i in 0..1000 {
        cache
            .set(&conn, &format!("key-{i:04}"), "value", 0)
            .unwrap();
    }
    c.bench_function("cache_keys", |b| {
        b.iter(|| {
            black_box(cache.keys());
        });
    });
}

fn bench_incr(c: &mut Criterion) {
    let cache = CacheStore::new();
    let conn = bench_conn();
    c.bench_function("cache_incr", |b| {
        b.iter(|| {
            black_box(
                cache
                    .incr(black_box(&conn), black_box("counter"), black_box(1))
                    .unwrap(),
            );
        });
    });
}

fn bench_list_push(c: &mut Criterion) {
    let cache = CacheStore::new();
    let conn = bench_conn();
    c.bench_function("cache_list_push", |b| {
        b.iter(|| {
            black_box(
                cache
                    .list_push(
                        black_box(&conn),
                        black_box("queue"),
                        vec![json!("job")],
                        false,
                    )
                    .unwrap(),
            );
        });
    });
}

fn bench_hash_set(c: &mut Criterion) {
    let cache = CacheStore::new();
    let conn = bench_conn();
    let mut fields = serde_json::Map::new();
    fields.insert("field".to_string(), json!("value"));
    c.bench_function("cache_hash_set", |b| {
        b.iter(|| {
            black_box(
                cache
                    .hash_set(black_box(&conn), black_box("user:1"), fields.clone())
                    .unwrap(),
            );
        });
    });
}

criterion_group!(
    benches,
    bench_set,
    bench_get,
    bench_keys,
    bench_incr,
    bench_list_push,
    bench_hash_set
);
criterion_main!(benches);
