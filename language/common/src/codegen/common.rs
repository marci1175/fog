use std::{collections::HashMap, sync::Arc};

use crate::{
    codegen::ty::Type,
    parser::common::{CustomItem, StatementVariant},
};
use anyhow::Result;
use indexmap::IndexMap;
use inkwell::{basic_block::BasicBlock, types::BasicMetadataTypeEnum, values::PointerValue};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PreAllocationEntry<'ctx>
{
    AllocationMap(HashMap<StatementVariant, PreAllocationEntry<'ctx>>),
    PreAllocationPtr((PointerValue<'ctx>, BasicMetadataTypeEnum<'ctx>, Type)),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FunctionArgumentIdentifier<IDENT, IDX>
{
    Identifier(IDENT),
    Index(IDX),
}

pub fn fn_arg_to_string(fn_name: &str, fn_arg: &FunctionArgumentIdentifier<String, usize>)
-> String
{
    match fn_arg {
        FunctionArgumentIdentifier::Identifier(ident) => ident.to_string(),
        FunctionArgumentIdentifier::Index(idx) => {
            format!("{fn_name}_idx_{idx}_arg")
        },
    }
}

/// Serves as a way to store information about the current loop body we are currently in.
#[derive(Debug, Clone)]
pub struct LoopBodyBlocks<'ctx>
{
    /// The BasicBlock of the loop's body
    pub loop_body: BasicBlock<'ctx>,

    /// The BasicBlock of the code's continuation. This gets executed when we break out of the `loop_body`.
    pub loop_body_exit: BasicBlock<'ctx>,
}

impl<'ctx> LoopBodyBlocks<'ctx>
{
    pub fn new(loop_body: BasicBlock<'ctx>, loop_body_exit: BasicBlock<'ctx>) -> Self
    {
        Self {
            loop_body,
            loop_body_exit,
        }
    }
}

pub fn fetch_nested_pointer_ty(
    custom_types: &Arc<IndexMap<String, CustomItem>>,
    pointer_ty: Type,
) -> Result<Type>
{
    match pointer_ty.clone().try_as_pointer() {
        Some(pointer_inner) => {
            match pointer_inner {
                Some(inner_token) => Ok(fetch_nested_pointer_ty(custom_types, *inner_token)?),
                None => Ok(pointer_ty),
            }
        },
        None => Ok(pointer_ty),
    }
}
