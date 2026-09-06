use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{parser::function::FunctionSignature, ty::Type};

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryImport
{
    pub name: String,
    pub version: String,
}

/// Specifies the type of an import.
/// These do not serve as standalone keys, they are always stored with their name as their key. (To check for naming collisions)
/// Since imports can be aliased - the key that is associated with this import may not be the top level name of the actual import. (Key might differ from actual imported function or item's name).
#[derive(Debug, Clone)]
pub enum ImportType
{
    /// A path import is used to import files from the host machine.
    File(PathBuf),
    /// A dependency import is used to import actual items (such as functions or structs) from either a dependecy of the project, or a previously imported source file.
    /// All dependency paths have to be fully defined. (For example you cannot import namespace `a` from dependency `b` and then refer to its items later as `a::foo` instead you need to use `b::a::foo`)
    Dependency(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum FFIDeclType
{
    Static(Type),
    Function(FunctionSignature),
}
