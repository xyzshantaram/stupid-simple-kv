use crate::keys::Key;
use crate::storages::kv_backend::{KvBackend, KvResult};
use rusqlite::{Connection, OptionalExtension, params};

pub struct SqliteBackend {
    conn: Connection,
}

impl SqliteBackend {
    pub fn in_memory() -> KvResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS kv (key BLOB PRIMARY KEY, value BLOB NOT NULL);",
        )?;
        Ok(SqliteBackend { conn })
    }
    pub fn file(path: &str) -> KvResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS kv (key BLOB PRIMARY KEY, value BLOB NOT NULL);",
        )?;
        Ok(SqliteBackend { conn })
    }
}

impl KvBackend for SqliteBackend {
    fn set(&mut self, key: Key, value: Vec<u8>) -> KvResult<()> {
        self.conn.execute(
            "REPLACE INTO kv (key, value) VALUES (?1, ?2)",
            params![key.0, value],
        )?;
        Ok(())
    }
    fn get(&self, key: &Key) -> KvResult<Option<Vec<u8>>> {
        self.conn
            .query_row(
                "SELECT value FROM kv WHERE key = ?1",
                params![&key.0],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }
    fn delete(&mut self, key: &Key) -> KvResult<()> {
        self.conn
            .execute("DELETE FROM kv WHERE key = ?1", params![&key.0])?;
        Ok(())
    }
    fn clear(&mut self) -> KvResult<()> {
        self.conn.execute("DELETE FROM kv", [])?;
        Ok(())
    }
    fn get_many(
        &self,
        keys: Vec<Key>,
    ) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + Send + Sync + 'static>> {
        let mut results = Vec::new();
        for k in keys {
            if let Some(v) = self.get(&k)? {
                results.push(v);
            }
        }
        Ok(Box::new(results.into_iter()))
    }
    fn keys(&self) -> KvResult<Box<dyn Iterator<Item = Key> + Send + Sync + 'static>> {
        let mut stmt = self.conn.prepare("SELECT key FROM kv ORDER BY key")?;
        let keys = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<Vec<u8>>, _>>()?;
        Ok(Box::new(keys.into_iter().map(Key)))
    }
}

#[cfg(test)]
mod tests {
    use crate::storages::kv_backend::KvBackend;
    use crate::storages::sqlite_backend::SqliteBackend;
    use crate::{IntoKey, Kv};

    #[test]
    fn sqlite_set_and_get() {
        let mut backend = SqliteBackend::in_memory().unwrap();
        let k = ("hello",).into_key();
        backend.set(k.clone(), vec![1, 2, 3]).unwrap();
        assert_eq!(backend.get(&k).unwrap(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn sqlite_kv_set_get_delete() {
        let mut backend = SqliteBackend::in_memory().unwrap();
        let mut kv = Kv::new(backend);
        let k = ("num",).into_key();
        kv.set(k.clone(), 42u32).unwrap();
        assert_eq!(kv.get::<_, u32>(&k).unwrap(), Some(42));
        kv.delete(&k).unwrap();
        assert_eq!(kv.get::<_, u32>(&k).unwrap(), None);
    }

    #[test]
    fn sqlite_prefix_iter() {
        let mut backend = SqliteBackend::in_memory().unwrap();
        let mut kv = Kv::new(backend);
        for i in 0..5u8 {
            kv.set(("users", i), i as u16).unwrap();
        }
        let pref = ("users",).into_key();
        let vals: Vec<_> = kv
            .list::<u16>()
            .prefix(&pref)
            .iter()
            .map(|(_, v)| v)
            .collect();
        assert_eq!(vals.len(), 5);
        assert!(vals.contains(&2));
    }
}
