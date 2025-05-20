use criterion::{Criterion, black_box, criterion_group, criterion_main};
use stupid_simple_kv::IntoKey;

fn bench_key_encoding(c: &mut Criterion) {
    let items: Vec<_> = (0..10000u64).collect();
    c.bench_function("key_encoding", |b| {
        b.iter(|| {
            for &i in &items {
                black_box(("foo", i, "bar", true).to_key());
            }
        });
    });
}

criterion_group!(keys_benches, bench_key_encoding);
criterion_main!(keys_benches);
