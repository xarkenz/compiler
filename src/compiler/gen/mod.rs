pub mod llvm;

use llvm::*;
use crate::token;
use crate::ast;
use crate::sema::*;

use std::io::Write;

pub struct Generator<W: Write> {
    emitter: Emitter<W>,
    context: GlobalContext,
    next_anonymous_constant_id: usize,
}

impl Generator<std::fs::File> {
    pub fn from_filename(filename: String, context: GlobalContext) -> crate::Result<Self> {
        Emitter::from_filename(filename)
            .map(|emitter| Self::new(emitter, context))
    }
}

impl<W: Write> Generator<W> {
    pub fn new(emitter: Emitter<W>, context: GlobalContext) -> Self {
        Self {
            emitter,
            context,
            next_anonymous_constant_id: 0,
        }
    }

    pub fn emitter(&self) -> &Emitter<W> {
        &self.emitter
    }

    pub fn emitter_mut(&mut self) -> &mut Emitter<W> {
        &mut self.emitter
    }

    pub fn context(&self) -> &GlobalContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut GlobalContext {
        &mut self.context
    }

    pub fn generate_all<'a>(
        mut self,
        top_level_statements: impl IntoIterator<Item = &'a ast::Node>,
        file_id: usize,
        filenames: &[String],
    ) -> crate::Result<()> {
        self.emitter.emit_preamble(file_id, &filenames[file_id])?;

        for top_level_statement in top_level_statements {
            self.generate_global_node(top_level_statement)?;
        }

        self.emitter.emit_postamble()
    }

    pub fn generate_global_node(&mut self, node: &ast::Node) -> crate::Result<Value> {
        match node {
            ast::Node::Let { value, global_register, .. } => {
                let global_register = global_register.as_ref().expect("register should be valid after fill phase");
                self.generate_global_let_statement(value.as_deref(), global_register)
            }
            ast::Node::Constant { value, global_register, .. } => {
                let global_register = global_register.as_ref().expect("register should be valid after fill phase");
                self.generate_global_constant_statement(value, global_register)
            }
            ast::Node::Function { name, parameters, body, global_register, .. } => {
                let global_register = global_register.as_ref().expect("register should be valid after fill phase");
                if let Some(body) = body {
                    self.generate_function_definition(name, parameters, body, global_register)
                }
                else {
                    self.generate_function_declaration(global_register)
                }
            }
            ast::Node::Structure { members, self_type, .. } => {
                if members.is_some() {
                    self.generate_structure_definition(*self_type)
                }
                else {
                    self.generate_opaque_structure_definition(*self_type)
                }
            }
            ast::Node::Implement { self_type, statements } => {
                self.generate_implement_block(self_type, statements)
            }
            ast::Node::Module { statements, namespace, .. } => {
                self.generate_module_block(statements, *namespace)
            }
            ast::Node::Import { .. } | ast::Node::GlobImport { .. } => {
                // The fill phase has done all of the work for us already
                Ok(Value::Void)
            }
            _ => {
                Err(Box::new(crate::Error::UnexpectedExpression {}))
            }
        }
    }

    pub fn generate_local_node(&mut self, node: &ast::Node, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        if let Ok(constant) = self.generate_constant_node(node, Some(local_context), expected_type) {
            return Ok(Value::Constant(constant));
        }

        let result = match node {
            ast::Node::Literal(literal) => {
                self.generate_literal(literal, local_context, expected_type)?
            }
            ast::Node::Path { segments } => {
                let path = self.context.get_absolute_path(segments)?;
                self.context.get_path_value(&path)?
            }
            ast::Node::Unary { operation, operand } => {
                self.generate_unary_operation(*operation, operand, local_context, expected_type)?
            }
            ast::Node::Binary { operation, lhs, rhs } => {
                self.generate_binary_operation(*operation, lhs, rhs, local_context, expected_type)?
            }
            ast::Node::Call { callee, arguments } => {
                self.generate_call_operation(callee, arguments, local_context)?
            }
            ast::Node::ArrayLiteral { items } => {
                self.generate_array_literal(items, local_context, expected_type)?
            }
            ast::Node::StructureLiteral { structure_type: type_name, members } => {
                self.generate_structure_literal(type_name, members, local_context)?
            }
            ast::Node::Grouping { content } => {
                // Fine to bypass validation steps since this is literally just parentheses
                return self.generate_local_node(content, local_context, expected_type);
            }
            ast::Node::Scope { statements, tail } => {
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
            ast::Node::Conditional { condition, consequent, alternative } => {
                self.generate_conditional(condition, consequent, alternative.as_deref(), local_context, expected_type)?
            }
            ast::Node::While { condition, body } => {
                self.generate_while_loop(condition, body, local_context)?
            }
            ast::Node::Break => {
                let break_label = local_context.break_label()
                    .ok_or_else(|| Box::new(crate::Error::InvalidBreak {}))?;

                self.emitter.emit_unconditional_branch(break_label)?;

                Value::Break
            }
            ast::Node::Continue => {
                let continue_label = local_context.continue_label()
                    .ok_or_else(|| Box::new(crate::Error::InvalidContinue {}))?;

                self.emitter.emit_unconditional_branch(continue_label)?;

                Value::Continue
            }
            ast::Node::Return { value } => {
                let return_type = local_context.return_type();

                if let Some(value) = value {
                    if return_type == TypeHandle::VOID {
                        return Err(Box::new(crate::Error::UnexpectedReturnValue {
                            function_name: local_context.function_path().to_string(),
                        }));
                    }
                    else {
                        let value = self.generate_local_node(value, local_context, Some(return_type))?;
                        let value = self.coerce_to_rvalue(value, local_context)?;

                        self.emitter.emit_return(&value, &self.context)?;
                    }
                }
                else if return_type != TypeHandle::VOID {
                    return Err(Box::new(crate::Error::ExpectedReturnValue {
                        function_name: local_context.function_path().to_string(),
                    }));
                }
                else {
                    self.emitter.emit_return(&Value::Void, &self.context)?;
                }

                Value::Never
            }
            ast::Node::Let { name, value_type, is_mutable, value, .. } => {
                self.generate_local_let_statement(name, value_type.as_ref(), *is_mutable, value.as_deref(), local_context)?
            }
            ast::Node::Constant { name, value_type, value, .. } => {
                self.generate_local_constant_statement(name, value_type, value, local_context)?
            }
            _ => {
                return Err(Box::new(crate::Error::UnexpectedExpression {}));
            }
        };

        if let Some(expected_type) = expected_type {
            self.enforce_type(result, expected_type, local_context)
                // For debugging purposes. This information is often useful
                .inspect_err(|_| println!("node: {node:?}"))
        }
        else {
            Ok(result)
        }
    }

    pub fn new_anonymous_constant(&mut self, pointer_type: TypeHandle) -> Register {
        let id = self.next_anonymous_constant_id;
        self.next_anonymous_constant_id += 1;

        Register::new_global(format!(".const.{id}"), pointer_type)
    }

    pub fn start_new_block(&mut self, label: &Label, local_context: &mut LocalContext) -> crate::Result<()> {
        local_context.set_current_block_label(label.clone());
        self.emitter.emit_label(label)
    }

    pub fn enforce_type(&mut self, value: Value, target_type: TypeHandle, local_context: &mut LocalContext) -> crate::Result<Value> {
        let got_type = value.get_type();

        if got_type == TypeHandle::NEVER || self.context.types_are_equivalent(got_type, target_type) {
            Ok(value)
        }
        // TODO: should from_mutable be true here?
        else if self.context.can_coerce_type(got_type, target_type, true) {
            if let Value::Constant(constant) = value {
                Ok(Value::Constant(Constant::BitwiseCast {
                    value: Box::new(constant),
                    result_type: target_type,
                }))
            }
            else {
                let value = self.coerce_to_rvalue(value, local_context)?;
                let result = local_context.new_anonymous_register(target_type);
                self.emitter.emit_bitwise_cast(&result, &value, &self.context)?;

                Ok(Value::Register(result))
            }
        }
        else {
            Err(Box::new(crate::Error::IncompatibleTypes {
                expected_type: target_type.path(self.context()).to_string(),
                got_type: got_type.path(self.context()).to_string(),
            }))
        }
    }

    pub fn enforce_constant_type(&mut self, constant: Constant, target_type: TypeHandle) -> crate::Result<Constant> {
        let got_type = constant.get_type();

        if self.context.types_are_equivalent(got_type, target_type) {
            Ok(constant)
        }
        // TODO: should from_mutable be true here?
        else if self.context.can_coerce_type(got_type, target_type, true) {
            Ok(Constant::BitwiseCast {
                value: Box::new(constant),
                result_type: target_type,
            })
        }
        else {
            Err(Box::new(crate::Error::IncompatibleTypes {
                expected_type: target_type.path(self.context()).to_string(),
                got_type: got_type.path(self.context()).to_string(),
            }))
        }
    }

    pub fn change_type(&mut self, value: Value, target_type: TypeHandle, local_context: &mut LocalContext) -> crate::Result<Value> {
        let original_type = value.get_type();

        if self.context.types_are_equivalent(original_type, target_type) {
            Ok(value)
        }
        else {
            // TODO: pointer to int, int to pointer
            match (original_type.repr(&self.context), target_type.repr(&self.context)) {
                (
                    TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
                    TypeRepr::Pointer { .. } | TypeRepr::Function { .. },
                ) => {
                    let result = local_context.new_anonymous_register(target_type);
                    self.emitter.emit_bitwise_cast(&result, &value, &self.context)?;

                    Ok(Value::Register(result))
                }
                (
                    TypeRepr::Integer { size: from_size, .. },
                    TypeRepr::Integer { size: to_size, .. },
                ) => {
                    match to_size.cmp(from_size) {
                        std::cmp::Ordering::Greater => {
                            let result = local_context.new_anonymous_register(target_type);
                            self.emitter.emit_extension(&result, &value, &self.context)?;

                            Ok(Value::Register(result))
                        }
                        std::cmp::Ordering::Less => {
                            let result = local_context.new_anonymous_register(target_type);
                            self.emitter.emit_truncation(&result, &value, &self.context)?;

                            Ok(Value::Register(result))
                        }
                        std::cmp::Ordering::Equal => {
                            Ok(value)
                        }
                    }
                }
                (
                    original_repr @ TypeRepr::Integer { .. },
                    TypeRepr::Boolean,
                ) => {
                    let result = local_context.new_anonymous_register(target_type);
                    let zero = IntegerValue::new(0, original_repr).unwrap();
                    self.emitter.emit_cmp_not_equal(&result, &value, &zero.into(), &self.context)?;

                    Ok(Value::Register(result))
                }
                (
                    TypeRepr::Boolean,
                    TypeRepr::Integer { .. },
                ) => {
                    let result = local_context.new_anonymous_register(target_type);
                    self.emitter.emit_zero_extension(&result, &value, &self.context)?;

                    Ok(Value::Register(result))
                }
                _ => {
                    Err(Box::new(crate::Error::InconvertibleTypes {
                        original_type: target_type.path(self.context()).to_string(),
                        target_type: original_type.path(self.context()).to_string(),
                    }))
                }
            }
        }
    }

    pub fn coerce_to_rvalue(&mut self, value: Value, local_context: &mut LocalContext) -> crate::Result<Value> {
        let (pointer, mut pointee_type) = match value {
            Value::Indirect { pointer, pointee_type } => (*pointer, pointee_type),
            Value::Constant(Constant::Indirect { pointer, pointee_type }) => ((*pointer).into(), pointee_type),
            value => return Ok(value)
        };

        // If the pointee is `*mut _`, copy the semantics from the pointer to the pointee
        if let &TypeRepr::Pointer {
            pointee_type: next_pointee_type,
            semantics: PointerSemantics::Mutable,
        } = pointee_type.repr(&self.context) {
            let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(&self.context) else {
                panic!("indirect value pointer is not a pointer type")
            };
            pointee_type = self.context.get_pointer_type(next_pointee_type, semantics);
        }

        let result = local_context.new_anonymous_register(pointee_type);
        self.emitter.emit_load(&result, &pointer, &self.context)?;

        Ok(Value::Register(result))
    }

    fn generate_literal(&mut self, literal: &token::Literal, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let result = match *literal {
            token::Literal::Name(ref name) => {
                if let Some(value) = local_context.find_symbol(name) {
                    value.clone()
                }
                else {
                    self.context.get_symbol_value(self.context.current_module(), name)
                        .map_err(|mut error| {
                            if let crate::Error::UndefinedGlobalSymbol { .. } = error.as_ref() {
                                *error = crate::Error::UndefinedSymbol {
                                    name: name.clone(),
                                };
                            }
                            error
                        })?
                }
            }
            token::Literal::Integer(value) => {
                let value_type = expected_type.unwrap_or(TypeHandle::I32);
                let Some(value) = IntegerValue::new(value, value_type.repr(&self.context)) else {
                    return Err(Box::new(crate::Error::IncompatibleValueType {
                        value: value.to_string(),
                        type_name: value_type.path(&self.context).to_string(),
                    }));
                };
                Value::Constant(Constant::Integer(value))
            }
            token::Literal::Boolean(value) => {
                Value::Constant(Constant::Boolean(value))
            }
            token::Literal::NullPointer => {
                Value::Constant(Constant::NullPointer(expected_type.unwrap_or_else(|| {
                    self.context.get_pointer_type(TypeHandle::VOID, PointerSemantics::Immutable)
                })))
            }
            token::Literal::String(ref value) => {
                let constant = Constant::String {
                    array_type: self.context.get_array_type(TypeHandle::U8, Some(value.len())),
                    value: value.clone(),
                };
                let pointer = self.new_anonymous_constant(constant.get_type());

                self.emitter.emit_anonymous_constant(&pointer, &constant, &self.context)?;

                Value::Constant(Constant::Register(pointer))
            }
            token::Literal::PrimitiveType(primitive_type) => {
                Value::Type(primitive_type.handle)
            }
        };

        Ok(result)
    }

    fn generate_array_literal(&mut self, items: &[Box<ast::Node>], local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let Some(array_type) = expected_type else {
            return Err(Box::new(crate::Error::UnknownArrayType {}));
        };
        let &TypeRepr::Array { item_type, .. } = array_type.repr(&self.context) else {
            return Err(Box::new(crate::Error::UnknownArrayType {}));
        };

        let mut non_constant_items = Vec::new();

        let constant_items: Vec<Constant> = crate::Result::from_iter(items.iter().enumerate().map(|(index, item)| {
            let item_value = self.generate_local_node(item, local_context, Some(item_type))?;

            if let Value::Constant(item_constant) = item_value {
                Ok(item_constant)
            }
            else {
                let item_value = self.coerce_to_rvalue(item_value, local_context)?;
                non_constant_items.push((index, item_value));

                Ok(Constant::Undefined(item_type))
            }
        }))?;

        let array_pointer_type = self.context.get_pointer_type(array_type, PointerSemantics::Immutable);
        let array_pointer = local_context.new_anonymous_register(array_pointer_type);
        let initial_value = Value::Constant(Constant::Array {
            array_type,
            items: constant_items,
        });

        self.emitter.emit_local_allocation(&array_pointer, &self.context)?;

        let array_pointer = Value::Register(array_pointer);

        self.emitter.emit_store(&initial_value, &array_pointer, &self.context)?;

        for (index, item) in non_constant_items {
            let item_pointer_type = self.context.get_pointer_type(item.get_type(), PointerSemantics::Mutable);
            let item_pointer = local_context.new_anonymous_register(item_pointer_type);
            let zero = Value::from(IntegerValue::Signed32(0));
            let index = Value::from(IntegerValue::Unsigned64(index as u64));

            self.emitter.emit_get_element_pointer(&item_pointer, &array_pointer, &[zero, index], &self.context)?;
            self.emitter.emit_store(&item, &item_pointer.into(), &self.context)?;
        }

        Ok(Value::Indirect {
            pointer: Box::new(array_pointer),
            pointee_type: array_type,
        })
    }

    fn generate_structure_literal(&mut self, type_name: &ast::Node, initializer_members: &[(String, Box<ast::Node>)], local_context: &mut LocalContext) -> crate::Result<Value> {
        let struct_type = self.context.interpret_node_as_type(type_name)?;

        let TypeRepr::Structure { name: type_name, members } = struct_type.repr(&self.context).clone() else {
            return Err(Box::new(crate::Error::NonStructType { type_name: type_name.to_string() }));
        };

        let mut initializer_members = initializer_members.to_vec();
        let mut non_constant_members = Vec::new();
        let mut missing_member_names = Vec::new();

        let constant_members: Vec<Constant> = crate::Result::from_iter(members.iter().enumerate().map(|(index, member)| {
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
                missing_member_names.push(member.name.clone());

                Ok(Constant::Undefined(member.member_type))
            }
        }))?;

        if !missing_member_names.is_empty() {
            return Err(Box::new(crate::Error::MissingStructMembers {
                member_names: missing_member_names,
                type_name: type_name.clone(),
            }))
        }

        if !initializer_members.is_empty() {
            let member_names = Vec::from_iter(initializer_members.iter().map(|(name, _)| name.clone()));

            return Err(Box::new(crate::Error::ExtraStructMembers {
                member_names,
                type_name: type_name.clone(),
            }));
        }

        let structure_pointer_type = self.context.get_pointer_type(struct_type, PointerSemantics::Immutable);
        let structure_pointer = local_context.new_anonymous_register(structure_pointer_type);
        let initial_value = Value::Constant(Constant::Structure {
            struct_type,
            members: constant_members,
        });

        self.emitter.emit_local_allocation(&structure_pointer, &self.context)?;

        let structure_pointer = Value::Register(structure_pointer);

        self.emitter.emit_store(&initial_value, &structure_pointer, &self.context)?;

        for (index, member) in non_constant_members {
            let member_pointer_type = self.context.get_pointer_type(member.get_type(), PointerSemantics::Mutable);
            let member_pointer = local_context.new_anonymous_register(member_pointer_type);
            let zero = Value::from(IntegerValue::Signed32(0));
            let index = Value::from(IntegerValue::Signed32(index as i32));

            self.emitter.emit_get_element_pointer(&member_pointer, &structure_pointer, &[zero, index], &self.context)?;
            self.emitter.emit_store(&member, &member_pointer.into(), &self.context)?;
        }

        Ok(Value::Indirect {
            pointer: Box::new(structure_pointer),
            pointee_type: struct_type,
        })
    }

    fn generate_unary_operation(&mut self, operation: ast::UnaryOperation, operand: &ast::Node, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let result = match operation {
            ast::UnaryOperation::Positive => {
                let operand = self.generate_local_node(operand, local_context, expected_type)?;

                self.coerce_to_rvalue(operand, local_context)?
            }
            ast::UnaryOperation::Negative => {
                let operand = self.generate_local_node(operand, local_context, expected_type)?;
                let operand = self.coerce_to_rvalue(operand, local_context)?;
                let result = local_context.new_anonymous_register(expected_type.unwrap_or_else(|| operand.get_type()));

                self.emitter.emit_negation(&result, &operand, &self.context)?;

                Value::Register(result)
            }
            ast::UnaryOperation::BitwiseNot => {
                let operand = self.generate_local_node(operand, local_context, expected_type)?;
                let operand = self.coerce_to_rvalue(operand, local_context)?;
                let result = local_context.new_anonymous_register(expected_type.unwrap_or_else(|| operand.get_type()));

                self.emitter.emit_inversion(&result, &operand, &self.context)?;

                Value::Register(result)
            }
            ast::UnaryOperation::LogicalNot => {
                let operand = self.generate_local_node(operand, local_context, Some(TypeHandle::BOOL))?;
                let operand = self.coerce_to_rvalue(operand, local_context)?;
                let result = local_context.new_anonymous_register(TypeHandle::BOOL);

                self.emitter.emit_inversion(&result, &operand, &self.context)?;

                Value::Register(result)
            }
            ast::UnaryOperation::Reference => {
                let operand = self.generate_local_node(operand, local_context, None)?;

                if let Value::Indirect { pointer, .. } = operand {
                    *pointer
                }
                else {
                    return Err(Box::new(crate::Error::ExpectedLValue {}));
                }
            }
            ast::UnaryOperation::Dereference => {
                let expected_type = expected_type.map(|expected_type| {
                    self.context.get_pointer_type(expected_type, PointerSemantics::Immutable)
                });

                let operand = self.generate_local_node(operand, local_context, expected_type)?;
                let operand = self.coerce_to_rvalue(operand, local_context)?;

                if let &TypeRepr::Pointer { pointee_type, .. } = operand.get_type().repr(&self.context) {
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
                    return Err(Box::new(crate::Error::ExpectedPointer {
                        type_name: operand.get_type().path(&self.context).to_string(),
                    }));
                }
            }
            ast::UnaryOperation::GetSize => {
                let ast::Node::Type(type_node) = operand else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type operand for 'sizeof'");
                };

                let value_type = self.context.interpret_type_node(type_node)?;
                let Some(size) = self.context.type_size(value_type) else {
                    return Err(Box::new(crate::Error::UnknownTypeSize {
                        type_name: value_type.path(&self.context).to_string(),
                    }));
                };

                Value::Constant(Constant::Integer(IntegerValue::Unsigned64(size as u64)))
            }
            ast::UnaryOperation::GetAlign => {
                let ast::Node::Type(type_node) = operand else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type operand for 'alignof'");
                };

                let value_type = self.context.interpret_type_node(type_node)?;
                let Some(alignment) = self.context.type_alignment(value_type) else {
                    return Err(Box::new(crate::Error::UnknownTypeSize {
                        type_name: value_type.path(&self.context).to_string(),
                    }));
                };

                Value::Constant(Constant::Integer(IntegerValue::Unsigned64(alignment as u64)))
            }
        };

        Ok(result)
    }

    fn generate_binary_operation(&mut self, operation: ast::BinaryOperation, lhs: &ast::Node, rhs: &ast::Node, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let result = match operation {
            ast::BinaryOperation::Subscript => {
                self.generate_subscript_operation(lhs, rhs, local_context)?
            }
            ast::BinaryOperation::Access => {
                let lhs = self.generate_local_node(lhs, local_context, None)?;

                if let &TypeRepr::Pointer { pointee_type, .. } = lhs.get_type().repr(&self.context) {
                    let pointer = self.coerce_to_rvalue(lhs, local_context)?;
                    let structure = Value::Indirect {
                        pointer: Box::new(pointer),
                        pointee_type,
                    };

                    self.generate_member_access(structure, rhs, local_context)?
                }
                else {
                    self.generate_member_access(lhs, rhs, local_context)?
                }
            }
            ast::BinaryOperation::Convert => {
                let ast::Node::Type(type_node) = rhs else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type rhs for 'as'");
                };

                let value = self.generate_local_node(lhs, local_context, None)?;
                let value = self.coerce_to_rvalue(value, local_context)?;

                let target_type = self.context.interpret_type_node(type_node)?;

                self.change_type(value, target_type, local_context)?
            }
            ast::BinaryOperation::Add => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_addition(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Subtract => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_subtraction(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Multiply => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_multiplication(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Divide => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_division(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Remainder => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_remainder(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::ShiftLeft => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_shift_left(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::ShiftRight => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_shift_right(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseAnd => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_bitwise_and(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseOr => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_bitwise_or(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseXor => {
                let (result, lhs, rhs) = self.generate_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_bitwise_xor(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Equal => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, local_context)?;

                self.emitter.emit_cmp_equal(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::NotEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, local_context)?;

                self.emitter.emit_cmp_not_equal(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::LessThan => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, local_context)?;

                self.emitter.emit_cmp_less_than(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::LessEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, local_context)?;

                self.emitter.emit_cmp_less_equal(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::GreaterThan => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, local_context)?;

                self.emitter.emit_cmp_greater_than(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::GreaterEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, local_context)?;

                self.emitter.emit_cmp_greater_equal(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::LogicalAnd => {
                let lhs = self.generate_local_node(lhs, local_context, Some(TypeHandle::BOOL))?;
                let lhs = self.coerce_to_rvalue(lhs, local_context)?;

                let lhs_true_label = local_context.new_block_label();
                let tail_label = local_context.new_block_label();

                self.emitter.emit_conditional_branch(&lhs, &lhs_true_label, &tail_label, &self.context)?;
                let short_circuit_label = local_context.current_block_label().clone();
                self.start_new_block(&lhs_true_label, local_context)?;

                let rhs = self.generate_local_node(rhs, local_context, Some(TypeHandle::BOOL))?;
                let rhs = self.coerce_to_rvalue(rhs, local_context)?;

                self.emitter.emit_unconditional_branch(&tail_label)?;
                let rhs_output_label = local_context.current_block_label().clone();
                self.start_new_block(&tail_label, local_context)?;

                let result = local_context.new_anonymous_register(TypeHandle::BOOL);

                self.emitter.emit_phi(&result, [
                    (&Value::from(false), &short_circuit_label),
                    (&rhs, &rhs_output_label),
                ], &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::LogicalOr => {
                let lhs = self.generate_local_node(lhs, local_context, Some(TypeHandle::BOOL))?;
                let lhs = self.coerce_to_rvalue(lhs, local_context)?;

                let lhs_false_label = local_context.new_block_label();
                let tail_label = local_context.new_block_label();

                self.emitter.emit_conditional_branch(&lhs, &tail_label, &lhs_false_label, &self.context)?;
                let short_circuit_label = local_context.current_block_label().clone();
                self.start_new_block(&lhs_false_label, local_context)?;

                let rhs = self.generate_local_node(rhs, local_context, Some(TypeHandle::BOOL))?;
                let rhs = self.coerce_to_rvalue(rhs, local_context)?;

                self.emitter.emit_unconditional_branch(&tail_label)?;
                let rhs_output_label = local_context.current_block_label().clone();
                self.start_new_block(&tail_label, local_context)?;

                let result = local_context.new_anonymous_register(TypeHandle::BOOL);

                self.emitter.emit_phi(&result, [
                    (&Value::from(true), &short_circuit_label),
                    (&rhs, &rhs_output_label),
                ], &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Assign => {
                let lhs = self.generate_local_node(lhs, local_context, expected_type)?;
                let (pointer, pointee_type) = lhs.into_mutable_lvalue(&self.context)?;
                let rhs = self.generate_local_node(rhs, local_context, Some(pointee_type))?;
                let rhs = self.coerce_to_rvalue(rhs, local_context)?;

                self.emitter.emit_store(&rhs, &pointer, &self.context)?;

                rhs
            }
            ast::BinaryOperation::MultiplyAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_multiplication(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::DivideAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_division(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::RemainderAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_remainder(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::AddAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_addition(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::SubtractAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_subtraction(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::ShiftLeftAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_shift_left(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::ShiftRightAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_shift_right(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::BitwiseAndAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_bitwise_and(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::BitwiseXorAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_bitwise_xor(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
            ast::BinaryOperation::BitwiseOrAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_bitwise_or(&result, &lhs, &rhs, &self.context)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer, &self.context)?;

                result
            }
        };

        Ok(result)
    }

    fn generate_subscript_operation(&mut self, lhs: &ast::Node, rhs: &ast::Node, local_context: &mut LocalContext) -> crate::Result<Value> {
        let lhs = self.generate_local_node(lhs, local_context, None)?;
        let rhs = self.generate_local_node(rhs, local_context, None)?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let lhs_type = lhs.get_type();
        let rhs_type = rhs.get_type();

        let TypeRepr::Integer { .. } = rhs_type.repr(&self.context) else {
            return Err(Box::new(crate::Error::ExpectedInteger {
                type_name: rhs_type.path(&self.context).to_string(),
            }));
        };

        let cannot_index_error = |context: &GlobalContext| {
            Box::new(crate::Error::ExpectedArray {
                type_name: lhs_type.path(context).to_string(),
            })
        };

        match lhs {
            Value::Indirect { pointer, pointee_type } => match *pointee_type.repr(&self.context) {
                TypeRepr::Array { item_type, length } => {
                    // &[T; N], &[T]
                    let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(&self.context) else {
                        panic!("indirect value pointer is not a pointer type")
                    };
                    let element_pointer_type = self.context.get_pointer_type(item_type, semantics);
                    let element_pointer = local_context.new_anonymous_register(element_pointer_type);
                    let indices = match length {
                        Some(..) => vec![Value::from(IntegerValue::Signed32(0)), rhs],
                        None => vec![rhs],
                    };

                    self.emitter.emit_get_element_pointer(&element_pointer, &pointer, &indices, &self.context)?;

                    Ok(Value::Indirect {
                        pointer: Box::new(Value::Register(element_pointer)),
                        pointee_type: item_type,
                    })
                }
                TypeRepr::Pointer { pointee_type: array_type, semantics } => match *array_type.repr(&self.context) {
                    TypeRepr::Array { item_type, length } => {
                        // &*[T; N], &*[T]
                        let &TypeRepr::Pointer { semantics: outer_semantics, .. } = pointer.get_type().repr(&self.context) else {
                            panic!("indirect value pointer is not a pointer type")
                        };
                        let semantics = match semantics {
                            PointerSemantics::Mutable => outer_semantics,
                            _ => semantics
                        };
                        let array_pointer = local_context.new_anonymous_register(pointee_type);
                        let element_pointer_type = self.context.get_pointer_type(item_type, semantics);
                        let element_pointer = local_context.new_anonymous_register(element_pointer_type);
                        let indices = match length {
                            Some(..) => vec![Value::from(IntegerValue::Signed32(0)), rhs],
                            None => vec![rhs],
                        };

                        self.emitter.emit_load(&array_pointer, &pointer, &self.context)?;
                        self.emitter.emit_get_element_pointer(&element_pointer, &array_pointer.into(), &indices, &self.context)?;

                        Ok(Value::Indirect {
                            pointer: Box::new(Value::Register(element_pointer)),
                            pointee_type: item_type,
                        })
                    }
                    _ => Err(cannot_index_error(&self.context))
                }
                _ => Err(cannot_index_error(&self.context))
            }
            Value::Register(register) => match *register.get_type().repr(&self.context) {
                TypeRepr::Pointer { pointee_type, semantics } => match *pointee_type.repr(&self.context) {
                    TypeRepr::Array { item_type, length } => {
                        // *[T; N], *[T]
                        let element_pointer_type = self.context.get_pointer_type(item_type, semantics);
                        let element_pointer = local_context.new_anonymous_register(element_pointer_type);
                        let indices = match length {
                            Some(_) => vec![Value::from(IntegerValue::Signed32(0)), rhs],
                            None => vec![rhs],
                        };

                        self.emitter.emit_get_element_pointer(&element_pointer, &register.into(), &indices, &self.context)?;

                        Ok(Value::Indirect {
                            pointer: Box::new(Value::Register(element_pointer)),
                            pointee_type: item_type,
                        })
                    }
                    _ => Err(cannot_index_error(&self.context))
                }
                _ => Err(cannot_index_error(&self.context))
            }
            _ => Err(cannot_index_error(&self.context))
        }
    }

    fn fold_subscript_operation(&mut self, lhs: &ast::Node, rhs: &ast::Node, constant_id: &mut usize, local_context: Option<&LocalContext>) -> crate::Result<(Constant, Vec<(Register, Constant)>)> {
        let (lhs, mut intermediate_constants) = self.fold_as_constant(lhs, constant_id, local_context, None)?;
        let (rhs, mut constants) = self.fold_as_constant(rhs, constant_id, local_context, None)?;
        intermediate_constants.append(&mut constants);

        let lhs_type = lhs.get_type();
        let rhs_type = rhs.get_type();

        let TypeRepr::Integer { .. } = rhs_type.repr(&self.context) else {
            return Err(Box::new(crate::Error::ExpectedInteger {
                type_name: rhs_type.path(&self.context).to_string(),
            }));
        };

        let cannot_index_error = |context: &GlobalContext| {
            Box::new(crate::Error::ExpectedArray {
                type_name: lhs_type.path(context).to_string(),
            })
        };

        let constant = match lhs {
            Constant::Indirect { pointer, pointee_type } => match pointee_type.repr(&self.context) {
                &TypeRepr::Array { item_type, length } => {
                    // const &[T; N], const &[T]
                    let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(&self.context) else {
                        panic!("bad pointer for indirect");
                    };
                    let indices = match length {
                        Some(_) => vec![Constant::from(IntegerValue::Signed32(0)), rhs],
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
                _ => return Err(cannot_index_error(&self.context))
            }
            Constant::Register(register) => match register.get_type().repr(&self.context) {
                &TypeRepr::Pointer { pointee_type, semantics } => match pointee_type.repr(&self.context) {
                    &TypeRepr::Array { item_type, length } => {
                        // const *[T; N], const *[T]
                        let indices = match length {
                            Some(_) => vec![Constant::from(IntegerValue::Signed32(0)), rhs],
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
                    _ => return Err(cannot_index_error(&self.context))
                }
                _ => return Err(cannot_index_error(&self.context))
            }
            _ => return Err(cannot_index_error(&self.context))
        };

        Ok((constant, intermediate_constants))
    }

    fn generate_member_access(&mut self, lhs: Value, member_name: &ast::Node, local_context: &mut LocalContext) -> crate::Result<Value> {
        let lhs_type = lhs.get_type();
        let member_name = member_name.as_name()?;

        let cannot_access_error = |context: &GlobalContext| {
            Box::new(crate::Error::ExpectedStruct {
                type_name: lhs_type.path(context).to_string(),
            })
        };

        match lhs {
            Value::Indirect { pointer, pointee_type } => match pointee_type.repr(&self.context).clone() {
                TypeRepr::Structure { members, .. } => {
                    let &TypeRepr::Pointer { semantics, .. } = pointer.get_type().repr(&self.context) else {
                        panic!("indirect value pointer is not a pointer type")
                    };

                    let Some(member_index) = members.iter().position(|member| member.name == member_name) else {
                        return Err(Box::new(crate::Error::UndefinedStructMember {
                            member_name: member_name.to_string(),
                            type_name: lhs_type.path(&self.context).to_string(),
                        }));
                    };
                    let member_type = members[member_index].member_type;
                    let member_pointer_type = self.context.get_pointer_type(member_type, semantics);
                    let member_pointer = local_context.new_anonymous_register(member_pointer_type);
                    let indices = &[
                        Value::from(IntegerValue::Signed32(0)),
                        Value::from(IntegerValue::Signed32(member_index as i32)),
                    ];

                    self.emitter.emit_get_element_pointer(&member_pointer, &pointer, indices, &self.context)?;

                    Ok(Value::Indirect {
                        pointer: Box::new(Value::Register(member_pointer)),
                        pointee_type: member_type,
                    })
                }
                _ => Err(cannot_access_error(&self.context))
            }
            _ => Err(cannot_access_error(&self.context))
        }
    }

    fn generate_arithmetic_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_local_node(lhs, local_context, expected_type)?;
        let lhs = self.coerce_to_rvalue(lhs, local_context)?;

        let rhs = self.generate_local_node(rhs, local_context, Some(lhs.get_type()))?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let result = local_context.new_anonymous_register(expected_type.unwrap_or_else(|| lhs.get_type()));

        Ok((result, lhs, rhs))
    }

    fn generate_comparison_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, local_context: &mut LocalContext) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_local_node(lhs, local_context, None)?;
        let lhs = self.coerce_to_rvalue(lhs, local_context)?;

        let rhs = self.generate_local_node(rhs, local_context, Some(lhs.get_type()))?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let result = local_context.new_anonymous_register(TypeHandle::BOOL);

        Ok((result, lhs, rhs))
    }

    fn generate_assignment_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<(Register, Value, Value, Value)> {
        let lhs = self.generate_local_node(lhs, local_context, expected_type)?;
        let (pointer, pointee_type) = lhs.into_mutable_lvalue(&self.context)?;

        let rhs = self.generate_local_node(rhs, local_context, Some(pointee_type))?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let lhs = local_context.new_anonymous_register(pointee_type);
        let result = local_context.new_anonymous_register(pointee_type);

        self.emitter.emit_load(&lhs, &pointer, &self.context)?;

        Ok((result, pointer, Value::Register(lhs), rhs))
    }

    fn generate_call_operation(&mut self, callee: &ast::Node, arguments: &[Box<ast::Node>], local_context: &mut LocalContext) -> crate::Result<Value> {
        // Determine which kind of call operation this is
        let callee = match callee {
            // Method call operation in the format `value.method(..)`
            ast::Node::Binary { operation: ast::BinaryOperation::Access, lhs, rhs } => {
                let method_name = rhs.as_name()?;
                let lhs = self.generate_local_node(lhs, local_context, None)?;

                // If lhs is a pointer, perform an implicit dereference (this is also done before
                // member accesses)
                let lhs = match lhs.get_type().repr(&self.context) {
                    &TypeRepr::Pointer { pointee_type, .. } => {
                        let pointer = self.coerce_to_rvalue(lhs, local_context)?;
                        Value::Indirect {
                            pointer: Box::new(pointer),
                            pointee_type,
                        }
                    }
                    _ => lhs
                };

                // Search in the type's implementation namespace for a matching method
                let lhs_namespace = self.context.type_namespace(lhs.get_type());
                if let Some(Symbol::Value(value)) = self.context.namespace_info(lhs_namespace).find(method_name) {
                    // A method was found, so bind lhs as self and use it as the callee
                    Value::BoundFunction {
                        self_value: Box::new(lhs),
                        function_value: Box::new(value.clone()),
                    }
                }
                else {
                    return Err(Box::new(crate::Error::NoSuchMethod {
                        type_name: lhs.get_type().path(&self.context).to_string(),
                        method_name: method_name.to_string(),
                    }));
                }
            }
            // Normal call operation
            _ => {
                let callee = self.generate_local_node(callee, local_context, None)?;
                self.coerce_to_rvalue(callee, local_context)?
            }
        };

        // Ensure the callee is, in fact, a function that can be called
        let TypeRepr::Function { signature } = callee.get_type().repr(&self.context).clone() else {
            return Err(Box::new(crate::Error::ExpectedFunction {
                type_name: callee.get_type().path(&self.context).to_string(),
            }));
        };

        let mut argument_values = Vec::new();

        // Ensure that when arguments and parameter formats are zipped, all arguments are generated
        // This is important for variadic arguments, which don't have corresponding parameters
        let mut parameters_iter = signature.parameter_types().iter()
            .map(|&parameter_type| Some(parameter_type))
            .chain(std::iter::repeat(None));

        if let Value::BoundFunction { self_value, .. } = &callee {
            // This is a method call, and so we need to match the self value to the first parameter
            let Some(self_parameter_type) = parameters_iter.next().unwrap() else {
                return Err(Box::new(crate::Error::ExpectedSelfParameter {}));
            };

            // Make an effort to convert the bound self value to the parameter type
            let self_argument = match self_parameter_type.repr(&self.context) {
                TypeRepr::Pointer { .. } => match self_value.as_ref() {
                    Value::Indirect { pointer, .. } => {
                        pointer.as_ref().clone()
                    }
                    _ => {
                        // Allocate temporary space on the stack for the value so it can be pointed to
                        let self_pointer_type = self.context.get_pointer_type(self_value.get_type(), PointerSemantics::Immutable);
                        let self_pointer = local_context.new_anonymous_register(self_pointer_type);

                        self.emitter.emit_local_allocation(&self_pointer, &self.context)?;
                        let self_value_pointer = Value::Register(self_pointer);
                        self.emitter.emit_store(self_value, &self_value_pointer, &self.context)?;

                        self_value_pointer
                    }
                }
                _ => {
                    self.coerce_to_rvalue(self_value.as_ref().clone(), local_context)?
                }
            };

            let self_argument = self.enforce_type(self_argument, self_parameter_type, local_context)?;

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
        if !signature.is_variadic() && got_count > expected_count {
            return Err(Box::new(crate::Error::ExtraFunctionArguments { expected_count, got_count }));
        }
        else if got_count < expected_count {
            return Err(Box::new(crate::Error::MissingFunctionArguments { expected_count, got_count }));
        }

        // Generate the function call itself, which will look different depending on return type
        if signature.return_type() == TypeHandle::NEVER {
            self.emitter.emit_function_call(None, &callee, &argument_values, &self.context)?;
            self.emitter.emit_unreachable()?;

            Ok(Value::Never)
        }
        else if signature.return_type() == TypeHandle::VOID {
            self.emitter.emit_function_call(None, &callee, &argument_values, &self.context)?;

            Ok(Value::Void)
        }
        else {
            let result = local_context.new_anonymous_register(signature.return_type());

            self.emitter.emit_function_call(Some(&result), &callee, &argument_values, &self.context)?;

            Ok(Value::Register(result))
        }
    }

    fn generate_conditional(&mut self, condition: &ast::Node, consequent: &ast::Node, alternative: Option<&ast::Node>, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let condition = self.generate_local_node(condition, local_context, Some(TypeHandle::BOOL))?;
        let condition = self.coerce_to_rvalue(condition, local_context)?;

        let consequent_label = local_context.new_block_label();
        let alternative_label = local_context.new_block_label();

        self.emitter.emit_conditional_branch(&condition, &consequent_label, &alternative_label, &self.context)?;

        if let Some(alternative) = alternative {
            let mut tail_label = None;

            self.start_new_block(&consequent_label, local_context)?;
            let consequent_value = self.generate_local_node(consequent, local_context, expected_type)?;
            let consequent_type = consequent_value.get_type();
            if consequent_type != TypeHandle::NEVER {
                let tail_label = tail_label.get_or_insert_with(|| local_context.new_block_label());
                self.emitter.emit_unconditional_branch(tail_label)?;
            }
            let consequent_end_label = local_context.current_block_label().clone();

            let expected_type = expected_type.or_else(|| {
                (consequent_type != TypeHandle::NEVER).then_some(consequent_type)
            });

            self.start_new_block(&alternative_label, local_context)?;
            let alternative_value = self.generate_local_node(alternative, local_context, expected_type)?;
            let alternative_type = alternative_value.get_type();
            if alternative_type != TypeHandle::NEVER {
                let tail_label = tail_label.get_or_insert_with(|| local_context.new_block_label());
                self.emitter.emit_unconditional_branch(tail_label)?;
            }
            let alternative_end_label = local_context.current_block_label().clone();

            let result_type = expected_type.unwrap_or(alternative_type);

            if let Some(tail_label) = tail_label {
                self.start_new_block(&tail_label, local_context)?;

                if result_type == TypeHandle::VOID {
                    Ok(Value::Void)
                }
                else {
                    let result = local_context.new_anonymous_register(result_type);

                    self.emitter.emit_phi(&result, [
                        (&consequent_value, &consequent_end_label),
                        (&alternative_value, &alternative_end_label),
                    ], &self.context)?;

                    Ok(Value::Register(result))
                }
            }
            else {
                Ok(Value::Never)
            }
        }
        else {
            self.start_new_block(&consequent_label, local_context)?;
            let consequent_value = self.generate_local_node(consequent, local_context, Some(TypeHandle::VOID))?;
            if consequent_value.get_type() != TypeHandle::NEVER {
                self.emitter.emit_unconditional_branch(&alternative_label)?;
            }

            self.start_new_block(&alternative_label, local_context)?;

            Ok(Value::Void)
        }
    }

    fn generate_while_loop(&mut self, condition: &ast::Node, body: &ast::Node, local_context: &mut LocalContext) -> crate::Result<Value> {
        // TODO: handling never, break/continue vs. return
        let condition_label = local_context.new_block_label();

        self.emitter.emit_unconditional_branch(&condition_label)?;

        self.start_new_block(&condition_label, local_context)?;

        let condition = self.generate_local_node(condition, local_context, Some(TypeHandle::BOOL))?;
        let condition = self.coerce_to_rvalue(condition, local_context)?;

        let body_label = local_context.new_block_label();
        let tail_label = local_context.new_block_label();

        local_context.push_break_label(tail_label.clone());
        local_context.push_continue_label(condition_label.clone());

        self.emitter.emit_conditional_branch(&condition, &body_label, &tail_label, &self.context)?;

        self.start_new_block(&body_label, local_context)?;
        self.generate_local_node(body, local_context, Some(TypeHandle::VOID))?;
        self.emitter.emit_unconditional_branch(&condition_label)?;

        self.start_new_block(&tail_label, local_context)?;

        local_context.pop_break_label();
        local_context.pop_continue_label();

        Ok(Value::Void)
    }

    fn generate_local_let_statement(&mut self, name: &str, type_node: Option<&ast::TypeNode>, is_mutable: bool, value: Option<&ast::Node>, local_context: &mut LocalContext) -> crate::Result<Value> {
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
                    return Err(Box::new(crate::Error::MustSpecifyTypeForUninitialized {
                        name: name.to_string(),
                    }));
                };
                value.get_type()
            }
        };

        let semantics = PointerSemantics::from_flag(is_mutable);
        let pointer_type = self.context.get_pointer_type(value_type, semantics);
        let pointer = local_context.define_indirect_symbol(name.into(), pointer_type, value_type);

        self.emitter.emit_local_allocation(&pointer, &self.context)?;
        if let Some(value) = &value {
            self.emitter.emit_store(value, &pointer.into(), &self.context)?;
        }

        Ok(Value::Void)
    }

    fn generate_local_constant_statement(&mut self, name: &str, type_node: &ast::TypeNode, value: &ast::Node, local_context: &mut LocalContext) -> crate::Result<Value> {
        let value_type = self.context.interpret_type_node(type_node)?;
        let value = self.generate_constant_node(value, Some(local_context), Some(value_type))?;

        let pointer_type = self.context.get_pointer_type(value_type, PointerSemantics::Immutable);
        let pointer = local_context.define_indirect_constant_symbol(name.into(), pointer_type, value_type);

        self.emitter.emit_global_allocation(&pointer, &value, true, &self.context)?;

        Ok(Value::Void)
    }

    fn generate_global_let_statement(&mut self, value: Option<&ast::Node>, global_register: &Register) -> crate::Result<Value> {
        // The fill phase has done most of the work for us already
        let TypeRepr::Pointer { pointee_type, .. } = *self.context.type_repr(global_register.get_type()) else {
            panic!("invalid global value register type");
        };

        let init_value = if let Some(node) = value {
            self.generate_constant_node(node, None, Some(pointee_type))?
        }
        else {
            Constant::ZeroInitializer(pointee_type)
        };

        self.emitter.emit_global_allocation(global_register, &init_value, false, &self.context)?;

        Ok(Value::Void)
    }

    fn generate_global_constant_statement(&mut self, value: &ast::Node, global_register: &Register) -> crate::Result<Value> {
        // The fill phase has done most of the work for us already
        let TypeRepr::Pointer { pointee_type, .. } = *self.context.type_repr(global_register.get_type()) else {
            panic!("invalid global value register type");
        };

        let value = self.generate_constant_node(value, None, Some(pointee_type))?;

        self.emitter.emit_global_allocation(global_register, &value, true, &self.context)?;

        Ok(Value::Void)
    }

    fn generate_function_definition(&mut self, name: &str, parameters: &[ast::FunctionParameterNode], body: &ast::Node, function_register: &Register) -> crate::Result<Value> {
        // The fill phase has done a lot of the initial work for us already
        let TypeRepr::Function { signature } = self.context.type_repr(function_register.get_type()) else {
            panic!("invalid global value register type");
        };
        let signature = signature.clone();

        let mut local_context = LocalContext::new(
            self.context.current_namespace_info().path().child(name),
            signature.return_type(),
        );

        let (parameter_registers, parameter_pointers): (Vec<Register>, Vec<Register>) = parameters.iter()
            .zip(signature.parameter_types())
            .map(|(ast::FunctionParameterNode { name, is_mutable, .. }, &parameter_type)| {
                let input_register = local_context.new_anonymous_register(parameter_type);

                let semantics = PointerSemantics::from_flag(*is_mutable);
                let pointer_type = self.context.get_pointer_type(parameter_type, semantics);
                let pointer = local_context.define_indirect_symbol(name.into(), pointer_type, parameter_type);

                (input_register, pointer)
            })
            .collect();

        self.emitter.emit_function_enter(function_register, &parameter_registers, &self.context)?;
        self.emitter.emit_label(local_context.current_block_label())?;

        for (input_register, pointer) in std::iter::zip(parameter_registers, parameter_pointers) {
            self.emitter.emit_local_allocation(&pointer, &self.context)?;
            self.emitter.emit_store(&input_register.into(), &pointer.into(), &self.context)?;
        }

        let body_result = self.generate_local_node(body, &mut local_context, Some(signature.return_type()))?;

        // Insert a return instruction if necessary
        if body_result.get_type() != TypeHandle::NEVER {
            self.emitter.emit_return(&body_result, &self.context)?;
        }

        self.emitter.emit_function_exit()?;

        Ok(Value::Void)
    }

    fn generate_function_declaration(&mut self, function_register: &Register) -> crate::Result<Value> {
        // The fill phase has done basically all of the work for us already
        self.emitter.emit_function_declaration(function_register, &self.context)?;

        Ok(Value::Void)
    }

    fn generate_structure_definition(&mut self, self_type: TypeHandle) -> crate::Result<Value> {
        // The fill phase has done basically all of the work for us already
        self.emitter.emit_type_definition(self_type, &self.context)?;

        Ok(Value::Void)
    }

    fn generate_opaque_structure_definition(&mut self, self_type: TypeHandle) -> crate::Result<Value> {
        // The fill phase has done basically all of the work for us already
        self.emitter.emit_type_declaration(self_type, &self.context)?;

        Ok(Value::Void)
    }

    fn generate_implement_block(&mut self, self_type: &ast::TypeNode, statements: &[Box<ast::Node>]) -> crate::Result<Value> {
        let self_type = self.context.interpret_type_node(self_type)?;

        self.context.set_self_type(self_type);

        for statement in statements {
            self.generate_global_node(statement)?;
        }

        self.context.unset_self_type();

        Ok(Value::Void)
    }

    fn generate_module_block(&mut self, statements: &[Box<ast::Node>], namespace: NamespaceHandle) -> crate::Result<Value> {
        self.context.enter_module(namespace);

        for statement in statements {
            self.generate_global_node(statement)?;
        }

        self.context.exit_module();

        Ok(Value::Void)
    }

    pub fn generate_constant_node(&mut self, node: &ast::Node, local_context: Option<&LocalContext>, expected_type: Option<TypeHandle>) -> crate::Result<Constant> {
        let mut constant_id = self.next_anonymous_constant_id;
        let (constant, intermediate_constants) = self.fold_as_constant(node, &mut constant_id,  local_context, expected_type)?;
        self.next_anonymous_constant_id = constant_id;

        for (pointer, intermediate_constant) in &intermediate_constants {
            self.emitter.emit_anonymous_constant(pointer, intermediate_constant, &self.context)?;
        }

        Ok(constant)
    }

    pub fn fold_as_constant(&mut self, node: &ast::Node, constant_id: &mut usize, local_context: Option<&LocalContext>, expected_type: Option<TypeHandle>) -> crate::Result<(Constant, Vec<(Register, Constant)>)> {
        let mut new_intermediate_constant = |constant: Constant, context: &mut GlobalContext| {
            let pointer = Register::new_global(
                format!(".const.{constant_id}"),
                context.get_pointer_type(constant.get_type(), PointerSemantics::Immutable),
            );
            *constant_id += 1;
            (pointer, constant)
        };

        let mut intermediate_constants = Vec::new();

        let constant = match node {
            ast::Node::Literal(literal) => {
                match *literal {
                    token::Literal::Name(ref name) => {
                        let value = if let Some(value) = local_context.and_then(|ctx| ctx.find_symbol(name)) {
                            value.clone()
                        }
                        else {
                            self.context.get_symbol_value(self.context.current_module(), name)
                                .map_err(|mut error| {
                                    if let crate::Error::UndefinedGlobalSymbol { .. } = error.as_ref() {
                                        *error = crate::Error::UndefinedSymbol {
                                            name: name.clone(),
                                        };
                                    }
                                    error
                                })?
                        };

                        if let Value::Constant(constant) = value {
                            constant
                        }
                        else {
                            return Err(Box::new(crate::Error::NonConstantSymbol {
                                name: name.clone(),
                            }));
                        }
                    }
                    token::Literal::Integer(value) => {
                        let value_type = expected_type.unwrap_or(TypeHandle::I32);
                        let value = IntegerValue::new(value, value_type.repr(&self.context))
                            .ok_or_else(|| Box::new(crate::Error::IncompatibleValueType {
                                value: value.to_string(),
                                type_name: value_type.path(&self.context).to_string(),
                            }))?;

                        Constant::Integer(value)
                    }
                    token::Literal::Boolean(value) => {
                        Constant::Boolean(value)
                    }
                    token::Literal::NullPointer => {
                        Constant::NullPointer(expected_type.unwrap_or_else(|| {
                            self.context.get_pointer_type(TypeHandle::VOID, PointerSemantics::Immutable)
                        }))
                    }
                    token::Literal::String(ref value) => {
                        let constant = Constant::String {
                            array_type: self.context.get_array_type(TypeHandle::U8, Some(value.len())),
                            value: value.clone(),
                        };
                        let (pointer, constant) = new_intermediate_constant(constant, &mut self.context);
                        intermediate_constants.push((pointer.clone(), constant));

                        Constant::Register(pointer)
                    }
                    token::Literal::PrimitiveType(primitive_type) => {
                        Constant::Type(primitive_type.handle)
                    }
                }
            }
            ast::Node::Path { segments } => {
                let path = self.context.get_absolute_path(segments)?;
                match self.context.get_path_value(&path)? {
                    Value::Constant(constant) => constant,
                    _ => return Err(Box::new(crate::Error::NonConstantSymbol {
                        name: path.to_string(),
                    }))
                }
            }
            ast::Node::ArrayLiteral { items } => {
                if let Some(expected_type) = expected_type {
                    let &TypeRepr::Array { item_type, .. } = expected_type.repr(&self.context) else {
                        return Err(Box::new(crate::Error::UnknownArrayType {}));
                    };
                    let items: Vec<Constant> = crate::Result::from_iter(items.iter().map(|item| {
                        let (item, mut constants) = self.fold_as_constant(item, constant_id, local_context, Some(item_type))?;

                        intermediate_constants.append(&mut constants);
                        Ok(item)
                    }))?;

                    Constant::Array {
                        array_type: expected_type,
                        items,
                    }
                }
                else {
                    // TODO
                    return Err(Box::new(crate::Error::UnknownArrayType {}));
                }
            }
            ast::Node::StructureLiteral { structure_type, members: initializer_members } => {
                let struct_type = self.context.interpret_node_as_type(structure_type)?;

                if let TypeRepr::Structure { name: type_name, members } = struct_type.repr(&self.context).clone() {
                    let mut initializer_members = initializer_members.clone();
                    let mut missing_member_names = Vec::new();

                    let members: Vec<Constant> = crate::Result::from_iter(members.iter().map(|member| {
                        if let Some(initializer_index) = initializer_members.iter().position(|(name, _)| &member.name == name) {
                            let (_, member_value) = &initializer_members[initializer_index];
                            let (member_value, mut constants) = self.fold_as_constant(member_value, constant_id, local_context, Some(member.member_type))?;

                            intermediate_constants.append(&mut constants);
                            initializer_members.swap_remove(initializer_index);

                            Ok(member_value)
                        }
                        else {
                            missing_member_names.push(member.name.clone());

                            Ok(Constant::Undefined(member.member_type))
                        }
                    }))?;

                    if !missing_member_names.is_empty() {
                        return Err(Box::new(crate::Error::MissingStructMembers {
                            member_names: missing_member_names,
                            type_name: type_name.clone(),
                        }))
                    }

                    if !initializer_members.is_empty() {
                        let member_names = Vec::from_iter(initializer_members.iter().map(|(name, _)| name.clone()));

                        return Err(Box::new(crate::Error::ExtraStructMembers {
                            member_names,
                            type_name: type_name.clone(),
                        }));
                    }

                    Constant::Structure {
                        members,
                        struct_type,
                    }
                }
                else {
                    return Err(Box::new(crate::Error::NonStructType {
                        type_name: structure_type.to_string(),
                    }));
                }
            }
            ast::Node::Binary { operation, lhs, rhs } => match operation {
                ast::BinaryOperation::Subscript => {
                    let (value, mut constants) = self.fold_subscript_operation(lhs, rhs, constant_id, local_context)?;
                    intermediate_constants.append(&mut constants);

                    value
                }
                ast::BinaryOperation::Convert => {
                    let ast::Node::Type(type_node) = rhs.as_ref() else {
                        // If parsing rules are followed, this should not occur
                        panic!("non-type rhs for 'as'");
                    };

                    let (value, mut constants) = self.fold_as_constant(lhs, constant_id, local_context, None)?;
                    intermediate_constants.append(&mut constants);

                    let target_type = self.context.interpret_type_node(type_node)?;

                    if let Constant::Integer(integer) = value {
                        let converted_integer = IntegerValue::new(integer.expanded_value(), target_type.repr(&self.context))
                            .ok_or_else(|| Box::new(crate::Error::InconvertibleTypes {
                                original_type: integer.get_type().path(&self.context).to_string(),
                                target_type: target_type.path(&self.context).to_string(),
                            }))?;

                        Constant::Integer(converted_integer)
                    }
                    else {
                        return Err(Box::new(crate::Error::InconvertibleTypes {
                            original_type: value.get_type().path(&self.context).to_string(),
                            target_type: target_type.path(&self.context).to_string(),
                        }));
                    }
                }
                _ => {
                    return Err(Box::new(crate::Error::UnsupportedConstantExpression {}));
                }
            }
            ast::Node::Grouping { content } => {
                // Fine to bypass validation steps since this is literally just parentheses
                return self.fold_as_constant(content, constant_id, local_context, expected_type);
            }
            _ => {
                return Err(Box::new(crate::Error::UnsupportedConstantExpression {}));
            }
        };

        if let Some(expected_type) = expected_type {
            self.enforce_constant_type(constant, expected_type)
                .map(|constant| (constant, intermediate_constants))
        }
        else {
            Ok((constant, intermediate_constants))
        }
    }
}
