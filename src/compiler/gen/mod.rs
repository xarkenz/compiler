pub mod llvm;

use llvm::*;
use crate::token;
use crate::ast;
use crate::sema::*;

use std::io::{BufRead, Write};

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

    pub fn new_anonymous_constant(&mut self, pointer_type: TypeHandle) -> Register {
        let id = self.next_anonymous_constant_id;
        self.next_anonymous_constant_id += 1;

        Register::new_global(format!(".const.{id}"), pointer_type)
    }

    pub fn enforce_type(&mut self, value: Value, target_type: TypeHandle, local_context: &mut LocalContext) -> crate::Result<Value> {
        let got_type = value.get_type();

        if self.context.types_are_equivalent(got_type, target_type) {
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
                expected_type: target_type.identifier(self.context()).into(),
                got_type: got_type.identifier(self.context()).into(),
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
                expected_type: target_type.identifier(self.context()).into(),
                got_type: got_type.identifier(self.context()).into(),
            }))
        }
    }

    pub fn change_type(&mut self, value: Value, target_type: TypeHandle, local_context: &mut LocalContext) -> crate::Result<Value> {
        let original_type = value.get_type();

        if self.context.types_are_equivalent(original_type, target_type) {
            Ok(value)
        }
        else {
            match (original_type.info(&self.context), target_type.info(&self.context)) {
                (TypeInfo::Pointer { .. }, TypeInfo::Pointer { .. }) => {
                    let result = local_context.new_anonymous_register(target_type);
                    self.emitter.emit_bitwise_cast(&result, &value, &self.context)?;

                    Ok(Value::Register(result))
                },
                (TypeInfo::Integer { size: from_size, .. }, TypeInfo::Integer { size: to_size, .. }) => {
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
                },
                (original_info @ TypeInfo::Integer { .. }, TypeInfo::Boolean) => {
                    let result = local_context.new_anonymous_register(target_type);
                    let zero = IntegerValue::new(0, original_info).unwrap();
                    self.emitter.emit_cmp_not_equal(&result, &value, &zero.into(), &self.context)?;
                    
                    Ok(Value::Register(result))
                },
                (TypeInfo::Boolean, TypeInfo::Integer { .. }) => {
                    let result = local_context.new_anonymous_register(target_type);
                    self.emitter.emit_zero_extension(&result, &value, &self.context)?;
                    
                    Ok(Value::Register(result))
                },
                _ => {
                    Err(Box::new(crate::Error::IncompatibleTypes {
                        expected_type: target_type.identifier(self.context()).into(),
                        got_type: original_type.identifier(self.context()).into(),
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

        // If the pointee is a `*own _`, copy the semantics from the pointer to the pointee
        if let &TypeInfo::Pointer {
            pointee_type: next_pointee_type,
            semantics: PointerSemantics::Owned,
        } = pointee_type.info(&self.context) {
            let &TypeInfo::Pointer { semantics, .. } = pointer.get_type().info(&self.context) else {
                panic!("indirect value pointer is not a pointer type")
            };
            pointee_type = self.context.get_pointer_type(next_pointee_type, semantics);
        }

        let result = local_context.new_anonymous_register(pointee_type);
        self.emitter.emit_load(&result, &pointer, &self.context)?;

        Ok(Value::Register(result))
    }

    pub fn get_type_from_node(&mut self, node: &ast::Node) -> crate::Result<TypeHandle> {
        let mut current_container = ContainerHandle::Module(self.context.current_module());
        let mut name_stack = Vec::new();
        let mut subpath = node;

        loop {
            match subpath {
                ast::Node::Binary { operation: ast::BinaryOperation::StaticAccess, lhs, rhs } => {
                    let ast::Node::Literal(token::Literal::Identifier(name)) = rhs.as_ref() else {
                        return Err(Box::new(crate::Error::InvalidStaticAccess {}));
                    };
                    name_stack.push(name);
                    subpath = lhs.as_ref();
                }
                ast::Node::Literal(token::Literal::Identifier(name)) => {
                    name_stack.push(name);
                    break;
                }
                ast::Node::Type(type_node) => {
                    current_container = ContainerHandle::Type(self.context.get_type_from_node(type_node)?);
                    break;
                }
                _ => return Err(Box::new(crate::Error::InvalidStaticAccess {}))
            }
        }

        while let Some(name) = name_stack.pop() {
            let Some(container) = current_container.container_binding(name, &self.context) else {
                return Err(Box::new(crate::Error::UndefinedModule { name: name.clone() }));
            };
            current_container = container;
        }

        match current_container {
            ContainerHandle::Module(handle) => {
                Err(Box::new(crate::Error::UnknownType { type_name: self.context.module_info(handle).identifier().into() }))
            }
            ContainerHandle::Type(handle) => {
                Ok(handle)
            }
        }
    }

    pub fn get_symbol_from_path(&mut self, lhs: &ast::Node, rhs: &ast::Node) -> crate::Result<&GlobalSymbol> {
        let ast::Node::Literal(token::Literal::Identifier(symbol_name)) = rhs else {
            return Err(Box::new(crate::Error::InvalidStaticAccess {}));
        };
        
        if let ast::Node::Literal(token::Literal::Identifier(type_name)) = lhs {
            if let Some(named_type) = self.context.get_existing_named_type(self.context.current_module(), type_name) {
                return if let Some(symbol) = self.context.find_type_implementation_symbol(named_type, symbol_name) {
                    Ok(symbol)
                }
                else {
                    Err(Box::new(crate::Error::UndefinedSymbol { name: symbol_name.clone() }))
                }
            }
        }
        
        let mut current_container = ContainerHandle::Module(self.context.current_module());
        let mut name_stack = Vec::new();
        let mut subpath = lhs;

        loop {
            match subpath {
                ast::Node::Binary { operation: ast::BinaryOperation::StaticAccess, lhs, rhs } => {
                    let ast::Node::Literal(token::Literal::Identifier(name)) = rhs.as_ref() else {
                        return Err(Box::new(crate::Error::InvalidStaticAccess {}));
                    };
                    name_stack.push(name);
                    subpath = lhs.as_ref();
                }
                ast::Node::Literal(token::Literal::Identifier(name)) => {
                    name_stack.push(name);
                    break;
                }
                ast::Node::Type(type_node) => {
                    current_container = ContainerHandle::Type(self.context.get_type_from_node(type_node)?);
                    break;
                }
                _ => return Err(Box::new(crate::Error::InvalidStaticAccess {}))
            }
        }

        while let Some(name) = name_stack.pop() {
            if let Some(container) = current_container.container_binding(name, &self.context) {
                current_container = container;
            }
            else {
                return Err(Box::new(crate::Error::UndefinedModule { name: name.clone() }));
            };
        }

        let Some(symbol) = current_container.find_symbol(symbol_name, &self.context) else {
            return Err(Box::new(crate::Error::UndefinedSymbol { name: symbol_name.clone() }));
        };

        Ok(symbol)
    }

    pub fn generate<T: BufRead>(mut self, parser: &mut ast::parse::Parser<T>, filenames: &[String]) -> crate::Result<()> {
        let file_id = parser.file_id();

        self.emitter.emit_preamble(file_id, &filenames[file_id])?;

        while let Some(statement) = parser.parse_top_level_statement()? {
            self.generate_global_node(&statement)?;
        }

        self.emitter.emit_postamble()
    }
    
    pub fn generate_global_node(&mut self, node: &ast::Node) -> crate::Result<Value> {
        match node {
            ast::Node::Let { name, value_type, is_mutable, value } => {
                self.generate_let_statement(name, value_type, *is_mutable, value.as_deref(), None)
            }
            ast::Node::Constant { name, value_type, value } => {
                self.generate_let_constant_statement(name, value_type, value, None)
            }
            ast::Node::Function { name, parameters, is_variadic, return_type, body: Some(body) } => {
                self.generate_function_definition(name, parameters, *is_variadic, return_type, body)
            }
            ast::Node::Function { name, parameters, is_variadic, return_type, body: _ } => {
                self.generate_function_declaration(name, parameters, *is_variadic, return_type)
            }
            ast::Node::Structure { name, members: Some(members) } => {
                self.generate_structure_definition(name, members)
            }
            ast::Node::Structure { name, members: _ } => {
                self.generate_structure_declaration(name)
            }
            ast::Node::Implement { self_type, statements } => {
                self.generate_implement_block(self_type, statements)
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
            ast::Node::StructureLiteral { type_name, members } => {
                self.generate_structure_literal(type_name, members, local_context)?
            }
            ast::Node::Scope { statements } => {
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

                local_context.exit_scope();

                result
            }
            ast::Node::Conditional { condition, consequent, alternative } => {
                let condition = self.generate_local_node(condition, local_context, Some(TypeHandle::BOOL))?;
                let condition = self.coerce_to_rvalue(condition, local_context)?;

                let consequent_label = local_context.new_block_label();
                let alternative_label = local_context.new_block_label();

                self.emitter.emit_conditional_branch(&condition, &consequent_label, &alternative_label, &self.context)?;
                
                if let Some(alternative) = alternative {
                    let mut tail_label = None;

                    self.emitter.emit_label(&consequent_label)?;
                    let consequent_value = self.generate_local_node(consequent, local_context, None)?;
                    if consequent_value.get_type() != TypeHandle::NEVER {
                        let tail_label = tail_label.get_or_insert_with(|| local_context.new_block_label());
                        self.emitter.emit_unconditional_branch(tail_label)?;
                    }

                    self.emitter.emit_label(&alternative_label)?;
                    let alternative_value = self.generate_local_node(alternative, local_context, None)?;
                    if alternative_value.get_type() != TypeHandle::NEVER {
                        let tail_label = tail_label.get_or_insert_with(|| local_context.new_block_label());
                        self.emitter.emit_unconditional_branch(tail_label)?;
                    }

                    if let Some(tail_label) = tail_label {
                        self.emitter.emit_label(&tail_label)?;

                        Value::Void
                    }
                    else {
                        Value::Never
                    }
                }
                else {
                    self.emitter.emit_label(&consequent_label)?;
                    let consequent_value = self.generate_local_node(consequent, local_context, None)?;
                    if consequent_value.get_type() != TypeHandle::NEVER {
                        self.emitter.emit_unconditional_branch(&alternative_label)?;
                    }

                    self.emitter.emit_label(&alternative_label)?;

                    Value::Void
                }
            }
            ast::Node::While { condition, body } => {
                // TODO: handling never, break/continue vs. return
                let condition_label = local_context.new_block_label();

                self.emitter.emit_unconditional_branch(&condition_label)?;

                self.emitter.emit_label(&condition_label)?;

                let condition = self.generate_local_node(condition, local_context, Some(TypeHandle::BOOL))?;
                let condition = self.coerce_to_rvalue(condition, local_context)?;

                let body_label = local_context.new_block_label();
                let tail_label = local_context.new_block_label();
                
                local_context.push_break_label(tail_label.clone());
                local_context.push_continue_label(condition_label.clone());

                self.emitter.emit_conditional_branch(&condition, &body_label, &tail_label, &self.context)?;

                self.emitter.emit_label(&body_label)?;
                self.generate_local_node(body, local_context, None)?;
                self.emitter.emit_unconditional_branch(&condition_label)?;

                self.emitter.emit_label(&tail_label)?;
                
                local_context.pop_break_label();
                local_context.pop_continue_label();

                Value::Void
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
                        return Err(Box::new(crate::Error::UnexpectedReturnValue {}));
                    }
                    else {
                        let value = self.generate_local_node(value, local_context, Some(return_type.clone()))?;
                        let value = self.coerce_to_rvalue(value, local_context)?;

                        self.emitter.emit_return(Some(&value), &self.context)?;
                    }
                }
                else if return_type == TypeHandle::VOID {
                    self.emitter.emit_return(None, &self.context)?;
                }
                else {
                    return Err(Box::new(crate::Error::ExpectedReturnValue {}));
                }

                Value::Never
            }
            ast::Node::Let { name, value_type, is_mutable, value } => {
                self.generate_let_statement(name, value_type, *is_mutable, value.as_deref(), Some(local_context))?
            }
            ast::Node::Constant { name, value_type, value } => {
                self.generate_let_constant_statement(name, value_type, value, Some(local_context))?
            }
            _ => {
                return Err(Box::new(crate::Error::UnexpectedExpression {}));
            }
        };

        if let Some(expected_type) = expected_type {
            self.enforce_type(result, expected_type, local_context)
        }
        else {
            Ok(result)
        }
    }

    fn generate_literal(&mut self, literal: &token::Literal, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let result = match *literal {
            token::Literal::Identifier(ref name) => {
                if let Some(value) = local_context.find_symbol(name) {
                    value.clone()
                }
                else if let Some(symbol) = self.context.current_module_info().find_symbol(name) {
                    symbol.value.clone()
                }
                else {
                    return Err(Box::new(crate::Error::UndefinedSymbol { name: name.clone() }));
                }
            }
            token::Literal::Integer(value) => {
                let value_type = expected_type.unwrap_or(TypeHandle::I32);
                let Some(value) = IntegerValue::new(value, value_type.info(&self.context)) else {
                    return Err(Box::new(crate::Error::IncompatibleValueType {
                        value: value.to_string(),
                        type_name: value_type.identifier(&self.context).into(),
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
        };

        Ok(result)
    }

    fn generate_array_literal(&mut self, items: &[Box<ast::Node>], local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<Value> {
        let Some(array_type) = expected_type else {
            return Err(Box::new(crate::Error::UnknownArrayType {}));
        };
        let &TypeInfo::Array { item_type, .. } = array_type.info(&self.context) else {
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
        let struct_type = self.get_type_from_node(type_name)?;

        let TypeInfo::Structure { name: type_name, members } = struct_type.info(&self.context).clone() else {
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
            return Err(Box::new(crate::Error::MissingStructMembers { member_names: missing_member_names, type_name: type_name.clone() }))
        }

        if !initializer_members.is_empty() {
            let member_names = Vec::from_iter(initializer_members.iter().map(|(name, _)| name.clone()));

            return Err(Box::new(crate::Error::ExtraStructMembers { member_names, type_name: type_name.clone() }));
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
            ast::UnaryOperation::PostIncrement => todo!(),
            ast::UnaryOperation::PostDecrement => todo!(),
            ast::UnaryOperation::PreIncrement => todo!(),
            ast::UnaryOperation::PreDecrement => todo!(),
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
                
                if let &TypeInfo::Pointer { pointee_type, .. } = operand.get_type().info(&self.context) {
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
                        type_name: operand.get_type().identifier(&self.context).into(),
                    }));
                }
            }
            ast::UnaryOperation::GetSize => {
                let ast::Node::Type(type_node) = operand else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type operand for 'sizeof'");
                };

                let value_type = self.context.get_type_from_node(type_node)?;
                let Some(size) = self.context.type_size(value_type) else {
                    return Err(Box::new(crate::Error::UnknownTypeSize {
                        type_name: value_type.identifier(&self.context).into(),
                    }));
                };

                Value::Constant(Constant::Integer(IntegerValue::Unsigned64(size as u64)))
            }
            ast::UnaryOperation::GetAlign => {
                let ast::Node::Type(type_node) = operand else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type operand for 'alignof'");
                };

                let value_type = self.context.get_type_from_node(type_node)?;
                let alignment = self.context.type_alignment(value_type);
                if alignment == 0 {
                    return Err(Box::new(crate::Error::UnknownTypeSize {
                        type_name: value_type.identifier(&self.context).into(),
                    }));
                }

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

                if let &TypeInfo::Pointer { pointee_type, .. } = lhs.get_type().info(&self.context) {
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
            ast::BinaryOperation::StaticAccess => {
                let symbol = self.get_symbol_from_path(lhs, rhs)?;

                symbol.value.clone()
            }
            ast::BinaryOperation::Convert => {
                let ast::Node::Type(type_node) = rhs else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type rhs for 'as'");
                };

                let value = self.generate_local_node(lhs, local_context, None)?;
                let value = self.coerce_to_rvalue(value, local_context)?;

                let target_type = self.context.get_type_from_node(type_node)?;

                self.change_type(value, target_type, local_context)?
            }
            ast::BinaryOperation::Add => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_addition(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Subtract => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_subtraction(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Multiply => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_multiplication(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Divide => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_division(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::Remainder => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_remainder(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::ShiftLeft => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_shift_left(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::ShiftRight => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_shift_right(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseAnd => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_bitwise_and(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseOr => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

                self.emitter.emit_bitwise_or(&result, &lhs, &rhs, &self.context)?;

                Value::Register(result)
            }
            ast::BinaryOperation::BitwiseXor => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, local_context, expected_type)?;

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
            ast::BinaryOperation::LogicalAnd => todo!(),
            ast::BinaryOperation::LogicalOr => todo!(),
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
        
        let TypeInfo::Integer { .. } = rhs_type.info(&self.context) else {
            return Err(Box::new(crate::Error::ExpectedInteger { type_name: rhs_type.identifier(&self.context).into() }));
        };

        let cannot_index_error = |context: &GlobalContext| {
            Box::new(crate::Error::ExpectedArray { type_name: lhs_type.identifier(context).into() })
        };

        match lhs {
            Value::Indirect { pointer, pointee_type } => match *pointee_type.info(&self.context) {
                TypeInfo::Array { item_type, length } => {
                    // &[T; N], &[T]
                    let &TypeInfo::Pointer { semantics, .. } = pointer.get_type().info(&self.context) else {
                        panic!("bad pointer for indirect");
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
                TypeInfo::Pointer { pointee_type: array_type, semantics } => match *array_type.info(&self.context) {
                    TypeInfo::Array { item_type, length } => {
                        // &*[T; N], &*[T]
                        let &TypeInfo::Pointer { semantics: outer_semantics, .. } = pointer.get_type().info(&self.context) else {
                            panic!("bad pointer for indirect");
                        };
                        let semantics = match semantics {
                            PointerSemantics::Owned => outer_semantics,
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
            Value::Register(register) => match *register.get_type().info(&self.context) {
                TypeInfo::Pointer { pointee_type, semantics } => match *pointee_type.info(&self.context) {
                    TypeInfo::Array { item_type, length } => {
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

        let TypeInfo::Integer { .. } = rhs_type.info(&self.context) else {
            return Err(Box::new(crate::Error::ExpectedInteger { type_name: rhs_type.identifier(&self.context).into() }));
        };

        let cannot_index_error = |context: &GlobalContext| {
            Box::new(crate::Error::ExpectedArray { type_name: lhs_type.identifier(context).into() })
        };

        let constant = match lhs {
            Constant::Indirect { pointer, pointee_type } => match pointee_type.info(&self.context) {
                &TypeInfo::Array { item_type, length } => {
                    // const &[T; N], const &[T]
                    let &TypeInfo::Pointer { semantics, .. } = pointer.get_type().info(&self.context) else {
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
            Constant::Register(register) => match register.get_type().info(&self.context) {
                &TypeInfo::Pointer { pointee_type, semantics } => match pointee_type.info(&self.context) {
                    &TypeInfo::Array { item_type, length } => {
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

        let cannot_access_error = |context: &GlobalContext| {
            Box::new(crate::Error::ExpectedStruct { type_name: lhs_type.identifier(context).into() })
        };

        let ast::Node::Literal(token::Literal::Identifier(member_name)) = member_name else {
            todo!("need to integrate `Span` into codegen")
            // return Err(Box::new(crate::Error::ExpectedIdentifier { span: ??? }));
        };

        // First, search for a method implemented for the type
        if let Some(symbol) = self.context.find_type_implementation_symbol(lhs_type, member_name) {
            // A method was found, so bind self and return it
            return Ok(Value::BoundFunction {
                self_value: Box::new(lhs),
                function_value: Box::new(symbol.value.clone()),
            });
        }

        // No method was found, so attempt to get a struct member
        match lhs {
            Value::Indirect { pointer, pointee_type } => match pointee_type.info(&self.context).clone() {
                TypeInfo::Structure { members, .. } => {
                    let &TypeInfo::Pointer { semantics, .. } = pointer.get_type().info(&self.context) else {
                        panic!("bad pointer for indirect");
                    };
                    
                    let Some(member_index) = members.iter().position(|member| &member.name == member_name) else {
                        return Err(Box::new(crate::Error::UndefinedStructMember {
                            member_name: member_name.clone(),
                            type_name: lhs_type.identifier(&self.context).into(),
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

    fn generate_binary_arithmetic_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, local_context: &mut LocalContext, expected_type: Option<TypeHandle>) -> crate::Result<(Register, Value, Value)> {
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

        let rhs = self.generate_local_node(rhs, local_context, Some(pointee_type.clone()))?;
        let rhs = self.coerce_to_rvalue(rhs, local_context)?;

        let lhs = local_context.new_anonymous_register(pointee_type);
        let result = local_context.new_anonymous_register(pointee_type);

        self.emitter.emit_load(&lhs, &pointer, &self.context)?;

        Ok((result, pointer, Value::Register(lhs), rhs))
    }

    fn generate_call_operation(&mut self, callee: &ast::Node, arguments: &[Box<ast::Node>], local_context: &mut LocalContext) -> crate::Result<Value> {
        let callee = self.generate_local_node(callee, local_context, None)?;
        let callee = self.coerce_to_rvalue(callee, local_context)?;

        let TypeInfo::Function { signature } = callee.get_type().info(&self.context).clone() else {
            return Err(Box::new(crate::Error::ExpectedFunction {
                type_name: callee.get_type().identifier(&self.context).into(),
            }));
        };
        
        let mut argument_values = Vec::new();

        // Ensure that when arguments and parameter formats are zipped, all arguments are generated
        // This is important for variadic arguments, which don't have corresponding parameters
        let mut parameters_iter = signature.parameter_types().iter()
            .map(|&parameter_type| Some(parameter_type))
            .chain(std::iter::repeat(None));

        if let Value::BoundFunction { self_value, .. } = &callee {
            let Some(self_parameter_type) = parameters_iter.next().unwrap() else {
                return Err(Box::new(crate::Error::ExpectedSelfParameter {}));
            };

            // TODO: figure out what happens if methods are implemented on pointers
            let self_argument = match self_parameter_type.info(&self.context) {
                TypeInfo::Pointer { .. } => match self_value.as_ref() {
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

        for (argument, parameter_type) in arguments.iter().zip(parameters_iter) {
            let argument = self.generate_local_node(argument, local_context, parameter_type)?;
            let argument = self.coerce_to_rvalue(argument, local_context)?;

            argument_values.push(argument);
        }

        let expected_count = signature.parameter_types().len();
        let got_count = argument_values.len();
        if !signature.is_variadic() && got_count > expected_count {
            return Err(Box::new(crate::Error::ExtraFunctionArguments { expected_count, got_count }));
        }
        else if got_count < expected_count {
            return Err(Box::new(crate::Error::MissingFunctionArguments { expected_count, got_count }));
        }

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

    fn generate_let_statement(&mut self, name: &str, type_node: &ast::TypeNode, is_mutable: bool, value: Option<&ast::Node>, local_context: Option<&mut LocalContext>) -> crate::Result<Value> {
        let value_type = self.context.get_type_from_node(type_node)?;
        let semantics = match is_mutable {
            false => PointerSemantics::Immutable,
            true => PointerSemantics::Mutable,
        };

        if let Some(local_context) = local_context {
            let pointer_type = self.context.get_pointer_type(value_type, semantics);
            let pointer = local_context.define_indirect_symbol(name.into(), pointer_type, value_type);

            self.emitter.emit_local_allocation(&pointer, &self.context)?;

            if let Some(node) = value {
                let value = self.generate_local_node(node, local_context, Some(value_type))?;
                let value = self.coerce_to_rvalue(value, local_context)?;

                self.emitter.emit_store(&value, &pointer.into(), &self.context)?;
            }
        }
        else {
            let init_value = if let Some(node) = value {
                self.generate_constant_node(node, None, Some(value_type))?
            }
            else {
                Constant::ZeroInitializer(value_type)
            };

            let identifier = self.context.create_member_identifier(name);
            let pointer_type = self.context.get_pointer_type(value_type, semantics);
            let pointer = Register::new_global(identifier, pointer_type);

            self.emitter.emit_global_allocation(&pointer, &init_value, false, &self.context)?;

            self.context.define_symbol(name.into(), init_value.into())?;
        }

        Ok(Value::Void)
    }

    fn generate_let_constant_statement(&mut self, name: &str, type_node: &ast::TypeNode, value: &ast::Node, local_context: Option<&mut LocalContext>) -> crate::Result<Value> {
        let value_type = self.context.get_type_from_node(type_node)?;

        if let Some(local_context) = local_context {
            let value = self.generate_constant_node(value, Some(local_context), Some(value_type))?;
            
            let pointer_type = self.context.get_pointer_type(value_type, PointerSemantics::Immutable);
            let pointer = local_context.define_indirect_constant_symbol(name.into(), pointer_type, value_type);
            
            self.emitter.emit_global_allocation(&pointer, &value, true, &self.context)?;
        }
        else {
            let value = self.generate_constant_node(value, None, Some(value_type))?;

            let identifier = self.context.create_member_identifier(name);
            let pointer_type = self.context.get_pointer_type(value_type, PointerSemantics::Immutable);
            let pointer = Register::new_global(identifier, pointer_type);

            self.emitter.emit_global_allocation(&pointer, &value, true, &self.context)?;

            self.context.define_symbol(name.into(), value.into())?;
        }

        Ok(Value::Void)
    }

    fn generate_function_declaration(&mut self, name: &str, parameters: &[ast::FunctionParameter], is_variadic: bool, return_type: &ast::TypeNode) -> crate::Result<Value> {
        let return_type = self.context.get_type_from_node(return_type)?;
        let parameter_types: Box<[TypeHandle]> = Result::from_iter(parameters.iter().map(|ast::FunctionParameter { type_node, ..}| {
            self.context.get_type_from_node(type_node)
        }))?;

        let signature = FunctionSignature::new(return_type, parameter_types, is_variadic);
        let function_type = self.context.get_function_type(&signature);
        let identifier = self.context.create_member_identifier(name);
        let function_register = Register::new_global(identifier, function_type);

        self.emitter.queue_function_declaration(&function_register, &self.context);
        
        self.context.declare_symbol(name.into(), function_register.into())?;

        Ok(Value::Void)
    }

    fn generate_function_definition(&mut self, name: &str, parameters: &[ast::FunctionParameter], is_variadic: bool, return_type: &ast::TypeNode, body: &ast::Node) -> crate::Result<Value> {
        let return_type = self.context.get_type_from_node(return_type)?;
        let parameter_types: Box<[TypeHandle]> = Result::from_iter(parameters.iter().map(|ast::FunctionParameter { type_node, .. }| {
            self.context.get_type_from_node(type_node)
        }))?;

        let signature = FunctionSignature::new(return_type, parameter_types, is_variadic);
        let function_type = self.context.get_function_type(&signature);
        let identifier = self.context.create_member_identifier(name);
        let mut local_context = LocalContext::new(identifier.clone(), return_type);
        let function_register = Register::new_global(identifier, function_type);

        let (parameter_registers, parameter_pointers): (Vec<Register>, Vec<Register>) = parameters.iter()
            .zip(signature.parameter_types())
            .map(|(ast::FunctionParameter { name, is_mutable, .. }, &parameter_type)| {
                let input_register = local_context.new_anonymous_register(parameter_type);

                let semantics = match is_mutable {
                    false => PointerSemantics::Immutable,
                    true => PointerSemantics::Mutable,
                };

                let pointer_type = self.context.get_pointer_type(parameter_type, semantics);
                let pointer = local_context.define_indirect_symbol(name.into(), pointer_type, parameter_type);

                (input_register, pointer)
            })
            .collect();

        self.emitter.emit_function_enter(&function_register, &parameter_registers, &self.context)?;

        // Define a symbol for the function, overwriting the function declaration symbol if it exists
        self.context.define_symbol(name.into(), function_register.into())?;

        let entry_label = local_context.new_block_label();
        self.emitter.emit_label(&entry_label)?;

        for (input_register, pointer) in std::iter::zip(parameter_registers, parameter_pointers) {
            self.emitter.emit_local_allocation(&pointer, &self.context)?;
            self.emitter.emit_store(&input_register.into(), &pointer.into(), &self.context)?;
        }

        let body_result = self.generate_local_node(body, &mut local_context, None)?;

        // Insert a void return if necessary
        if body_result.get_type() != TypeHandle::NEVER {
            if return_type == TypeHandle::VOID {
                self.emitter.emit_return(None, &self.context)?;
            }
            else {
                return Err(Box::new(crate::Error::MissingReturnStatement { function_name: name.into() }));
            }
        }

        self.emitter.emit_function_exit()?;

        Ok(Value::Void)
    }

    fn generate_structure_declaration(&mut self, type_name: &str) -> crate::Result<Value> {
        if let Some(container) = self.context.current_module_info().container_binding(type_name) {
            if let ContainerHandle::Module(module) = container {
                return Err(Box::new(crate::Error::TypeSymbolConflict {
                    name: self.context.module_info(module).identifier().into(),
                }));
            }
        }
        else {
            let type_handle = self.context.get_named_type(self.context.current_module(), type_name)?;

            self.emitter.queue_type_declaration(type_handle, &self.context);
        }

        Ok(Value::Void)
    }

    fn generate_structure_definition(&mut self, type_name: &str, members: &[(String, ast::TypeNode)]) -> crate::Result<Value> {
        match self.context.current_module_info().container_binding(type_name) {
            Some(ContainerHandle::Module(module)) => {
                return Err(Box::new(crate::Error::TypeSymbolConflict {
                    name: self.context.module_info(module).identifier().into(),
                }));
            }
            Some(ContainerHandle::Type(type_handle)) => {
                let TypeInfo::Undefined { .. } = type_handle.info(&self.context) else {
                    return Err(Box::new(crate::Error::TypeSymbolConflict {
                        name: type_handle.identifier(&self.context).into(),
                    }));
                };
            }
            None => {}
        }

        let members: Box<[StructureMember]> = crate::Result::from_iter(members.iter().map(|(member_name, member_type)| {
            Ok(StructureMember {
                name: member_name.clone(),
                member_type: self.context.get_type_from_node(member_type)?,
            })
        }))?;
        
        let structure_info = TypeInfo::Structure {
            name: type_name.into(),
            members,
        };
        
        let structure_type = self.context.define_named_type(self.context.current_module(), type_name, structure_info)?;

        self.emitter.emit_type_definition(structure_type, &self.context)?;

        Ok(Value::Void)
    }

    fn generate_implement_block(&mut self, self_type: &ast::TypeNode, statements: &[Box<ast::Node>]) -> crate::Result<Value> {
        let self_type = self.context.get_type_from_node(self_type)?;
        
        self.context.enter_implement_block(self_type);
        
        for statement in statements {
            self.generate_global_node(statement)?;
        }
        
        self.context.exit_implement_block();

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
                    token::Literal::Identifier(ref name) => {
                        let value = if let Some(value) = local_context.and_then(|ctx| ctx.find_symbol(name)) {
                            value
                        }
                        else if let Some(symbol) = self.context.current_module_info().find_symbol(name) {
                            &symbol.value
                        }
                        else {
                            return Err(Box::new(crate::Error::UndefinedSymbol { name: name.clone() }));
                        };

                        match value {
                            Value::Constant(constant) => constant.clone(),
                            _ => return Err(Box::new(crate::Error::NonConstantSymbol { name: name.clone() }))
                        }
                    }
                    token::Literal::Integer(value) => {
                        let value_type = expected_type.unwrap_or(TypeHandle::I32);
                        let value = IntegerValue::new(value, value_type.info(&self.context))
                            .ok_or_else(|| Box::new(crate::Error::IncompatibleValueType {
                                value: value.to_string(),
                                type_name: value_type.identifier(&self.context).into(),
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
                }
            }
            ast::Node::ArrayLiteral { items } => {
                if let Some(expected_type) = expected_type {
                    let &TypeInfo::Array { item_type, .. } = expected_type.info(&self.context) else {
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
            ast::Node::StructureLiteral { type_name, members: initializer_members } => {
                let struct_type = self.get_type_from_node(type_name)?;
                
                if let TypeInfo::Structure { name: type_name, members } = struct_type.info(&self.context).clone() {
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
                        return Err(Box::new(crate::Error::MissingStructMembers { member_names: missing_member_names, type_name: type_name.clone() }))
                    }
        
                    if !initializer_members.is_empty() {
                        let member_names = Vec::from_iter(initializer_members.iter().map(|(name, _)| name.clone()));
        
                        return Err(Box::new(crate::Error::ExtraStructMembers { member_names, type_name: type_name.clone() }));
                    }

                    Constant::Structure {
                        members,
                        struct_type,
                    }
                }
                else {
                    return Err(Box::new(crate::Error::NonStructType { type_name: type_name.to_string() }));
                }
            }
            ast::Node::Binary { operation, lhs, rhs } => match operation {
                ast::BinaryOperation::Subscript => {
                    let (value, mut constants) = self.fold_subscript_operation(lhs, rhs, constant_id, local_context)?;
                    intermediate_constants.append(&mut constants);

                    value
                }
                ast::BinaryOperation::StaticAccess => {
                    let symbol = self.get_symbol_from_path(lhs, rhs)?;

                    match &symbol.value {
                        Value::Constant(constant) => constant.clone(),
                        _ => return Err(Box::new(crate::Error::UnsupportedConstantExpression {}))
                    }
                }
                ast::BinaryOperation::Convert => {
                    let ast::Node::Type(type_node) = rhs.as_ref() else {
                        // If parsing rules are followed, this should not occur
                        panic!("non-type rhs for 'as'");
                    };

                    let (value, mut constants) = self.fold_as_constant(lhs, constant_id, local_context, None)?;
                    intermediate_constants.append(&mut constants);

                    let target_type = self.context.get_type_from_node(type_node)?;

                    if let Constant::Integer(integer) = value {
                        let converted_integer = IntegerValue::new(integer.expanded_value(), target_type.info(&self.context))
                            .ok_or_else(|| Box::new(crate::Error::InconvertibleTypes {
                                original_type: integer.get_type().identifier(&self.context).into(),
                                target_type: target_type.identifier(&self.context).into(),
                            }))?;

                        Constant::Integer(converted_integer)
                    }
                    else {
                        return Err(Box::new(crate::Error::InconvertibleTypes {
                            original_type: value.get_type().identifier(&self.context).into(),
                            target_type: target_type.identifier(&self.context).into(),
                        }));
                    }
                }
                _ => {
                    return Err(Box::new(crate::Error::UnsupportedConstantExpression {}));
                }
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
