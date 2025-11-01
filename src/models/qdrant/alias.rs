//! Qdrant alias management models

use serde::{Deserialize, Serialize};

/// Alias change operations request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantChangeAliasesOperation {
    /// Actions to apply
    pub actions: Vec<QdrantAliasOperations>,
}

/// Possible alias operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QdrantAliasOperations {
    /// Create alias operation
    Create(QdrantCreateAliasOperation),
    /// Delete alias operation
    Delete(QdrantDeleteAliasOperation),
    /// Rename alias operation
    Rename(QdrantRenameAliasOperation),
}

/// Create alias operation wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCreateAliasOperation {
    /// Operation payload
    pub create_alias: QdrantCreateAlias,
}

/// Delete alias operation wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDeleteAliasOperation {
    /// Operation payload
    pub delete_alias: QdrantDeleteAlias,
}

/// Rename alias operation wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRenameAliasOperation {
    /// Operation payload
    pub rename_alias: QdrantRenameAlias,
}

/// Create alias payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCreateAlias {
    /// Target collection name
    pub collection_name: String,
    /// Alias name
    pub alias_name: String,
}

/// Delete alias payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantDeleteAlias {
    /// Alias name to delete
    pub alias_name: String,
}

/// Rename alias payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantRenameAlias {
    /// Existing alias name
    pub old_alias_name: String,
    /// New alias name
    pub new_alias_name: String,
}

/// Alias description returned by Qdrant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantAliasDescription {
    /// Alias name
    pub alias_name: String,
    /// Target collection name
    pub collection_name: String,
}

/// Collection aliases response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantCollectionsAliasesResponse {
    /// List of aliases
    pub aliases: Vec<QdrantAliasDescription>,
}
