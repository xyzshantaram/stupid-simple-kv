#![feature(test)]

extern crate test;

use stupid_simple_kv::{IntoKey, Kv, MemoryBackend};

#[cfg(test)]
mod bench_memory {
    use super::*;
    use test::{Bencher, black_box};

    #[bench]
    fn bench_memory_set(b: &mut Bencher) {
        let keys: Vec<_> = (0..1000u64)
            .map(|i| (String::from("num"), i).to_key())
            .collect();
        let values: Vec<_> = (0..1000u64).collect();
        b.iter(|| {
            for (k, &v) in keys.iter().zip(&values) {
                black_box(k.clone());
                black_box(v);
            }
        });
    }

    #[bench]
    fn bench_memory_get(b: &mut Bencher) {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);
        let keys: Vec<_> = (0..1000u64)
            .map(|i| (String::from("num"), i).to_key())
            .collect();
        let values: Vec<_> = (0..1000u64).collect();

        for (k, &v) in keys.iter().zip(&values) {
            kv.set(k, v.into()).unwrap();
        }

        b.iter(|| {
            for k in &keys {
                let _ = kv.get(k).unwrap();
            }
        });
    }
}
