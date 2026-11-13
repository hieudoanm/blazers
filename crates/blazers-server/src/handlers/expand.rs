//! `?expand=` relation resolution for record lists.

use serde_json::Value;

use super::*;

/// Applies `expand` to a page of records: for each expanded field, values of
/// the form `"collection:id"` (or a bare id when a collection shares the
/// field's name) are replaced by the fetched record.
pub fn expand_records(
    conn: &rusqlite::Connection,
    records: &mut [Record],
    expand: &str,
) -> std::result::Result<(), AppError> {
    for field in expand.split(',').map(str::trim).filter(|f| !f.is_empty()) {
        // Collect (record_idx, target_collection, target_id) tuples.
        let mut lookups: Vec<(usize, String, String)> = Vec::new();
        for (i, rec) in records.iter().enumerate() {
            let Some(v) = rec.data.get(field) else {
                continue;
            };
            match v {
                Value::String(s) => {
                    if let Some((col, id)) = s.split_once(':') {
                        lookups.push((i, col.to_string(), id.to_string()));
                    } else if db::get_collection(conn, field)?.is_some() {
                        lookups.push((i, field.to_string(), s.clone()));
                    }
                }
                Value::Object(_) => {}
                _ => {}
            }
        }
        for (i, col, id) in lookups {
            if let Some(target) = db::get_record(conn, &col, &id)? {
                let entry = serde_json::json!({
                    "id": target.id,
                    "collection": col,
                    "data": target.data,
                    "created": target.created_at,
                    "updated": target.updated_at,
                });
                if let Some(obj) = records[i].data.as_object_mut() {
                    obj.insert(format!("expand_{field}"), entry);
                }
            }
        }
    }
    Ok(())
}
