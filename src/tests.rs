use crate::{Kv, KvBackend, MemoryBackend, key};

#[test]
fn memory_backend_insert_get_delete() {
    let mut backend = MemoryBackend::new();
    let k = key!["foo"];
    backend.set(k.clone(), vec![1, 2, 3]).unwrap();
    assert_eq!(backend.get(&k).unwrap(), Some(vec![1, 2, 3]));
    backend.delete(&k).unwrap();
    assert_eq!(backend.get(&k).unwrap(), None);
}

#[test]
fn kv_set_and_get() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(key!["a"], 123u32).unwrap();
    kv.set(key!["b"], 321u32).unwrap();
    assert_eq!(kv.get::<u32>(&key!["a"]).unwrap(), Some(123));
    assert_eq!(kv.get::<u32>(&key!["b"]).unwrap(), Some(321));
}

#[test]
fn get_and_delete_nonexistent_key() {
    let mut kv = Kv::new(MemoryBackend::new());
    let k = key!["no_such_key"];
    assert_eq!(kv.get::<u32>(&k).unwrap(), None);
    kv.delete(&k).unwrap();
}

#[test]
fn clear_removes_all_data() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(key!["a"], 111u32).unwrap();
    kv.set(key!["b"], 222u32).unwrap();
    kv.clear().unwrap();
    assert_eq!(kv.get::<u32>(&key!["a"]).unwrap(), None);
    assert_eq!(kv.get::<u32>(&key!["b"]).unwrap(), None);
    assert_eq!(kv.keys().unwrap().count(), 0);
}

#[test]
fn keys_are_iterable_and_dynamic_after_mutation() {
    let mut kv = Kv::new(MemoryBackend::new());
    let k1 = key!["1"];
    let k2 = key!["2"];
    let k3 = key!["3"];
    for (k, v) in &[(&k1, 10u8), (&k2, 11u8), (&k3, 12u8)] {
        kv.set(k.to_vec(), *v).unwrap();
    }
    let mut keys: Vec<_> = kv.keys().unwrap().collect();
    keys.sort();
    assert_eq!(keys, vec![k1.clone(), k2.clone(), k3.clone()]);
    kv.delete(&k2).unwrap();
    let mut keys2: Vec<_> = kv.keys().unwrap().collect();
    keys2.sort();
    assert_eq!(keys2, vec![k1, k3]);
}

#[test]
fn values_iter_with_prefix_works() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(key!["a", 1u8], 1u8).unwrap();
    kv.set(key!["a", 2u8], 2u8).unwrap();
    kv.set(key!["b", 1u8], 3u8).unwrap();
    let vals: Vec<_> = kv
        .list::<u8>()
        .prefix(&key!["a"])
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
        kv.set(key![format!("k{i}")], i as u8).unwrap();
    }
    let vals: Vec<_> = kv
        .list::<u8>()
        .start(&key!["k2"])
        .end(&key!["k7"])
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
    kv.set(key!["int"], 55i64).unwrap();
    kv.set(key!["str"], "world".to_string()).unwrap();
    assert_eq!(kv.get::<i64>(&key!["int"]).unwrap(), Some(55));
    assert_eq!(
        kv.get::<String>(&key!["str"]).unwrap(),
        Some("world".to_string())
    );
}

#[test]
fn filter_accepts_prefix_start_end_combinations() {
    let mut kv = Kv::new(MemoryBackend::new());
    for idx in 0..5 {
        kv.set(key!["xa", idx as u64], idx as u64).unwrap();
        kv.set(key!["yb", idx as u64], idx as u64).unwrap();
    }
    let mut found: Vec<_> = kv
        .list::<u64>()
        .prefix(&key!["xa"])
        .start(&key!["xa", 2u64])
        .iter()
        .map(|(_, v)| v)
        .collect();
    found.sort();
    assert_eq!(found, vec![2, 3, 4]);
}

#[test]
fn filter_matches_none_yields_no_results() {
    let mut kv = Kv::new(MemoryBackend::new());
    kv.set(key!["foo"], 99u8).unwrap();
    let vals: Vec<_> = kv
        .list::<u8>()
        .prefix(&key!["bar"])
        .iter()
        .map(|(_, v)| v)
        .collect();
    assert!(vals.is_empty());
}

#[test]
fn composite_key_sorting_works() {
    let mut kv = Kv::new(MemoryBackend::new());
    for id in [10u64, 2u64] {
        kv.set(key!["user", id], id).unwrap();
    }
    let mut ids: Vec<_> = kv
        .list::<u64>()
        .prefix(&key!["user"])
        .iter()
        .map(|(_, v)| v)
        .collect();
    ids.sort();
    assert_eq!(ids, vec![2, 10]);
}
