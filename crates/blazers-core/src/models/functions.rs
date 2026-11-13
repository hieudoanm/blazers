use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A stored edge function: JS source executed by the embedded QuickJS runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub code: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertFunctionRequest {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct InvokeFunctionRequest {
    #[serde(default)]
    pub input: Value,
}
