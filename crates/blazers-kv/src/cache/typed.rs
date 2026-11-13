use super::codec::*;
use super::*;

impl CacheStore {
    /// Look up an entry while the caller holds the write guard: memory first
    /// (with expiry check), then the DB, backfilling the map on a hit.
    pub(crate) fn lookup(
        data: &mut HashMap<String, CacheEntry>,
        conn: &Connection,
        key: &str,
    ) -> Result<Option<CacheEntry>> {
        if let Some(entry) = data.get(key) {
            if is_expired(entry) {
                data.remove(key);
                let _ = db::delete_cache_entry(conn, key);
                return Ok(None);
            }
            return Ok(Some(entry.clone()));
        }
        if let Some(entry) = db::get_cache_entry(conn, key)? {
            data.insert(key.to_string(), entry.clone());
            return Ok(Some(entry));
        }
        Ok(None)
    }

    pub(crate) fn decode_entry(entry: Option<CacheEntry>) -> Result<Option<CacheValue>> {
        Ok(entry.map(|e| decode_value(&e.value)))
    }

    /// Persist `value` under `key`, preserving the existing TTL when the entry
    /// already exists; new keys get no expiry.
    pub(crate) fn write_value(
        data: &mut HashMap<String, CacheEntry>,
        conn: &Connection,
        key: &str,
        value: CacheValue,
    ) -> Result<CacheEntry> {
        let raw = encode_value(&value);
        if let Some(existing) = Self::lookup(data, conn, key)? {
            db::update_cache_value(conn, key, &raw)?;
            let now = Utc::now().to_rfc3339();
            if let Some(slot) = data.get_mut(key) {
                slot.value = raw.clone();
                slot.updated_at = now.clone();
            }
            return Ok(CacheEntry {
                value: raw,
                updated_at: now,
                ..existing
            });
        }
        let entry = db::set_cache_entry(conn, key, &raw, 0)?;
        data.insert(key.to_string(), entry.clone());
        Ok(entry)
    }

    /// INCRBY — atomically add `by` to an integer string value. Missing keys
    /// start at 0; non-integer values are an error.
    pub fn incr(&self, conn: &Connection, key: &str, by: i64) -> Result<i64> {
        let mut data = self.data.write().unwrap();
        let current = match Self::decode_entry(Self::lookup(&mut data, conn, key)?)? {
            None => 0,
            Some(CacheValue::Str(s)) => s
                .trim()
                .parse::<i64>()
                .map_err(|_| wrong_type("string", "integer"))?,
            Some(CacheValue::List(_)) => return Err(wrong_type("list", "string")),
            Some(CacheValue::Hash(_)) => return Err(wrong_type("hash", "string")),
        };
        let next = current + by;
        Self::write_value(&mut data, conn, key, CacheValue::Str(next.to_string()))?;
        Ok(next)
    }

    /// APPEND — atomically append to a string value. Missing keys start empty.
    pub fn append(&self, conn: &Connection, key: &str, value: &str) -> Result<String> {
        let mut data = self.data.write().unwrap();
        let mut current = match Self::decode_entry(Self::lookup(&mut data, conn, key)?)? {
            None => String::new(),
            Some(CacheValue::Str(s)) => s,
            Some(CacheValue::List(_)) => return Err(wrong_type("list", "string")),
            Some(CacheValue::Hash(_)) => return Err(wrong_type("hash", "string")),
        };
        current.push_str(value);
        Self::write_value(&mut data, conn, key, CacheValue::Str(current.clone()))?;
        Ok(current)
    }

    /// EXPIRE — set a TTL in seconds (0 clears expiry). None when key missing.
    pub fn expire(&self, conn: &Connection, key: &str, ttl: i64) -> Result<Option<()>> {
        let mut data = self.data.write().unwrap();
        if Self::lookup(&mut data, conn, key)?.is_none() {
            return Ok(None);
        }
        if !db::expire_cache_entry(conn, key, ttl)? {
            return Ok(None);
        }
        let now = Utc::now().to_rfc3339();
        if let Some(entry) = data.get_mut(key) {
            entry.ttl = ttl;
            entry.expires_at = if ttl > 0 {
                (Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339()
            } else {
                String::new()
            };
            entry.updated_at = now;
        }
        Ok(Some(()))
    }

    /// TTL — seconds until expiry: positive = remaining, -1 = no expiry,
    /// None = key missing or already expired.
    pub fn ttl(&self, conn: &Connection, key: &str) -> Result<Option<i64>> {
        let Some(entry) = self.get(conn, key)? else {
            return Ok(None);
        };
        if entry.ttl <= 0 || entry.expires_at.is_empty() {
            return Ok(Some(-1));
        }
        let expires = chrono::DateTime::parse_from_rfc3339(&entry.expires_at)
            .map_err(|e| AppError::Internal(format!("bad expires_at: {e}")))?
            .with_timezone(&Utc);
        Ok(Some((expires - Utc::now()).num_seconds().max(0)))
    }
}

impl CacheStore {
    /// EXISTS — whether the key is present and unexpired.
    pub fn exists(&self, conn: &Connection, key: &str) -> Result<bool> {
        Ok(self.get(conn, key)?.is_some())
    }

    /// KEYS — sorted names of all unexpired entries.
    pub fn keys(&self) -> Vec<String> {
        let data = self.data.read().unwrap();
        let mut keys: Vec<String> = data
            .iter()
            .filter(|(_, e)| !is_expired(e))
            .map(|(k, _)| k.clone())
            .collect();
        keys.sort();
        keys
    }

    /// DEL many — delete each key, returning the count removed.
    pub fn delete_many(&self, conn: &Connection, keys: &[String]) -> Result<usize> {
        let mut removed = 0;
        for key in keys {
            if self.delete(conn, key)? {
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// TYPE — "string", "list", "hash" or "none" for a missing key.
    pub fn kind(&self, conn: &Connection, key: &str) -> Result<&'static str> {
        let Some(entry) = self.get(conn, key)? else {
            return Ok("none");
        };
        Ok(match decode_value(&entry.value) {
            CacheValue::Str(_) => "string",
            CacheValue::List(_) => "list",
            CacheValue::Hash(_) => "hash",
        })
    }
}
