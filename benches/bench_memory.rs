use criterion::{Criterion, black_box, criterion_group, criterion_main};
use stupid_simple_kv::{IntoKey, Kv, MemoryBackend};

fn bench_memory_set(c: &mut Criterion) {
    let keys: Vec<_> = (0..1000u64).map(|i| ("num", i).to_key()).collect();
    let values: Vec<_> = (0..1000u64).collect();

    c.bench_function("memory_set", |b| {
        b.iter(|| {
            for (k, &v) in keys.iter().zip(&values) {
                black_box(k.clone());
                black_box(v);
            }
        });
    });
}

fn bench_memory_get(c: &mut Criterion) {
    let backend = Box::new(MemoryBackend::new());
    let mut kv = Kv::new(backend);
    let keys: Vec<_> = (0..1000u64).map(|i| ("num", i).to_key()).collect();
    let values: Vec<_> = (0..1000i64).collect();

    for (k, &v) in keys.iter().zip(&values) {
        kv.set(k, v.into()).unwrap();
    }

    c.bench_function("memory_get", |b| {
        b.iter(|| {
            for k in &keys {
                let _ = kv.get(k).unwrap();
            }
        });
    });
}

criterion_group!(memory_benches, bench_memory_set, bench_memory_get);
criterion_main!(memory_benches);
