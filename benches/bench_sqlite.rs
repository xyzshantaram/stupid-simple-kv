#![feature(test)]

extern crate test;

#[cfg(feature = "sqlite")]
use stupid_simple_kv::storages::sqlite_backend::SqliteBackend;
#[allow(unused_imports)]
use stupid_simple_kv::{IntoKey, Kv};

#[cfg(test)]
mod bench_sqlite {
    use super::*;
    #[allow(unused_imports)]
    use test::{Bencher, black_box};

    // SQLite benchmark using `iter_batched` for setup/teardown isolation
    #[cfg(feature = "sqlite")]
    #[bench]
    fn bench_sqlite_set_get(b: &mut Bencher) {
        let backend = SqliteBackend::in_memory().unwrap();
        let mut kv = Kv::new(backend);

        b.iter(|| {
            // Process items in batches manually
            for i in 0..1000u64 {
                let k = ("x", i).into_key();
                kv.set(k.clone(), black_box(i)).unwrap();
                let _ = kv.get::<_, u64>(&k).unwrap();
            }
        });
    }
}
