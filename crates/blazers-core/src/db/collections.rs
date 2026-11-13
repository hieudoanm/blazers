use rusqlite::{Connection, params};

use crate::models::*;

const COLS: &str = "name, schema, list_rule, view_rule, create_rule, update_rule, delete_rule, type, created_at, updated_at";

fn row_to_collection(row: &rusqlite::Row) -> rusqlite::Result<Collection> {
    Ok(Collection {
        name: row.get(0)?,
        schema: row.get(1)?,
        list_rule: row.get(2)?,
        view_rule: row.get(3)?,
        create_rule: row.get(4)?,
        update_rule: row.get(5)?,
        delete_rule: row.get(6)?,
        r#type: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

pub fn insert_collection(conn: &Connection, name: &str, schema: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO _collections (name, schema) VALUES (?1, ?2)",
        params![name, schema],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("collection already exists".into())
        } else {
            AppError::Internal(format!("insert collection: {e}"))
        }
    })?;
    Ok(())
}

pub fn get_collection(conn: &Connection, name: &str) -> Result<Option<Collection>> {
    let mut stmt = conn
        .prepare(&format!("SELECT {COLS} FROM _collections WHERE name = ?1"))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut rows = stmt
        .query_map(params![name], row_to_collection)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    match rows.next() {
        Some(Ok(c)) => Ok(Some(c)),
        Some(Err(e)) => Err(AppError::Internal(e.to_string())),
        None => Ok(None),
    }
}

pub fn list_collections(conn: &Connection) -> Result<Vec<Collection>> {
    let mut stmt = conn
        .prepare(&format!("SELECT {COLS} FROM _collections ORDER BY name"))
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let rows = stmt
        .query_map([], row_to_collection)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut cols = Vec::new();
    for row in rows {
        cols.push(row.map_err(|e| AppError::Internal(e.to_string()))?);
    }
    Ok(cols)
}

pub fn update_collection_schema(conn: &Connection, name: &str, schema: &str) -> Result<()> {
    conn.execute(
        "UPDATE _collections SET schema = ?1, updated_at = datetime('now') WHERE name = ?2",
        params![schema, name],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn update_collection_rules(
    conn: &Connection,
    name: &str,
    rules: &CollectionRules,
) -> Result<()> {
    conn.execute(
        "UPDATE _collections SET list_rule = ?1, view_rule = ?2, create_rule = ?3, \
         update_rule = ?4, delete_rule = ?5, updated_at = datetime('now') WHERE name = ?6",
        params![
            rules.list_rule,
            rules.view_rule,
            rules.create_rule,
            rules.update_rule,
            rules.delete_rule,
            name
        ],
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}

pub fn delete_collection(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM _collections WHERE name = ?1", params![name])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let sql = format!("DROP TABLE IF EXISTS \"_data_{name}\"");
    conn.execute(&sql, [])
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(())
}
