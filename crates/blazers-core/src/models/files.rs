use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub is_public: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
    #[serde(default)]
    pub is_public: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub bucket: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct FilesPage {
    pub files: Vec<FileRecord>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}
