use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    let dir = std::env::var("BLAZERS_DATA").unwrap_or_default();
    if dir.is_empty() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".blazers")
    } else {
        PathBuf::from(dir)
    }
}

pub struct ConnectionManager {
    pub path: PathBuf,
}

impl deadpool::managed::Manager for ConnectionManager {
    type Type = rusqlite::Connection;
    type Error = rusqlite::Error;

    async fn create(&self) -> std::result::Result<Self::Type, Self::Error> {
        rusqlite::Connection::open(&self.path)
    }

    async fn recycle(
        &self,
        _: &mut Self::Type,
        _: &deadpool::managed::Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        Ok(())
    }
}
