#![feature(test)]

extern crate test;

#[cfg(feature = "sqlite")]
use stupid_simple_kv::SqliteBackend;
#[allow(unused_imports)]
use stupid_simple_kv::{IntoKey, Kv};

#[cfg(test)]
mod bench_sqlite {
    use super::*;
    use stupid_simple_kv::KvResult;
    #[allow(unused_imports)]
    use test::{Bencher, black_box};

    #[cfg(feature = "sqlite")]
    #[bench]
    fn bench_sqlite_set_get(b: &mut Bencher) -> KvResult<()> {
        let backend = Box::new(SqliteBackend::in_memory()?);
        let mut kv = Kv::new(backend);

        b.iter(|| {
            for i in 0..1000u64 {
                let k = ("sqlite-key", i).to_key();
                black_box(kv.set(&k, i.into())).unwrap();
                let _ = kv.get(&k).unwrap();
            }
        });
        Ok(())
    }
}
