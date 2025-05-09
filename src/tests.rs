use crate::{DecodeError, FromKey, IntoKey, Key, Kv, KvBackend, MemoryBackend};

#[test]
fn memory_backend_insert_get_delete() {
    let mut backend = MemoryBackend::new();
    let k = ("foo",).into_key();
    backend.set(k.clone(), vec![1, 2, 3]).unwrap();
    assert_eq!(backend.get(&k).unwrap(), Some(vec![1, 2, 3]));
    backend.delete(&k).unwrap();
    assert_eq!(backend.get(&k).unwrap(), None);
}

#[test]
fn kv_set_and_get() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(("a",), 123u32).unwrap();
    kv.set(("b",), 321u32).unwrap();
    assert_eq!(kv.get::<_, u32>(("a",)).unwrap(), Some(123));
    assert_eq!(kv.get::<_, u32>(("b",)).unwrap(), Some(321));
}

#[test]
fn get_and_delete_nonexistent_key() {
    let mut kv = Kv::new(MemoryBackend::new());
    let k = ("no_such_key",);
    assert_eq!(kv.get::<_, u32>(k).unwrap(), None);
    kv.delete(k).unwrap();
}

#[test]
fn clear_removes_all_data() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(("a",), 111u32).unwrap();
    kv.set(("b",), 222u32).unwrap();
    kv.clear().unwrap();
    assert_eq!(kv.get::<_, u32>(("a",)).unwrap(), None);
    assert_eq!(kv.get::<_, u32>(("b",)).unwrap(), None);
    assert_eq!(kv.keys().unwrap().count(), 0);
}

#[test]
fn keys_are_iterable_and_dynamic_after_mutation() {
    let mut kv = Kv::new(MemoryBackend::new());
    let k1 = ("1",).into_key();
    let k2 = ("2",).into_key();
    let k3 = ("3",).into_key();
    for (k, v) in [k1.clone(), k2.clone(), k3.clone()]
        .iter()
        .zip(&[10u8, 11u8, 12u8])
    {
        kv.set(k.clone(), *v).unwrap();
    }
    let mut keys: Vec<_> = kv.keys().unwrap().collect();
    keys.sort();
    assert_eq!(keys, vec![k1.clone(), k2.clone(), k3.clone()]);
    kv.delete((&k2,)).unwrap();
    let mut keys2: Vec<_> = kv.keys().unwrap().collect();
    keys2.sort();
    assert_eq!(keys2, vec![k1, k3]);
}

#[test]
fn values_iter_with_prefix_works() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(("a", 1u8), 1u8).unwrap();
    kv.set(("a", 2u8), 2u8).unwrap();
    kv.set(("b", 1u8), 3u8).unwrap();
    let vals: Vec<_> = kv
        .list::<u8>()
        .prefix(("a",))
        .iter()
        .map(|(_, v)| v)
        .collect();
    assert!(vals.contains(&1));
    assert!(vals.contains(&2));
    assert_eq!(vals.len(), 2);
}

#[test]
fn values_iter_with_range_filters() {
    let mut kv = Kv::new(MemoryBackend::new());
    for i in 0..10 {
        kv.set((format!("k{i}"),), i as u8).unwrap();
    }
    let vals: Vec<_> = kv
        .list::<u8>()
        .start(("k2",).into_key())
        .end(("k7",).into_key())
        .iter()
        .map(|(_, v)| v)
        .collect();

    assert_eq!(vals.len(), 6);
    for v in 2u8..=7 {
        assert!(vals.contains(&v), "vals: {vals:?} missing {v}");
    }
}

#[test]
fn can_store_and_retrieve_mixed_types() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(("int",), 55i64).unwrap();
    kv.set(("str",), "world".to_string()).unwrap();
    assert_eq!(kv.get::<_, i64>(("int",)).unwrap(), Some(55));
    assert_eq!(
        kv.get::<_, String>(("str",)).unwrap(),
        Some("world".to_string())
    );
}

#[test]
fn filter_accepts_prefix_start_end_combinations() {
    let mut kv = Kv::new(MemoryBackend::new());
    for idx in 0..5 {
        kv.set(("xa", idx as u64), idx as u64).unwrap();
        kv.set(("yb", idx as u64), idx as u64).unwrap();
    }
    let mut found: Vec<_> = kv
        .list::<u64>()
        .prefix(("xa",).into_key())
        .start(("xa", 2u64).into_key())
        .iter()
        .map(|(_, v)| v)
        .collect();
    found.sort();
    assert_eq!(found, vec![2, 3, 4]);
}

#[test]
fn filter_matches_none_yields_no_results() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(("foo",), 99u8).unwrap();
    let vals: Vec<_> = kv
        .list::<u8>()
        .prefix(("bar",))
        .iter()
        .map(|(_, v)| v)
        .collect();
    assert!(vals.is_empty());
}

#[test]
fn composite_key_sorting_works() {
    let mut kv = Kv::new(MemoryBackend::new());
    for id in [10u64, 2u64] {
        kv.set(("user", id), id).unwrap();
    }
    let mut ids: Vec<_> = kv
        .list::<u64>()
        .prefix(("user",))
        .iter()
        .map(|(_, v)| v)
        .collect();
    ids.sort();
    assert_eq!(ids, vec![2, 10]);
}

#[test]
fn numeric_segment_sorts_numerically_not_lexically() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(("foo", 2u64), "small").unwrap();
    kv.set(("foo", 10u64), "large").unwrap();
    let prefix = ("foo",).into_key();
    let results: Vec<_> = kv.list::<String>().prefix(&prefix).iter().collect();
    let decoded: Vec<_> = results.iter().map(|(_, v)| v.as_str()).collect();
    assert_eq!(decoded, vec!["small", "large"]);
}

#[test]
fn struct_key_roundtrip() {
    #[derive(Debug, PartialEq, Eq, Clone)]
    struct AssetKey {
        scope: String,
        name: String,
        id: u32,
    }
    impl IntoKey for AssetKey {
        fn into_key(self) -> Key {
            (self.scope, self.name, self.id).into_key()
        }
    }
    impl FromKey for AssetKey {
        fn from_key(key: &Key) -> Result<Self, DecodeError> {
            let (scope, name, id): (String, String, u32) = FromKey::from_key(key)?;
            Ok(Self { scope, name, id })
        }
    }
    let original = AssetKey {
        scope: "global".to_string(),
        name: "item".to_string(),
        id: 101,
    };
    let key = original.clone().into_key();
    let roundtripped = AssetKey::from_key(&key).unwrap();
    assert_eq!(original, roundtripped);
}

#[test]
fn arbitrary_struct_roundtrip() {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    struct MyStruct {
        n: i32,
        txt: String,
        opt: Option<f64>,
    }

    let mut kv = Kv::new(MemoryBackend::new());
    let value = MyStruct {
        n: 42,
        txt: "hello".into(),
        opt: Some(std::f64::consts::PI),
    };
    kv.set(("struct",), value.clone()).unwrap();

    let loaded = kv.get::<_, MyStruct>(("struct",)).unwrap();
    assert_eq!(loaded, Some(value));
}
