use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use common::{
    anyhow::Result,
    codegen::{CustomType, ty_enum_to_metadata_ty_enum, ty_to_llvm_ty},
    error::codegen::CodeGenError,
    indexmap::IndexMap,
    inkwell::{
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        types::{ArrayType, BasicMetadataTypeEnum},
        values::{FunctionValue, IntValue, PointerValue},
    },
    parser::{
        common::{ParsedToken, ParsedTokenInstance},
        function::FunctionDefinition,
        variable::VariableReference,
    },
    ty::Type,
};

use crate::{irgen::create_ir_from_parsed_token, pointer::access_variable_ptr};

pub fn allocate_string<'a>(
    builder: &'a Builder<'_>,
    i8_type: common::inkwell::types::IntType<'a>,
    string_buffer: String,
) -> Result<(PointerValue<'a>, ArrayType<'a>)>
{
    // Create a buffer from the String
    let mut string_bytes = string_buffer.as_bytes().to_vec();

    // If the last byte is not a null byte we automaticly add the null byte
    if let Some(last_byte) = string_bytes.last() {
        // Check if the last byte is not a null byte
        if *last_byte != 0 {
            // Push the \0 byte
            string_bytes.push(0);
        }
    }
    // Create a Sized array type with every element being an i8
    let sized_array_ty = i8_type.array_type(string_bytes.len() as u32);

    // Allocate the Array based on its type
    let buffer_ptr = builder.build_alloca(sized_array_ty, "string_buffer")?;

    // Create a String array from the byte values of the string and the array
    let str_array = i8_type.const_array(
        string_bytes
            .iter()
            .map(|byte| i8_type.const_int(*byte as u64, false))
            .collect::<Vec<IntValue>>()
            .as_slice(),
    );

    // Store the array in the buffer ptr
    builder.build_store(buffer_ptr, str_array)?;

    // Return the buffer's ptr
    Ok((buffer_ptr, sized_array_ty))
}

/// Creates a new variable from a `TypeDiscriminant`.
/// It is UB to access the value of the variable created here before initilazing it with actual data.
pub fn create_new_variable<'a, 'b>(
    ctx: &'a Context,
    builder: &'a Builder<'_>,
    var_name: &str,
    var_type: &Type,
    custom_types: Rc<IndexMap<String, CustomType>>,
) -> Result<(PointerValue<'a>, BasicMetadataTypeEnum<'a>)>
{
    // Turn a `TypeDiscriminant` into an LLVM type
    let var_type = ty_to_llvm_ty(ctx, var_type, custom_types.clone())?;

    // Allocate an instance of the converted type
    let v_ptr = builder.build_alloca(var_type, var_name)?;

    // Return the pointer of the allocation and the type
    Ok((v_ptr, var_type.into()))
}
