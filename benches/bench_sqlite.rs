#![feature(test)]

extern crate test;

#[cfg(feature = "sqlite")]
use stupid_simple_kv::SqliteBackend;
#[allow(unused_imports)]
use stupid_simple_kv::{IntoKey, Kv};

#[cfg(test)]
mod bench_sqlite {
    use super::*;
    #[allow(unused_imports)]
    use test::{Bencher, black_box};

    #[cfg(feature = "sqlite")]
    #[bench]
    fn bench_sqlite_set_get(b: &mut Bencher) {
        let mut backend = SqliteBackend::in_memory().unwrap();
        let mut kv = Kv::new(&mut backend);

        b.iter(|| {
            for i in 0..1000u64 {
                let k = ("sqlite-key", i).to_key();
                black_box(kv.set(&k, i.into())).unwrap();
                let _ = kv.get(&k).unwrap();
            }
        });
    }
}
