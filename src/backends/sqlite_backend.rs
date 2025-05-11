use crate::{KvBackend, KvError, KvKey, KvResult};
use rusqlite::{Connection, params};

pub struct SqliteBackend {
    conn: Connection,
}

impl SqliteBackend {
    pub fn in_memory() -> KvResult<Self> {
        let conn = Connection::open_in_memory().map_err(KvError::SqliteError)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS kv (key BLOB PRIMARY KEY, value BLOB NOT NULL);",
        )
        .map_err(KvError::SqliteError)?;
        Ok(SqliteBackend { conn })
    }

    pub fn file(path: &str) -> KvResult<Self> {
        let conn = Connection::open(path).map_err(KvError::SqliteError)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS kv (key BLOB PRIMARY KEY, value BLOB NOT NULL);",
        )
        .map_err(KvError::SqliteError)?;
        Ok(SqliteBackend { conn })
    }
}

impl KvBackend for SqliteBackend {
    fn get_range(
        &self,
        start: Option<KvKey>,
        end: Option<KvKey>,
    ) -> KvResult<Vec<(KvKey, Vec<u8>)>> {
        // Build SQL WHERE clause for start/end
        let mut sql = String::from("SELECT key, value FROM kv");
        let mut clauses = Vec::new();
        let mut params_vec: Vec<Vec<u8>> = Vec::new();

        if let Some(start_key) = &start {
            clauses.push("key >= ?".to_string());
            params_vec.push(start_key.0.clone());
        }
        if let Some(end_key) = &end {
            clauses.push("key < ?".to_string());
            params_vec.push(end_key.0.clone());
        }
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY key ASC");

        let mut stmt = self.conn.prepare(&sql).map_err(KvError::SqliteError)?;
        let params: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();
        let rows = stmt
            .query_map(&params[..], |row| {
                let key: Vec<u8> = row.get(0)?;
                let value: Vec<u8> = row.get(1)?;
                Ok((KvKey(key), value))
            })
            .map_err(KvError::SqliteError)?;

        let results = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(KvError::SqliteError)?;
        Ok(results)
    }

    fn set(&mut self, key: KvKey, value: Option<Vec<u8>>) -> KvResult<()> {
        match value {
            Some(val) => {
                self.conn
                    .execute(
                        "REPLACE INTO kv (key, value) VALUES (?1, ?2)",
                        params![key.0, val],
                    )
                    .map_err(KvError::SqliteError)?;
            }
            None => {
                self.conn
                    .execute("DELETE FROM kv WHERE key = ?1", params![key.0])
                    .map_err(KvError::SqliteError)?;
            }
        }
        Ok(())
    }

    fn clear(&mut self) -> KvResult<()> {
        self.conn
            .execute("DELETE FROM kv", [])
            .map_err(KvError::SqliteError)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Kv, KvValue};

    #[test]
    fn sqlite_set_and_get() -> KvResult<()> {
        let backend = Box::new(SqliteBackend::in_memory()?);
        let mut kv = Kv::new(backend);
        let tup = (String::from("hello"),);
        let value = KvValue::String("world".to_string());

        kv.set(&tup, value.clone())?;
        let got = kv.get(&tup)?;
        assert_eq!(got, Some(value));
        Ok(())
    }

    #[test]
    fn sqlite_kv_set_get_delete() -> KvResult<()> {
        let backend = Box::new(SqliteBackend::in_memory()?);
        let mut kv = Kv::new(backend);
        let tup = (String::from("num"),);
        let value = KvValue::U64(42);

        kv.set(&tup, value.clone())?;
        assert_eq!(kv.get(&tup)?, Some(value.clone()));
        kv.delete(&tup)?;
        assert_eq!(kv.get(&tup)?, None);
        Ok(())
    }

    #[test]
    fn sqlite_prefix_iter() -> KvResult<()> {
        let backend = Box::new(SqliteBackend::in_memory()?);
        let mut kv = Kv::new(backend);
        for i in 0..5u8 {
            let key = (String::from("users"), i as u64);
            kv.set(&key, KvValue::U64(i as u64))?;
        }
        let results = kv.list().prefix(&(String::from("users"),)).entries()?;
        assert_eq!(results.len(), 5);
        let vals: Vec<_> = results.into_iter().map(|(_, v)| v).collect();
        assert!(vals.contains(&KvValue::U64(2)));
        Ok(())
    }
}
