use criterion::{Criterion, black_box, criterion_group, criterion_main};

#[cfg(feature = "sqlite")]
use stupid_simple_kv::SqliteBackend;
use stupid_simple_kv::{IntoKey, Kv};

#[cfg(feature = "sqlite")]
fn bench_sqlite_set_get(c: &mut Criterion) {
    c.bench_function("sqlite_set_get", |b| {
        b.iter(|| {
            let backend = Box::new(SqliteBackend::in_memory().unwrap());
            let mut kv = Kv::new(backend);

            for i in 0..1000i64 {
                let k = ("sqlite-key", i).to_key();
                black_box(kv.set(&k, i.into())).unwrap();
                let _ = kv.get(&k).unwrap();
            }
        });
    });
}

#[cfg(feature = "sqlite")]
criterion_group!(sqlite_benches, bench_sqlite_set_get);
#[cfg(feature = "sqlite")]
criterion_main!(sqlite_benches);
