use common::{
    anyhow,
    codegen::{
        common::FunctionArgumentIdentifier,
        ty::{NotNan, Type, Value},
    },
    error::{Spanned, analyzer::AnalyzerError},
    parser::common::StatementVariant,
};

#[macro_export]
macro_rules! create_conversion_chain {
    {$destination_ty_var:ident, $modifyable_value:ident, $raw_value:ident, $original_value_ty:ty, $(($destination_ty:ident, $destination_value_ty:ty),)*} => {
        $(
            if $destination_ty_var == Type::$destination_ty {
                match <$destination_value_ty>::try_from($raw_value) {
                    Ok(converted_val) => {
                        *$modifyable_value = Value::$destination_ty(converted_val);
                    }
                    Err(_) => {
                        return Err(AnalyzerError::UnsupportedValueByType($modifyable_value.clone(), $destination_ty_var).into());
                    }
                }

                return Ok(());
            }
        )*
    };
}

/// This function is only ran on values which do not have a type cast. (ie. the user did not specifically cast the value to a specific type.)
fn resize(dest_ty: Type, value: &mut Value) -> anyhow::Result<()>
{
    // Get original value's type
    let original_ty = value.get_type();

    // If the original value is not a number return an error
    if !(original_ty.is_float() || original_ty.is_int() || original_ty.is_uint()) {
        return Err(AnalyzerError::InternalValueNotNumerical(original_ty).into());
    }

    // If the original type matches the desitnation type then we dont have to do anything really, we can just return Ok
    if original_ty == dest_ty {
        return Ok(());
    }

    // Only the numerical value's size needs to be figured out as the numbers type is resolved unlike their size. (Except int to uint)
    match value.clone() {
        Value::F64(_) => {
            // If a value is parsed as an f64 it cannot be casted to anything else without data loss.
            if original_ty != dest_ty {
                return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
            }
        },
        Value::U64(val) => {
            create_conversion_chain! {
                dest_ty, value, val, u64,
                (I64, i64),
            };

            // The macro produces code that should automatically return, so if the code executes favourably this error will never be returned.
            return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
        },
        Value::F32(val) => {
            if dest_ty == Type::F64 {
                // Cast the inner value (this cannot really return an error since we are already dereferencing a type that is NotNan<T>)
                *value = Value::F64(NotNan::new(*val as f64)?);

                return Ok(());
            }

            return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
        },
        Value::U32(val) => {
            create_conversion_chain! {
                dest_ty, value, val, u32,
                (I32, i32),
                (I64, i64),
                (U64, u64),
            };

            // The macro produces code that should automatically return, so if the code executes favourably this error will never be returned.
            return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
        },
        Value::F16(val) => {
            if dest_ty == Type::F64 {
                // Cast the inner value (this cannot really return an error since we are already dereferencing a type that is NotNan<T>)
                *value = Value::F64(NotNan::new(*val as f64)?);

                return Ok(());
            }
            else if dest_ty == Type::F32 {
                // Cast the inner value (this cannot really return an error since we are already dereferencing a type that is NotNan<T>)
                *value = Value::F32(NotNan::new(*val as f32)?);

                return Ok(());
            }

            return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
        },
        Value::U16(val) => {
            create_conversion_chain! {
                dest_ty, value, val, u16,
                (I16, i16),
                (I32, i32),
                (I64, i64),
                (U32, u32),
                (U64, u64),
            };

            // The macro produces code that should automatically return, so if the code executes favourably this error will never be returned.
            return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
        },
        Value::U8(val) => {
            create_conversion_chain! {
                dest_ty, value, val, u8,
                (I16, i16),
                (I32, i32),
                (I64, i64),
                (U16, u16),
                (U32, u32),
                (U64, u64),
            };

            // The macro produces code that should automatically return, so if the code executes favourably this error will never be returned.
            return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
        },

        // These are usually not emitted by the AST parser.
        // Negative numbers are parsed as NegateValue(uint)
        Value::I64(_) => {
            if original_ty != dest_ty {
                return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
            }
        },
        Value::I32(val) => {
            create_conversion_chain! {
                dest_ty, value, val, i32,
                (I64, i64),
                (U64, u64),
                (U32, u32),
            };

            return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
        },
        Value::I16(val) => {
            create_conversion_chain! {
                dest_ty, value, val, i16,
                (I64, i64),
                (I32, i32),
                (U64, u64),
                (U32, u32),
                (U16, u16),
            };

            return Err(AnalyzerError::UnsupportedValueByType(value.clone(), dest_ty).into());
        },

        _ => return Err(AnalyzerError::InternalValueNotNumerical(original_ty).into()),
    }

    // No modification has been done to the passed in value, all passed without issue.
    Ok(())
}

fn resolve_numerical_size_in_statement(
    stmt: &mut Spanned<StatementVariant>,
    errors: &mut Vec<Spanned<anyhow::Error>>,
) -> anyhow::Result<()>
{
    // Number literals can only be in a limited  variants of statements
    // When creating a new variable
    match &mut stmt.inner {
        // Parse the arguments of a function call
        StatementVariant::FunctionCall {
            identifier,
            arguments: call_args,
        } => {
            if let StatementVariant::InternalCheckedReference {
                identifier: _,
                referenced_item,
            } = identifier.get_inner()
            {
                match referenced_item {
                    // Function pointers are not supported right now, just return an error for this or any other unsupported item.
                    common::parser::common::ResolvedItemReference::Static(_)
                    | common::parser::common::ResolvedItemReference::Type(_)
                    | common::parser::common::ResolvedItemReference::Variable => {
                        errors.push(Spanned {
                            inner: AnalyzerError::ItemNotCallableAsFunction.into(),
                            span: *identifier.get_span(),
                        });
                    },

                    common::parser::common::ResolvedItemReference::Function(function_signature) => {
                        let args = &function_signature.args.arguments;

                        for (arg_ident, value) in call_args.iter_mut() {
                            // Look the function's argument up and fetch its type
                            let argument_query = match arg_ident {
                                FunctionArgumentIdentifier::Identifier(arg_name) => {
                                    args.get(arg_name)
                                },
                                FunctionArgumentIdentifier::Index(idx) => {
                                    args.get_index(*idx).map(|(_, v)| v)
                                },
                            };

                            // Verify that the function call's arguments match up with the function's signature
                            if let Some((ty, _id)) = argument_query {
                                // If the value is a numeric value we can convert it
                                if let Some(numeric_value) = value.inner.try_as_value_mut() {
                                    // If the argument is not convertible just skip the argument
                                    // The resize function is only callable with types that can have numeric values converted to them
                                    // This is basically just to avoid an error in the resize function.
                                    // This will return an error later at the type checking.
                                    if !ty.is_number() {
                                        // Parse next argument
                                        continue;
                                    }

                                    // Resize numeric literal
                                    resize(ty.clone(), numeric_value)?;
                                }
                                else {
                                    // Resolve its argument recursively
                                    resolve_numerical_size_in_statement(value, errors)?;
                                }
                            }
                            else {
                                // Store error of the argument nad continue parsing
                                errors.push(Spanned {
                                    inner: {
                                        if let FunctionArgumentIdentifier::Index(argument_idx) =
                                            arg_ident
                                        {
                                            AnalyzerError::FunctionArgumentCountInvalid(
                                                args.len(),
                                                *argument_idx,
                                            )
                                        }
                                        else {
                                            AnalyzerError::UnknownFunctionNamedArgument
                                        }
                                    }
                                    .into(),
                                    span: value.span,
                                });
                            }
                        }
                    },

                    // Continue analyzing the function and skip this statement
                    common::parser::common::ResolvedItemReference::Unresolved => return Ok(()),
                }
            }
            else {
                return Err(
                    AnalyzerError::InternalIdentUnresolved(stmt.get_inner().clone()).into(),
                );
            }
        },
        _ => (),
    }

    Ok(())
}

pub fn resolve_numerical_size(
    body: &mut Vec<Spanned<StatementVariant>>,
    errors: &mut Vec<Spanned<anyhow::Error>>,
) -> anyhow::Result<()>
{
    // We can simply iterate over all the statements since these arent need to be applied globally
    for stmt in body {
        resolve_numerical_size_in_statement(stmt, errors)?;
    }

    Ok(())
}

/// This function resolves statements that have a return type by default. Such as values (0, 42, 5), variables' type, function calls, etc.
/// Statements like `return foo;` do not have a directly returned type (or value).
pub fn resolve_statement_type(stmt: &Spanned<StatementVariant>) -> anyhow::Result<Option<Type>> {
    let resolved_type = match stmt.get_inner() {
        StatementVariant::InternalCheckedReference { identifier, referenced_item } => {
            match referenced_item {
                common::parser::common::ResolvedItemReference::Variable => {
                    // Lookup variable
                    todo!()
                },
                common::parser::common::ResolvedItemReference::Type(custom_item) => {
                    match custom_item.clone() {
                        common::parser::common::CustomItem::Struct(struct_definition) => Some(Type::Struct { name: struct_definition.name, fields: struct_definition.fields, attributes: struct_definition.attributes }),
                        common::parser::common::CustomItem::Enum { name, ty, variants } => Some(Type::Enum { name, ty: Box::new(ty), variants }),
                        common::parser::common::CustomItem::Trait { name, functions, access_path } => Some(Type::Trait { name: name, access_path, functions }),
                    }
                },
                common::parser::common::ResolvedItemReference::Function(function_signature) => todo!(),
                common::parser::common::ResolvedItemReference::Static(_) => todo!(),
                common::parser::common::ResolvedItemReference::Unresolved => todo!(),
            }
        },
        StatementVariant::NewVariable { variable_name, variable_type, variable_value, variable_id, is_mutable } => None,
        
        StatementVariant::BasicReference { identifier } => todo!(),
        StatementVariant::ArrayReference { variable_reference, index } => todo!(),
        StatementVariant::StructPropertyAccess { variable_reference, field_name } => todo!(),

        StatementVariant::Value(value) => Some(value.get_type()),
        StatementVariant::TypeCast(spanned, target) => Some(target.clone()),
        StatementVariant::MathematicalExpression { lhs, symbol, rhs } => resolve_statement_type(lhs)?,
        StatementVariant::NegateValue(stmt) => resolve_statement_type(stmt)?,
        StatementVariant::FunctionCall { identifier, arguments } => {
            // Resolve function signature, by resolving its name and return type
            todo!()
        },
        StatementVariant::SetValue { receiver, value } => None,
        StatementVariant::ModifyValueArithmetic { receiver, symbol, value } => None,
        StatementVariant::ReturnValue { value } => None,
        StatementVariant::Comparison { lhs, ord, rhs } => Some(Type::Boolean),
        StatementVariant::LogicalOperation { lhs, op, rhs } => resolve_statement_type(lhs)?,
        StatementVariant::If(_) => {
            // Resolve branch return values (check if they match and return that)
            todo!()
        },
        StatementVariant::CodeBlock(spanneds) => {
            // Do the same thing basically as with the branches of if.
            todo!()
        },
        StatementVariant::Grouping { inner_expr } => resolve_statement_type(inner_expr)?,
        StatementVariant::Loop(spanneds) => None,
        StatementVariant::ControlFlow(control_flow_type) => None,
        StatementVariant::ArrayInitialization { values } => Some(Type::Array { ty: todo!(), len: values.len() }),
        StatementVariant::GetPointerTo(spanned) => Some(Type::Pointer(resolve_statement_type(spanned)?.map(|ty| Box::new(ty)))),
        StatementVariant::DerefPointer(spanned) => resolve_statement_type(spanned)?,
    };

    Ok(resolved_type)
}