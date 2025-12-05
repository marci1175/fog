use common::{assert_same_fields, chrono::NaiveDate, serde};
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

assert_same_fields!(common::dependency_manager::DependencyInformation, DependencyInformation, { dependency_name, dependency_source_path, dependency_version, author, date_added, secret });
assert_same_fields!(common::dependency_manager::DependencyUpload, DependencyUpload, { dependency_name, source_files, dependency_version, author });
assert_same_fields!(common::dependency_manager::DependencyUpdateRequest, DependencyUpdateRequest, { dependency_name, updated_source, secret });
assert_same_fields!(
    common::dependency_manager::DependencyUploadReply,
    DependencyUploadReply,
    { secret_to_dep }
);
assert_same_fields!(common::dependency_manager::Dependency, Dependency, {
    source
});
