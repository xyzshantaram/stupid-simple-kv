#![feature(test)]

extern crate test;

#[cfg(feature = "sqlite")]
use stupid_simple_kv::storages::sqlite_backend::SqliteBackend;
use stupid_simple_kv::{IntoKey, Kv, MemoryBackend};

#[cfg(test)]
mod benches {
    use super::*;
    use test::{Bencher, black_box};

    #[bench]
    fn bench_memory_set_get(b: &mut Bencher) {
        let mut kv = Kv::new(MemoryBackend::new());
        b.iter(|| {
            for i in 0..1000u64 {
                let k = ("num", i).into_key();
                kv.set(k.clone(), black_box(i)).unwrap();
                let _ = kv.get::<_, u64>(&k).unwrap();
            }
        });
    }

    #[bench]
    fn bench_key_encoding(b: &mut Bencher) {
        b.iter(|| {
            for i in 0u64..10000 {
                black_box(("foo", i, "bar", true).into_key());
            }
        });
    }

    #[cfg(feature = "sqlite")]
    #[bench]
    fn bench_sqlite_set_get(b: &mut Bencher) {
        let backend = SqliteBackend::in_memory().unwrap();
        let mut kv = Kv::new(backend);
        b.iter(|| {
            for i in 0..1000u64 {
                let k = ("x", i).into_key();
                kv.set(k.clone(), black_box(i)).unwrap();
                let _ = kv.get::<u64>(&k).unwrap();
            }
        });
    }
}
