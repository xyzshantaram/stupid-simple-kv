#[cfg(test)]
mod kv_integration_tests {
    #[cfg(feature = "sqlite")]
    use crate::SqliteBackend;
    use crate::{Kv, KvResult, KvValue, MemoryBackend, keys::IntoKey};

    #[test]
    fn set_and_get_single_value() -> KvResult<()> {
        let backend = Box::new(MemoryBackend::new());
        let mut kv = Kv::new(backend);

        let tup = (1001u64, "foo".to_string());
        let key = tup.to_key();
        let value = KvValue::String("bar".to_string());

        kv.set(&tup, value.clone())?;
        let out = kv.get(&key)?;

        assert_eq!(out, Some(value));
        Ok(())
    }

    #[test]
    fn get_nonexistent_returns_none() -> KvResult<()> {
        let backend = Box::new(MemoryBackend::new());
        let kv = Kv::new(backend);
        let tup = (9u64, "not_there".to_string());
        let out = kv.get(&tup)?;
        assert_eq!(out, None);
        Ok(())
    }

    #[test]
    fn delete_removes_value() -> KvResult<()> {
        let backend = Box::new(MemoryBackend::new());
        let mut kv = Kv::new(backend);

        let tup = (512u64, "z".to_string());
        let key = tup.to_key();
        kv.set(&tup, KvValue::Bool(true))?;
        let del = kv.delete(&key)?;
        assert!(del.is_some());
        let after = kv.get(&key)?;
        assert!(after.is_none());
        Ok(())
    }

    #[test]
    fn overwrite_value() -> KvResult<()> {
        let backend = Box::new(MemoryBackend::new());
        let mut kv = Kv::new(backend);

        let tup = (17u64, "a".to_string());
        kv.set(&tup, KvValue::I64(123))?;
        kv.set(&tup, KvValue::I64(456))?;

        let out = kv.get(&tup)?;
        assert_eq!(out, Some(KvValue::I64(456)));
        Ok(())
    }

    #[test]
    fn list_prefix() -> KvResult<()> {
        let backend = Box::new(MemoryBackend::new());
        let mut kv = Kv::new(backend);

        for i in 0..10i64 {
            let tup = (1u64, i);
            kv.set(&tup, KvValue::I64(i))?;
        }
        for j in 0..10i64 {
            let tup = (2u64, j);
            kv.set(&tup, KvValue::I64(j))?;
        }
        // List all keys starting with (1, _)
        let results = kv.list().prefix(&(1u64,)).entries()?;
        assert_eq!(results.len(), 10);
        for (k, v) in results {
            let (_prefix, idx): (u64, i64) = k.try_into()?;
            assert_eq!(v, KvValue::I64(idx));
        }
        Ok(())
    }

    #[test]
    fn list_range() -> KvResult<()> {
        let backend = Box::new(MemoryBackend::new());
        let mut kv = Kv::new(backend);

        for i in 1..=5i64 {
            let tup = (99u64, i);
            kv.set(&tup, KvValue::I64(i * 10))?;
        }
        // List from (99,2) up to but not including (99,5)
        let results = kv
            .list()
            .start(&(99u64, 2i64))
            .end(&(99u64, 5i64))
            .entries()?;

        let want = vec![20, 30, 40];
        let got: Vec<i64> = results
            .into_iter()
            .map(|(_k, v)| if let KvValue::I64(n) = v { n } else { 0 })
            .collect();

        assert_eq!(got, want);
        Ok(())
    }

    #[test]
    fn clear_backend() -> KvResult<()> {
        let backend = Box::new(MemoryBackend::new());
        let mut kv = Kv::new(backend);

        for i in 0..5i64 {
            let tup = (777u64, i);
            kv.set(&tup, KvValue::I64(i))?;
        }
        kv.backend.clear()?;
        let items = kv.entries()?;
        assert_eq!(items.len(), 0);
        Ok(())
    }

    #[test]
    fn json_roundtrip_memory() {
        let backend = Box::new(MemoryBackend::new());
        let mut kv = Kv::new(backend);

        kv.set(&(1u64,), KvValue::String("foo".to_string()))
            .unwrap();
        kv.set(&(2u64,), KvValue::Bool(true)).unwrap();
        kv.set(&(3u64,), KvValue::I64(999)).unwrap();

        let orig_entries = { kv.entries().unwrap() };
        // Dump and reload
        let json = kv.dump_json().unwrap();
        println!("Serialized: {json}");
        let backend2 = Box::new(MemoryBackend::new());
        let mut kv2 = Kv::from_json_string(backend2, json).unwrap();

        let new_entries = kv2.entries().unwrap();
        assert_eq!(orig_entries, new_entries);
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn json_roundtrip_sqlite() -> KvResult<()> {
        let backend = Box::new(SqliteBackend::in_memory()?);
        let mut kv = Kv::new(backend);
        kv.set(&(1u64, "foo"), KvValue::I64(-42)).unwrap();
        kv.set(&(2u64, "bar"), KvValue::String("baz".to_owned()))
            .unwrap();
        kv.set(&(99u64, "wat"), KvValue::Bool(false)).unwrap();

        let orig_entries = { kv.entries().unwrap() };

        let json = kv.dump_json().unwrap();
        let backend2 = Box::new(SqliteBackend::in_memory()?);
        let mut kv2 = Kv::from_json_string(backend2, json).unwrap();
        let new_entries = kv2.entries().unwrap();
        assert_eq!(orig_entries, new_entries);
        Ok(())
    }
}
