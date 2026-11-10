use chrono::Utc;
use rusqlite::{Connection, params};

use crate::models::*;

pub fn insert_bucket(conn: &Connection, name: &str, is_public: bool) -> Result<Bucket> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _buckets (name, is_public, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, is_public as i64, now, now],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("bucket already exists".into())
        } else {
            AppError::Internal(format!("insert bucket: {e}"))
        }
    })?;
    Ok(Bucket {
        name: name.to_string(),
        is_public,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_bucket(conn: &Connection, name: &str) -> Result<Option<Bucket>> {
    let mut stmt = conn
        .prepare("SELECT name, is_public, created_at, updated_at FROM _buckets WHERE name = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![name], |row| {
            Ok(Bucket {
                name: row.get(0)?,
                is_public: row.get::<_, i64>(1)? != 0,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(b)) => Ok(Some(b)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn list_buckets(conn: &Connection) -> Result<Vec<Bucket>> {
    let mut stmt = conn
        .prepare("SELECT name, is_public, created_at, updated_at FROM _buckets ORDER BY name")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Bucket {
                name: row.get(0)?,
                is_public: row.get::<_, i64>(1)? != 0,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut buckets = Vec::new();
    for row in rows {
        buckets.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(buckets)
}

pub fn delete_bucket(conn: &Connection, name: &str) -> Result<Vec<FileRecord>> {
    let files = list_files_all(conn, name)?;
    conn.execute("DELETE FROM _files WHERE bucket = ?1", params![name])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    conn.execute("DELETE FROM _buckets WHERE name = ?1", params![name])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(files)
}

fn list_files_all(conn: &Connection, bucket: &str) -> Result<Vec<FileRecord>> {
    let mut stmt = conn
        .prepare("SELECT id, bucket, filename, mime_type, size, created_at, updated_at FROM _files WHERE bucket = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map(params![bucket], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                bucket: row.get(1)?,
                filename: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut files = Vec::new();
    for row in rows {
        files.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(files)
}

pub fn insert_file(
    conn: &Connection,
    bucket: &str,
    id: &str,
    filename: &str,
    mime_type: &str,
    size: i64,
) -> Result<FileRecord> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _files (id, bucket, filename, mime_type, size, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, bucket, filename, mime_type, size, now, now],
    )
    .map_err(|e| AppError::Internal(format!("insert file: {e}")))?;
    Ok(FileRecord {
        id: id.to_string(),
        bucket: bucket.to_string(),
        filename: filename.to_string(),
        mime_type: mime_type.to_string(),
        size,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_file(conn: &Connection, id: &str) -> Result<Option<FileRecord>> {
    let mut stmt = conn
        .prepare("SELECT id, bucket, filename, mime_type, size, created_at, updated_at FROM _files WHERE id = ?1")
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                bucket: row.get(1)?,
                filename: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(f)) => Ok(Some(f)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn list_files(conn: &Connection, bucket: &str, page: i64, per_page: i64) -> Result<FilesPage> {
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _files WHERE bucket = ?1",
            params![bucket],
            |row| row.get(0),
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let total_pages = std::cmp::max(1, (total + per_page - 1) / per_page);
    let offset = (page - 1) * per_page;

    let mut stmt = conn
        .prepare(
            "SELECT id, bucket, filename, mime_type, size, created_at, updated_at FROM _files WHERE bucket = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let rows = stmt
        .query_map(params![bucket, per_page, offset], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                bucket: row.get(1)?,
                filename: row.get(2)?,
                mime_type: row.get(3)?,
                size: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut files = Vec::new();
    for row in rows {
        files.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }

    Ok(FilesPage {
        files,
        total,
        page,
        per_page,
        total_pages,
    })
}

pub fn delete_file(conn: &Connection, id: &str) -> Result<Option<FileRecord>> {
    let existing = get_file(conn, id)?;
    if existing.is_some() {
        conn.execute("DELETE FROM _files WHERE id = ?1", params![id])
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(existing)
}
