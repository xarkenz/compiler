pub mod llvm;

use llvm::*;
use crate::token;
use crate::ast;

use std::io::{BufRead, Write};

#[derive(Clone, Debug)]
pub struct FunctionContext {
    name: String,
    containing_scope: Scope,
    return_format: Format,
}

impl FunctionContext {
    pub fn new(name: String, containing_scope: Scope, return_format: Format) -> Self {
        Self {
            name,
            containing_scope,
            return_format,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn containing_scope(&self) -> &Scope {
        &self.containing_scope
    }

    pub fn return_format(&self) -> &Format {
        &self.return_format
    }
}

#[derive(Clone, Debug)]
pub struct ScopeContext {
    function_context: Option<FunctionContext>,
    break_label: Option<Label>,
    continue_label: Option<Label>,
}

impl ScopeContext {
    pub fn new() -> Self {
        Self {
            function_context: None,
            break_label: None,
            continue_label: None,
        }
    }

    pub fn is_global(&self) -> bool {
        self.function_context.is_none()
    }

    pub fn function(&self) -> Option<&FunctionContext> {
        self.function_context.as_ref()
    }

    pub fn break_label(&self) -> Option<&Label> {
        self.break_label.as_ref()
    }

    pub fn continue_label(&self) -> Option<&Label> {
        self.continue_label.as_ref()
    }

    pub fn self_format(&self) -> Option<&Format> {
        self.self_format.as_ref()
    }

    pub fn enter_function(&self, current_scope: &Scope, name: String, return_format: Format) -> Self {
        let mut new_scope = self.clone();
        new_scope.function_context = Some(FunctionContext::new(name, current_scope.clone(), return_format));
        new_scope
    }

    pub fn enter_loop(&self, break_label: Label, continue_label: Label) -> Self {
        let mut new_scope = self.clone();
        new_scope.break_label = Some(break_label);
        new_scope.continue_label = Some(continue_label);
        new_scope
    }

    pub fn enter_implement(&self, self_format: Format) -> Self {
        let mut new_scope = self.clone();
        new_scope.self_format = Some(self_format);
        new_scope
    }
}

pub struct Generator<W: Write> {
    emitter: Emitter<W>,
    symbol_table: SymbolTable,
    next_anonymous_register_id: usize,
    next_block_id: usize,
    next_anonymous_constant_id: usize,
}

impl Generator<std::fs::File> {
    pub fn from_filename(filename: String) -> crate::Result<Self> {
        Emitter::from_filename(filename)
            .map(|emitter| Self::new(emitter))
    }
}

impl<W: Write> Generator<W> {
    const DEFAULT_SYMBOL_TABLE_CAPACITY: usize = 128;

    pub fn new(emitter: Emitter<W>) -> Self {
        Self {
            emitter,
            symbol_table: SymbolTable::new(Self::DEFAULT_SYMBOL_TABLE_CAPACITY),
            next_anonymous_register_id: 0,
            next_block_id: 0,
            next_anonymous_constant_id: 0,
        }
    }

    pub fn new_anonymous_register(&mut self, format: Format) -> Register {
        let id = self.next_anonymous_register_id;
        self.next_anonymous_register_id += 1;

        Register::new_local(format!("{id}"), format)
    }

    pub fn new_block_label(&mut self) -> Label {
        let id = self.next_block_id;
        self.next_block_id += 1;

        Label::new(format!(".block.{id}"))
    }

    pub fn new_anonymous_constant(&mut self, format: Format) -> Register {
        let id = self.next_anonymous_constant_id;
        self.next_anonymous_constant_id += 1;

        Register::new_global(format!(".const.{id}"), format.into_pointer(PointerSemantics::Immutable))
    }

    pub fn get_symbol(&self, name: &str) -> crate::Result<&Symbol> {
        self.symbol_table.find(name, None)
            .ok_or_else(|| Box::new(crate::Error::UndefinedSymbol { name: name.into() }))
    }

    pub fn get_type_symbol(&self, type_name: &str) -> crate::Result<&TypeSymbol> {
        self.symbol_table.find(type_name, None)
            .and_then(Symbol::type_value)
            .ok_or_else(|| Box::new(crate::Error::UnknownType { type_name: type_name.into() }))
    }

    pub fn get_type_definition_format(&self, type_name: &str) -> crate::Result<Format> {
        self.get_type_symbol(type_name)?
            .type_handle()
            .cloned()
            .ok_or_else(|| Box::new(crate::Error::PartialType { type_name: type_name.into() }))
    }

    pub fn get_type_identifier(&self, type_name: &str) -> crate::Result<String> {
        let type_identifier = self.get_type_symbol(type_name)?
            .member_scope()
            .name()
            .expect("member scope without a name");

        Ok(type_identifier.into())
    }

    pub fn get_format_from_name(&self, type_name: &str) -> crate::Result<Format> {
        let builtin_format = BUILTIN_FORMATS.iter()
            .find_map(|&(builtin_type_name, ref builtin_format)| {
                (type_name == builtin_type_name).then_some(builtin_format)
            });
        
        let format = match builtin_format {
            Some(builtin_format) => builtin_format.clone(),
            None => Format::Identified {
                type_identifier: self.get_type_identifier(type_name)?,
                type_name: type_name.into(),
            },
        };

        Ok(format)
    }

    pub fn get_format_from_node(&self, type_node: &ast::TypeNode, allow_unsized: bool, context: &ScopeContext) -> crate::Result<Format> {
        let format = match type_node {
            ast::TypeNode::Path { names: type_name } => {
                self.get_format_from_name(type_name)?
            },
            ast::TypeNode::Pointer { pointee_type, semantics } => {
                self.get_format_from_node(pointee_type, true, context)?
                    .into_pointer(*semantics)
            },
            ast::TypeNode::Array { item_type, length: Some(length) } => {
                if let ast::Node::Literal(token::Literal::Integer(length)) = length.as_ref() {
                    Format::Array {
                        item_format: Box::new(self.get_format_from_node(item_type, false, context)?),
                        length: Some(*length as usize),
                    }
                }
                else {
                    return Err(Box::new(crate::Error::NonConstantArrayLength {}));
                }
            },
            ast::TypeNode::Array { item_type, length: None } => {
                Format::Array {
                    item_format: Box::new(self.get_format_from_node(item_type, false, context)?),
                    length: None,
                }
            },
            ast::TypeNode::Function { parameter_types, is_variadic: is_varargs, return_type } => {
                Format::Function {
                    signature: Box::new(FunctionSignature::new(
                        self.get_format_from_node(return_type, false, context)?,
                        Result::from_iter(parameter_types.iter().map(|parameter_type| {
                            self.get_format_from_node(parameter_type, false, context)
                        }))?,
                        *is_varargs,
                    )),
                }
            },
            ast::TypeNode::SelfType => {
                context.self_format()
                    .cloned()
                    .ok_or_else(|| Box::new(crate::Error::SelfOutsideImplement {}))?
            },
        };

        if !allow_unsized {
            format.expect_size(&self.symbol_table)?;
        }

        Ok(format)
    }

    pub fn enforce_format(&mut self, value: Value, target_format: &Format) -> crate::Result<Value> {
        let got_format = value.get_type();

        if &got_format == target_format {
            Ok(value)
        }
        else if got_format.can_coerce_to(target_format, true) {
            if !got_format.requires_bitcast_to(target_format) {
                Ok(value)
            }
            else if let Value::Constant(constant) = value {
                Ok(Value::Constant(Constant::BitwiseCast {
                    value: Box::new(constant),
                    result_type: target_format.clone(),
                }))
            }
            else {
                let value = self.coerce_to_rvalue(value)?;
                let result = self.new_anonymous_register(target_format.clone());
                self.emitter.emit_bitwise_cast(&result, &value)?;

                Ok(Value::Register(result))
            }
        }
        else {
            Err(Box::new(crate::Error::IncompatibleTypes { expected_type: target_format.rich_name(), got_type: got_format.rich_name() }))
        }
    }

    pub fn enforce_constant_format(&self, constant: Constant, target_format: &Format) -> crate::Result<Constant> {
        let got_format = constant.get_type();

        if &got_format == target_format {
            Ok(constant)
        }
        else if got_format.can_coerce_to(target_format, true) {
            if got_format.requires_bitcast_to(target_format) {
                Ok(Constant::BitwiseCast {
                    value: Box::new(constant),
                    result_type: target_format.clone(),
                })
            }
            else {
                Ok(constant)
            }
        }
        else {
            Err(Box::new(crate::Error::IncompatibleTypes { expected_type: target_format.rich_name(), got_type: got_format.rich_name() }))
        }
    }

    pub fn change_format(&mut self, value: Value, target_format: &Format) -> crate::Result<Value> {
        let original_format = value.get_type();

        if &original_format == target_format {
            Ok(value)
        }
        else {
            match (&original_format, target_format) {
                (Format::Pointer { .. }, Format::Pointer { .. }) => {
                    let result = self.new_anonymous_register(target_format.clone());
                    self.emitter.emit_bitwise_cast(&result, &value)?;

                    Ok(Value::Register(result))
                },
                (Format::Integer { size: from_size, .. }, Format::Integer { size: to_size, .. }) => {
                    if to_size > from_size {
                        let result = self.new_anonymous_register(target_format.clone());
                        self.emitter.emit_extension(&result, &value)?;

                        Ok(Value::Register(result))
                    }
                    else if to_size < from_size {
                        let result = self.new_anonymous_register(target_format.clone());
                        self.emitter.emit_truncation(&result, &value)?;

                        Ok(Value::Register(result))
                    }
                    else {
                        Ok(value)
                    }
                },
                (Format::Integer { .. }, Format::Boolean) => {
                    let result = self.new_anonymous_register(Format::Boolean);
                    let zero = IntegerValue::new(0, &original_format).unwrap();
                    self.emitter.emit_cmp_not_equal(&result, &value, &Value::Constant(Constant::Integer(zero)))?;
                    
                    Ok(Value::Register(result))
                },
                (Format::Boolean, Format::Integer { .. }) => {
                    let result = self.new_anonymous_register(target_format.clone());
                    self.emitter.emit_zero_extension(&result, &value)?;
                    
                    Ok(Value::Register(result))
                },
                _ => {
                    Err(Box::new(crate::Error::InconvertibleTypes { original_type: original_format.rich_name(), target_type: target_format.rich_name() }))
                }
            }
        }
    }

    pub fn coerce_to_rvalue(&mut self, value: Value) -> crate::Result<Value> {
        if let Value::Indirect { pointer, pointee_type: mut loaded_format } = value {
            if let Format::Pointer { semantics, .. } = &mut loaded_format {
                if let PointerSemantics::Owned = semantics {
                    *semantics = pointer.get_type().pointer_semantics().unwrap();
                }
            }

            let result = self.new_anonymous_register(loaded_format);
            self.emitter.emit_load(&result, pointer.as_ref())?;
            
            Ok(Value::Register(result))
        }
        else if let Value::Constant(Constant::Indirect { pointer, pointee_type: mut loaded_format }) = value {
            if let Format::Pointer { semantics, .. } = &mut loaded_format {
                if let PointerSemantics::Owned = semantics {
                    *semantics = pointer.get_type().pointer_semantics().unwrap();
                }
            }

            let result = self.new_anonymous_register(loaded_format);
            self.emitter.emit_load(&result, &Value::Constant(*pointer))?;

            Ok(Value::Register(result))
        }
        else {
            Ok(value)
        }
    }

    pub fn add_builtins_to_symbol_table(&mut self) {
        for &(type_name, ref definition_format) in BUILTIN_FORMATS {
            let member_scope = self.symbol_table.create_inactive_scope(Some(type_name.into()));
            let symbol = self.symbol_table.create_type_symbol(
                type_name.into(),
                Some(definition_format.clone()),
                member_scope,
            );

            self.symbol_table.insert(symbol);
        }
    }

    pub fn generate<T: BufRead>(mut self, parser: &mut ast::parse::Parser<T>, filenames: &[String]) -> crate::Result<()> {
        let module_id = parser.file_id();
        let global_context = ScopeContext::new();
        
        self.add_builtins_to_symbol_table();

        self.emitter.emit_preamble(module_id, &filenames[module_id])?;

        while let Some(statement) = parser.parse_top_level_statement()? {
            self.generate_node(statement.as_ref(), &global_context, None)?;
        }

        self.emitter.emit_postamble()
    }

    pub fn generate_node(&mut self, node: &ast::Node, context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<Value> {
        if let Ok(constant) = self.generate_constant_node(node, context, expected_format.clone()) {
            return Ok(Value::Constant(constant));
        }

        let result = match node {
            ast::Node::Literal(literal) => {
                self.generate_literal(literal, context, expected_format.as_ref())?
            },
            ast::Node::Unary { operation, operand } => {
                self.generate_unary_operation(*operation, operand, context, expected_format.as_ref())?
            },
            ast::Node::Binary { operation, lhs, rhs } => {
                self.generate_binary_operation(*operation, lhs, rhs, context, expected_format.as_ref())?
            },
            ast::Node::Call { callee, arguments } => {
                self.generate_call_operation(callee, arguments, context)?
            },
            ast::Node::ArrayLiteral { items } => {
                self.generate_array_literal(items, context, expected_format.as_ref())?
            },
            ast::Node::StructureLiteral { type_name, members } => {
                self.generate_structure_literal(type_name, members, context)?
            },
            ast::Node::Scope { statements } => {
                self.symbol_table.enter_new_scope();

                let mut result = Value::Void;
                for statement in statements {
                    let statement_value = self.generate_node(statement, context, None)?;

                    if statement_value.get_type() == Format::Never {
                        // The rest of the statements in the block will never be executed, so they don't need to be generated
                        result = statement_value;
                        break;
                    }
                }

                self.symbol_table.leave_scope();

                result
            },
            ast::Node::Conditional { condition, consequent, alternative } => {
                let condition = self.generate_node(condition, context, Some(Format::Boolean))?;
                let condition = self.coerce_to_rvalue(condition)?;

                let consequent_label = self.new_block_label();
                let alternative_label = self.new_block_label();

                self.emitter.emit_conditional_branch(&condition, &consequent_label, &alternative_label)?;
                
                if let Some(alternative) = alternative {
                    let mut tail_label = None;

                    self.emitter.emit_label(&consequent_label)?;
                    let consequent_value = self.generate_node(consequent, context, None)?;
                    if consequent_value.get_type() != Format::Never {
                        let tail_label = tail_label.get_or_insert_with(|| self.new_block_label());
                        self.emitter.emit_unconditional_branch(tail_label)?;
                    }

                    self.emitter.emit_label(&alternative_label)?;
                    let alternative_value = self.generate_node(alternative, context, None)?;
                    if alternative_value.get_type() != Format::Never {
                        let tail_label = tail_label.get_or_insert_with(|| self.new_block_label());
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
                    let consequent_value = self.generate_node(consequent, context, None)?;
                    if consequent_value.get_type() != Format::Never {
                        self.emitter.emit_unconditional_branch(&alternative_label)?;
                    }

                    self.emitter.emit_label(&alternative_label)?;

                    Value::Void
                }
            },
            ast::Node::While { condition, body } => {
                // TODO: handling never, break/continue vs. return
                let condition_label = self.new_block_label();

                self.emitter.emit_unconditional_branch(&condition_label)?;

                self.emitter.emit_label(&condition_label)?;

                let condition = self.generate_node(condition, context, Some(Format::Boolean))?;
                let condition = self.coerce_to_rvalue(condition)?;

                let body_label = self.new_block_label();
                let tail_label = self.new_block_label();
                let loop_context = context.clone().enter_loop(tail_label.clone(), condition_label.clone());

                self.emitter.emit_conditional_branch(&condition, &body_label, &tail_label)?;

                self.emitter.emit_label(&body_label)?;
                self.generate_node(body, &loop_context, None)?;
                self.emitter.emit_unconditional_branch(&condition_label)?;

                self.emitter.emit_label(&tail_label)?;

                Value::Void
            },
            ast::Node::Break => {
                let break_label = context.break_label()
                    .ok_or_else(|| Box::new(crate::Error::InvalidBreak {}))?;

                self.emitter.emit_unconditional_branch(break_label)?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                // self.next_anonymous_register_id += 1;

                Value::Break
            },
            ast::Node::Continue => {
                let continue_label = context.continue_label()
                    .ok_or_else(|| Box::new(crate::Error::InvalidContinue {}))?;

                self.emitter.emit_unconditional_branch(continue_label)?;

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                // self.next_anonymous_register_id += 1;

                Value::Continue
            },
            ast::Node::Return { value } => {
                let return_format = context.function().map(FunctionContext::return_format)
                    .ok_or_else(|| Box::new(crate::Error::InvalidReturn {}))?;

                if let Some(value) = value {
                    if return_format == &Format::Void {
                        return Err(Box::new(crate::Error::UnexpectedReturnValue {}));
                    }
                    else {
                        let value = self.generate_node(value, context, Some(return_format.clone()))?;
                        let value = self.coerce_to_rvalue(value)?;

                        self.emitter.emit_return(Some(&value))?;
                    }
                }
                else {
                    if return_format == &Format::Void {
                        self.emitter.emit_return(None)?;
                    }
                    else {
                        return Err(Box::new(crate::Error::ExpectedReturnValue {}));
                    }
                }

                // Consume an anonymous ID corresponding to the implicit label inserted after the terminator instruction
                // self.next_anonymous_register_id += 1;

                Value::Never
            },
            ast::Node::Let { name, value_type, is_mutable, value } => {
                self.generate_let_statement(name, value_type, *is_mutable, value.as_deref(), context)?
            },
            ast::Node::Constant { name, value_type, value } => {
                self.generate_let_constant_statement(name, value_type, value, context)?
            },
            ast::Node::Function { name, parameters, is_varargs, return_type, body: None } => {
                self.generate_function_declaration(name, parameters, *is_varargs, return_type, context)?
            },
            ast::Node::Function { name, parameters, is_varargs, return_type, body: Some(body) } => {
                self.generate_function_definition(name, parameters, *is_varargs, return_type, body, context)?
            },
            ast::Node::Structure { name, members: None } => {
                self.generate_structure_declaration(name, context)?
            },
            ast::Node::Structure { name, members: Some(members) } => {
                self.generate_structure_definition(name, members, context)?
            },
            ast::Node::Implement { self_type, statements } => {
                self.generate_implement_block(self_type, statements, context)?
            },
            _ => {
                return Err(Box::new(crate::Error::UnexpectedExpression {}));
            }
        };

        if let Some(expected_format) = &expected_format {
            self.enforce_format(result, expected_format)
        }
        else {
            Ok(result)
        }
    }

    fn generate_literal(&mut self, literal: &token::Literal, _context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<Value> {
        let result = match literal {
            token::Literal::Identifier(name) => {
                self.get_symbol(name)?.value()
            },
            token::Literal::Integer(value) => {
                let format = expected_format.cloned().unwrap_or(Format::Integer { size: 4, signed: true });
                let value = IntegerValue::new(*value, &format)
                    .ok_or_else(|| Box::new(crate::Error::IncompatibleValueType { value: value.to_string(), type_name: format.rich_name() }))?;
                Value::Constant(Constant::Integer(value))
            },
            token::Literal::Boolean(value) => {
                Value::Constant(Constant::Boolean(*value))
            },
            token::Literal::NullPointer => {
                Value::Constant(Constant::NullPointer(expected_format.cloned().unwrap_or_else(Format::opaque_pointer)))
            },
            token::Literal::String(value) => {
                let constant = Constant::String(value.clone());
                let pointer = self.new_anonymous_constant(constant.get_type());

                self.emitter.emit_anonymous_constant(&pointer, &constant)?;

                Value::Constant(Constant::Register(pointer))
            },
        };

        Ok(result)
    }

    fn generate_array_literal(&mut self, items: &[Box<ast::Node>], context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<Value> {
        if let Some(array_format) = expected_format {
            if let Format::Array { item_format, .. } = array_format {
                let mut non_constant_items = Vec::new();

                let constant_items: Vec<Constant> = crate::Result::from_iter(items.iter().enumerate().map(|(index, item)| {
                    let item_value = self.generate_node(item, context, Some(item_format.as_ref().clone()))?;

                    if let Value::Constant(item_constant) = item_value {
                        Ok(item_constant)
                    }
                    else {
                        let item_value = self.coerce_to_rvalue(item_value)?;
                        non_constant_items.push((index, item_value));

                        Ok(Constant::Undefined(item_format.as_ref().clone()))
                    }
                }))?;

                let array_pointer = self.new_anonymous_register(array_format.clone().into_pointer(PointerSemantics::Immutable));
                let initial_value = Value::Constant(Constant::Array {
                    items: constant_items,
                    item_type: item_format.as_ref().clone(),
                });

                self.emitter.emit_local_allocation(&array_pointer, array_format)?;

                let array_pointer = Value::Register(array_pointer);

                self.emitter.emit_store(&initial_value, &array_pointer)?;
                
                for (index, member) in non_constant_items {
                    let item_pointer = self.new_anonymous_register(member.get_type().into_pointer(PointerSemantics::Mutable));
                    let zero = Value::Constant(Constant::Integer(IntegerValue::Signed32(0)));
                    let index = Value::Constant(Constant::Integer(IntegerValue::Unsigned64(index as u64)));

                    self.emitter.emit_get_element_pointer(&item_pointer, array_format, &array_pointer, &[zero, index])?;
                    self.emitter.emit_store(&member, &Value::Register(item_pointer))?;
                }

                Ok(Value::Indirect {
                    pointer: Box::new(array_pointer),
                    pointee_type: array_format.clone(),
                })
            }
            else {
                Err(Box::new(crate::Error::UnknownArrayType {}))
            }
        }
        else {
            // TODO
            Err(Box::new(crate::Error::UnknownArrayType {}))
        }
    }

    fn generate_structure_literal(&mut self, type_name: &ast::Node, initializer_members: &[(String, Box<ast::Node>)], context: &ScopeContext) -> crate::Result<Value> {
        let structure_format = match type_name {
            ast::Node::Literal(token::Literal::Identifier(name)) => {
                self.get_symbol(name)?
                    .type_value()
                    .ok_or_else(|| Box::new(crate::Error::NonStructSymbol { name: name.clone() }))?
                    .type_handle()
                    .ok_or_else(|| Box::new(crate::Error::PartialType { type_name: name.clone() }))?
                    .clone()
            },
            ast::Node::Type(type_node) => {
                // FIXME: we need the *definition* format, not the identified format
                self.get_format_from_node(type_node, false, context)?
            }
            _ => return Err(Box::new(crate::Error::InvalidStructIdentifier {}))
        };

        if let Format::Structure { type_name: Some(type_name), members } = structure_format {
            let result_format = Format::Identified {
                type_identifier: self.get_type_identifier(&type_name)?,
                type_name: type_name.clone(),
            };

            let mut initializer_members = initializer_members.to_vec();
            let mut non_constant_members = Vec::new();
            let mut missing_member_names = Vec::new();

            let constant_members: Vec<Constant> = crate::Result::from_iter(members.iter().enumerate().map(|(index, (member_name, member_format))| {
                if let Some(initializer_index) = initializer_members.iter().position(|(name, _)| member_name == name) {
                    let (_, member_value) = &initializer_members[initializer_index];
                    let member_value = self.generate_node(member_value, context, Some(member_format.clone()))?;

                    initializer_members.swap_remove(initializer_index);

                    if let Value::Constant(member_constant) = member_value {
                        Ok(member_constant)
                    }
                    else {
                        let member_value = self.coerce_to_rvalue(member_value)?;
                        non_constant_members.push((index, member_value));

                        Ok(Constant::Undefined(member_format.clone()))
                    }
                }
                else {
                    missing_member_names.push(member_name.clone());

                    Ok(Constant::Undefined(member_format.clone()))
                }
            }))?;

            if !missing_member_names.is_empty() {
                return Err(Box::new(crate::Error::MissingStructMembers { member_names: missing_member_names, type_name: type_name.clone() }))
            }

            if !initializer_members.is_empty() {
                let member_names = Vec::from_iter(initializer_members.iter().map(|(name, _)| name.clone()));

                return Err(Box::new(crate::Error::ExtraStructMembers { member_names, type_name: type_name.clone() }));
            }

            let structure_pointer = self.new_anonymous_register(result_format.clone().into_pointer(PointerSemantics::Immutable));
            let initial_value = Value::Constant(Constant::Structure {
                members: constant_members,
                struct_type: result_format.clone(),
            });

            self.emitter.emit_local_allocation(&structure_pointer, &result_format)?;

            let structure_pointer = Value::Register(structure_pointer);

            self.emitter.emit_store(&initial_value, &structure_pointer)?;
            
            for (index, member) in non_constant_members {
                let member_pointer = self.new_anonymous_register(member.get_type().into_pointer(PointerSemantics::Mutable));
                let zero = Value::Constant(Constant::Integer(IntegerValue::Signed32(0)));
                let index = Value::Constant(Constant::Integer(IntegerValue::Signed32(index as i32)));

                self.emitter.emit_get_element_pointer(&member_pointer, &result_format, &structure_pointer, &[zero, index])?;
                self.emitter.emit_store(&member, &Value::Register(member_pointer))?;
            }

            Ok(Value::Indirect {
                pointer: Box::new(structure_pointer),
                pointee_type: result_format,
            })
        }
        else {
            Err(Box::new(crate::Error::NonStructType { type_name: type_name.to_string() }))
        }
    }

    fn generate_unary_operation(&mut self, operation: ast::UnaryOperation, operand: &ast::Node, context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<Value> {
        let result = match operation {
            ast::UnaryOperation::PostIncrement => todo!(),
            ast::UnaryOperation::PostDecrement => todo!(),
            ast::UnaryOperation::PreIncrement => todo!(),
            ast::UnaryOperation::PreDecrement => todo!(),
            ast::UnaryOperation::Positive => {
                let operand = self.generate_node(operand, context, expected_format.cloned())?;

                self.coerce_to_rvalue(operand)?
            },
            ast::UnaryOperation::Negative => {
                let operand = self.generate_node(operand, context, expected_format.cloned())?;
                let operand = self.coerce_to_rvalue(operand)?;
                let result = self.new_anonymous_register(expected_format.cloned().unwrap_or(operand.get_type()));

                self.emitter.emit_negation(&result, &operand)?;

                Value::Register(result)
            },
            ast::UnaryOperation::BitwiseNot => {
                let operand = self.generate_node(operand, context, expected_format.cloned())?;
                let operand = self.coerce_to_rvalue(operand)?;
                let result = self.new_anonymous_register(expected_format.cloned().unwrap_or(operand.get_type()));

                self.emitter.emit_inversion(&result, &operand)?;

                Value::Register(result)
            },
            ast::UnaryOperation::LogicalNot => {
                let operand = self.generate_node(operand, context, Some(Format::Boolean))?;
                let operand = self.coerce_to_rvalue(operand)?;
                let result = self.new_anonymous_register(Format::Boolean);

                self.emitter.emit_inversion(&result, &operand)?;

                Value::Register(result)
            },
            ast::UnaryOperation::Reference => {
                let operand = self.generate_node(operand, context, None)?;

                if let Value::Indirect { pointer, .. } = operand {
                    *pointer
                }
                else {
                    return Err(Box::new(crate::Error::ExpectedLValue {}));
                }
            },
            ast::UnaryOperation::Dereference => {
                let operand = self.generate_node(operand, context, expected_format.map(|format| format.clone().into_pointer(PointerSemantics::Immutable)))?;
                let operand = self.coerce_to_rvalue(operand)?;
                
                if let Format::Pointer { pointee_format, .. } = operand.get_type() {
                    if let Value::Constant(constant) = operand {
                        Value::Constant(Constant::Indirect {
                            pointer: Box::new(constant),
                            pointee_type: *pointee_format,
                        })
                    }
                    else {
                        Value::Indirect {
                            pointer: Box::new(operand),
                            pointee_type: *pointee_format,
                        }
                    }
                }
                else {
                    return Err(Box::new(crate::Error::ExpectedPointer { type_name: operand.get_type().rich_name() }));
                }
            },
            ast::UnaryOperation::GetSize => {
                if let ast::Node::Type(type_node) = operand {
                    let format = self.get_format_from_node(type_node, false, context)?;
                    let size = format.expect_size(&self.symbol_table)?;

                    Value::Constant(Constant::Integer(IntegerValue::Unsigned64(size as u64)))
                }
                else {
                    panic!("non-type operand for 'sizeof'");
                }
            },
            ast::UnaryOperation::GetAlign => {
                if let ast::Node::Type(type_node) = operand {
                    let format = self.get_format_from_node(type_node, false, context)?;
                    let align = format.expect_alignment(&self.symbol_table)?;

                    Value::Constant(Constant::Integer(IntegerValue::Unsigned64(align as u64)))
                }
                else {
                    panic!("non-type operand for 'alignof'");
                }
            },
        };

        Ok(result)
    }

    fn generate_binary_operation(&mut self, operation: ast::BinaryOperation, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<Value> {
        let result = match operation {
            ast::BinaryOperation::Subscript => {
                self.generate_subscript_operation(lhs, rhs, context)?
            },
            ast::BinaryOperation::Access => {
                let lhs = self.generate_node(lhs, context, None)?;

                if let Format::Pointer { pointee_format, .. } = lhs.get_type() {
                    let pointer = self.coerce_to_rvalue(lhs)?;
                    let structure = Value::Indirect {
                        pointer: Box::new(pointer),
                        pointee_type: *pointee_format,
                    };

                    self.generate_member_access(structure, rhs, context)?
                }
                else {
                    self.generate_member_access(lhs, rhs, context)?
                }
            },
            ast::BinaryOperation::StaticAccess => {
                let containing_scope = self.generate_constant_node(lhs, context, Some(Format::Scope))?;

                let containing_scope = match containing_scope {
                    Constant::Scope(scope) => scope,
                    _ => return Err(Box::new(crate::Error::InvalidStaticAccess {}))
                };
                let member_name = match rhs {
                    ast::Node::Literal(token::Literal::Identifier(name)) => name,
                    _ => return Err(Box::new(crate::Error::InvalidStaticAccess {}))
                };

                self.symbol_table.find(member_name, Some(&containing_scope))
                    .ok_or_else(|| Box::new(crate::Error::UndefinedSymbol { name: member_name.clone() }))?
                    .value()
            },
            ast::BinaryOperation::Convert => {
                if let ast::Node::Type(type_node) = rhs {
                    let value = self.generate_node(lhs, context, None)?;
                    let value = self.coerce_to_rvalue(value)?;
                    
                    let target_format = self.get_format_from_node(type_node, false, context)?;

                    self.change_format(value, &target_format)?
                }
                else {
                    // If parsing rules are followed, this should not occur
                    panic!("non-type rhs for 'as'");
                }
            },
            ast::BinaryOperation::Add => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_addition(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Subtract => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_subtraction(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Multiply => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_multiplication(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Divide => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_division(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Remainder => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_remainder(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::ShiftLeft => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_shift_left(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::ShiftRight => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_shift_right(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::BitwiseAnd => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_and(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::BitwiseOr => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_or(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::BitwiseXor => {
                let (result, lhs, rhs) = self.generate_binary_arithmetic_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_xor(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::Equal => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_equal(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::NotEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_not_equal(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::LessThan => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_less_than(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::LessEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_less_equal(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::GreaterThan => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_greater_than(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::GreaterEqual => {
                let (result, lhs, rhs) = self.generate_comparison_operands(lhs, rhs, context)?;

                self.emitter.emit_cmp_greater_equal(&result, &lhs, &rhs)?;

                Value::Register(result)
            },
            ast::BinaryOperation::LogicalAnd => todo!(),
            ast::BinaryOperation::LogicalOr => todo!(),
            ast::BinaryOperation::Assign => {
                let lhs = self.generate_node(lhs, context, expected_format.cloned())?;
                let (pointer, loaded_format) = lhs.into_mutable_lvalue()?;
                let rhs = self.generate_node(rhs, context, Some(loaded_format))?;
                let rhs = self.coerce_to_rvalue(rhs)?;

                self.emitter.emit_store(&rhs, &pointer)?;

                rhs
            },
            ast::BinaryOperation::MultiplyAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_multiplication(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::DivideAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_division(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::RemainderAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_remainder(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::AddAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_addition(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::SubtractAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_subtraction(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::ShiftLeftAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_shift_left(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::ShiftRightAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_shift_right(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::BitwiseAndAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_and(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::BitwiseXorAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_xor(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
            ast::BinaryOperation::BitwiseOrAssign => {
                let (result, pointer, lhs, rhs) = self.generate_assignment_operands(lhs, rhs, context, expected_format)?;

                self.emitter.emit_bitwise_or(&result, &lhs, &rhs)?;
                let result = Value::Register(result);
                self.emitter.emit_store(&result, &pointer)?;

                result
            },
        };

        Ok(result)
    }

    fn generate_subscript_operation(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext) -> crate::Result<Value> {
        let lhs = self.generate_node(lhs, context, None)?;
        let rhs = self.generate_node(rhs, context, None)?;
        let rhs = self.coerce_to_rvalue(rhs)?;
        
        if !(matches!(rhs.get_type(), Format::Integer { .. })) {
            return Err(Box::new(crate::Error::ExpectedInteger { type_name: rhs.get_type().rich_name() }));
        }

        let lhs_format = lhs.get_type();
        let cannot_index_error = || Box::new(crate::Error::ExpectedArray { type_name: lhs_format.rich_name() });

        match lhs {
            Value::Indirect { pointer, pointee_type: loaded_format } => match &loaded_format {
                Format::Array { item_format, length } => {
                    // &[T; N], &[T]
                    let semantics = pointer.get_type().pointer_semantics().unwrap();
                    let element_pointer = self.new_anonymous_register(item_format.as_ref().clone().into_pointer(semantics));
                    let indices = match length {
                        Some(_) => vec![Value::Constant(Constant::Integer(IntegerValue::Signed32(0))), rhs],
                        None => vec![rhs],
                    };

                    self.emitter.emit_get_element_pointer(
                        &element_pointer,
                        &loaded_format,
                        pointer.as_ref(),
                        &indices,
                    )?;

                    Ok(Value::Indirect {
                        pointer: Box::new(Value::Register(element_pointer)),
                        pointee_type: item_format.as_ref().clone(),
                    })
                },
                Format::Pointer { pointee_format, semantics } => match pointee_format.as_ref() {
                    Format::Array { item_format, length } => {
                        // &*[T; N], &*[T]
                        let semantics = match semantics {
                            PointerSemantics::Owned => pointer.get_type().pointer_semantics().unwrap(),
                            _ => *semantics
                        };
                        let loaded_pointer = self.new_anonymous_register(loaded_format.clone());
                        let element_pointer = self.new_anonymous_register(item_format.as_ref().clone().into_pointer(semantics));
                        let indices = match length {
                            Some(_) => vec![Value::Constant(Constant::Integer(IntegerValue::Signed32(0))), rhs],
                            None => vec![rhs],
                        };

                        self.emitter.emit_load(&loaded_pointer, &pointer)?;
                        self.emitter.emit_get_element_pointer(
                            &element_pointer,
                            &pointee_format,
                            &Value::Register(loaded_pointer),
                            &indices,
                        )?;

                        Ok(Value::Indirect {
                            pointer: Box::new(Value::Register(element_pointer)),
                            pointee_type: item_format.as_ref().clone(),
                        })
                    },
                    _ => Err(cannot_index_error())
                },
                _ => Err(cannot_index_error())
            },
            Value::Register(register) => match register.value_type().clone() {
                Format::Pointer { pointee_format, semantics } => match pointee_format.as_ref() {
                    Format::Array { item_format, length } => {
                        // *[T; N], *[T]
                        let element_pointer = self.new_anonymous_register(item_format.as_ref().clone().into_pointer(semantics));
                        let indices = match length {
                            Some(_) => vec![Value::Constant(Constant::Integer(IntegerValue::Signed32(0))), rhs],
                            None => vec![rhs],
                        };

                        self.emitter.emit_get_element_pointer(
                            &element_pointer,
                            &pointee_format,
                            &Value::Register(register),
                            &indices,
                        )?;

                        Ok(Value::Indirect {
                            pointer: Box::new(Value::Register(element_pointer)),
                            pointee_type: item_format.as_ref().clone(),
                        })
                    },
                    _ => Err(cannot_index_error())
                },
                _ => Err(cannot_index_error())
            },
            _ => Err(cannot_index_error())
        }
    }

    fn fold_subscript_operation(&self, lhs: &ast::Node, rhs: &ast::Node, constant_id: &mut usize, context: &ScopeContext) -> crate::Result<(Constant, Vec<(Register, Constant)>)> {
        let (lhs, mut intermediate_constants) = self.fold_as_constant(lhs, constant_id, context, None)?;
        let (rhs, mut constants) = self.fold_as_constant(rhs, constant_id, context, None)?;
        intermediate_constants.append(&mut constants);

        if !(matches!(rhs.get_type(), Format::Integer { .. })) {
            return Err(Box::new(crate::Error::ExpectedInteger { type_name: rhs.get_type().rich_name() }));
        }

        let lhs_format = lhs.get_type();
        let cannot_index_error = || Box::new(crate::Error::ExpectedArray { type_name: lhs_format.rich_name() });

        let constant = match lhs {
            Constant::Indirect { pointer, pointee_type: loaded_format } => match &loaded_format {
                Format::Array { item_format, length } => {
                    // const &[T; N], const &[T]
                    let semantics = pointer.get_type().pointer_semantics().unwrap();
                    let indices = match length {
                        Some(_) => vec![Constant::Integer(IntegerValue::Signed32(0)), rhs],
                        None => vec![rhs],
                    };

                    let element_pointer = Constant::GetElementPointer {
                        element_type: item_format.as_ref().clone().into_pointer(semantics),
                        aggregate_type: loaded_format.clone(),
                        pointer,
                        indices,
                        semantics,
                    };

                    Constant::Indirect {
                        pointer: Box::new(element_pointer),
                        pointee_type: item_format.as_ref().clone(),
                    }
                },
                _ => return Err(cannot_index_error())
            },
            Constant::Register(register) => match register.value_type().clone() {
                Format::Pointer { pointee_format, semantics } => match pointee_format.as_ref() {
                    Format::Array { item_format, length } => {
                        // const *[T; N], const *[T]
                        let indices = match length {
                            Some(_) => vec![Constant::Integer(IntegerValue::Signed32(0)), rhs],
                            None => vec![rhs],
                        };

                        let element_pointer = Constant::GetElementPointer {
                            element_type: item_format.as_ref().clone().into_pointer(semantics),
                            aggregate_type: pointee_format.as_ref().clone(),
                            pointer: Box::new(Constant::Register(register)),
                            indices,
                            semantics,
                        };

                        Constant::Indirect {
                            pointer: Box::new(element_pointer),
                            pointee_type: item_format.as_ref().clone(),
                        }
                    },
                    _ => return Err(cannot_index_error())
                },
                _ => return Err(cannot_index_error())
            },
            _ => return Err(cannot_index_error())
        };

        Ok((constant, intermediate_constants))
    }

    fn generate_member_access(&mut self, container: Value, member_name: &ast::Node, _context: &ScopeContext) -> crate::Result<Value> {
        let container_format = container.get_type();
        let cannot_access_error = || Box::new(crate::Error::ExpectedStruct { type_name: container_format.rich_name() });

        if let ast::Node::Literal(token::Literal::Identifier(member_name)) = member_name {
            // First, search for a method on the container's type
            let type_name = container_format.rich_name();
            if let Ok(type_symbol) = self.get_type_symbol(&type_name) {
                let member_scope = type_symbol.member_scope();
                if let Some(function_symbol) = self.symbol_table.find(member_name, Some(member_scope)).and_then(Symbol::function_value) {
                    // A method was found, so return it
                    return Ok(Value::BoundFunction {
                        self_value: Box::new(container),
                        function_register: function_symbol.register().clone(),
                    });
                }
            }

            // No method was found, so attempt to get a struct member
            match container {
                Value::Indirect { pointer, pointee_type: loaded_format } => match &loaded_format {
                    Format::Identified { type_name, .. } => match self.get_type_definition_format(type_name)? {
                        Format::Structure { members, .. } => {
                            let semantics = pointer.get_type().pointer_semantics().unwrap();
                            let member_index = members.iter().position(|(name, _)| name == member_name)
                                .ok_or_else(|| Box::new(crate::Error::UndefinedStructMember { member_name: member_name.clone(), type_name: container_format.rich_name() }))?;
                            let member_format = members[member_index].1.clone();
                            let member_pointer = self.new_anonymous_register(member_format.clone().into_pointer(semantics));
                            let indices = &[
                                Value::Constant(Constant::Integer(IntegerValue::Signed32(0))),
                                Value::Constant(Constant::Integer(IntegerValue::Signed32(member_index as i32))),
                            ];

                            self.emitter.emit_get_element_pointer(
                                &member_pointer,
                                &loaded_format,
                                &pointer,
                                indices,
                            )?;

                            Ok(Value::Indirect {
                                pointer: Box::new(Value::Register(member_pointer)),
                                pointee_type: member_format,
                            })
                        },
                        _ => Err(cannot_access_error())
                    },
                    _ => Err(cannot_access_error())
                },
                _ => Err(cannot_access_error())
            }
        }
        else {
            todo!("need to integrate `Span` into codegen")
            // return Err(Box::new(crate::Error::ExpectedIdentifier { span: todo!() }));
        }
    }

    fn generate_binary_arithmetic_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_node(lhs, context, expected_format.cloned())?;
        let lhs = self.coerce_to_rvalue(lhs)?;

        let rhs = self.generate_node(rhs, context, Some(lhs.get_type()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let result = self.new_anonymous_register(expected_format.cloned().unwrap_or_else(|| lhs.get_type()));

        Ok((result, lhs, rhs))
    }

    fn generate_comparison_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext) -> crate::Result<(Register, Value, Value)> {
        let lhs = self.generate_node(lhs, context, None)?;
        let lhs = self.coerce_to_rvalue(lhs)?;

        let rhs = self.generate_node(rhs, context, Some(lhs.get_type()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let result = self.new_anonymous_register(Format::Boolean);

        Ok((result, lhs, rhs))
    }

    fn generate_assignment_operands(&mut self, lhs: &ast::Node, rhs: &ast::Node, context: &ScopeContext, expected_format: Option<&Format>) -> crate::Result<(Register, Value, Value, Value)> {
        let lhs = self.generate_node(lhs, context, expected_format.cloned())?;
        let (pointer, loaded_format) = lhs.into_mutable_lvalue()?;

        let rhs = self.generate_node(rhs, context, Some(loaded_format.clone()))?;
        let rhs = self.coerce_to_rvalue(rhs)?;

        let lhs = self.new_anonymous_register(loaded_format.clone());
        let result = self.new_anonymous_register(loaded_format);

        self.emitter.emit_load(&lhs, &pointer)?;

        Ok((result, pointer, Value::Register(lhs), rhs))
    }

    fn generate_call_operation(&mut self, callee: &ast::Node, arguments: &[Box<ast::Node>], context: &ScopeContext) -> crate::Result<Value> {
        let callee = self.generate_node(callee, context, None)?;
        let callee = self.coerce_to_rvalue(callee)?;

        if let Format::Function { signature } = callee.get_type() {
            let mut argument_values = Vec::new();

            // Ensure that when arguments and parameter formats are zipped, all arguments are generated
            // This is important for variadic arguments, which don't have corresponding parameters
            let mut parameters_iter = signature.parameter_formats().iter()
                .map(|parameter_format| Some(parameter_format))
                .chain(std::iter::repeat(None));

            if let Value::BoundFunction { self_value, .. } = &callee {
                let self_target_format = parameters_iter.next().unwrap()
                    .ok_or_else(|| Box::new(crate::Error::ExpectedSelfParameter {}))?;

                // FIXME: once 'implement' works on arbitrary sema, this logic will be flawed
                let self_argument = match self_target_format {
                    Format::Pointer { .. } => match self_value.as_ref() {
                        Value::Indirect { pointer, .. } => {
                            pointer.as_ref().clone()
                        },
                        _ => {
                            // Allocate temporary space on the stack for the value so it can be pointed to
                            let self_format = self_value.get_type();
                            let self_value_pointer = self.new_anonymous_register(self_format.clone().into_pointer(PointerSemantics::Immutable));

                            self.emitter.emit_local_allocation(&self_value_pointer, &self_format)?;

                            let self_value_pointer = Value::Register(self_value_pointer);

                            self.emitter.emit_store(self_value, &self_value_pointer)?;

                            self_value_pointer
                        }
                    },
                    _ => {
                        self.coerce_to_rvalue(self_value.as_ref().clone())?
                    }
                };
                
                let self_argument = self.enforce_format(self_argument, self_target_format)?;

                // Pass the bound 'self' value as the first argument
                argument_values.push(self_argument);
            }

            for (argument, parameter_format) in arguments.iter().zip(parameters_iter) {
                let argument = self.generate_node(argument, context, parameter_format.cloned())?;
                let argument = self.coerce_to_rvalue(argument)?;

                argument_values.push(argument);
            }

            let expected_count = signature.parameter_formats().len();
            let got_count = argument_values.len();
            if !signature.is_varargs() && got_count > expected_count {
                return Err(Box::new(crate::Error::ExtraFunctionArguments { expected_count, got_count }));
            }
            else if got_count < expected_count {
                return Err(Box::new(crate::Error::MissingFunctionArguments { expected_count, got_count }));
            }

            match signature.return_format() {
                Format::Never => {
                    self.emitter.emit_function_call(None, &callee, &argument_values)?;
                    self.emitter.emit_unreachable()?;

                    Ok(Value::Never)
                },
                Format::Void => {
                    self.emitter.emit_function_call(None, &callee, &argument_values)?;

                    Ok(Value::Void)
                },
                return_format => {
                    let result = self.new_anonymous_register(return_format.clone());

                    self.emitter.emit_function_call(Some(&result), &callee, &argument_values)?;

                    Ok(Value::Register(result))
                }
            }
        }
        else {
            Err(Box::new(crate::Error::ExpectedFunction { type_name: callee.get_type().rich_name() }))
        }
    }

    fn generate_let_statement(&mut self, name: &str, value_type: &ast::TypeNode, is_mutable: bool, value: Option<&ast::Node>, context: &ScopeContext) -> crate::Result<Value> {
        let format = self.get_format_from_node(value_type, false, context)?;

        if context.is_global() {
            let init_value = if let Some(node) = value {
                self.generate_constant_node(node, context, Some(format.clone()))?
            }
            else {
                Constant::ZeroInitializer(format.clone())
            };

            if self.symbol_table.find(name, None).is_some() {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: name.into() }));
            }

            let (symbol, pointer) = self.symbol_table.create_global_indirect_symbol(name.into(), format.clone(), is_mutable);

            self.emitter.emit_global_allocation(&pointer, &init_value, false)?;

            self.symbol_table.insert(symbol);
        }
        else {
            let (symbol, pointer) = self.symbol_table.create_local_indirect_symbol(name.into(), format.clone(), is_mutable);

            self.emitter.emit_local_allocation(&pointer, &format)?;

            if let Some(node) = value {
                let value = self.generate_node(node, context, Some(format.clone()))?;
                let value = self.coerce_to_rvalue(value)?;

                self.emitter.emit_store(&value, &Value::Register(pointer))?;
            }

            self.symbol_table.insert(symbol);
        }

        Ok(Value::Void)
    }

    fn generate_let_constant_statement(&mut self, name: &str, value_type: &ast::TypeNode, value: &ast::Node, context: &ScopeContext) -> crate::Result<Value> {
        let format = self.get_format_from_node(value_type, false, context)?;

        if let Some(function) = context.function() {
            let constant = self.generate_constant_node(value, context, Some(format.clone()))?;
            
            let (symbol, pointer) = self.symbol_table.create_indirect_local_constant_symbol(name.into(), format.clone(), function.name());
            self.symbol_table.insert(symbol);

            self.emitter.emit_global_allocation(&pointer, &constant, true)?;
        }
        else {
            let constant = self.generate_constant_node(value, context, Some(format.clone()))?;

            if self.symbol_table.find(name, None).is_some() {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: name.into() }));
            }

            let (symbol, pointer) = self.symbol_table.create_global_indirect_symbol(name.into(), format.clone(), false);
            self.symbol_table.insert(symbol);

            self.emitter.emit_global_allocation(&pointer, &constant, true)?;
        }

        Ok(Value::Void)
    }

    fn generate_function_declaration(&mut self, name: &str, parameters: &[ast::FunctionParameter], is_varargs: bool, return_type: &ast::TypeNode, context: &ScopeContext) -> crate::Result<Value> {
        let return_format = self.get_format_from_node(return_type, false, context)?;
        let parameter_formats: Vec<Format> = Result::from_iter(parameters.iter().map(|ast::FunctionParameter { type_node, ..}| {
            self.get_format_from_node(type_node, false, context)
        }))?;

        let new_signature = FunctionSignature::new(return_format.clone(), parameter_formats.clone(), is_varargs);

        if let Some(old_symbol) = self.symbol_table.find(name, None) {
            // Ensure the new declaration doesn't conflict with the existing symbol
            if let Some(old_symbol) = old_symbol.function_value() {
                if &new_signature != old_symbol.signature() {
                    return Err(Box::new(crate::Error::FunctionSignatureConflict { function_name: name.into(), old_type: old_symbol.signature().rich_name(), new_type: new_signature.rich_name() }));
                }
            }
            else {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: name.into() }));
            }
        }
        else {
            // Create a symbol for the function since one does not exist already
            let (symbol, function_register) = self.symbol_table.create_function_symbol(name.into(), new_signature, false);
            self.symbol_table.insert(symbol);

            self.emitter.queue_function_declaration(&function_register, &return_format, &parameter_formats, is_varargs);
        }

        Ok(Value::Void)
    }

    fn clear_local_context(&mut self) {
        self.symbol_table.clear_locals();

        self.next_anonymous_register_id = 0;
        self.next_block_id = 0;
    }

    fn generate_function_definition(&mut self, name: &str, parameters: &[ast::FunctionParameter], is_varargs: bool, return_type: &ast::TypeNode, body: &ast::Node, context: &ScopeContext) -> crate::Result<Value> {
        let return_format = self.get_format_from_node(return_type, false, context)?;
        let parameter_formats: Vec<Format> = Result::from_iter(parameters.iter().map(|ast::FunctionParameter { type_node, .. }| {
            self.get_format_from_node(type_node, false, context)
        }))?;

        let parameter_handles: Vec<(Register, Symbol, Register)> = parameters.iter()
            .zip(parameter_formats.iter())
            .map(|(ast::FunctionParameter { name, is_mutable, .. }, format)| {
                let input_register = Register::new_local(format!(".arg.{name}"), format.clone());

                let (symbol, pointer) = self.symbol_table.create_local_indirect_symbol(name.clone(), format.clone(), *is_mutable);
                
                (input_register, symbol, pointer)
            })
            .collect();

        let new_signature = FunctionSignature::new(return_format.clone(), parameter_formats.clone(), is_varargs);

        if let Some(old_symbol) = self.symbol_table.find(name, None) {
            // Ensure the new definition doesn't conflict with the existing symbol
            if let Some(old_symbol) = old_symbol.function_value() {
                if &new_signature != old_symbol.signature() {
                    return Err(Box::new(crate::Error::FunctionSignatureConflict { function_name: name.into(), old_type: old_symbol.signature().rich_name(), new_type: new_signature.rich_name() }));
                }
                if old_symbol.is_defined() {
                    return Err(Box::new(crate::Error::MultipleFunctionDefinition { function_name: name.into() }));
                }
            }
            else {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: name.into() }));
            }
        }
        
        let function_context = context.enter_function(self.symbol_table.current_scope(), name.into(), return_format.clone());

        // Define a symbol for the function, overwriting the function declaration symbol if it exists
        let (symbol, function_register) = self.symbol_table.create_function_symbol(name.into(), new_signature, true);
        self.symbol_table.insert(symbol);

        self.symbol_table.enter_new_scope();

        let input_registers: Vec<Register> = parameter_handles.iter()
            .map(|(register, _, _)| register.clone())
            .collect();
        self.emitter.emit_function_enter(&function_register, &return_format, &input_registers, is_varargs)?;
        let entry_label = self.new_block_label();
        self.emitter.emit_label(&entry_label)?;

        for (input_register, symbol, pointer) in parameter_handles {
            self.emitter.emit_local_allocation(&pointer, input_register.value_type())?;
            self.emitter.emit_store(&Value::Register(input_register), &Value::Register(pointer))?;
            self.symbol_table.insert(symbol);
        }

        let body_result = self.generate_node(body, &function_context, None)?;

        if body_result.get_type() != Format::Never {
            if &return_format == &Format::Void {
                self.emitter.emit_return(None)?;
            }
            else {
                return Err(Box::new(crate::Error::MissingReturnStatement { function_name: name.into() }));
            }
        }

        self.emitter.emit_function_exit()?;

        self.symbol_table.leave_scope();
        self.clear_local_context();

        Ok(Value::Void)
    }

    fn generate_structure_declaration(&mut self, type_name: &str, _context: &ScopeContext) -> crate::Result<Value> {
        if let Some(old_symbol) = self.symbol_table.find(type_name, None) {
            if old_symbol.type_value().is_none() {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: type_name.into() }));
            }
        }
        else {
            // TODO: allow Self as the structure type
            let type_identifier = self.symbol_table.current_scope().get_member_identifier(type_name);
            let identified_format = Format::Identified {
                type_identifier: type_identifier.clone(),
                type_name: type_name.into(),
            };

            self.emitter.queue_type_declaration(&identified_format);

            let member_scope = self.symbol_table.create_inactive_scope(Some(type_identifier));

            let symbol = self.symbol_table.create_type_symbol(type_name.into(), None, member_scope);
            self.symbol_table.insert(symbol);
        }

        Ok(Value::Void)
    }

    fn generate_structure_definition(&mut self, type_name: &str, members: &[(String, ast::TypeNode)], context: &ScopeContext) -> crate::Result<Value> {
        let member_scope;
        if let Some(old_symbol) = self.symbol_table.find(type_name, Some(self.symbol_table.current_scope())) {
            if let Some(old_symbol) = old_symbol.type_value() {
                if old_symbol.type_handle().is_none() {
                    member_scope = old_symbol.member_scope().clone();
                }
                else {
                    return Err(Box::new(crate::Error::GlobalSymbolConflict { name: type_name.into() }));
                }
            }
            else {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: type_name.into() }));
            }
        }
        else {
            let type_identifier = self.symbol_table.current_scope().get_member_identifier(type_name);
            member_scope = self.symbol_table.create_inactive_scope(Some(type_identifier));

            // Declare the symbol beforehand to allow recursive structure definitions
            let symbol = self.symbol_table.create_type_symbol(type_name.into(), None, member_scope.clone());
            self.symbol_table.insert(symbol);
        }

        let type_identifier = member_scope.name()
            .expect("member scope without a name");
        let identified_format = Format::Identified {
            type_identifier: type_identifier.into(),
            type_name: type_name.into(),
        };
        let members: Vec<(String, Format)> = crate::Result::from_iter(members.iter().map(|(member_name, member_type)| {
            let member_format = self.get_format_from_node(member_type, false, context)?;
            Ok((member_name.clone(), member_format))
        }))?;
        let structure_format = Format::Structure {
            type_name: Some(type_name.into()),
            members,
        };

        self.emitter.emit_type_definition(&identified_format, &structure_format)?;

        // Redefine the symbol with the definition attached
        let symbol = self.symbol_table.create_type_symbol(type_name.into(), Some(structure_format), member_scope);
        self.symbol_table.insert(symbol);

        Ok(Value::Void)
    }

    fn generate_implement_block(&mut self, self_type: &ast::TypeNode, statements: &[Box<ast::Node>], context: &ScopeContext) -> crate::Result<Value> {
        if let ast::TypeNode::Path { names: type_name } = self_type {
            let type_symbol = self.get_type_symbol(type_name)?;

            let self_format = self.get_format_from_name(type_name)?;
            let implement_context = context.enter_implement(self_format);

            let member_scope = type_symbol.member_scope().clone();
            self.symbol_table.enter_scope(member_scope);

            for statement in statements {
                self.generate_node(statement, &implement_context, Some(Format::Void))?;
            }

            self.symbol_table.leave_scope();
        }
        else {
            todo!("'implement' on arbitrary types")
        }

        Ok(Value::Void)
    }

    pub fn generate_constant_node(&mut self, node: &ast::Node, context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<Constant> {
        let mut constant_id = self.next_anonymous_constant_id;
        let (constant, intermediate_constants) = self.fold_as_constant(node, &mut constant_id, context, expected_format)?;
        self.next_anonymous_constant_id = constant_id;

        for (pointer, intermediate_constant) in &intermediate_constants {
            self.emitter.emit_anonymous_constant(pointer, intermediate_constant)?;
        }

        Ok(constant)
    }

    pub fn fold_as_constant(&self, node: &ast::Node, constant_id: &mut usize, context: &ScopeContext, expected_format: Option<Format>) -> crate::Result<(Constant, Vec<(Register, Constant)>)> {
        let mut new_intermediate_constant = |constant: Constant| {
            let pointer = Register::new_global(format!(".const.{constant_id}"), constant.get_type().into_pointer(PointerSemantics::Immutable));
            *constant_id += 1;
            (pointer, constant)
        };

        let mut intermediate_constants = Vec::new();

        let constant = match node {
            ast::Node::Literal(literal) => {
                match literal {
                    token::Literal::Identifier(name) => {
                        let value = self.get_symbol(name)?.value();

                        match value {
                            Value::Constant(constant) => constant,
                            _ => return Err(Box::new(crate::Error::NonConstantSymbol { name: name.clone() }))
                        }
                    },
                    token::Literal::Integer(value) => {
                        let format = expected_format.clone().unwrap_or(Format::Integer { size: 4, signed: true });
                        let value = IntegerValue::new(*value, &format)
                            .ok_or_else(|| Box::new(crate::Error::IncompatibleValueType { value: value.to_string(), type_name: format.rich_name() }))?;

                        Constant::Integer(value)
                    },
                    token::Literal::Boolean(value) => {
                        Constant::Boolean(*value)
                    },
                    token::Literal::NullPointer => {
                        Constant::NullPointer(expected_format.clone().unwrap_or_else(Format::opaque_pointer))
                    },
                    token::Literal::String(value) => {
                        let (pointer, constant) = new_intermediate_constant(Constant::String(value.clone()));
                        intermediate_constants.push((pointer.clone(), constant));

                        Constant::Register(pointer)
                    },
                }
            },
            ast::Node::ArrayLiteral { items } => {
                if let Some(Format::Array { item_format, .. }) = &expected_format {
                    let items: Vec<Constant> = crate::Result::from_iter(items.iter().map(|item| {
                        let (item, mut constants) = self.fold_as_constant(item, constant_id, context, Some(item_format.as_ref().clone()))?;

                        intermediate_constants.append(&mut constants);
                        Ok(item)
                    }))?;

                    Constant::Array {
                        items,
                        item_type: item_format.as_ref().clone(),
                    }
                }
                else {
                    // TODO
                    return Err(Box::new(crate::Error::UnknownArrayType {}));
                }
            },
            ast::Node::StructureLiteral { type_name, members: initializer_members } => {
                let structure_format = match type_name.as_ref() {
                    ast::Node::Literal(token::Literal::Identifier(name)) => {
                        self.get_symbol(name)?.type_value()
                            .ok_or_else(|| Box::new(crate::Error::NonStructSymbol { name: name.clone() }))?
                            .type_handle()
                            .ok_or_else(|| Box::new(crate::Error::PartialType { type_name: name.clone() }))?
                            .clone()
                    },
                    ast::Node::Type(type_node) => {
                        self.get_format_from_node(type_node, false, context)?
                    }
                    _ => return Err(Box::new(crate::Error::InvalidStructIdentifier {}))
                };
                
                if let Format::Structure { type_name: Some(type_name), members } = structure_format {
                    let result_format = Format::Identified {
                        type_identifier: self.get_type_identifier(&type_name)?,
                        type_name: type_name.clone(),
                    };

                    let mut initializer_members = initializer_members.clone();
                    let mut missing_member_names = Vec::new();

                    let members: Vec<Constant> = crate::Result::from_iter(members.iter().map(|(member_name, member_format)| {
                        if let Some(initializer_index) = initializer_members.iter().position(|(name, _)| member_name == name) {
                            let (_, member_value) = &initializer_members[initializer_index];
                            let (member_value, mut constants) = self.fold_as_constant(member_value.as_ref(), constant_id, context, Some(member_format.clone()))?;

                            intermediate_constants.append(&mut constants);
                            initializer_members.swap_remove(initializer_index);

                            Ok(member_value)
                        }
                        else {
                            missing_member_names.push(member_name.clone());

                            Ok(Constant::Undefined(member_format.clone()))
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
                        struct_type: result_format,
                    }
                }
                else {
                    return Err(Box::new(crate::Error::NonStructType { type_name: type_name.to_string() }));
                }
            },
            ast::Node::Binary { operation, lhs, rhs } => match operation {
                ast::BinaryOperation::Subscript => {
                    let (value, mut constants) = self.fold_subscript_operation(lhs, rhs, constant_id, context)?;
                    intermediate_constants.append(&mut constants);

                    value
                },
                ast::BinaryOperation::StaticAccess => {
                    let (containing_scope, mut constants) = self.fold_as_constant(lhs, constant_id, context, Some(Format::Scope))?;
                    intermediate_constants.append(&mut constants);

                    let containing_scope = match containing_scope {
                        Constant::Scope(scope) => scope,
                        _ => return Err(Box::new(crate::Error::InvalidStaticAccess {}))
                    };
                    let member_name = match rhs.as_ref() {
                        ast::Node::Literal(token::Literal::Identifier(name)) => name,
                        _ => return Err(Box::new(crate::Error::InvalidStaticAccess {}))
                    };

                    let member_value = self.symbol_table.find(member_name, Some(&containing_scope))
                        .ok_or_else(|| Box::new(crate::Error::UndefinedSymbol { name: member_name.clone() }))?
                        .value();

                    match member_value {
                        Value::Constant(constant) => constant,
                        _ => return Err(Box::new(crate::Error::UnsupportedConstantExpression {}))
                    }
                },
                ast::BinaryOperation::Convert => {
                    if let ast::Node::Type(type_node) = rhs.as_ref() {
                        let (value, mut constants) = self.fold_as_constant(lhs, constant_id, context, None)?;
                        intermediate_constants.append(&mut constants);
                        
                        let target_format = self.get_format_from_node(type_node, false, context)?;
    
                        if let Constant::Integer(integer) = value {
                            let converted_integer = IntegerValue::new(integer.expanded_value(), &target_format)
                                .ok_or_else(|| Box::new(crate::Error::InconvertibleTypes { original_type: integer.value_type().rich_name(), target_type: target_format.rich_name() }))?;

                            Constant::Integer(converted_integer)
                        }
                        else {
                            return Err(Box::new(crate::Error::InconvertibleTypes { original_type: value.get_type().rich_name(), target_type: target_format.rich_name() }));
                        }
                    }
                    else {
                        // If parsing rules are followed, this should not occur
                        panic!("non-type rhs for 'as'");
                    }
                },
                _ => {
                    return Err(Box::new(crate::Error::UnsupportedConstantExpression {}));
                }
            },
            _ => {
                return Err(Box::new(crate::Error::UnsupportedConstantExpression {}));
            }
        };

        if let Some(expected_format) = &expected_format {
            self.enforce_constant_format(constant, expected_format)
                .map(|constant| (constant, intermediate_constants))
        }
        else {
            Ok((constant, intermediate_constants))
        }
    }
}
