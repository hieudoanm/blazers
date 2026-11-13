use super::codec::*;
use super::*;

impl CacheStore {
    pub(crate) fn read_list(
        data: &mut HashMap<String, CacheEntry>,
        conn: &Connection,
        key: &str,
    ) -> Result<Vec<serde_json::Value>> {
        match Self::decode_entry(Self::lookup(data, conn, key)?)? {
            None => Ok(Vec::new()),
            Some(CacheValue::List(items)) => Ok(items),
            Some(CacheValue::Str(_)) => Err(wrong_type("string", "list")),
            Some(CacheValue::Hash(_)) => Err(wrong_type("hash", "list")),
        }
    }

    /// RPUSH/LPUSH — push values onto a list; returns the new length.
    pub fn list_push(
        &self,
        conn: &Connection,
        key: &str,
        values: Vec<serde_json::Value>,
        left: bool,
    ) -> Result<usize> {
        let mut data = self.data.write().unwrap();
        let mut items = Self::read_list(&mut data, conn, key)?;
        if left {
            for v in values.into_iter().rev() {
                items.insert(0, v);
            }
        } else {
            items.extend(values);
        }
        let len = items.len();
        Self::write_value(&mut data, conn, key, CacheValue::List(items))?;
        Ok(len)
    }

    /// LPOP/RPOP — pop `count` items (default 1) from a list end.
    /// Single item returns the value; count>1 returns an array.
    pub fn list_pop(
        &self,
        conn: &Connection,
        key: &str,
        left: bool,
        count: Option<usize>,
    ) -> Result<Option<serde_json::Value>> {
        let mut data = self.data.write().unwrap();
        let mut items = Self::read_list(&mut data, conn, key)?;
        if items.is_empty() {
            return Ok(None);
        }
        let n = count.unwrap_or(1).min(items.len());
        let popped: Vec<serde_json::Value> = if left {
            items.drain(..n).collect()
        } else {
            let start = items.len() - n;
            items.drain(start..).collect()
        };
        let single = count.is_none() || count == Some(1);
        let result = if single {
            popped.into_iter().next().unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Array(popped)
        };
        if items.is_empty() {
            data.remove(key);
            db::delete_cache_entry(conn, key)?;
        } else {
            Self::write_value(&mut data, conn, key, CacheValue::List(items))?;
        }
        Ok(Some(result))
    }

    /// LRANGE — return list items in [start, stop] (negative = from end).
    pub fn list_range(
        &self,
        conn: &Connection,
        key: &str,
        start: i64,
        stop: i64,
    ) -> Result<Vec<serde_json::Value>> {
        let mut data = self.data.write().unwrap();
        let items = Self::read_list(&mut data, conn, key)?;
        let len = items.len() as i64;
        let norm = |i: i64| -> i64 {
            if i < 0 {
                (len + i).max(-1)
            } else {
                i.min(len - 1)
            }
        };
        let (s, e) = (norm(start).max(0), norm(stop) + 1);
        if s >= e || s >= len {
            return Ok(Vec::new());
        }
        Ok(items[s as usize..(e as usize).min(len as usize)].to_vec())
    }

    /// LLEN — list length, 0 when the key is missing.
    pub fn list_len(&self, conn: &Connection, key: &str) -> Result<usize> {
        let mut data = self.data.write().unwrap();
        Ok(Self::read_list(&mut data, conn, key)?.len())
    }
}
