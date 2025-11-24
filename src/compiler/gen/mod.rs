use crate::ast;
use crate::ir::{FunctionDefinition, GlobalVariable, GlobalVariableKind};
use crate::ir::instr::{Instruction, PhiInstruction, TerminatorInstruction};
use crate::ir::value::*;
use crate::sema::*;
use crate::token;

pub struct Generator<'ctx> {
    context: &'ctx mut GlobalContext,
    next_anonymous_constant_id: usize,
}

impl<'ctx> Generator<'ctx> {
    pub fn new(context: &'ctx mut GlobalContext) -> Self {
        Self {
            context,
            next_anonymous_constant_id: 0,
        }
    }

    pub fn context(&self) -> &GlobalContext {
        self.context
    }

    pub fn context_mut(&mut self) -> &mut GlobalContext {
        self.context
    }

    pub fn generate_package<'a>(mut self, modules: impl IntoIterator<Item = &'a ast::parse::ParsedModule>) -> crate::Result<()> {
        for parsed_module in modules {
            let parent_module = self.context.replace_current_module(parsed_module.namespace());

            self.generate_global_statements(parsed_module.statements())?;

            self.context.replace_current_module(parent_module);
        }

        Ok(())
    }

    pub fn generate_global_statements<'a>(&mut self, global_statements: impl IntoIterator<Item = &'a ast::GlobalNode>) -> crate::Result<()> {
        for global_statement in global_statements {
            self.generate_global_statement(global_statement)?;
        }

        Ok(())
    }

    pub fn generate_global_statement(&mut self, node: &ast::GlobalNode) -> crate::Result<Value> {
        match node.kind() {
            ast::GlobalNodeKind::Let { value, register, .. } => {
                let register = register.as_ref().expect("register should be valid after fill phase");
                if let Some(value) = value {
                    self.generate_global_let_statement(value, register)
                }
                else {
                    // The work has already been done for us
                    Ok(Value::Void)
                }
            }
            ast::GlobalNodeKind::Function { name, parameters, body, register, .. } => {
                let register = register.as_ref().expect("register should be valid after fill phase");
                if let Some(body) = body {
                    self.generate_function_definition(name, parameters, body, register)
                }
                else {
                    // The work has already been done for us
                    Ok(Value::Void)
                }
            }
            ast::GlobalNodeKind::Structure { self_type, .. } => {
                self.generate_structure_definition(*self_type)
            }
            ast::GlobalNodeKind::Implement { self_type, statements } => {
                self.generate_implement_block(self_type, statements)
            }
            ast::GlobalNodeKind::Module { statements, namespace, .. } => {
                self.generate_module_block(statements, *namespace)
            }
            ast::GlobalNodeKind::ModuleFile { .. } |
            ast::GlobalNodeKind::Import { .. } |
            ast::GlobalNodeKind::GlobImport { .. } => {
                // The work has already been done for us
                Ok(Value::Void)
            }
        }
    }

    pub fn generate_local_node(&mut self, node: &ast::LocalNode, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        if let Ok(constant) = self.generate_constant_node(node, Some(local_context), expected_type) {
            return Ok(Value::Constant(constant));
        }

        let result = match node.kind() {
            ast::LocalNodeKind::Literal(literal) => {
                self.generate_literal(node.span(), literal, local_context, expected_type)?
            }
            ast::LocalNodeKind::Path { segments } => {
                let path = self.context.get_absolute_path(node.span(), segments)?;
                self.context.get_path_value(&path, Some(&node.span()))?
            }
            ast::LocalNodeKind::Unary { operation, operand } => {
                self.generate_unary_operation(*operation, operand, local_context, expected_type)?
            }
            ast::LocalNodeKind::Binary { operation, lhs, rhs } => {
                self.generate_binary_operation(*operation, lhs, rhs, local_context, expected_type)?
            }
            ast::LocalNodeKind::Call { callee, arguments } => {
                self.generate_call_operation(callee, arguments, local_context)?
            }
            ast::LocalNodeKind::ArrayLiteral { items } => {
                self.generate_array_literal(node.span(), items, local_context, expected_type)?
            }
            ast::LocalNodeKind::TupleLiteral { items } => {
                self.generate_tuple_literal(node.span(), items, local_context, expected_type)?
            }
            ast::LocalNodeKind::StructureLiteral { structure_type: type_name, members } => {
                self.generate_structure_literal(node.span(), type_name, members, local_context)?
            }
            ast::LocalNodeKind::Grouping { content } => {
                // Fine to bypass validation steps since this is literally just parentheses
                return self.generate_local_node(content, local_context, expected_type);
            }
            ast::LocalNodeKind::Scope { statements, tail } => {
                local_context.enter_scope();

                let mut result = Value::Void;
                for statement in statements {
                    let statement_value = self.generate_local_node(statement, local_context, None)?;

                    if statement_value.get_type() == TypeHandle::NEVER {
                        // The rest of the statements in the block will never be executed, so they don't need to be generated
                        result = statement_value;
                        break;
                    }
                }
                if let Some(tail) = tail {
                    if result.get_type() != TypeHandle::NEVER {
                        let tail_value = self.generate_local_node(tail, local_context, expected_type)?;
                        result = self.coerce_to_rvalue(tail_value, local_context)?;
                    }
                }

                local_context.exit_scope();

                result
            }
            ast::LocalNodeKind::Conditional { condition, consequent, alternative } => {
                self.generate_conditional(condition, consequent, alternative.as_deref(), local_context, expected_type)?
            }
            ast::LocalNodeKind::While { condition, body } => {
                self.generate_while_loop(condition, body, local_context)?
            }
            ast::LocalNodeKind::Break => {
                let break_label = local_context.break_label()
                    .ok_or_else(|| Box::new(crate::Error::new(
                        Some(node.span()),
                        crate::ErrorKind::InvalidBreak,
                    )))?;

                local_context.set_terminator(TerminatorInstruction::Branch {
                    to_label: break_label.clone(),
                });

                Value::Break
            }
            ast::LocalNodeKind::Continue => {
                let continue_label = local_context.continue_label()
                    .ok_or_else(|| Box::new(crate::Error::new(
                        Some(node.span()),
                        crate::ErrorKind::InvalidContinue,
                    )))?;

                local_context.set_terminator(TerminatorInstruction::Branch {
                    to_label: continue_label.clone(),
                });

                Value::Continue
            }
            ast::LocalNodeKind::Return { value } => {
                let return_type = local_context.return_type();

                let return_value;
                if let Some(value) = value {
                    if return_type == TypeHandle::VOID {
                        return Err(Box::new(crate::Error::new(
                            Some(value.span()),
                            crate::ErrorKind::UnexpectedReturnValue {
                                function_name: local_context.function_path().to_string(),
                            },
                        )));
                    }
                    else {
                        let value = self.generate_local_node(value, local_context, Some(return_type))?;
                        return_value = self.coerce_to_rvalue(value, local_context)?;
                    }
                }
                else if return_type != TypeHandle::VOID {
                    return Err(Box::new(crate::Error::new(
                        Some(node.span().tail_point()),
                        crate::ErrorKind::ExpectedReturnValue {
                            function_name: local_context.function_path().to_string(),
                        },
                    )));
                }
                else {
                    return_value = Value::Void;
                }

                local_context.set_terminator(TerminatorInstruction::Return {
                    value: return_value,
                });

                Value::Never
            }
            ast::LocalNodeKind::Let { name, value_type, is_mutable, value, .. } => {
                self.generate_local_let_statement(node.span(), name, value_type.as_deref(), *is_mutable, value.as_deref(), local_context)?
            }
            _ => {
                return Err(Box::new(crate::Error::new(
                    Some(node.span()),
                    crate::ErrorKind::UnexpectedExpression,
                )));
            }
        };

        if let Some(expected_type) = expected_type {
            self.enforce_type(result, expected_type, node.span(), local_context)
                // For debugging purposes. This information is often useful
                .inspect_err(|_| eprintln!("problematic node: {node:?}"))
        }
        else {
            Ok(result)
        }
    }

    pub fn new_anonymous_constant(&mut self, pointer_type: TypeHandle) -> GlobalRegister {
        let id = self.next_anonymous_constant_id;
        self.next_anonymous_constant_id += 1;

        let identifier = format!(".const.{}.{id}", self.context.package().info().name());
        GlobalRegister::new(identifier.as_bytes().into(), pointer_type)
    }

    pub fn enforce_type(&mut self, value: Value, expected_type: TypeHandle, span: crate::Span, local_context: &mut LocalContext) -> crate::Result<Value> {
        let got_type = value.get_type();

        let conversion = self.context.try_implicit_conversion(got_type, expected_type, true)
            .ok_or_else(|| Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::IncompatibleTypes {
                    expected_type: expected_type.path(self.context()).to_string(),
                    got_type: got_type.path(self.context()).to_string(),
                },
            )))?;

        self.convert_value(value, expected_type, conversion, local_context)
    }

    pub fn enforce_constant_type(&mut self, constant: Constant, expected_type: TypeHandle, span: crate::Span) -> crate::Result<Constant> {
        let got_type = constant.get_type();

        let conversion = self.context.try_implicit_conversion(got_type, expected_type, true)
            .ok_or_else(|| Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::IncompatibleTypes {
                    expected_type: expected_type.path(self.context()).to_string(),
                    got_type: got_type.path(self.context()).to_string(),
                },
            )))?;

        self.convert_constant(constant, expected_type, conversion)
    }

    pub fn explicitly_convert(&mut self, value: Value, to_type: TypeHandle, span: crate::Span, local_context: &mut LocalContext) -> crate::Result<Value> {
        let from_type = value.get_type();

        let conversion = self.context.try_explicit_conversion(from_type, to_type, true)
            .ok_or_else(|| Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::IncompatibleTypes {
                    expected_type: to_type.path(self.context()).to_string(),
                    got_type: from_type.path(self.context()).to_string(),
                },
            )))?;

        self.convert_value(value, to_type, conversion, local_context)
    }

    pub fn convert_value(&mut self, mut value: Value, to_type: TypeHandle, conversion: Conversion, local_context: &mut LocalContext) -> crate::Result<Value> {
        if let Some(operation) = conversion.operation_needed {
            if let Value::Constant(constant) = value {
                Ok(Value::Constant(Constant::Convert {
                    operation,
                    value: Box::new(constant),
                    result_type: to_type,
                }))
            }
            else {
                let value = self.coerce_to_rvalue(value, local_context)?;
                let result = local_context.new_anonymous_register(to_type);

                local_context.add_instruction(Instruction::Convert {
                    operation,
                    result: result.clone(),
                    value,
                });

                Ok(Value::Register(result))
            }
        }
        else {
            value.set_type(to_type);
            Ok(value)
        }
    }

    pub fn convert_constant(&mut self, mut constant: Constant, to_type: TypeHandle, conversion: Conversion) -> crate::Result<Constant> {
        if let Some(operation) = conversion.operation_needed {
            Ok(Constant::Convert {
                operation,
                value: Box::new(constant),
                result_type: to_type,
            })
        }
        else {
            constant.set_type(to_type);
            Ok(constant)
        }
    }

    pub fn coerce_to_rvalue(&mut self, value: Value, local_context: &mut LocalContext) -> crate::Result<Value> {
        let (pointer, mut pointee_type) = match value {
            Value::Indirect { pointer, pointee_type } => {
                (*pointer, pointee_type)
            }
            Value::Constant(Constant::Indirect { pointer, pointee_type }) => {
                (Value::Constant(*pointer), pointee_type)
            }
            value => return Ok(value)
        };

        // If the pointee is `*mut _`, copy the semantics from the pointer to the pointee
        if let &TypeRepr::Pointer {
            pointee_type: next_pointee_type,
            semantics: PointerSemantics::Mutable,
        } = pointee_type.repr(self.context) {
            let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(self.context) else {
                panic!("indirect value pointer is not a pointer type")
            };
            if !matches!(semantics, PointerSemantics::ImmutableSymbol) {
                pointee_type = self.context.get_pointer_type(next_pointee_type, semantics);
            }
        }

        let result = local_context.new_anonymous_register(pointee_type);

        local_context.add_instruction(Instruction::Load {
            result: result.clone(),
            pointer,
        });

        Ok(Value::Register(result))
    }

    fn generate_literal(&mut self, span: crate::Span, literal: &token::Literal, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let result = match *literal {
            token::Literal::Name(ref name) => {
                if let Some(value) = local_context.find_symbol(name) {
                    value.clone()
                }
                else {
                    self.context.get_symbol_value(self.context.current_module(), name, Some(&span))
                        .map_err(|mut error| {
                            if let crate::ErrorKind::UndefinedGlobalSymbol { .. } = error.kind() {
                                *error.kind_mut() = crate::ErrorKind::UndefinedSymbol {
                                    name: name.to_string(),
                                };
                            }
                            error
                        })?
                }
            }
            token::Literal::Integer(value, suffix) => {
                let value_type = match suffix {
                    Some(suffix) => suffix.as_handle(),
                    None => expected_type.unwrap_or(TypeHandle::I32),
                };
                let Some(value) = IntegerValue::from_unknown_type(value, value_type, self.context.target()) else {
                    return Err(Box::new(crate::Error::new(
                        Some(span),
                        crate::ErrorKind::IncompatibleValueType {
                            value: value.to_string(),
                            type_name: value_type.path(self.context).to_string(),
                        },
                    )));
                };

                Value::from(value)
            }
            token::Literal::Float(value, suffix) => {
                let value_type = match suffix {
                    Some(suffix) => suffix.as_handle(),
                    None => expected_type.unwrap_or(TypeHandle::F64),
                };
                let Some(value) = FloatValue::from_unknown_type(value, value_type, self.context.target()) else {
                    return Err(Box::new(crate::Error::new(
                        Some(span),
                        crate::ErrorKind::IncompatibleValueType {
                            value: value.to_string(),
                            type_name: value_type.path(self.context).to_string(),
                        },
                    )));
                };

                Value::from(value)
            }
            token::Literal::Boolean(value) => {
                Value::from(value)
            }
            token::Literal::NullPointer => {
                Value::Constant(Constant::NullPointer(expected_type.unwrap_or_else(|| {
                    self.context.get_pointer_type(TypeHandle::VOID, PointerSemantics::Immutable)
                })))
            }
            token::Literal::String(ref value) => {
                let constant = Constant::String {
                    array_type: self.context.get_array_type(TypeHandle::U8, Some(value.len() as u64)),
                    value: value.clone(),
                };
                let pointer = self.new_anonymous_constant(constant.get_type());

                self.context.package_mut().output_mut().add_global_variable(GlobalVariable::new(
                    pointer.clone(),
                    GlobalVariableKind::AnonymousConstant,
                    constant,
                ));

                Value::Constant(Constant::Register(pointer))
            }
            token::Literal::PrimitiveType(primitive_type) => {
                Value::Constant(Constant::Type(primitive_type.handle))
            }
        };

        Ok(result)
    }

    fn generate_array_literal(&mut self, span: crate::Span, items: &[ast::LocalNode], local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let Some(array_type) = expected_type else {
            return Err(Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::UnknownArrayType,
            )));
        };
        let &TypeRepr::Array { item_type, .. } = array_type.repr(self.context) else {
            return Err(Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::UnknownArrayType,
            )));
        };

        let mut non_constant_items = Vec::new();

        let constant_items: Vec<Constant> = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let item_value = self.generate_local_node(item, local_context, Some(item_type))?;

                if let Value::Constant(item_constant) = item_value {
                    Ok(item_constant)
                }
                else {
                    let item_value = self.coerce_to_rvalue(item_value, local_context)?;
                    non_constant_items.push((index, item_value));

                    Ok(Constant::Undefined(item_type))
                }
            })
            .collect::<crate::Result<_>>()?;

        let array_pointer_type = self.context.get_pointer_type(array_type, PointerSemantics::Immutable);
        let array_pointer = local_context.new_anonymous_register(array_pointer_type);

        local_context.add_instruction(Instruction::StackAllocate {
            result: array_pointer.clone(),
        });

        let array_pointer = Value::Register(array_pointer);

        // Store the constant items, if any
        if non_constant_items.len() < items.len() {
            local_context.add_instruction(Instruction::Store {
                value: Value::Constant(Constant::Array {
                    array_type,
                    items: constant_items,
                }),
                pointer: array_pointer.clone(),
            });
        }

        for (index, item_value) in non_constant_items {
            let item_pointer_type = self.context.get_pointer_type(item_value.get_type(), PointerSemantics::Mutable);
            let item_pointer = local_context.new_anonymous_register(item_pointer_type);

            local_context.add_instruction(Instruction::GetElementPointer {
                result: item_pointer.clone(),
                pointer: array_pointer.clone(),
                indices: [
                    Value::from(IntegerValue::new(IntegerType::I32, 0)),
                    Value::from(IntegerValue::new(IntegerType::Usize, index as i128))
                ].into(),
            });
            local_context.add_instruction(Instruction::Store {
                value: item_value,
                pointer: item_pointer.into(),
            });
        }

        Ok(Value::Indirect {
            pointer: Box::new(array_pointer),
            pointee_type: array_type,
        })
    }

    fn generate_tuple_literal(&mut self, span: crate::Span, items: &[ast::LocalNode], local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let Some(tuple_type) = expected_type else {
            return Err(Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::UnknownTupleType,
            )));
        };
        let TypeRepr::Tuple { item_types } = tuple_type.repr(self.context).clone() else {
            return Err(Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::UnknownTupleType,
            )));
        };
        if items.len() != item_types.len() {
            // TODO: better error
            return Err(Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::UnknownTupleType,
            )));
        }

        let mut non_constant_items = Vec::new();

        let constant_items: Vec<Constant> = items
            .iter()
            .zip(&item_types)
            .enumerate()
            .map(|(index, (item, &item_type))| {
                let item_value = self.generate_local_node(item, local_context, Some(item_type))?;

                if let Value::Constant(item_constant) = item_value {
                    Ok(item_constant)
                }
                else {
                    let item_value = self.coerce_to_rvalue(item_value, local_context)?;
                    non_constant_items.push((index, item_value));

                    Ok(Constant::Undefined(item_type))
                }
            })
            .collect::<crate::Result<_>>()?;

        let tuple_pointer_type = self.context.get_pointer_type(tuple_type, PointerSemantics::Immutable);
        let tuple_pointer = local_context.new_anonymous_register(tuple_pointer_type);

        local_context.add_instruction(Instruction::StackAllocate {
            result: tuple_pointer.clone(),
        });

        let tuple_pointer = Value::Register(tuple_pointer);

        // Store the constant items, if any
        if non_constant_items.len() < items.len() {
            local_context.add_instruction(Instruction::Store {
                value: Value::Constant(Constant::Tuple {
                    tuple_type,
                    items: constant_items,
                }),
                pointer: tuple_pointer.clone(),
            });
        }

        for (index, item_value) in non_constant_items {
            let item_pointer_type = self.context.get_pointer_type(item_value.get_type(), PointerSemantics::Mutable);
            let item_pointer = local_context.new_anonymous_register(item_pointer_type);

            local_context.add_instruction(Instruction::GetElementPointer {
                result: item_pointer.clone(),
                pointer: tuple_pointer.clone(),
                indices: [
                    Value::from(IntegerValue::new(IntegerType::I32, 0)),
                    Value::from(IntegerValue::new(IntegerType::I32, index as i128))
                ].into(),
            });
            local_context.add_instruction(Instruction::Store {
                value: item_value,
                pointer: item_pointer.into(),
            });
        }

        Ok(Value::Indirect {
            pointer: Box::new(tuple_pointer),
            pointee_type: tuple_type,
        })
    }

    fn generate_structure_literal(&mut self, span: crate::Span, type_name: &ast::LocalNode, initializer_members: &[(Box<str>, ast::LocalNode)], local_context: &mut LocalContext) -> crate::Result<Value> {
        let struct_type = self.context.interpret_node_as_type(type_name)?;

        let TypeRepr::Structure {
            name: type_name,
            members,
            is_external,
        } = struct_type.repr(self.context).clone() else {
            return Err(Box::new(crate::Error::new(
                Some(type_name.span()),
                crate::ErrorKind::NonStructType {
                    type_name: type_name.to_string(),
                },
            )));
        };

        // This was probably already handled, but it can't hurt
        if is_external {
            self.context.use_external(
                struct_type.path(self.context).clone(),
                Constant::Type(struct_type),
            );
        }

        let mut initializer_members = initializer_members.to_vec();
        let mut non_constant_members = Vec::new();
        let mut missing_member_names = Vec::new();

        let constant_members: Vec<Constant> = members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                if let Some(initializer_index) = initializer_members.iter().position(|(name, _)| &member.name == name) {
                    let (_, member_value) = &initializer_members[initializer_index];
                    let member_value = self.generate_local_node(member_value, local_context, Some(member.member_type))?;

                    initializer_members.swap_remove(initializer_index);

                    if let Value::Constant(member_constant) = member_value {
                        Ok(member_constant)
                    }
                    else {
                        let member_value = self.coerce_to_rvalue(member_value, local_context)?;
                        non_constant_members.push((index, member_value));

                        Ok(Constant::Undefined(member.member_type))
                    }
                }
                else {
                    missing_member_names.push(member.name.to_string());

                    Ok(Constant::Undefined(member.member_type))
                }
            })
            .collect::<crate::Result<_>>()?;

        if !missing_member_names.is_empty() {
            return Err(Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::MissingStructMembers {
                    member_names: missing_member_names,
                    type_name: type_name.to_string(),
                },
            )))
        }

        if !initializer_members.is_empty() {
            return Err(Box::new(crate::Error::new(
                Some(span),
                crate::ErrorKind::ExtraStructMembers {
                    member_names: initializer_members
                        .iter()
                        .map(|(name, _)| name.to_string())
                        .collect(),
                    type_name: type_name.to_string(),
                },
            )));
        }

        let structure_pointer_type = self.context.get_pointer_type(struct_type, PointerSemantics::Immutable);
        let structure_pointer = local_context.new_anonymous_register(structure_pointer_type);

        local_context.add_instruction(Instruction::StackAllocate {
            result: structure_pointer.clone(),
        });

        let structure_pointer = Value::Register(structure_pointer);

        // Store the constant members, if any
        if non_constant_members.len() < members.len() {
            local_context.add_instruction(Instruction::Store {
                value: Value::Constant(Constant::Structure {
                    struct_type,
                    members: constant_members,
                }),
                pointer: structure_pointer.clone(),
            });
        }

        for (index, member_value) in non_constant_members {
            let member_pointer_type = self.context.get_pointer_type(member_value.get_type(), PointerSemantics::Mutable);
            let member_pointer = local_context.new_anonymous_register(member_pointer_type);

            local_context.add_instruction(Instruction::GetElementPointer {
                result: member_pointer.clone(),
                pointer: structure_pointer.clone(),
                indices: [
                    Value::from(IntegerValue::new(IntegerType::I32, 0)),
                    Value::from(IntegerValue::new(IntegerType::I32, index as i128))
                ].into(),
            });
            local_context.add_instruction(Instruction::Store {
                value: member_value,
                pointer: member_pointer.into(),
            });
        }

        Ok(Value::Indirect {
            pointer: Box::new(structure_pointer),
            pointee_type: struct_type,
        })
    }

    fn generate_unary_operation(&mut self, operation: ast::UnaryOperation, operand_node: &ast::LocalNode, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let result = match operation {
            ast::UnaryOperation::Positive => {
                let operand = self.generate_local_node(operand_node, local_context, expected_type)?;

                self.coerce_to_rvalue(operand, local_context)?
            }
            ast::UnaryOperation::Negative => {
                let operand = self.generate_local_node(operand_node, local_context, expected_type)?;
                let operand = self.coerce_to_rvalue(operand, local_context)?;
                let result = local_context.new_anonymous_register(expected_type.unwrap_or_else(|| operand.get_type()));

                local_context.add_instruction(Instruction::Negate {
                    result: result.clone(),
                    operand,
                });

                Value::Register(result)
            }
            ast::UnaryOperation::BitwiseNot => {
                let operand = self.generate_local_node(operand_node, local_context, expected_type)?;
                let operand = self.coerce_to_rvalue(operand, local_context)?;
                let result = local_context.new_anonymous_register(expected_type.unwrap_or_else(|| operand.get_type()));

                local_context.add_instruction(Instruction::Not {
                    result: result.clone(),
                    operand,
                });

                Value::Register(result)
            }
            ast::UnaryOperation::LogicalNot => {
                let operand = self.generate_local_node(operand_node, local_context, Some(TypeHandle::BOOL))?;
                let operand = self.coerce_to_rvalue(operand, local_context)?;
                let result = local_context.new_anonymous_register(TypeHandle::BOOL);

                local_context.add_instruction(Instruction::Not {
                    result: result.clone(),
                    operand,
                });

                Value::Register(result)
            }
            ast::UnaryOperation::Reference => {
                let operand = self.generate_local_node(operand_node, local_context, None)?;

                if let Value::Indirect { mut pointer, .. } = operand {
                    pointer.map_pointer_semantics(self.context, |_, semantics| semantics.normalized());
                    *pointer
                }
                else {
                    return Err(Box::new(crate::Error::new(
                        Some(operand_node.span()),
                        crate::ErrorKind::ExpectedLValue,
                    )));
                }
            }
            ast::UnaryOperation::Dereference => {
                let expected_type = expected_type.map(|expected_type| {
                    self.context.get_pointer_type(expected_type, PointerSemantics::Immutable)
                });

                let operand = self.generate_local_node(operand_node, local_context, expected_type)?;
                let operand = self.coerce_to_rvalue(operand, local_context)?;

                if let &TypeRepr::Pointer { pointee_type, .. } = operand.get_type().repr(self.context) {
                    if let Value::Constant(constant) = operand {
                        Value::Constant(Constant::Indirect {
                            pointer: Box::new(constant),
                            pointee_type,
                        })
                    }
                    else {
                        Value::Indirect {
                            pointer: Box::new(operand),
                            pointee_type,
                        }
                    }
                }
                else {
                    return Err(Box::new(crate::Error::new(
                        Some(operand_node.span()),
                        crate::ErrorKind::ExpectedPointer {
                            type_name: operand.get_type().path(self.context).to_string(),
                        },
                    )));
                }
            }
            ast::UnaryOperation::GetSize => {
                let ast::LocalNodeKind::Type(type_node) = operand_node.kind() else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type operand for 'sizeof'");
                };

                let value_type = self.context.interpret_type_node(type_node)?;
                let Some(size) = self.context.type_size(value_type) else {
                    return Err(Box::new(crate::Error::new(
                        Some(type_node.span()),
                        crate::ErrorKind::UnknownTypeSize {
                            type_name: value_type.path(self.context).to_string(),
                        },
                    )));
                };

                Value::from(IntegerValue::new(IntegerType::Usize, size as i128))
            }
            ast::UnaryOperation::GetAlign => {
                let ast::LocalNodeKind::Type(type_node) = operand_node.kind() else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type operand for 'alignof'");
                };

                let value_type = self.context.interpret_type_node(type_node)?;
                let Some(alignment) = self.context.type_alignment(value_type) else {
                    return Err(Box::new(crate::Error::new(
                        Some(type_node.span()),
                        crate::ErrorKind::UnknownTypeAlignment {
                            type_name: value_type.path(self.context).to_string(),
                        },
                    )));
                };

                Value::from(IntegerValue::new(IntegerType::Usize, alignment as i128))
            }
        };

        Ok(result)
    }

    fn generate_binary_operation(&mut self, operation: ast::BinaryOperation, lhs_node: &ast::LocalNode, rhs_node: &ast::LocalNode, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let result = match operation {
            ast::BinaryOperation::Subscript => {
                self.generate_subscript_operation(lhs_node, rhs_node, local_context)?
            }
            ast::BinaryOperation::Access => {
                let lhs = self.generate_local_node(lhs_node, local_context, None)?;

                if let &TypeRepr::Pointer { pointee_type, .. } = lhs.get_type().repr(self.context) {
                    let pointer = self.coerce_to_rvalue(lhs, local_context)?;
                    let structure = Value::Indirect {
                        pointer: Box::new(pointer),
                        pointee_type,
                    };

                    self.generate_member_access(structure, rhs_node, local_context)?
                }
                else {
                    self.generate_member_access(lhs, rhs_node, local_context)?
                }
            }
            ast::BinaryOperation::Convert => {
                let ast::LocalNodeKind::Type(type_node) = rhs_node.kind() else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type rhs for 'as'");
                };

                let value = self.generate_local_node(lhs_node, local_context, None)?;
                let value = self.coerce_to_rvalue(value, local_context)?;

                let target_type = self.context.interpret_type_node(type_node)?;

                self.explicitly_convert(value, target_type, type_node.span(), local_context)?
            }
            ast::BinaryOperation::Add => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Add {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::Subtract => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Subtract {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::Multiply => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Multiply {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::Divide => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Divide {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::Remainder => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Remainder {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::ShiftLeft => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::ShiftLeft {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::ShiftRight => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::ShiftRight {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseAnd => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::And {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseOr => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Or {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseXor => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Xor {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::Equal => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs_node, rhs_node, local_context)?;

                local_context.add_instruction(Instruction::CompareEqual {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::NotEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs_node, rhs_node, local_context)?;

                local_context.add_instruction(Instruction::CompareNotEqual {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::LessThan => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs_node, rhs_node, local_context)?;

                local_context.add_instruction(Instruction::CompareLessThan {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::LessEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs_node, rhs_node, local_context)?;

                local_context.add_instruction(Instruction::CompareLessEqual {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::GreaterThan => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs_node, rhs_node, local_context)?;

                local_context.add_instruction(Instruction::CompareGreaterThan {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::GreaterEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs_node, rhs_node, local_context)?;

                local_context.add_instruction(Instruction::CompareGreaterEqual {
                    result: result.clone(),
                    lhs,
                    rhs,
                });

                Value::Register(result)
            }
            ast::BinaryOperation::LogicalAnd => {
                let lhs = self.generate_local_node(lhs_node, local_context, Some(TypeHandle::BOOL))?;
                let lhs = self.coerce_to_rvalue(lhs, local_context)?;

                let lhs_true_label = local_context.new_block_label();
                let tail_label = local_context.new_block_label();

                local_context.set_terminator(TerminatorInstruction::ConditionalBranch {
                    condition: lhs,
                    consequent_label: lhs_true_label.clone(),
                    alternative_label: tail_label.clone(),
                });
                let short_circuit_label = local_context.start_new_block(lhs_true_label).clone();

                let rhs = self.generate_local_node(rhs_node, local_context, Some(TypeHandle::BOOL))?;
                let rhs = self.coerce_to_rvalue(rhs, local_context)?;

                local_context.set_terminator(TerminatorInstruction::Branch {
                    to_label: tail_label.clone(),
                });
                let rhs_output_label = local_context.start_new_block(tail_label).clone();

                let result = local_context.new_anonymous_register(TypeHandle::BOOL);

                local_context.add_phi(PhiInstruction {
                    result: result.clone(),
                    inputs: [
                        (Value::from(false), short_circuit_label),
                        (rhs, rhs_output_label),
                    ].into(),
                });

                Value::Register(result)
            }
            ast::BinaryOperation::LogicalOr => {
                let lhs = self.generate_local_node(lhs_node, local_context, Some(TypeHandle::BOOL))?;
                let lhs = self.coerce_to_rvalue(lhs, local_context)?;

                let lhs_false_label = local_context.new_block_label();
                let tail_label = local_context.new_block_label();

                local_context.set_terminator(TerminatorInstruction::ConditionalBranch {
                    condition: lhs,
                    consequent_label: tail_label.clone(),
                    alternative_label: lhs_false_label.clone(),
                });
                let short_circuit_label = local_context.start_new_block(lhs_false_label).clone();

                let rhs = self.generate_local_node(rhs_node, local_context, Some(TypeHandle::BOOL))?;
                let rhs = self.coerce_to_rvalue(rhs, local_context)?;

                local_context.set_terminator(TerminatorInstruction::Branch {
                    to_label: tail_label.clone(),
                });
                let rhs_output_label = local_context.start_new_block(tail_label).clone();

                let result = local_context.new_anonymous_register(TypeHandle::BOOL);

                local_context.add_phi(PhiInstruction {
                    result: result.clone(),
                    inputs: [
                        (Value::from(true), short_circuit_label),
                        (rhs, rhs_output_label),
                    ].into(),
                });

                Value::Register(result)
            }
            ast::BinaryOperation::Assign => {
                let lhs = self.generate_local_node(lhs_node, local_context, expected_type)?;
                let (pointer, pointee_type) = lhs.into_mutable_lvalue(lhs_node.span(), self.context)?;
                let rhs = self.generate_local_node(rhs_node, local_context, Some(pointee_type))?;
                let rhs = self.coerce_to_rvalue(rhs, local_context)?;

                local_context.add_instruction(Instruction::Store {
                    value: rhs.clone(),
                    pointer,
                });

                rhs
            }
            ast::BinaryOperation::AddAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Add {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::SubtractAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Subtract {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::MultiplyAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Multiply {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::DivideAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Divide {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::RemainderAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Remainder {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::ShiftLeftAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::ShiftLeft {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::ShiftRightAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::ShiftRight {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::BitwiseAndAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::And {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::BitwiseOrAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Or {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
            ast::BinaryOperation::BitwiseXorAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs_node, rhs_node, local_context, expected_type)?;

                local_context.add_instruction(Instruction::Xor {
                    result: result.clone(),
                    lhs,
                    rhs,
                });
                let result = Value::Register(result);
                local_context.add_instruction(Instruction::Store {
                    value: result.clone(),
                    pointer,
                });

                result
            }
        };

        Ok(result)
    }

    fn generate_subscript_operation(&mut self, lhs_node: &ast::LocalNode, rhs_node: &ast::LocalNode, local_context: &mut LocalContext) -> crate::Result<Value> {
        let lhs = self.generate_local_node(lhs_node, local_context, None)?;
        let rhs = self.generate_local_node(rhs_node, local_context, None)?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let lhs_type = lhs.get_type();
        let rhs_type = rhs.get_type();

        let TypeRepr::Integer { .. } = rhs_type.repr(self.context) else {
            return Err(Box::new(crate::Error::new(
                Some(rhs_node.span()),
                crate::ErrorKind::ExpectedInteger {
                    type_name: rhs_type.path(self.context).to_string(),
                },
            )));
        };

        let cannot_index_error = |context: &GlobalContext| {
            Box::new(crate::Error::new(
                Some(lhs_node.span()),
                crate::ErrorKind::ExpectedArray {
                    type_name: lhs_type.path(context).to_string(),
                },
            ))
        };

        match lhs {
            Value::Indirect { pointer, pointee_type } => match *pointee_type.repr(self.context) {
                TypeRepr::Array { item_type, length } => {
                    // &[T; N], &[T]
                    let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(self.context) else {
                        panic!("indirect value pointer is not a pointer type")
                    };
                    let element_pointer_type = self.context.get_pointer_type(item_type, semantics);
                    let element_pointer = local_context.new_anonymous_register(element_pointer_type);

                    local_context.add_instruction(Instruction::GetElementPointer {
                        result: element_pointer.clone(),
                        pointer: *pointer,
                        indices: match length {
                            Some(..) => [Value::from(IntegerValue::new(IntegerType::I32, 0)), rhs].into(),
                            None => [rhs].into(),
                        },
                    });

                    Ok(Value::Indirect {
                        pointer: Box::new(Value::Register(element_pointer)),
                        pointee_type: item_type,
                    })
                }
                TypeRepr::Pointer { pointee_type: array_type, semantics } => match *array_type.repr(self.context) {
                    TypeRepr::Array { item_type, length } => {
                        // &*[T; N], &*[T]
                        let &TypeRepr::Pointer { semantics: outer_semantics, .. } = pointer.get_type().repr(self.context) else {
                            panic!("indirect value pointer is not a pointer type")
                        };
                        let semantics = match (outer_semantics, semantics) {
                            (PointerSemantics::ImmutableSymbol, PointerSemantics::Mutable) => {
                                PointerSemantics::Mutable
                            },
                            (_, PointerSemantics::Mutable) => outer_semantics,
                            _ => semantics
                        };
                        let array_pointer = local_context.new_anonymous_register(pointee_type);
                        let element_pointer_type = self.context.get_pointer_type(item_type, semantics);
                        let element_pointer = local_context.new_anonymous_register(element_pointer_type);

                        local_context.add_instruction(Instruction::Load {
                            result: array_pointer.clone(),
                            pointer: *pointer,
                        });
                        local_context.add_instruction(Instruction::GetElementPointer {
                            result: element_pointer.clone(),
                            pointer: array_pointer.into(),
                            indices: match length {
                                Some(..) => [Value::from(IntegerValue::new(IntegerType::I32, 0)), rhs].into(),
                                None => [rhs].into(),
                            },
                        });

                        Ok(Value::Indirect {
                            pointer: Box::new(Value::Register(element_pointer)),
                            pointee_type: item_type,
                        })
                    }
                    _ => Err(cannot_index_error(self.context))
                }
                _ => Err(cannot_index_error(self.context))
            }
            Value::Register(register) => match *register.get_type().repr(self.context) {
                TypeRepr::Pointer { pointee_type, semantics } => match *pointee_type.repr(self.context) {
                    TypeRepr::Array { item_type, length } => {
                        // *[T; N], *[T]
                        let element_pointer_type = self.context.get_pointer_type(item_type, semantics);
                        let element_pointer = local_context.new_anonymous_register(element_pointer_type);

                        local_context.add_instruction(Instruction::GetElementPointer {
                            result: element_pointer.clone(),
                            pointer: register.into(),
                            indices: match length {
                                Some(..) => [Value::from(IntegerValue::new(IntegerType::I32, 0)), rhs].into(),
                                None => [rhs].into(),
                            },
                        });

                        Ok(Value::Indirect {
                            pointer: Box::new(Value::Register(element_pointer)),
                            pointee_type: item_type,
                        })
                    }
                    _ => Err(cannot_index_error(self.context))
                }
                _ => Err(cannot_index_error(self.context))
            }
            _ => Err(cannot_index_error(self.context))
        }
    }

    fn fold_subscript_operation(&mut self, lhs_node: &ast::LocalNode, rhs_node: &ast::LocalNode, constant_id: &mut usize, local_context: Option<&LocalContext>) -> crate::Result<(Constant, Vec<GlobalVariable>)> {
        let (lhs, mut intermediate_constants) = self.fold_as_constant(lhs_node, constant_id, local_context, None)?;
        let (rhs, mut constants) = self.fold_as_constant(rhs_node, constant_id, local_context, None)?;
        intermediate_constants.append(&mut constants);

        let lhs_type = lhs.get_type();
        let rhs_type = rhs.get_type();

        let TypeRepr::Integer { .. } = rhs_type.repr(self.context) else {
            return Err(Box::new(crate::Error::new(
                Some(rhs_node.span()),
                crate::ErrorKind::ExpectedInteger {
                    type_name: rhs_type.path(self.context).to_string(),
                },
            )));
        };

        let cannot_index_error = |context: &GlobalContext| {
            Box::new(crate::Error::new(
                Some(lhs_node.span()),
                crate::ErrorKind::ExpectedArray {
                    type_name: lhs_type.path(context).to_string(),
                },
            ))
        };

        let constant = match lhs {
            Constant::Indirect { pointer, pointee_type } => match pointee_type.repr(self.context) {
                &TypeRepr::Array { item_type, length } => {
                    // const &[T; N], const &[T]
                    let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(self.context) else {
                        panic!("bad pointer for indirect");
                    };
                    let indices = match length {
                        Some(_) => vec![Constant::from(IntegerValue::new(IntegerType::I32, 0)), rhs],
                        None => vec![rhs],
                    };

                    let element_pointer = Constant::GetElementPointer {
                        result_type: self.context.get_pointer_type(item_type, semantics),
                        aggregate_type: pointee_type,
                        pointer,
                        indices,
                    };

                    Constant::Indirect {
                        pointer: Box::new(element_pointer),
                        pointee_type: item_type,
                    }
                }
                _ => return Err(cannot_index_error(self.context))
            }
            Constant::Register(register) => match register.get_type().repr(self.context) {
                &TypeRepr::Pointer { pointee_type, semantics } => match pointee_type.repr(self.context) {
                    &TypeRepr::Array { item_type, length } => {
                        // const *[T; N], const *[T]
                        let indices = match length {
                            Some(_) => vec![Constant::from(IntegerValue::new(IntegerType::I32, 0)), rhs],
                            None => vec![rhs],
                        };

                        let element_pointer = Constant::GetElementPointer {
                            result_type: self.context.get_pointer_type(item_type, semantics),
                            aggregate_type: pointee_type,
                            pointer: Box::new(Constant::Register(register)),
                            indices,
                        };

                        Constant::Indirect {
                            pointer: Box::new(element_pointer),
                            pointee_type: item_type,
                        }
                    }
                    _ => return Err(cannot_index_error(self.context))
                }
                _ => return Err(cannot_index_error(self.context))
            }
            _ => return Err(cannot_index_error(self.context))
        };

        Ok((constant, intermediate_constants))
    }

    fn generate_member_access(&mut self, lhs: Value, member_name_node: &ast::LocalNode, local_context: &mut LocalContext) -> crate::Result<Value> {
        let lhs_type = lhs.get_type();

        let cannot_access_error = |context: &GlobalContext| {
            Box::new(crate::Error::new(
                Some(member_name_node.span()),
                crate::ErrorKind::InvalidMemberAccess {
                    type_name: lhs_type.path(context).to_string(),
                },
            ))
        };

        match lhs {
            Value::Indirect { pointer, pointee_type } => match pointee_type.repr(self.context).clone() {
                TypeRepr::Tuple { item_types } => {
                    let item_index = member_name_node.as_tuple_member(item_types.len() as i32)?;
                    let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(self.context) else {
                        panic!("indirect value pointer is not a pointer type")
                    };

                    let item_type = item_types[item_index as usize];
                    let item_pointer_type = self.context.get_pointer_type(item_type, semantics);
                    let item_pointer = local_context.new_anonymous_register(item_pointer_type);

                    local_context.add_instruction(Instruction::GetElementPointer {
                        result: item_pointer.clone(),
                        pointer: *pointer,
                        indices: [
                            Value::from(IntegerValue::new(IntegerType::I32, 0)),
                            Value::from(IntegerValue::new(IntegerType::I32, item_index as i128)),
                        ].into(),
                    });

                    Ok(Value::Indirect {
                        pointer: Box::new(Value::Register(item_pointer)),
                        pointee_type: item_type,
                    })
                }
                TypeRepr::Structure { members, .. } => {
                    let member_name = member_name_node.as_name()?;
                    let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(self.context) else {
                        panic!("indirect value pointer is not a pointer type")
                    };

                    let (member_index, member_type) = members
                        .iter()
                        .enumerate()
                        .find_map(|(index, member)| {
                            (member.name.as_ref() == member_name).then_some((index, member.member_type))
                        })
                        .ok_or_else(|| Box::new(crate::Error::new(
                            Some(member_name_node.span()),
                            crate::ErrorKind::UndefinedMember {
                                member_name: member_name.to_string(),
                                type_name: lhs_type.path(self.context).to_string(),
                            },
                        )))?;
                    let member_pointer_type = self.context.get_pointer_type(member_type, semantics);
                    let member_pointer = local_context.new_anonymous_register(member_pointer_type);

                    local_context.add_instruction(Instruction::GetElementPointer {
                        result: member_pointer.clone(),
                        pointer: *pointer,
                        indices: [
                            Value::from(IntegerValue::new(IntegerType::I32, 0)),
                            Value::from(IntegerValue::new(IntegerType::I32, member_index as i128)),
                        ].into(),
                    });

                    Ok(Value::Indirect {
                        pointer: Box::new(Value::Register(member_pointer)),
                        pointee_type: member_type,
                    })
                }
                _ => Err(cannot_access_error(self.context))
            }
            _ => Err(cannot_access_error(self.context))
        }
    }

    fn generate_arithmetic_operands(&mut self, lhs_node: &ast::LocalNode, rhs_node: &ast::LocalNode, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<(LocalRegister, Value, Value)> {
        let lhs = self.generate_local_node(lhs_node, local_context, expected_type)?;
        let lhs = self.coerce_to_rvalue(lhs, local_context)?;

        let rhs = self.generate_local_node(rhs_node, local_context, Some(lhs.get_type()))?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let result = local_context.new_anonymous_register(expected_type.unwrap_or_else(|| lhs.get_type()));

        Ok((result, lhs, rhs))
    }

    fn generate_comparison_operands(&mut self, lhs_node: &ast::LocalNode, rhs_node: &ast::LocalNode, local_context: &mut LocalContext) -> crate::Result<(LocalRegister, Value, Value)> {
        let lhs = self.generate_local_node(lhs_node, local_context, None)?;
        let lhs = self.coerce_to_rvalue(lhs, local_context)?;

        let rhs = self.generate_local_node(rhs_node, local_context, Some(lhs.get_type()))?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let result = local_context.new_anonymous_register(TypeHandle::BOOL);

        Ok((result, lhs, rhs))
    }

    fn generate_assignment_operands(&mut self, lhs_node: &ast::LocalNode, rhs_node: &ast::LocalNode, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<(LocalRegister, Value, Value, Value)> {
        let lhs = self.generate_local_node(lhs_node, local_context, expected_type)?;
        let (pointer, pointee_type) = lhs.into_mutable_lvalue(lhs_node.span(), self.context)?;

        let rhs = self.generate_local_node(rhs_node, local_context, Some(pointee_type))?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let lhs = local_context.new_anonymous_register(pointee_type);
        let result = local_context.new_anonymous_register(pointee_type);

        local_context.add_instruction(Instruction::Load {
            result: lhs.clone(),
            pointer: pointer.clone(),
        });

        Ok((result, pointer, Value::Register(lhs), rhs))
    }

    fn generate_call_operation(&mut self, callee_node: &ast::LocalNode, arguments: &[ast::LocalNode], local_context: &mut LocalContext) -> crate::Result<Value> {
        // Determine which kind of call operation this is
        let callee = match callee_node.kind() {
            // Method call operation in the format `value.method(..)`
            ast::LocalNodeKind::Binary { operation: ast::BinaryOperation::Access, lhs, rhs } => {
                let method_name = rhs.as_name()?;
                let self_value = self.generate_local_node(lhs, local_context, None)?;

                // If lhs is a pointer, perform an implicit dereference (this is also done before
                // member accesses)
                let self_value = match self_value.get_type().repr(self.context) {
                    &TypeRepr::Pointer { pointee_type, .. } => {
                        let pointer = self.coerce_to_rvalue(self_value, local_context)?;
                        Value::Indirect {
                            pointer: Box::new(pointer),
                            pointee_type,
                        }
                    }
                    _ => self_value
                };

                // Search in the type's implementation namespace for a matching method
                let lhs_namespace = self.context.type_namespace(self_value.get_type());
                if let Ok(value) = self.context.get_symbol_value(lhs_namespace, method_name, Some(&rhs.span())) {
                    // A method was found, so bind lhs as self and use it as the callee
                    Value::BoundFunction {
                        self_value: Box::new((lhs.span(), self_value)),
                        function_value: Box::new(value.clone()),
                    }
                }
                else {
                    return Err(Box::new(crate::Error::new(
                        Some(rhs.span()),
                        crate::ErrorKind::NoSuchMethod {
                            type_name: self_value.get_type().path(self.context).to_string(),
                            method_name: method_name.to_string(),
                        },
                    )));
                }
            }
            // Normal call operation
            _ => {
                let callee = self.generate_local_node(callee_node, local_context, None)?;
                self.coerce_to_rvalue(callee, local_context)?
            }
        };

        // Ensure the callee is, in fact, a function that can be called
        let TypeRepr::Function { signature } = callee.get_type().repr(self.context).clone() else {
            return Err(Box::new(crate::Error::new(
                Some(callee_node.span()),
                crate::ErrorKind::ExpectedFunction {
                    type_name: callee.get_type().path(self.context).to_string(),
                },
            )));
        };

        let mut argument_values = Vec::new();

        // Ensure that when arguments and parameter formats are zipped, all arguments are generated
        // This is important for variadic arguments, which don't have corresponding parameters
        let mut parameters_iter = signature.parameter_types().iter()
            .map(|&parameter_type| Some(parameter_type))
            .chain(std::iter::repeat(None));

        if let Value::BoundFunction { self_value, .. } = &callee {
            let (self_span, self_value) = self_value.as_ref();
            // This is a method call, so we need to match the self value to the first parameter
            let Some(self_parameter_type) = parameters_iter.next().unwrap() else {
                return Err(Box::new(crate::Error::new(
                    Some(*self_span),
                    crate::ErrorKind::ExpectedSelfParameter,
                )));
            };

            // Make an effort to convert the bound self value to the parameter type
            let self_argument = match self_parameter_type.repr(self.context) {
                TypeRepr::Pointer { .. } => match self_value {
                    Value::Indirect { pointer, .. } => {
                        pointer.as_ref().clone()
                    }
                    _ => {
                        // Allocate temporary space on the stack for the value so it can be pointed to
                        let self_pointer_type = self.context.get_pointer_type(self_value.get_type(), PointerSemantics::Immutable);
                        let self_pointer = local_context.new_anonymous_register(self_pointer_type);

                        local_context.add_instruction(Instruction::StackAllocate {
                            result: self_pointer.clone(),
                        });
                        let self_value_pointer = Value::Register(self_pointer);
                        local_context.add_instruction(Instruction::Store {
                            value: self_value.clone(),
                            pointer: self_value_pointer.clone(),
                        });

                        self_value_pointer
                    }
                }
                _ => {
                    self.coerce_to_rvalue(self_value.clone(), local_context)?
                }
            };

            let self_argument = self.enforce_type(self_argument, self_parameter_type, *self_span, local_context)?;

            // Pass the bound 'self' value as the first argument
            argument_values.push(self_argument);
        }

        // Generate each argument for the call operation in order
        for (argument, parameter_type) in arguments.iter().zip(parameters_iter) {
            let argument = self.generate_local_node(argument, local_context, parameter_type)?;
            let argument = self.coerce_to_rvalue(argument, local_context)?;

            argument_values.push(argument);
        }

        // Ensure the number of arguments is correct
        let expected_count = signature.parameter_types().len();
        let got_count = argument_values.len();
        if (!signature.is_variadic() && got_count > expected_count) || got_count < expected_count {
            return Err(Box::new(crate::Error::new(
                Some(callee_node.span()),
                crate::ErrorKind::WrongFunctionArgumentCount {
                    expected_count,
                    got_count,
                },
            )));
        }

        // Generate the function call itself, which will look different depending on return type
        if signature.return_type() == TypeHandle::NEVER {
            local_context.add_instruction(Instruction::Call {
                result: None,
                callee,
                arguments: argument_values.into_boxed_slice(),
            });
            local_context.set_terminator(TerminatorInstruction::Unreachable);

            Ok(Value::Never)
        }
        else if signature.return_type() == TypeHandle::VOID {
            local_context.add_instruction(Instruction::Call {
                result: None,
                callee,
                arguments: argument_values.into_boxed_slice(),
            });

            Ok(Value::Void)
        }
        else {
            let result = local_context.new_anonymous_register(signature.return_type());

            local_context.add_instruction(Instruction::Call {
                result: Some(result.clone()),
                callee,
                arguments: argument_values.into_boxed_slice(),
            });

            Ok(Value::Register(result))
        }
    }

    fn generate_conditional(&mut self, condition: &ast::LocalNode, consequent: &ast::LocalNode, alternative: Option<&ast::LocalNode>, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let condition = self.generate_local_node(condition, local_context, Some(TypeHandle::BOOL))?;
        let condition = self.coerce_to_rvalue(condition, local_context)?;

        let consequent_label = local_context.new_block_label();
        let alternative_label = local_context.new_block_label();

        local_context.set_terminator(TerminatorInstruction::ConditionalBranch {
            condition,
            consequent_label: consequent_label.clone(),
            alternative_label: alternative_label.clone(),
        });

        if let Some(alternative) = alternative {
            let mut tail_label = None;

            local_context.start_new_block(consequent_label);

            let consequent_value = self.generate_local_node(consequent, local_context, expected_type)?;
            let consequent_type = consequent_value.get_type();
            if consequent_type != TypeHandle::NEVER {
                let tail_label = tail_label.get_or_insert_with(|| local_context.new_block_label());
                local_context.set_terminator(TerminatorInstruction::Branch {
                    to_label: tail_label.clone(),
                });
            }

            let expected_type = expected_type.or_else(|| {
                (consequent_type != TypeHandle::NEVER).then_some(consequent_type)
            });

            let consequent_end_label = local_context.start_new_block(alternative_label).clone();

            let alternative_value = self.generate_local_node(alternative, local_context, expected_type)?;
            let alternative_type = alternative_value.get_type();
            if alternative_type != TypeHandle::NEVER {
                let tail_label = tail_label.get_or_insert_with(|| local_context.new_block_label());
                local_context.set_terminator(TerminatorInstruction::Branch {
                    to_label: tail_label.clone(),
                });
            }

            let result_type = expected_type.unwrap_or(alternative_type);

            if let Some(tail_label) = tail_label {
                let alternative_end_label = local_context.start_new_block(tail_label).clone();

                if result_type == TypeHandle::VOID {
                    Ok(Value::Void)
                }
                else {
                    let result = local_context.new_anonymous_register(result_type);

                    local_context.add_phi(PhiInstruction {
                        result: result.clone(),
                        inputs: [
                            (consequent_value, consequent_end_label),
                            (alternative_value, alternative_end_label),
                        ].into(),
                    });

                    Ok(Value::Register(result))
                }
            }
            else {
                Ok(Value::Never)
            }
        }
        else {
            local_context.start_new_block(consequent_label);

            let consequent_value = self.generate_local_node(consequent, local_context, Some(TypeHandle::VOID))?;
            if consequent_value.get_type() != TypeHandle::NEVER {
                local_context.set_terminator(TerminatorInstruction::Branch {
                    to_label: alternative_label.clone(),
                });
            }

            local_context.start_new_block(alternative_label);

            Ok(Value::Void)
        }
    }

    fn generate_while_loop(&mut self, condition: &ast::LocalNode, body: &ast::LocalNode, local_context: &mut LocalContext) -> crate::Result<Value> {
        // TODO: handling never, break/continue vs. return
        let condition_label = local_context.new_block_label();

        local_context.set_terminator(TerminatorInstruction::Branch {
            to_label: condition_label.clone(),
        });

        local_context.start_new_block(condition_label.clone());

        let condition = self.generate_local_node(condition, local_context, Some(TypeHandle::BOOL))?;
        let condition = self.coerce_to_rvalue(condition, local_context)?;

        let body_label = local_context.new_block_label();
        let tail_label = local_context.new_block_label();

        local_context.push_break_label(tail_label.clone());
        local_context.push_continue_label(condition_label.clone());

        local_context.set_terminator(TerminatorInstruction::ConditionalBranch {
            condition,
            consequent_label: body_label.clone(),
            alternative_label: tail_label.clone(),
        });

        local_context.start_new_block(body_label);

        let body_value = self.generate_local_node(body, local_context, Some(TypeHandle::VOID))?;
        if body_value.get_type() != TypeHandle::NEVER {
            local_context.set_terminator(TerminatorInstruction::Branch {
                to_label: condition_label.clone(),
            });
        }

        local_context.start_new_block(tail_label);

        local_context.pop_break_label();
        local_context.pop_continue_label();

        Ok(Value::Void)
    }

    fn generate_local_let_statement(&mut self, span: crate::Span, name: &str, type_node: Option<&ast::TypeNode>, is_mutable: bool, value: Option<&ast::LocalNode>, local_context: &mut LocalContext) -> crate::Result<Value> {
        let value_type = match type_node {
            Some(type_node) => {
                Some(self.context.interpret_type_node(type_node)?)
            }
            None => None,
        };

        let value = match value {
            Some(node) => {
                let value = self.generate_local_node(node, local_context, value_type)?;
                Some(self.coerce_to_rvalue(value, local_context)?)
            }
            None => None,
        };

        let value_type = match value_type {
            Some(value_type) => value_type,
            None => {
                let Some(value) = &value else {
                    return Err(Box::new(crate::Error::new(
                        Some(span),
                        crate::ErrorKind::MustSpecifyTypeForUninitialized {
                            name: name.to_string(),
                        },
                    )));
                };
                value.get_type()
            }
        };

        let semantics = PointerSemantics::for_symbol(is_mutable);
        let pointer_type = self.context.get_pointer_type(value_type, semantics);
        let pointer = local_context.define_indirect_symbol(name.into(), pointer_type, value_type);

        local_context.add_instruction(Instruction::StackAllocate {
            result: pointer.clone(),
        });
        if let Some(value) = value {
            local_context.add_instruction(Instruction::Store {
                value,
                pointer: pointer.into(),
            });
        }

        Ok(Value::Void)
    }

    fn generate_global_let_statement(&mut self, value: &ast::LocalNode, global_register: &GlobalRegister) -> crate::Result<Value> {
        // The fill phase has done most of the work for us already
        let TypeRepr::Pointer { pointee_type, semantics } = *self.context.type_repr(global_register.get_type()) else {
            panic!("invalid global value register type");
        };

        let value = self.generate_constant_node(value, None, Some(pointee_type))?;

        self.context.package_mut().output_mut().add_global_variable(GlobalVariable::new(
            global_register.clone(),
            match semantics {
                PointerSemantics::Immutable |
                PointerSemantics::ImmutableSymbol => GlobalVariableKind::Constant,
                PointerSemantics::Mutable => GlobalVariableKind::Mutable,
            },
            value,
        ));

        Ok(Value::Void)
    }

    fn generate_function_definition(&mut self, name: &str, parameters: &[ast::FunctionParameterNode], body: &ast::LocalNode, function_register: &GlobalRegister) -> crate::Result<Value> {
        // The fill phase has done a lot of the initial work for us already
        let TypeRepr::Function { signature } = self.context.type_repr(function_register.get_type()) else {
            panic!("invalid global value register type");
        };
        let signature = signature.clone();

        let mut local_context = LocalContext::new(
            FunctionDefinition::new(
                function_register.clone(),
                signature.return_type(),
                signature.is_variadic(),
            ),
            self.context.current_namespace_info().path().child(name),
        );

        for (parameter, &parameter_type) in std::iter::zip(parameters, signature.parameter_types()) {
            let parameter_register = local_context.new_anonymous_register(parameter_type);
            local_context.function_mut().add_parameter_register(parameter_register.clone());

            let semantics = PointerSemantics::for_symbol(parameter.is_mutable);
            let pointer_type = self.context.get_pointer_type(parameter_type, semantics);
            let pointer = local_context.define_indirect_symbol(parameter.name.clone(), pointer_type, parameter_type);

            local_context.add_instruction(Instruction::StackAllocate {
                result: pointer.clone(),
            });
            local_context.add_instruction(Instruction::Store {
                value: parameter_register.into(),
                pointer: pointer.into(),
            });
        }

        let body_result = self.generate_local_node(body, &mut local_context, Some(signature.return_type()))?;

        // Insert a return instruction if necessary
        if body_result.get_type() != TypeHandle::NEVER {
            local_context.set_terminator(TerminatorInstruction::Return {
                value: body_result,
            });
        }

        self.context.package_mut().output_mut().add_function_definition(local_context.finish());

        Ok(Value::Void)
    }

    fn generate_structure_definition(&mut self, self_type: TypeHandle) -> crate::Result<Value> {
        // The fill phase has done basically all of the work for us already
        self.context.package_mut().output_mut().add_type_declaration(self_type);

        Ok(Value::Void)
    }

    fn generate_implement_block(&mut self, self_type: &ast::TypeNode, statements: &[ast::GlobalNode]) -> crate::Result<Value> {
        let self_type = self.context.interpret_type_node(self_type)?;

        self.context.set_self_type(self_type);

        for statement in statements {
            self.generate_global_statement(statement)?;
        }

        self.context.unset_self_type();

        Ok(Value::Void)
    }

    fn generate_module_block(&mut self, statements: &[ast::GlobalNode], namespace: NamespaceHandle) -> crate::Result<Value> {
        let parent_module = self.context.replace_current_module(namespace);

        for statement in statements {
            self.generate_global_statement(statement)?;
        }

        self.context.replace_current_module(parent_module);

        Ok(Value::Void)
    }

    pub fn generate_constant_node(&mut self, node: &ast::LocalNode, local_context: Option<&LocalContext>, expected_type: Option<TypeHandle>) -> crate::Result<Constant> {
        let mut constant_id = self.next_anonymous_constant_id;
        let (constant, intermediate_constants) = self.fold_as_constant(node, &mut constant_id,  local_context, expected_type)?;
        self.next_anonymous_constant_id = constant_id;

        for intermediate_constant in intermediate_constants {
            self.context.package_mut().output_mut().add_global_variable(intermediate_constant);
        }

        Ok(constant)
    }

    pub fn fold_as_constant(&mut self, node: &ast::LocalNode, constant_id: &mut usize, local_context: Option<&LocalContext>, expected_type: Option<TypeHandle>) -> crate::Result<(Constant, Vec<GlobalVariable>)> {
        let mut new_intermediate_constant = |constant: Constant, context: &mut GlobalContext| {
            let pointer = GlobalRegister::new(
                format!(".const.{}.{constant_id}", context.package().info().name()).as_bytes().into(),
                context.get_pointer_type(constant.get_type(), PointerSemantics::Immutable),
            );
            *constant_id += 1;
            GlobalVariable::new(
                pointer,
                GlobalVariableKind::AnonymousConstant,
                constant,
            )
        };

        let mut intermediate_constants = Vec::new();

        let constant = match node.kind() {
            ast::LocalNodeKind::Literal(literal) => {
                match *literal {
                    token::Literal::Name(ref name) => {
                        let value = if let Some(value) = local_context.and_then(|ctx| ctx.find_symbol(name)) {
                            value.clone()
                        }
                        else {
                            self.context.get_symbol_value(self.context.current_module(), name, Some(&node.span()))
                                .map_err(|mut error| {
                                    if let crate::ErrorKind::UndefinedGlobalSymbol { .. } = error.kind() {
                                        *error.kind_mut() = crate::ErrorKind::UndefinedSymbol {
                                            name: name.to_string(),
                                        };
                                    }
                                    error
                                })?
                        };

                        if let Value::Constant(constant) = value {
                            constant
                        }
                        else {
                            return Err(Box::new(crate::Error::new(
                                Some(node.span()),
                                crate::ErrorKind::NonConstantSymbol {
                                    name: name.to_string(),
                                },
                            )));
                        }
                    }
                    token::Literal::Integer(value, suffix) => {
                        let value_type = match suffix {
                            Some(suffix) => suffix.as_handle(),
                            None => expected_type.unwrap_or(TypeHandle::I32),
                        };
                        let Some(value) = IntegerValue::from_unknown_type(value, value_type, self.context.target()) else {
                            return Err(Box::new(crate::Error::new(
                                Some(node.span()),
                                crate::ErrorKind::IncompatibleValueType {
                                    value: value.to_string(),
                                    type_name: value_type.path(self.context).to_string(),
                                },
                            )));
                        };

                        Constant::from(value)
                    }
                    token::Literal::Float(value, suffix) => {
                        let value_type = match suffix {
                            Some(suffix) => suffix.as_handle(),
                            None => expected_type.unwrap_or(TypeHandle::F64),
                        };
                        let Some(value) = FloatValue::from_unknown_type(value, value_type, self.context.target()) else {
                            return Err(Box::new(crate::Error::new(
                                Some(node.span()),
                                crate::ErrorKind::IncompatibleValueType {
                                    value: value.to_string(),
                                    type_name: value_type.path(self.context).to_string(),
                                },
                            )));
                        };

                        Constant::from(value)
                    }
                    token::Literal::Boolean(value) => {
                        Constant::from(value)
                    }
                    token::Literal::NullPointer => {
                        Constant::NullPointer(expected_type.unwrap_or_else(|| {
                            self.context.get_pointer_type(TypeHandle::VOID, PointerSemantics::Immutable)
                        }))
                    }
                    token::Literal::String(ref value) => {
                        let constant = Constant::String {
                            array_type: self.context.get_array_type(TypeHandle::U8, Some(value.len() as u64)),
                            value: value.clone(),
                        };
                        let intermediate_constant = new_intermediate_constant(constant, self.context);
                        let pointer = intermediate_constant.register().clone();
                        intermediate_constants.push(intermediate_constant);

                        Constant::Register(pointer)
                    }
                    token::Literal::PrimitiveType(primitive_type) => {
                        Constant::Type(primitive_type.handle)
                    }
                }
            }
            ast::LocalNodeKind::Path { segments } => {
                let path = self.context.get_absolute_path(node.span(), segments)?;
                match self.context.get_path_value(&path, Some(&node.span()))? {
                    Value::Constant(constant) => constant,
                    _ => return Err(Box::new(crate::Error::new(
                        Some(node.span()),
                        crate::ErrorKind::NonConstantSymbol {
                            name: path.to_string(),
                        },
                    )))
                }
            }
            ast::LocalNodeKind::ArrayLiteral { items } => {
                if let Some(expected_type) = expected_type {
                    let &TypeRepr::Array { item_type, .. } = expected_type.repr(self.context) else {
                        return Err(Box::new(crate::Error::new(
                            Some(node.span()),
                            crate::ErrorKind::UnknownArrayType,
                        )));
                    };
                    let items: Vec<Constant> = items
                        .iter()
                        .map(|item| {
                            let (item, mut constants) = self.fold_as_constant(item, constant_id, local_context, Some(item_type))?;

                            intermediate_constants.append(&mut constants);
                            Ok(item)
                        })
                        .collect::<crate::Result<_>>()?;

                    Constant::Array {
                        array_type: expected_type,
                        items,
                    }
                }
                else {
                    // TODO
                    return Err(Box::new(crate::Error::new(
                        Some(node.span()),
                        crate::ErrorKind::UnknownArrayType,
                    )));
                }
            }
            ast::LocalNodeKind::StructureLiteral { structure_type, members: initializer_members } => {
                let struct_type = self.context.interpret_node_as_type(structure_type)?;

                let TypeRepr::Structure {
                    name: type_name,
                    members,
                    is_external,
                } = struct_type.repr(self.context).clone() else {
                    return Err(Box::new(crate::Error::new(
                        Some(structure_type.span()),
                        crate::ErrorKind::NonStructType {
                            type_name: structure_type.to_string(),
                        },
                    )));
                };

                // This was probably already handled, but it can't hurt
                if is_external {
                    self.context.use_external(
                        struct_type.path(self.context).clone(),
                        Constant::Type(struct_type),
                    );
                }

                let mut initializer_members = initializer_members.to_vec();
                let mut missing_member_names = Vec::new();

                let members: Vec<Constant> = members
                    .iter()
                    .map(|member| {
                        if let Some(initializer_index) = initializer_members.iter().position(|(name, _)| &member.name == name) {
                            let (_, member_value) = &initializer_members[initializer_index];
                            let (member_value, mut constants) = self.fold_as_constant(member_value, constant_id, local_context, Some(member.member_type))?;

                            intermediate_constants.append(&mut constants);
                            initializer_members.swap_remove(initializer_index);

                            Ok(member_value)
                        }
                        else {
                            missing_member_names.push(member.name.to_string());

                            Ok(Constant::Undefined(member.member_type))
                        }
                    })
                    .collect::<crate::Result<_>>()?;

                if !missing_member_names.is_empty() {
                    return Err(Box::new(crate::Error::new(
                        Some(node.span()),
                        crate::ErrorKind::MissingStructMembers {
                            member_names: missing_member_names,
                            type_name: type_name.to_string(),
                        },
                    )))
                }

                if !initializer_members.is_empty() {
                    return Err(Box::new(crate::Error::new(
                        Some(node.span()),
                        crate::ErrorKind::ExtraStructMembers {
                            member_names: initializer_members
                                .iter()
                                .map(|(name, _)| name.to_string())
                                .collect(),
                            type_name: type_name.to_string(),
                        },
                    )));
                }

                Constant::Structure {
                    members,
                    struct_type,
                }
            }
            ast::LocalNodeKind::Binary { operation, lhs, rhs } => match operation {
                ast::BinaryOperation::Subscript => {
                    let (value, mut constants) = self.fold_subscript_operation(lhs, rhs, constant_id, local_context)?;
                    intermediate_constants.append(&mut constants);

                    value
                }
                ast::BinaryOperation::Convert => {
                    let ast::LocalNodeKind::Type(type_node) = rhs.kind() else {
                        // If parsing rules are followed, this should not occur
                        panic!("non-type rhs for 'as'");
                    };

                    let (value, mut constants) = self.fold_as_constant(lhs, constant_id, local_context, None)?;
                    intermediate_constants.append(&mut constants);

                    let target_type = self.context.interpret_type_node(type_node)?;

                    match value {
                        Constant::Integer(integer) => {
                            let converted_integer = IntegerValue::from_unknown_type(integer.raw(), target_type, self.context.target())
                                .ok_or_else(|| Box::new(crate::Error::new(
                                    Some(type_node.span()),
                                    crate::ErrorKind::InconvertibleTypes {
                                        from_type: integer.integer_type().as_handle().path(self.context).to_string(),
                                        to_type: target_type.path(self.context).to_string(),
                                    },
                                )))?;

                            Constant::Integer(converted_integer)
                        }
                        Constant::Float(float) => {
                            let converted_float = FloatValue::from_unknown_type(float.raw(), target_type, self.context.target())
                                .ok_or_else(|| Box::new(crate::Error::new(
                                    Some(type_node.span()),
                                    crate::ErrorKind::InconvertibleTypes {
                                        from_type: float.float_type().as_handle().path(self.context).to_string(),
                                        to_type: target_type.path(self.context).to_string(),
                                    },
                                )))?;

                            Constant::Float(converted_float)
                        }
                        _ => {
                            return Err(Box::new(crate::Error::new(
                                Some(type_node.span()),
                                crate::ErrorKind::InconvertibleTypes {
                                    from_type: value.get_type().path(self.context).to_string(),
                                    to_type: target_type.path(self.context).to_string(),
                                },
                            )));
                        }
                    }
                }
                _ => {
                    return Err(Box::new(crate::Error::new(
                        Some(node.span()),
                        crate::ErrorKind::UnsupportedConstantExpression,
                    )));
                }
            }
            ast::LocalNodeKind::Grouping { content } => {
                // Fine to bypass validation steps since this is literally just parentheses
                return self.fold_as_constant(content, constant_id, local_context, expected_type);
            }
            _ => {
                return Err(Box::new(crate::Error::new(
                    Some(node.span()),
                    crate::ErrorKind::UnsupportedConstantExpression,
                )));
            }
        };

        if let Some(expected_type) = expected_type {
            self.enforce_constant_type(constant, expected_type, node.span())
                .map(|constant| (constant, intermediate_constants))
        }
        else {
            Ok((constant, intermediate_constants))
        }
    }
}
