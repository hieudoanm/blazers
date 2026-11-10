use std::sync::Arc;

use super::Registered;
use crate::handlers::*;
use crate::protocol::{PropertySchema, Schema, Tool};

pub(super) fn collection_tools(db: &DbHandle) -> Vec<Registered> {
    vec![
        tool(
            "blazers_collections_create",
            "Create a collection with an optional JSON schema",
            Schema::new(
                vec![
                    ("name", PropertySchema::string("Collection name")),
                    (
                        "schema",
                        PropertySchema::string("Optional JSON schema for record fields"),
                    ),
                ],
                &["name"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_collections_create(&db, args))
            },
        ),
        tool(
            "blazers_collections_delete",
            "Delete a collection and every record it holds",
            Schema::new(
                vec![("name", PropertySchema::string("Collection name"))],
                &["name"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_collections_delete(&db, args))
            },
        ),
        tool(
            "blazers_collections_list",
            "List all collections in the blazers database",
            Schema::empty(),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_collections_list(&db, args))
            },
        ),
        tool(
            "blazers_export",
            "Export every collection, record, bucket and file as JSON",
            Schema::new(
                vec![(
                    "format",
                    PropertySchema::enumeration("Export format", &["json"]),
                )],
                &["format"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_export(&db, args))
            },
        ),
        tool(
            "blazers_health",
            "Check if the blazers MCP server is operational",
            Schema::empty(),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_health(&db, args))
            },
        ),
    ]
}

pub(super) fn record_tools(db: &DbHandle) -> Vec<Registered> {
    vec![
        tool(
            "blazers_import",
            "Import a previously exported JSON payload",
            Schema::new(
                vec![
                    (
                        "format",
                        PropertySchema::enumeration("Import format", &["json"]),
                    ),
                    (
                        "data",
                        PropertySchema::string("The export payload as JSON text"),
                    ),
                ],
                &["format", "data"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_import(&db, args))
            },
        ),
        tool(
            "blazers_records_create",
            "Create a record, generating an id when none is given",
            Schema::new(
                vec![
                    ("collection", PropertySchema::string("Collection name")),
                    (
                        "id",
                        PropertySchema::string("Optional record id; generated when absent"),
                    ),
                    (
                        "data",
                        PropertySchema::object("Record body as a JSON object"),
                    ),
                ],
                &["collection", "data"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_records_create(&db, args))
            },
        ),
        tool(
            "blazers_records_delete",
            "Delete one record by id",
            Schema::new(
                vec![
                    ("collection", PropertySchema::string("Collection name")),
                    ("id", PropertySchema::string("Record id")),
                ],
                &["collection", "id"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_records_delete(&db, args))
            },
        ),
        tool(
            "blazers_records_get",
            "Fetch one record by id",
            Schema::new(
                vec![
                    ("collection", PropertySchema::string("Collection name")),
                    ("id", PropertySchema::string("Record id")),
                ],
                &["collection", "id"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_records_get(&db, args))
            },
        ),
        tool(
            "blazers_records_list",
            "List records in a collection, with search and paging",
            Schema::new(
                vec![
                    ("collection", PropertySchema::string("Collection name")),
                    (
                        "page",
                        PropertySchema::integer("1-based page number; default 1"),
                    ),
                    (
                        "per_page",
                        PropertySchema::integer("Records per page, 1-200; default 50"),
                    ),
                    (
                        "search",
                        PropertySchema::string("Free-text filter over record fields"),
                    ),
                ],
                &["collection"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_records_list(&db, args))
            },
        ),
        tool(
            "blazers_records_update",
            "Replace the body of an existing record",
            Schema::new(
                vec![
                    ("collection", PropertySchema::string("Collection name")),
                    ("id", PropertySchema::string("Record id")),
                    (
                        "data",
                        PropertySchema::object("The new record body as a JSON object"),
                    ),
                ],
                &["collection", "id", "data"],
            ),
            {
                let db = Arc::clone(db);
                Box::new(move |args| handle_records_update(&db, args))
            },
        ),
    ]
}

/// Builds one tool registration, keeping the name in one place.
pub(super) fn tool(
    name: &str,
    description: &str,
    input_schema: Schema,
    handler: Box<dyn Fn(&str) -> crate::protocol::ToolResult>,
) -> Registered {
    Registered {
        tool: Tool {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
        },
        handler,
    }
}
