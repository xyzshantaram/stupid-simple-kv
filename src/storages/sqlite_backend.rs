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
    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> KvResult<()> {
        self.conn.execute(
            "REPLACE INTO kv (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
    fn get(&self, key: &[u8]) -> KvResult<Option<Vec<u8>>> {
        self.conn
            .query_row("SELECT value FROM kv WHERE key = ?1", params![key], |row| {
                row.get(0)
            })
            .optional()
            .map_err(Into::into)
    }
    fn delete(&mut self, key: &[u8]) -> KvResult<()> {
        self.conn
            .execute("DELETE FROM kv WHERE key = ?1", params![key])?;
        Ok(())
    }
    fn clear(&mut self) -> KvResult<()> {
        self.conn.execute("DELETE FROM kv", [])?;
        Ok(())
    }
    fn get_many<'a>(
        &'a self,
        keys: Vec<Vec<u8>>,
    ) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + 'a>> {
        let mut results = Vec::new();
        for k in keys {
            if let Some(v) = self.get(&k)? {
                results.push(v);
            }
        }
        Ok(Box::new(results.into_iter()))
    }
    fn keys<'a>(&'a self) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + 'a>> {
        let mut stmt = self.conn.prepare("SELECT key FROM kv ORDER BY key")?;
        let keys = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<Vec<u8>>, _>>()?;
        Ok(Box::new(keys.into_iter()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Kv, key};

    #[test]
    fn sqlite_set_and_get() {
        let mut backend = SqliteBackend::in_memory().unwrap();
        let k = key!["hello"];
        backend.set(k.clone(), vec![1, 2, 3]).unwrap();
        assert_eq!(backend.get(&k).unwrap(), Some(vec![1, 2, 3]));
    }

    #[test]
    fn sqlite_kv_set_get_delete() {
        let mut backend = SqliteBackend::in_memory().unwrap();
        let mut kv = Kv::new(backend);
        let k = key!["num"];
        kv.set(k.clone(), 42u32).unwrap();
        assert_eq!(kv.get::<u32>(&k).unwrap(), Some(42));
        kv.delete(&k).unwrap();
        assert_eq!(kv.get::<u32>(&k).unwrap(), None);
    }

    #[test]
    fn sqlite_prefix_iter() {
        let mut backend = SqliteBackend::in_memory().unwrap();
        let mut kv = Kv::new(backend);
        for i in 0..5u8 {
            kv.set(key!["users", i], i as u16).unwrap();
        }
        let pref = key!["users"];
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
