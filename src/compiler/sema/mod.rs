use crate::ast::{Node, PathSegment, TypeNode};
use crate::token::Literal;
use std::collections::HashMap;
use std::num::NonZeroUsize;

mod local_ctx;
mod symbols;
mod types;
mod values;

pub use local_ctx::*;
pub use symbols::*;
pub use types::*;
pub use values::*;

struct TypeEntry {
    path: AbsolutePath,
    repr: TypeRepr,
    namespace: NamespaceHandle,
    alignment: Option<Option<usize>>,
    size: Option<Option<usize>>,
    llvm_syntax: Option<String>,
}

pub struct GlobalContext {
    /// The size of a pointer in bytes.
    pointer_size: usize,
    /// Table of all namespaces in existence.
    namespace_registry: Vec<NamespaceInfo>,
    /// The hierarchy of modules currently being analyzed.
    module_stack: Vec<NamespaceHandle>,
    /// Table of all types in existence.
    type_registry: Vec<TypeEntry>,
    /// The type `Self` currently represents, or `None` if not in an `implement` block or `struct`
    /// definition.
    current_self_type: Option<TypeHandle>,
    /// Find a type by absolute path.
    path_base_types: HashMap<PathBaseType, TypeHandle>,
    /// Find `*T`, `*mut T`, or `*own T`.
    pointer_types: HashMap<(TypeHandle, PointerSemantics), TypeHandle>,
    /// Find `[T; N]` or `[T]`.
    array_types: HashMap<(TypeHandle, Option<usize>), TypeHandle>,
    /// Find a function type by signature.
    function_types: HashMap<FunctionSignature, TypeHandle>,
    /// Flag for whether the fill phase has been completed. If so, all type properties are known.
    fill_phase_complete: bool,
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalContext {
    pub fn new() -> Self {
        let mut context = Self {
            pointer_size: size_of::<&()>(),
            namespace_registry: vec![NamespaceInfo::new(AbsolutePath::at_root())],
            module_stack: vec![NamespaceHandle::ROOT],
            type_registry: Vec::with_capacity(PRIMITIVE_TYPES.len()),
            current_self_type: None,
            path_base_types: HashMap::new(),
            pointer_types: HashMap::new(),
            array_types: HashMap::new(),
            function_types: HashMap::new(),
            fill_phase_complete: false,
        };

        for (registry_index, &(name, ref repr)) in PRIMITIVE_TYPES.iter().enumerate() {
            let base_type = PathBaseType::Primitive(PrimitiveType {
                handle: TypeHandle::new(registry_index),
                name,
            });
            let handle = context.create_type(
                AbsolutePath::at_base_type(Box::new(base_type.clone())),
                repr.clone(),
            );
            context.path_base_types.insert(base_type, handle);
        }

        context
    }

    pub fn pointer_size(&self) -> usize {
        self.pointer_size
    }

    pub fn current_module(&self) -> NamespaceHandle {
        *self.module_stack.last().unwrap()
    }

    pub fn current_self_type(&self) -> Option<TypeHandle> {
        self.current_self_type
    }

    pub fn namespace_info(&self, handle: NamespaceHandle) -> &NamespaceInfo {
        &self.namespace_registry[handle.registry_index()]
    }

    pub fn namespace_info_mut(&mut self, handle: NamespaceHandle) -> &mut NamespaceInfo {
        &mut self.namespace_registry[handle.registry_index()]
    }

    fn type_entry(&self, handle: TypeHandle) -> &TypeEntry {
        &self.type_registry[handle.registry_index()]
    }

    fn type_entry_mut(&mut self, handle: TypeHandle) -> &mut TypeEntry {
        &mut self.type_registry[handle.registry_index()]
    }

    pub fn type_path(&self, handle: TypeHandle) -> &AbsolutePath {
        &self.type_entry(handle).path
    }

    pub fn type_repr(&self, handle: TypeHandle) -> &TypeRepr {
        &self.type_entry(handle).repr
    }

    pub fn type_namespace(&self, handle: TypeHandle) -> NamespaceHandle {
        self.type_entry(handle).namespace
    }

    pub fn type_alignment(&self, handle: TypeHandle) -> Option<usize> {
        self.type_entry(handle).alignment
            .expect("type alignment cannot be known before fill phase is completed")
    }

    pub fn type_size(&self, handle: TypeHandle) -> Option<usize> {
        self.type_entry(handle).size
            .expect("type size cannot be known before fill phase is completed")
    }

    pub fn type_llvm_syntax(&self, handle: TypeHandle) -> &str {
        self.type_entry(handle).llvm_syntax.as_ref()
            .expect("type syntax cannot be known before fill phase is completed")
    }

    pub fn current_module_info(&self) -> &NamespaceInfo {
        self.namespace_info(self.current_module())
    }

    pub fn current_module_info_mut(&mut self) -> &mut NamespaceInfo {
        self.namespace_info_mut(self.current_module())
    }

    pub fn current_namespace(&self) -> NamespaceHandle {
        if let Some(self_type) = self.current_self_type() {
            self.type_namespace(self_type)
        }
        else {
            self.current_module()
        }
    }

    pub fn current_namespace_info(&self) -> &NamespaceInfo {
        self.namespace_info(self.current_namespace())
    }

    pub fn current_namespace_info_mut(&mut self) -> &mut NamespaceInfo {
        self.namespace_info_mut(self.current_namespace())
    }

    pub fn enter_module_outline(&mut self, name: &str) -> crate::Result<NamespaceHandle> {
        let current_namespace = self.current_namespace();
        let module_path = self.namespace_info(current_namespace).path().child(name);
        let module_namespace = self.create_namespace(module_path);

        self.namespace_info_mut(current_namespace).define(name, Symbol::Module(module_namespace))?;

        self.module_stack.push(module_namespace);

        Ok(module_namespace)
    }

    pub fn enter_module(&mut self, namespace: NamespaceHandle) {
        self.module_stack.push(namespace);
    }

    pub fn exit_module(&mut self) {
        self.module_stack.pop();

        if self.module_stack.is_empty() {
            panic!("exited root module");
        }
    }

    pub fn set_self_type(&mut self, self_type: TypeHandle) {
        if self.current_self_type().is_some() {
            panic!("Self type is already set");
        }

        self.current_self_type = Some(self_type);
    }

    pub fn unset_self_type(&mut self) -> TypeHandle {
        self.current_self_type.take().expect("Self type is not set")
    }

    pub fn get_super_namespace(&mut self, namespace: NamespaceHandle) -> crate::Result<NamespaceHandle> {
        let Some(super_path) = self.namespace_info(namespace).path().parent() else {
            return Err(Box::new(crate::Error::InvalidSuper {
                namespace: self.namespace_info(namespace).path().to_string(),
            }));
        };

        self.get_path_value(&super_path)?
            .as_namespace(self)
            .ok_or_else(|| Box::new(crate::Error::ExpectedNamespace {
                name: super_path.to_string(),
            }))
    }

    pub fn get_absolute_path(&self, segments: &[PathSegment]) -> crate::Result<AbsolutePath> {
        let mut path = self.namespace_info(self.current_module()).path().clone();

        for segment in segments {
            match segment {
                PathSegment::Name(name) => {
                    path = path.into_child(name);
                }
                PathSegment::RootModule => {
                    path = AbsolutePath::from_root(SimplePath::empty());
                }
                PathSegment::SuperModule => {
                    let Some(parent) = path.parent() else {
                        return Err(Box::new(crate::Error::InvalidSuper {
                            namespace: path.to_string(),
                        }));
                    };
                    path = parent;
                }
                PathSegment::SelfModule => {
                    // TODO: error for non-module?
                }
                PathSegment::SelfType => {
                    let Some(self_type) = self.current_self_type() else {
                        return Err(Box::new(crate::Error::NoSelfType {}));
                    };
                    path = self.type_path(self_type).clone();
                }
                PathSegment::PrimitiveType(primitive_type) => {
                    path = AbsolutePath::from_base_type(Box::new(PathBaseType::Primitive(*primitive_type)), SimplePath::empty());
                }
                PathSegment::Type(type_node) => {
                    path = self.type_path_for_type_node(type_node)?;
                }
            }
        }

        Ok(path)
    }

    pub fn type_path_for_type_node(&self, type_node: &TypeNode) -> crate::Result<AbsolutePath> {
        match type_node {
            TypeNode::Path { segments } => {
                Ok(self.get_absolute_path(segments)?)
            }
            TypeNode::Pointer { pointee_type, semantics } => {
                let base_type = PathBaseType::Pointer {
                    pointee_type: self.type_path_for_type_node(pointee_type)?,
                    semantics: *semantics,
                };
                Ok(AbsolutePath::from_base_type(Box::new(base_type), SimplePath::empty()))
            }
            TypeNode::Array { item_type, length } => {
                let base_type = PathBaseType::Array {
                    item_type: self.type_path_for_type_node(item_type)?,
                    length: match length {
                        Some(node) => Some(node.as_constant_usize()?),
                        None => None,
                    },
                };
                Ok(AbsolutePath::from_base_type(Box::new(base_type), SimplePath::empty()))
            }
            TypeNode::Function { parameter_types, is_variadic, return_type } => {
                let base_type = PathBaseType::Function {
                    parameter_types: Result::from_iter(parameter_types.iter()
                        .map(|type_node| self.type_path_for_type_node(type_node)))?,
                    is_variadic: *is_variadic,
                    return_type: self.type_path_for_type_node(return_type)?,
                };
                Ok(AbsolutePath::from_base_type(Box::new(base_type), SimplePath::empty()))
            }
        }
    }

    pub fn type_path_for_node(&self, node: &Node) -> crate::Result<AbsolutePath> {
        match node {
            Node::Type(type_node) => {
                self.type_path_for_type_node(type_node)
            }
            Node::Path { segments, .. } => {
                self.get_absolute_path(segments)
            }
            Node::Literal(Literal::Name(name)) => {
                Ok(self.current_module_info().path().child(name))
            }
            Node::Literal(Literal::PrimitiveType(primitive_type)) => {
                let base_type = PathBaseType::Primitive(*primitive_type);
                Ok(AbsolutePath::from_base_type(Box::new(base_type), SimplePath::empty()))
            }
            _ => {
                Err(Box::new(crate::Error::UnexpectedExpression {}))
            }
        }
    }

    pub fn interpret_type_node(&mut self, type_node: &TypeNode) -> crate::Result<TypeHandle> {
        let type_path = self.type_path_for_type_node(type_node)?;
        self.get_path_type(&type_path)
    }

    pub fn interpret_node_as_type(&mut self, node: &Node) -> crate::Result<TypeHandle> {
        let type_path = self.type_path_for_node(node)?;
        self.get_path_type(&type_path)
    }

    pub fn outline_structure_type(&mut self, name: String) -> crate::Result<TypeHandle> {
        let path = self.current_module_info().path().child(&name);
        let handle = self.create_type(path, TypeRepr::Unresolved);

        self.current_module_info_mut().define(&name, Symbol::Type(handle))?;

        Ok(handle)
    }

    pub fn get_symbol_value(&mut self, namespace: NamespaceHandle, name: &str) -> crate::Result<Value> {
        // FIXME: does not detect recursive import
        match self.namespace_info(namespace).find(name) {
            Some(Symbol::Alias(target_path)) => {
                let target_path = target_path.clone();
                self.get_path_value(&target_path)
            }
            Some(Symbol::Module(namespace)) => {
                Ok(Value::Module(*namespace))
            }
            Some(Symbol::Type(handle)) => {
                Ok(Value::Type(*handle))
            }
            Some(Symbol::Value(value)) => {
                Ok(value.clone())
            }
            None => {
                let glob_imports = self.namespace_info(namespace).glob_imports().to_vec();

                let search_results: Vec<(AbsolutePath, Value)> = glob_imports
                    .into_iter()
                    .filter_map(|glob_import_path| {
                        let test_path = glob_import_path.into_child(name);
                        self.get_path_value(&test_path)
                            .ok()
                            .map(|value| (test_path, value))
                    })
                    .collect();

                if search_results.is_empty() {
                    Err(Box::new(crate::Error::UndefinedGlobalSymbol {
                        namespace: self.namespace_info(namespace).path().to_string(),
                        name: name.to_string(),
                    }))
                }
                else if search_results.len() > 1 {
                    Err(Box::new(crate::Error::AmbiguousSymbol {
                        name: name.to_string(),
                        possible_paths: search_results
                            .into_iter()
                            .map(|(path, _)| path.to_string())
                            .collect(),
                    }))
                }
                else {
                    let (_, value) = search_results.into_iter().next().unwrap();
                    Ok(value)
                }
            }
        }
    }

    pub fn get_path_value(&mut self, path: &AbsolutePath) -> crate::Result<Value> {
        let mut value = match path.base_type() {
            Some(base_type) => {
                Value::Type(self.get_path_base_type(base_type)?)
            }
            None => {
                Value::Module(NamespaceHandle::ROOT)
            }
        };

        for segment in path.simple().segments() {
            let Some(namespace) = value.as_namespace(self) else {
                return Err(Box::new(crate::Error::ExpectedNamespace {
                    name: segment.clone(),
                }));
            };
            value = self.get_symbol_value(namespace, segment)?;
        }

        Ok(value)
    }

    pub fn get_path_type(&mut self, path: &AbsolutePath) -> crate::Result<TypeHandle> {
        self.get_path_value(path)?.as_type().ok_or_else(|| Box::new(crate::Error::NonTypeSymbol {
            name: path.to_string(),
        }))
    }

    pub fn get_path_base_type(&mut self, base_type: &PathBaseType) -> crate::Result<TypeHandle> {
        if let Some(handle) = self.path_base_types.get(base_type) {
            Ok(*handle)
        }
        else {
            let handle = match base_type {
                PathBaseType::Primitive(primitive) => {
                    // This should not occur since all valid primitive types were added to
                    // self.path_base_types during initialization.
                    panic!("invalid primitive type '{primitive}'")
                }
                PathBaseType::Pointer { pointee_type, semantics } => {
                    let pointee_type = self.get_path_type(&pointee_type)?;
                    self.get_pointer_type(pointee_type, *semantics)
                }
                PathBaseType::Array { item_type, length } => {
                    let item_type = self.get_path_type(&item_type)?;
                    self.get_array_type(item_type, *length)
                }
                PathBaseType::Function { parameter_types, is_variadic, return_type } => {
                    let parameter_types = Result::from_iter(parameter_types
                        .iter()
                        .map(|parameter_type| self.get_path_type(parameter_type)))?;
                    let return_type = self.get_path_type(&return_type)?;
                    let signature = FunctionSignature::new(return_type, parameter_types, *is_variadic);
                    self.get_function_type(&signature)
                }
            };
            self.path_base_types.insert(base_type.clone(), handle);
            Ok(handle)
        }
    }

    pub fn get_pointer_type(&mut self, pointee_type: TypeHandle, semantics: PointerSemantics) -> TypeHandle {
        if let Some(handle) = self.pointer_types.get(&(pointee_type, semantics)) {
            *handle
        }
        else {
            let info = TypeRepr::Pointer {
                pointee_type,
                semantics,
            };
            let path = AbsolutePath::at_base_type(Box::new(PathBaseType::Pointer {
                pointee_type: self.type_path(pointee_type).clone(),
                semantics,
            }));

            let handle = self.create_type(path, info);
            self.pointer_types.insert((pointee_type, semantics), handle);
            handle
        }
    }

    pub fn get_array_type(&mut self, item_type: TypeHandle, length: Option<usize>) -> TypeHandle {
        if let Some(handle) = self.array_types.get(&(item_type, length)) {
            *handle
        }
        else {
            let info = TypeRepr::Array {
                item_type,
                length,
            };
            let path = AbsolutePath::at_base_type(Box::new(PathBaseType::Array {
                item_type: self.type_path(item_type).clone(),
                length,
            }));

            let handle = self.create_type(path, info);
            self.array_types.insert((item_type, length), handle);
            handle
        }
    }

    pub fn get_function_type(&mut self, signature: &FunctionSignature) -> TypeHandle {
        if let Some(handle) = self.function_types.get(signature) {
            *handle
        }
        else {
            let info = TypeRepr::Function {
                signature: signature.clone(),
            };
            let path = AbsolutePath::at_base_type(Box::new(PathBaseType::Function {
                parameter_types: signature.parameter_types()
                    .iter()
                    .map(|&handle| self.type_path(handle).clone())
                    .collect(),
                is_variadic: signature.is_variadic(),
                return_type: self.type_path(signature.return_type()).clone(),
            }));

            let handle = self.create_type(path, info);
            self.function_types.insert(signature.clone(), handle);
            handle
        }
    }

    pub fn types_are_equivalent(&self, lhs_type: TypeHandle, rhs_type: TypeHandle) -> bool {
        lhs_type == rhs_type
    }

    pub fn can_coerce_type(&self, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> bool {
        from_type == to_type || match (self.type_repr(from_type), self.type_repr(to_type)) {
            (
                &TypeRepr::Pointer { pointee_type: from_pointee, semantics: from_semantics },
                &TypeRepr::Pointer { pointee_type: to_pointee, semantics: to_semantics },
            ) => {
                use PointerSemantics::*;
                match (from_semantics, to_semantics) {
                    (Immutable, Immutable) => self.can_coerce_type(from_pointee, to_pointee, false),
                    (Immutable, Mutable) => false,
                    (Mutable, Immutable) => self.can_coerce_type(from_pointee, to_pointee, true),
                    (Mutable, Mutable) => from_mutable && self.can_coerce_type(from_pointee, to_pointee, true),
                }
            }
            (
                &TypeRepr::Array { item_type: from_item, length: Some(from_length) },
                &TypeRepr::Array { item_type: to_item, length: Some(to_length) },
            ) => {
                from_length == to_length && self.can_coerce_type(from_item, to_item, from_mutable)
            }
            (
                &TypeRepr::Array { item_type: from_item, length: _ },
                &TypeRepr::Array { item_type: to_item, length: None },
            ) => {
                self.can_coerce_type(from_item, to_item, from_mutable)
            }
            (
                TypeRepr::Function { signature: from_signature },
                TypeRepr::Function { signature: to_signature },
            ) => {
                from_signature.is_variadic() == to_signature.is_variadic()
                    && self.can_coerce_type(from_signature.return_type(), to_signature.return_type(), false)
                    && from_signature.parameter_types().len() == to_signature.parameter_types().len()
                    && std::iter::zip(from_signature.parameter_types(), to_signature.parameter_types())
                        .all(|(&from_param, &to_param)| {
                            self.can_coerce_type(from_param, to_param, false)
                        })
            }
            (TypeRepr::Void, _) | (_, TypeRepr::Void) => true,
            _ => false
        }
    }

    pub fn process_global_statements<'a>(&mut self, top_level_statements: impl IntoIterator<Item = &'a mut Node>) -> crate::Result<()> {
        for top_level_statement in top_level_statements {
            self.process_global_statement(top_level_statement)?;
        }

        self.complete_fill_phase()
    }

    pub fn process_global_statement(&mut self, node: &mut Node) -> crate::Result<()> {
        match *node {
            Node::Let { ref name, ref value_type, is_mutable, ref mut global_register, .. } => {
                let Some(value_type) = value_type else {
                    return Err(Box::new(crate::Error::MustSpecifyTypeForGlobal {
                        name: self.current_namespace_info().path().child(name).to_string(),
                    }));
                };
                let value_type = self.interpret_type_node(value_type)?;
                *global_register = Some(self.define_global_value(name, value_type, is_mutable)?);
            }
            Node::Constant { ref name, ref value_type, ref mut global_register, .. } => {
                let value_type = self.interpret_type_node(value_type)?;
                *global_register = Some(self.define_global_value(name, value_type, false)?);
            }
            Node::Function { ref name, ref parameters, is_variadic, ref return_type, ref mut global_register, is_foreign, .. } => {
                let parameter_types = Result::from_iter(parameters
                    .iter()
                    .map(|parameter| {
                        self.interpret_type_node(&parameter.type_node)
                    }))?;
                let return_type = self.interpret_type_node(return_type)?;
                let signature = FunctionSignature::new(return_type, parameter_types, is_variadic);
                let function_type = self.get_function_type(&signature);

                let identifier = if is_foreign {
                    name.clone()
                } else {
                    self.current_namespace_info().path().child(name).to_string()
                };
                let register = Register::new_global(identifier, function_type);

                self.current_namespace_info_mut().define(
                    name,
                    Symbol::Value(Value::Constant(Constant::Register(register.clone()))),
                )?;

                *global_register = Some(register);
            }
            Node::Structure { ref name, ref members, self_type, is_foreign } => {
                if is_foreign {
                    self.update_type_repr(self_type, TypeRepr::ForeignStructure {
                        name: name.clone(),
                    });
                }
                else {
                    self.set_self_type(self_type);

                    let members = crate::Result::from_iter(members
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|member| Ok(StructureMember {
                            name: member.name.clone(),
                            member_type: self.interpret_type_node(&member.type_node)?,
                        })))?;
                    self.update_type_repr(self_type, TypeRepr::Structure {
                        name: name.clone(),
                        members,
                    });

                    self.unset_self_type();
                }
            }
            Node::Implement { ref self_type, ref mut statements, .. } => {
                let self_type = self.interpret_type_node(self_type)?;
                self.set_self_type(self_type);

                for statement in statements {
                    self.process_global_statement(statement)?;
                }

                self.unset_self_type();
            }
            Node::Module { ref mut statements, namespace, .. } => {
                self.enter_module(namespace);

                for statement in statements {
                    self.process_global_statement(statement)?;
                }

                self.exit_module();
            }
            _ => {}
        }

        Ok(())
    }

    fn define_global_value(&mut self, name: &str, value_type: TypeHandle, is_mutable: bool) -> crate::Result<Register> {
        let path = self.current_module_info().path().child(name);
        let pointer_type = self.get_pointer_type(value_type, PointerSemantics::from_flag(is_mutable));
        let register = Register::new_global(path.to_string(), pointer_type);

        self.current_namespace_info_mut().define(
            name,
            Symbol::Value(Value::Constant(Constant::Indirect {
                pointee_type: value_type,
                pointer: Box::new(Constant::Register(register.clone())),
            })),
        )?;

        Ok(register)
    }

    pub fn complete_fill_phase(&mut self) -> crate::Result<()> {
        let mut dependency_stack = Vec::new();
        for registry_index in 0 .. self.type_registry.len() {
            self.calculate_type_properties(
                TypeHandle::new(registry_index),
                true,
                true,
                true,
                &mut dependency_stack,
            )?;
            if !dependency_stack.is_empty() {
                panic!("dependencies were handled incorrectly while calculating properties for type {registry_index}");
            }
        }

        self.fill_phase_complete = true;
        Ok(())
    }

    fn calculate_type_properties(
        &mut self,
        handle: TypeHandle,
        get_alignment: bool,
        get_size: bool,
        get_llvm_syntax: bool,
        dependency_stack: &mut Vec<TypeHandle>,
    ) -> crate::Result<()> {
        let entry = self.type_entry(handle);
        let get_alignment = get_alignment && entry.alignment.is_none();
        let get_size = get_size && entry.size.is_none();
        let get_llvm_syntax = get_llvm_syntax && entry.llvm_syntax.is_none();

        if !get_alignment && !get_size && !get_llvm_syntax {
            return Ok(());
        }

        if dependency_stack.contains(&handle) {
            return Err(Box::new(crate::Error::RecursiveTypeDefinition {
                type_name: self.type_path(handle).to_string(),
            }));
        }
        dependency_stack.push(handle);

        let repr = self.type_repr(handle).clone();
        match repr {
            TypeRepr::Unresolved => {
                panic!("{handle:?} is still unresolved at the end of the fill phase");
            }
            TypeRepr::Pointer { pointee_type, .. } => {
                self.calculate_type_properties(
                    pointee_type,
                    false,
                    false,
                    get_llvm_syntax,
                    dependency_stack,
                )?;
            }
            TypeRepr::Array { item_type, length } => {
                self.calculate_type_properties(
                    item_type,
                    get_alignment,
                    get_size && length.is_some(),
                    get_llvm_syntax,
                    dependency_stack,
                )?;
            }
            TypeRepr::Structure { ref members, .. } => {
                for member in members {
                    self.calculate_type_properties(
                        member.member_type,
                        get_alignment || get_size,
                        get_size,
                        false,
                        dependency_stack,
                    )?;
                }
            }
            TypeRepr::Function { ref signature } if get_llvm_syntax => {
                for &parameter_type in signature.parameter_types() {
                    self.calculate_type_properties(
                        parameter_type,
                        false,
                        false,
                        get_llvm_syntax,
                        dependency_stack,
                    )?;
                }
                self.calculate_type_properties(
                    signature.return_type(),
                    false,
                    false,
                    get_llvm_syntax,
                    dependency_stack,
                )?;
            }
            _ => {}
        }

        dependency_stack.pop();

        if get_alignment {
            let alignment = self.calculate_alignment(&repr);
            self.type_entry_mut(handle).alignment = Some(alignment);
        }
        if get_size {
            let size = self.calculate_size(&repr);
            self.type_entry_mut(handle).size = Some(size);
        }
        if get_llvm_syntax {
            let identifier = self.type_path(handle).to_string();
            let llvm_syntax = self.generate_type_llvm_syntax(&identifier, &repr);
            self.type_entry_mut(handle).llvm_syntax = Some(llvm_syntax);
        }

        Ok(())
    }

    fn create_namespace(&mut self, path: AbsolutePath) -> NamespaceHandle {
        let handle = NamespaceHandle::new(self.namespace_registry.len());

        self.namespace_registry.push(NamespaceInfo::new(path));

        handle
    }

    fn create_type(&mut self, path: AbsolutePath, repr: TypeRepr) -> TypeHandle {
        let handle = TypeHandle::new(self.type_registry.len());

        let namespace = self.create_namespace(path.clone());
        let alignment = self.fill_phase_complete.then(|| self.calculate_alignment(&repr));
        let size = self.fill_phase_complete.then(|| self.calculate_size(&repr));
        let llvm_syntax = self.fill_phase_complete.then(|| self.generate_type_llvm_syntax(&path.to_string(), &repr));

        self.type_registry.push(TypeEntry {
            path,
            repr,
            namespace,
            alignment,
            size,
            llvm_syntax,
        });

        handle
    }

    fn update_type_repr(&mut self, handle: TypeHandle, repr: TypeRepr) {
        let alignment = self.fill_phase_complete.then(|| self.calculate_alignment(&repr));
        let size = self.fill_phase_complete.then(|| self.calculate_size(&repr));
        let llvm_syntax = self.fill_phase_complete.then(|| {
            let identifier = self.type_path(handle).to_string();
            self.generate_type_llvm_syntax(&identifier, &repr)
        });

        let entry = self.type_entry_mut(handle);
        entry.repr = repr;
        entry.alignment = alignment;
        entry.size = size;
        entry.llvm_syntax = llvm_syntax;
    }

    fn calculate_alignment(&self, repr: &TypeRepr) -> Option<usize> {
        match *repr {
            TypeRepr::Unresolved => None,
            TypeRepr::Meta => None,
            TypeRepr::Never => None,
            TypeRepr::Void => None,
            TypeRepr::Boolean => Some(1),
            TypeRepr::Integer { size, .. } => Some(size),
            TypeRepr::Pointer { .. } => Some(self.pointer_size),
            TypeRepr::Function { .. } => Some(self.pointer_size),
            TypeRepr::Array { item_type, .. } => self.type_alignment(item_type),
            TypeRepr::Structure { ref members, .. } => members.iter()
                .map(|member| self.type_alignment(member.member_type))
                .max().unwrap_or(Some(1)),
            TypeRepr::ForeignStructure { .. } => None,
        }
    }

    fn calculate_size(&self, repr: &TypeRepr) -> Option<usize> {
        match *repr {
            TypeRepr::Unresolved => None,
            TypeRepr::Meta => None,
            TypeRepr::Never => Some(0),
            TypeRepr::Void => Some(0),
            TypeRepr::Boolean => Some(1),
            TypeRepr::Integer { size, .. } => Some(size),
            TypeRepr::Pointer { .. } => Some(self.pointer_size),
            TypeRepr::Function { .. } => Some(self.pointer_size),
            TypeRepr::Array { item_type, length } => {
                let length = length?;
                let item_size = self.type_size(item_type)?;
                Some(length * item_size)
            }
            TypeRepr::Structure { ref members, .. } => {
                let mut current_size = 0;
                let mut max_alignment = 1;

                for member in members {
                    let alignment = self.type_alignment(member.member_type).unwrap_or(0);
                    max_alignment = max_alignment.max(alignment);

                    // Calculate padding
                    let intermediate_size = current_size + alignment - 1;
                    let padded_size = intermediate_size - intermediate_size % alignment;
                    current_size = padded_size + self.type_size(member.member_type)?;
                }

                // Pad for the largest member alignment
                let intermediate_size = current_size + max_alignment - 1;
                let padded_size = intermediate_size - intermediate_size % max_alignment;

                Some(padded_size)
            }
            TypeRepr::ForeignStructure { .. } => None,
        }
    }

    fn generate_type_llvm_syntax(&self, identifier: &str, repr: &TypeRepr) -> String {
        match *repr {
            TypeRepr::Unresolved => "<ERROR unresolved type>".to_owned(),
            TypeRepr::Meta => "<ERROR meta type>".to_owned(),
            TypeRepr::Never => "void".to_owned(),
            TypeRepr::Void => "void".to_owned(),
            TypeRepr::Boolean => "i1".to_owned(),
            TypeRepr::Integer { size, .. } => format!("i{}", size * 8),
            TypeRepr::Pointer { pointee_type, .. } => {
                match self.type_llvm_syntax(pointee_type) {
                    "void" => "{}*".to_owned(),
                    pointee_syntax => format!("{pointee_syntax}*")
                }
            }
            TypeRepr::Array { item_type, length } => match length {
                Some(length) => format!("[{} x {}]", length, self.type_llvm_syntax(item_type)),
                None => self.type_llvm_syntax(item_type).to_owned(),
            }
            TypeRepr::Structure { .. } | TypeRepr::ForeignStructure { .. } => {
                format!("%\"type.{identifier}\"")
            }
            TypeRepr::Function { ref signature } => {
                let mut syntax = format!("{}(", self.type_llvm_syntax(signature.return_type()));
                let mut parameters_iter = signature.parameter_types().iter();
                if let Some(&parameter) = parameters_iter.next() {
                    syntax.push_str(self.type_llvm_syntax(parameter));
                    for &parameter in parameters_iter {
                        syntax.push_str(", ");
                        syntax.push_str(self.type_llvm_syntax(parameter));
                    }
                    if signature.is_variadic() {
                        syntax.push_str(", ...");
                    }
                }
                else if signature.is_variadic() {
                    syntax.push_str("...");
                }
                syntax.push_str(")*");
                syntax
            }
        }
    }
}
