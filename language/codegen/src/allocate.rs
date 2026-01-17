use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use common::{
    anyhow::{self, Result},
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
        variable::{UniqueId, VariableReference},
    },
    ty::Type,
};

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
    var_id: Option<UniqueId>,
    allocation_table: &HashMap<UniqueId, PointerValue<'a>>,
    custom_types: Rc<IndexMap<String, CustomType>>,
) -> Result<(PointerValue<'a>, BasicMetadataTypeEnum<'a>)>
{
    // Turn a `TypeDiscriminant` into an LLVM type
    let var_type = ty_to_llvm_ty(ctx, var_type, custom_types.clone())?;

    // Check if we have already pre-allocated the variable
    // If yes, we should return the pointer to the pre-allocated variable
    if let Some(var_id) = var_id {
        if let Some(ptr) = allocation_table.get(&var_id) {
            return Ok((*ptr, var_type.into()));
        }
    }
    // If no, just allocate a new one
    // This assumes that we are not in a loop.
    // Allocate an instance of the converted type
    let v_ptr = builder.build_alloca(var_type, var_name)?;

    // Return the pointer of the allocation and the type
    Ok((v_ptr, var_type.into()))
}

pub fn create_allocation_table<'ctx>(
    ctx: &'ctx Context,
    builder: &'ctx Builder<'_>,
    parsed_tokens: &[ParsedTokenInstance],
    custom_types: Rc<IndexMap<String, CustomType>>,
) -> anyhow::Result<HashMap<UniqueId, PointerValue<'ctx>>>
{
    // Create allocation table to store the allocation in later
    let mut allocation_table: HashMap<UniqueId, PointerValue> = HashMap::new();

    for tkn_inst in parsed_tokens {
        // Inner token
        let tkn = &tkn_inst.inner;

        // If a NewVariable was created in the loop pre-allocate it
        // We dont need to preallocate variabled for functions called by this since they get deallocated automaticly.
        if let ParsedToken::NewVariable {
            variable_name,
            variable_type,
            variable_value: _,
            variable_id,
            is_mutable: _,
        } = tkn
        {
            // Allocate the variable here
            // We can ignore the initial value of the variable since NewVariables will be interpreted as setvalue for the variables preallocated.
            let variable_pointer = builder.build_alloca(
                ty_to_llvm_ty(ctx, variable_type, custom_types.clone())?,
                &format!("alloca_table_{variable_name}"),
            )?;

            // Store the pointer to the allocated variable
            allocation_table.insert(*variable_id, variable_pointer);
        }
    }

    Ok(allocation_table)
}
