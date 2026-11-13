use super::codec::*;
use super::*;

impl CacheStore {
    pub(crate) fn read_hash(
        data: &mut HashMap<String, CacheEntry>,
        conn: &Connection,
        key: &str,
    ) -> Result<serde_json::Map<String, serde_json::Value>> {
        match Self::decode_entry(Self::lookup(data, conn, key)?)? {
            None => Ok(serde_json::Map::new()),
            Some(CacheValue::Hash(fields)) => Ok(fields),
            Some(CacheValue::Str(_)) => Err(wrong_type("string", "hash")),
            Some(CacheValue::List(_)) => Err(wrong_type("list", "hash")),
        }
    }

    /// HSET — set one or more hash fields; returns fields written.
    pub fn hash_set(
        &self,
        conn: &Connection,
        key: &str,
        fields: serde_json::Map<String, serde_json::Value>,
    ) -> Result<usize> {
        let mut data = self.data.write().unwrap();
        let mut map = Self::read_hash(&mut data, conn, key)?;
        let n = fields.len();
        for (f, v) in fields {
            map.insert(f, v);
        }
        Self::write_value(&mut data, conn, key, CacheValue::Hash(map))?;
        Ok(n)
    }

    /// HGET — get one hash field; None when missing.
    pub fn hash_get(
        &self,
        conn: &Connection,
        key: &str,
        field: &str,
    ) -> Result<Option<serde_json::Value>> {
        let mut data = self.data.write().unwrap();
        Ok(Self::read_hash(&mut data, conn, key)?.get(field).cloned())
    }

    /// HGETALL — all hash fields.
    pub fn hash_get_all(
        &self,
        conn: &Connection,
        key: &str,
    ) -> Result<serde_json::Map<String, serde_json::Value>> {
        let mut data = self.data.write().unwrap();
        Self::read_hash(&mut data, conn, key)
    }

    /// HDEL — delete hash fields; returns count removed.
    pub fn hash_del(&self, conn: &Connection, key: &str, fields: &[String]) -> Result<usize> {
        let mut data = self.data.write().unwrap();
        let mut map = Self::read_hash(&mut data, conn, key)?;
        let mut removed = 0;
        for f in fields {
            if map.remove(f).is_some() {
                removed += 1;
            }
        }
        if map.is_empty() {
            data.remove(key);
            db::delete_cache_entry(conn, key)?;
        } else {
            Self::write_value(&mut data, conn, key, CacheValue::Hash(map))?;
        }
        Ok(removed)
    }

    /// HLEN — number of hash fields.
    pub fn hash_len(&self, conn: &Connection, key: &str) -> Result<usize> {
        let mut data = self.data.write().unwrap();
        Ok(Self::read_hash(&mut data, conn, key)?.len())
    }
}
