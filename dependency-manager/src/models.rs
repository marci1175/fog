use chrono::NaiveDate;
use common::serde;
use diesel::{
    Selectable,
    prelude::{Insertable, Queryable, QueryableByName},
};

#[derive(
    Debug,
    Clone,
    Selectable,
    QueryableByName,
    Queryable,
    serde::Deserialize,
    serde::Serialize,
    Insertable,
)]
#[diesel(table_name = crate::schema::dependencies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DependencyInformation
{
    pub dependency_name: String,
    pub dependency_source_path: String,
    pub dependency_version: String,
    pub author: String,
    pub date_added: NaiveDate,
    pub secret: String,
}

/// Contains both the raw compressed bytes and the information
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Dependency
{
    pub info: DependencyInformation,
    pub source: Vec<u8>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DependencyUpdateRequest
{
    pub dependency_name: String,
    // pub author: String,
    pub secret: String,
    pub updated_source: Vec<u8>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DependencyUploadReply
{
    /// Updates can ONLY be uploaded with the use of this secret
    pub secret_to_dep: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DependencyUpload
{
    pub dependency_name: String,
    pub dependency_version: String,
    pub author: String,
    /// Compressed ZIP source files
    pub source_files: Vec<u8>,
}
