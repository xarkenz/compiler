use crate::ast::{Node, NodeKind, PathSegment, TypeNode, TypeNodeKind};
use crate::ast::parse::ParsedModule;
use crate::target::TargetInfo;
use crate::token::Literal;
use std::path::Path;

mod local;
pub use local::*;

mod symbol;
pub use symbol::*;

mod types;
pub use types::*;

mod value;
pub use value::*;

mod package;
pub use package::*;

pub struct GlobalContext {
    target: TargetInfo,
    package_manager: PackageManager,
    namespace_registry: NamespaceRegistry,
    type_registry: TypeRegistry,
    package: PackageContext,
}

impl GlobalContext {
    pub fn new(package_path: impl AsRef<Path>, target: TargetInfo) -> crate::Result<Self> {
        let mut package_manager = PackageManager::generate(package_path)?;
        let package_info = package_manager.get_next_to_compile()
            .expect("there should be at least one package to compile");

        let mut namespace_registry = NamespaceRegistry::new();
        let package = namespace_registry.create_package_context(package_info);

        let mut type_registry = TypeRegistry::new();
        type_registry.populate_primitive_types(
            |path| namespace_registry.create_namespace(path),
            &target,
        );

        Ok(Self {
            target,
            package_manager,
            namespace_registry,
            type_registry,
            package,
        })
    }

    pub fn target(&self) -> &TargetInfo {
        &self.target
    }

    pub fn package_manager(&self) -> &PackageManager {
        &self.package_manager
    }

    pub fn package_manager_mut(&mut self) -> &mut PackageManager {
        &mut self.package_manager
    }

    pub fn namespace_registry(&self) -> &NamespaceRegistry {
        &self.namespace_registry
    }

    pub fn namespace_registry_mut(&mut self) -> &mut NamespaceRegistry {
        &mut self.namespace_registry
    }

    pub fn type_registry(&self) -> &TypeRegistry {
        &self.type_registry
    }

    pub fn type_registry_mut(&mut self) -> &mut TypeRegistry {
        &mut self.type_registry
    }

    pub fn package(&self) -> &PackageContext {
        &self.package
    }

    pub fn package_mut(&mut self) -> &mut PackageContext {
        &mut self.package
    }

    pub fn start_next_package(&mut self) -> bool {
        self.namespace_registry.finish_package();
        self.type_registry.finish_package();

        if let Some(package_info) = self.package_manager.get_next_to_compile() {
            self.package = self.namespace_registry.create_package_context(package_info);
            true
        }
        else {
            false
        }
    }

    pub fn prepare_next_source(&mut self) -> crate::Result<Option<(usize, NamespaceHandle)>> {
        loop {
            let Some(module_path) = self.package.get_next_module_to_parse() else {
                break Ok(None);
            };
            let file_path = self.package.get_file_path_for_module(&module_path);
            if let Some(source_id) = self.package.register_source_path(file_path) {
                let namespace = module_path.segments().iter().try_fold(
                    NamespaceHandle::GLOBAL_ROOT,
                    |parent_namespace, segment| {
                        self.get_or_create_module(parent_namespace, segment)
                    },
                )?;
                break Ok(Some((source_id, namespace)));
            }
        }
    }

    pub fn queue_module_file(&mut self, name: impl Into<Box<str>>) {
        let module_path = self.current_namespace_info().path().simple().child(name);
        self.package.queue_module_file(module_path);
    }

    pub fn namespace_info(&self, handle: NamespaceHandle) -> &NamespaceInfo {
        self.namespace_registry.namespace_info(handle)
    }

    pub fn namespace_info_mut(&mut self, handle: NamespaceHandle) -> &mut NamespaceInfo {
        self.namespace_registry.namespace_info_mut(handle)
    }

    pub fn type_path(&self, handle: TypeHandle) -> &AbsolutePath {
        self.type_registry.type_path(handle)
    }

    pub fn type_repr(&self, handle: TypeHandle) -> &TypeRepr {
        self.type_registry.type_repr(handle)
    }

    pub fn type_namespace(&self, handle: TypeHandle) -> NamespaceHandle {
        self.type_registry.type_namespace(handle)
    }

    pub fn type_alignment(&self, handle: TypeHandle) -> Option<u64> {
        self.type_registry.type_alignment(handle)
    }

    pub fn type_size(&self, handle: TypeHandle) -> Option<u64> {
        self.type_registry.type_size(handle)
    }

    pub fn type_llvm_syntax(&self, handle: TypeHandle) -> &str {
        self.type_registry.type_llvm_syntax(handle)
    }

    pub fn current_module(&self) -> NamespaceHandle {
        self.package.current_module()
    }

    pub fn current_module_info(&self) -> &NamespaceInfo {
        self.namespace_info(self.package.current_module())
    }

    pub fn current_module_info_mut(&mut self) -> &mut NamespaceInfo {
        self.namespace_info_mut(self.package.current_module())
    }

    pub fn current_namespace(&self) -> NamespaceHandle {
        if let Some(self_type) = self.package.current_self_type() {
            self.type_namespace(self_type)
        }
        else {
            self.package.current_module()
        }
    }

    pub fn current_namespace_info(&self) -> &NamespaceInfo {
        self.namespace_info(self.current_namespace())
    }

    pub fn current_namespace_info_mut(&mut self) -> &mut NamespaceInfo {
        self.namespace_info_mut(self.current_namespace())
    }

    pub fn get_or_create_module(&mut self, parent_module: NamespaceHandle, name: &str) -> crate::Result<NamespaceHandle> {
        let parent_module_info = self.namespace_info(parent_module);

        if let Some(namespace) = parent_module_info.find(name).and_then(Symbol::as_module) {
            // Extend the existing module
            Ok(namespace)
        }
        else {
            // Create a new module
            let module_path = parent_module_info.path().child(name);
            let namespace = self.namespace_registry.create_namespace(module_path);

            self.namespace_info_mut(parent_module).define(name, Symbol::new(SymbolKind::Module(namespace)))?;

            Ok(namespace)
        }
    }

    pub fn replace_current_module(&mut self, module: NamespaceHandle) -> NamespaceHandle {
        self.package.replace_current_module(module)
    }

    pub fn current_self_type(&self) -> Option<TypeHandle> {
        self.package.current_self_type()
    }

    pub fn set_self_type(&mut self, self_type: TypeHandle) {
        self.package.set_self_type(self_type);
    }

    pub fn unset_self_type(&mut self) -> TypeHandle {
        self.package.unset_self_type()
    }

    pub fn get_absolute_path(&self, path_span: crate::Span, segments: &[PathSegment]) -> crate::Result<AbsolutePath> {
        segments.iter().try_fold(
            self.current_module_info().path().clone(),
            |path, segment| match *segment {
                PathSegment::Name(ref name) => {
                    Ok(path.into_child(name.clone()))
                }
                PathSegment::RootModule => {
                    Ok(AbsolutePath::at_root())
                }
                PathSegment::SuperModule => {
                    path.parent()
                        .ok_or_else(|| Box::new(crate::Error::new(
                            Some(path_span),
                            crate::ErrorKind::InvalidSuper {
                                namespace: path.to_string(),
                            },
                        )))
                }
                PathSegment::SelfModule => {
                    Ok(path)
                }
                PathSegment::SelfType => {
                    self.current_self_type()
                        .ok_or_else(|| Box::new(crate::Error::new(
                            Some(path_span),
                            crate::ErrorKind::NoSelfType,
                        )))
                        .map(|self_type| self.type_path(self_type).clone())
                }
                PathSegment::PrimitiveType(primitive_type) => {
                    Ok(AbsolutePath::at_base_type(
                        Box::new(PathBaseType::Primitive(primitive_type)),
                    ))
                }
                PathSegment::Type(ref type_node) => {
                    self.type_path_for_type_node(type_node)
                }
            },
        )
    }

    pub fn type_path_for_type_node(&self, type_node: &TypeNode) -> crate::Result<AbsolutePath> {
        match type_node.kind() {
            TypeNodeKind::Path { segments } => {
                Ok(self.get_absolute_path(type_node.span(), segments)?)
            }
            TypeNodeKind::Pointer { pointee_type, semantics } => {
                let base_type = PathBaseType::Pointer {
                    pointee_type: self.type_path_for_type_node(pointee_type)?,
                    semantics: *semantics,
                };

                Ok(AbsolutePath::at_base_type(Box::new(base_type)))
            }
            TypeNodeKind::Array { item_type, length } => {
                let base_type = PathBaseType::Array {
                    item_type: self.type_path_for_type_node(item_type)?,
                    length: match length {
                        Some(node) => {
                            // Must be an integer literal (for now)
                            if let &NodeKind::Literal(Literal::Integer(raw, _)) = node.kind() {
                                // Must be an acceptable usize value
                                let Some(value) = IntegerValue::from_unknown_type(raw, TypeHandle::USIZE, self.target()) else {
                                    return Err(Box::new(crate::Error::new(
                                        Some(node.span()),
                                        crate::ErrorKind::IncompatibleValueType {
                                            value: raw.to_string(),
                                            type_name: self.type_path(TypeHandle::USIZE).to_string(),
                                        },
                                    )));
                                };
                                Some(value.raw() as u64)
                            }
                            else {
                                return Err(Box::new(crate::Error::new(
                                    Some(node.span()),
                                    crate::ErrorKind::NonConstantArrayLength,
                                )));
                            }
                        },
                        None => None,
                    },
                };

                Ok(AbsolutePath::at_base_type(Box::new(base_type)))
            }
            TypeNodeKind::Tuple { item_types } => {
                let base_type = PathBaseType::Tuple {
                    item_types: Result::from_iter(item_types
                        .iter()
                        .map(|item_type| self.type_path_for_type_node(item_type)))?,
                };

                Ok(AbsolutePath::at_base_type(Box::new(base_type)))
            }
            TypeNodeKind::Function { parameter_types, is_variadic, return_type } => {
                let base_type = PathBaseType::Function {
                    parameter_types: Result::from_iter(parameter_types
                        .iter()
                        .map(|parameter_type| self.type_path_for_type_node(parameter_type)))?,
                    is_variadic: *is_variadic,
                    return_type: self.type_path_for_type_node(return_type)?,
                };

                Ok(AbsolutePath::at_base_type(Box::new(base_type)))
            }
            TypeNodeKind::Grouping { content } => {
                self.type_path_for_type_node(content)
            }
        }
    }

    pub fn type_path_for_node(&self, node: &Node) -> crate::Result<AbsolutePath> {
        match node.kind() {
            NodeKind::Type(type_node) => {
                self.type_path_for_type_node(type_node)
            }
            NodeKind::Path { segments, .. } => {
                self.get_absolute_path(node.span(), segments)
            }
            NodeKind::Literal(Literal::Name(name)) => {
                Ok(self.current_module_info().path().child(name.clone()))
            }
            NodeKind::Literal(Literal::PrimitiveType(primitive_type)) => {
                let base_type = PathBaseType::Primitive(*primitive_type);
                Ok(AbsolutePath::at_base_type(Box::new(base_type)))
            }
            _ => {
                Err(Box::new(crate::Error::new(
                    Some(node.span()),
                    crate::ErrorKind::UnexpectedExpression,
                )))
            }
        }
    }

    pub fn interpret_type_node(&mut self, type_node: &TypeNode) -> crate::Result<TypeHandle> {
        let type_path = self.type_path_for_type_node(type_node)?;
        self.get_path_type(&type_path, Some(&type_node.span()))
    }

    pub fn interpret_node_as_type(&mut self, node: &Node) -> crate::Result<TypeHandle> {
        let type_path = self.type_path_for_node(node)?;
        self.get_path_type(&type_path, Some(&node.span()))
    }

    pub fn outline_structure_type(&mut self, name: Box<str>) -> crate::Result<TypeHandle> {
        let path = self.current_module_info().path().child(name.clone());
        let handle = self.type_registry.create_type(
            path.clone(),
            TypeRepr::Unresolved,
            self.namespace_registry.create_namespace(path),
            &self.target,
            self.package.fill_phase_complete(),
        );

        self.current_module_info_mut().define(&name, Symbol::new(SymbolKind::Type(handle)))?;

        Ok(handle)
    }

    pub fn get_symbol_value(&mut self, namespace: NamespaceHandle, name: &str, span: Option<&crate::Span>) -> crate::Result<Value> {
        // FIXME: does not detect recursive import
        if let Some(symbol) = self.namespace_info(namespace).find(name) {
            let is_external = symbol.is_external();

            let value = match symbol.kind() {
                SymbolKind::Alias(target_path) => {
                    let target_path = target_path.clone();
                    self.get_path_value(&target_path, span)?
                }
                SymbolKind::Module(namespace) => {
                    Value::Constant(Constant::Module(*namespace))
                }
                SymbolKind::Type(handle) => {
                    Value::Constant(Constant::Type(*handle))
                }
                SymbolKind::Value(value) => {
                    value.clone()
                }
            };

            if is_external {
                self.use_external_value(&value);
            }

            Ok(value)
        }
        else {
            let glob_imports = self.namespace_info(namespace)
                .glob_imports()
                .to_vec();

            // TODO: wtf, kinda
            let search_results: Vec<(AbsolutePath, Value)> = glob_imports
                .into_iter()
                .map(|glob_import_path| glob_import_path.into_child(name))
                .chain((namespace != NamespaceHandle::GLOBAL_ROOT).then(|| {
                    AbsolutePath::at_root().into_child(name)
                }))
                .filter_map(|test_path| {
                    self.get_path_value(&test_path, span)
                        .ok()
                        .map(|value| (test_path, value))
                })
                .collect();

            if search_results.is_empty() {
                Err(Box::new(crate::Error::new(
                    span.copied(),
                    crate::ErrorKind::UndefinedGlobalSymbol {
                        namespace: self.namespace_info(namespace).path().to_string(),
                        name: name.to_string(),
                    },
                )))
            }
            else if search_results.len() > 1 {
                Err(Box::new(crate::Error::new(
                    span.copied(),
                    crate::ErrorKind::AmbiguousSymbol {
                        name: name.to_string(),
                        possible_paths: search_results
                            .into_iter()
                            .map(|(path, _)| path.to_string())
                            .collect(),
                    },
                )))
            }
            else {
                let (_, value) = search_results.into_iter().next().unwrap();
                Ok(value)
            }
        }
    }

    pub fn get_path_value(&mut self, path: &AbsolutePath, span: Option<&crate::Span>) -> crate::Result<Value> {
        path.simple().segments().iter().try_fold(
            match path.base_type() {
                Some(base_type) => {
                    Value::Constant(Constant::Type(self.get_path_base_type(base_type, span)?))
                }
                None => {
                    Value::Constant(Constant::Module(NamespaceHandle::GLOBAL_ROOT))
                }
            },
            |value, segment| {
                value.as_namespace(self)
                    .ok_or_else(|| Box::new(crate::Error::new(
                        span.copied(),
                        crate::ErrorKind::ExpectedNamespace {
                            name: segment.to_string(),
                        },
                    )))
                    .and_then(|namespace| {
                        self.get_symbol_value(namespace, segment, span)
                    })
            },
        )
    }

    /// Get the type handle correponding to the given path.
    pub fn get_path_type(&mut self, path: &AbsolutePath, span: Option<&crate::Span>) -> crate::Result<TypeHandle> {
        self.get_path_value(path, span)?
            .as_type()
            .ok_or_else(|| Box::new(crate::Error::new(
                span.copied(),
                crate::ErrorKind::NonTypeSymbol {
                    name: path.to_string(),
                },
            )))
    }

    /// Get the type handle for a base type of a path.
    pub fn get_path_base_type(&mut self, base_type: &PathBaseType, span: Option<&crate::Span>) -> crate::Result<TypeHandle> {
        if let Some(handle) = self.type_registry.path_base_type(base_type) {
            Ok(handle)
        }
        else {
            let handle = match base_type {
                PathBaseType::Primitive(primitive) => {
                    // This should not occur since all valid primitive types were added to
                    // self.path_base_types during initialization.
                    panic!("invalid primitive type '{primitive}'")
                }
                PathBaseType::Pointer { pointee_type, semantics } => {
                    let pointee_type = self.get_path_type(pointee_type, span)?;
                    self.get_pointer_type(pointee_type, *semantics)
                }
                PathBaseType::Array { item_type, length } => {
                    let item_type = self.get_path_type(item_type, span)?;
                    self.get_array_type(item_type, *length)
                }
                PathBaseType::Tuple { item_types } => {
                    let item_types: Vec<TypeHandle> = Result::from_iter(item_types
                        .iter()
                        .map(|item_type| self.get_path_type(item_type, span)))?;
                    self.get_tuple_type(&item_types)
                }
                PathBaseType::Function { parameter_types, is_variadic, return_type } => {
                    let parameter_types = Result::from_iter(parameter_types
                        .iter()
                        .map(|parameter_type| self.get_path_type(parameter_type, span)))?;
                    let return_type = self.get_path_type(return_type, span)?;
                    let signature = FunctionSignature::new(return_type, parameter_types, *is_variadic);
                    self.get_function_type(&signature)
                }
            };
            self.type_registry.set_path_base_type(base_type.clone(), handle);
            Ok(handle)
        }
    }

    /// Get `*T` or `*mut T` from `T` and the pointer semantics.
    pub fn get_pointer_type(&mut self, pointee_type: TypeHandle, semantics: PointerSemantics) -> TypeHandle {
        self.type_registry.get_pointer_type(
            pointee_type,
            semantics,
            |path| self.namespace_registry.create_namespace(path),
            &self.target,
            self.package.fill_phase_complete(),
        )
    }

    /// Get `[T; N]` or `[T]` from `T` and an optional array length.
    pub fn get_array_type(&mut self, item_type: TypeHandle, length: Option<u64>) -> TypeHandle {
        self.type_registry.get_array_type(
            item_type,
            length,
            |path| self.namespace_registry.create_namespace(path),
            &self.target,
            self.package.fill_phase_complete(),
        )
    }

    /// Get a tuple type from its item types.
    pub fn get_tuple_type(&mut self, item_types: &[TypeHandle]) -> TypeHandle {
        self.type_registry.get_tuple_type(
            item_types,
            |path| self.namespace_registry.create_namespace(path),
            &self.target,
            self.package.fill_phase_complete(),
        )
    }

    /// Get a function type from its signature.
    pub fn get_function_type(&mut self, signature: &FunctionSignature) -> TypeHandle {
        self.type_registry.get_function_type(
            signature,
            |path| self.namespace_registry.create_namespace(path),
            &self.target,
            self.package.fill_phase_complete(),
        )
    }

    pub fn try_implicit_conversion(&self, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> Option<Conversion> {
        self.type_registry.try_implicit_conversion(from_type, to_type, from_mutable)
    }

    pub fn try_explicit_conversion(&self, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> Option<Conversion> {
        self.type_registry.try_explicit_conversion(from_type, to_type, from_mutable)
    }

    pub fn process_package<'a, I>(&mut self, modules: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = &'a mut ParsedModule>,
    {
        for parsed_module in modules {
            let previous_module = self.replace_current_module(parsed_module.namespace());

            self.process_global_statements(parsed_module.statements_mut())?;

            self.replace_current_module(previous_module);
        }

        self.complete_fill_phase()
    }

    pub fn process_global_statements<'a, I>(&mut self, global_statements: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = &'a mut Node>,
    {
        for global_statement in global_statements {
            self.process_global_statement(global_statement)?;
        }

        Ok(())
    }

    pub fn process_global_statement(&mut self, node: &mut Node) -> crate::Result<()> {
        let node_span = node.span();
        match node.kind_mut() {
            NodeKind::Let { name, value_type, is_mutable, global_register, .. } => {
                let Some(value_type) = value_type else {
                    return Err(Box::new(crate::Error::new(
                        Some(node_span),
                        crate::ErrorKind::MustSpecifyTypeForGlobal {
                            name: self.current_namespace_info().path().child(name.clone()).to_string(),
                        },
                    )));
                };
                let value_type = self.interpret_type_node(value_type)?;
                *global_register = Some(self.define_global_value(name, value_type, *is_mutable)?);
            }
            NodeKind::Constant { name, value_type, global_register, .. } => {
                let value_type = self.interpret_type_node(value_type)?;
                *global_register = Some(self.define_global_value(name, value_type, false)?);
            }
            NodeKind::Function { name, parameters, is_variadic, return_type, global_register, is_foreign, .. } => {
                let parameter_types = parameters
                    .iter()
                    .map(|parameter| {
                        self.interpret_type_node(&parameter.type_node)
                    })
                    .collect::<crate::Result<_>>()?;
                let return_type = self.interpret_type_node(return_type)?;
                let signature = FunctionSignature::new(return_type, parameter_types, *is_variadic);
                let function_type = self.get_function_type(&signature);

                let identifier = if *is_foreign {
                    name.to_string()
                } else {
                    self.current_namespace_info().path().child(name.clone()).to_string()
                };
                let register = Register::new_global(identifier, function_type);

                self.current_namespace_info_mut().define(
                    name,
                    Symbol::new(SymbolKind::Value(Value::Constant(Constant::Register(register.clone())))),
                )?;

                *global_register = Some(register);
            }
            NodeKind::Structure { name, members, self_type, is_foreign } => {
                if *is_foreign {
                    self.type_registry.update_type_repr(
                        *self_type,
                        TypeRepr::OpaqueStructure {
                            name: name.clone(),
                            is_external: false,
                        },
                        &self.target,
                        self.package.fill_phase_complete(),
                    );
                }
                else {
                    self.set_self_type(*self_type);

                    let members = crate::Result::from_iter(members
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|member| Ok(StructureMember {
                            name: member.name.clone(),
                            member_type: self.interpret_type_node(&member.type_node)?,
                        })))?;
                    self.type_registry.update_type_repr(
                        *self_type,
                        TypeRepr::Structure {
                            name: name.clone(),
                            members,
                            is_external: false,
                        },
                        &self.target,
                        self.package.fill_phase_complete(),
                    );

                    self.unset_self_type();
                }
            }
            NodeKind::Implement { self_type, statements, .. } => {
                let self_type = self.interpret_type_node(self_type)?;
                self.set_self_type(self_type);

                for statement in statements {
                    self.process_global_statement(statement)?;
                }

                self.unset_self_type();
            }
            NodeKind::Module { statements, namespace, .. } => {
                let parent_module = self.replace_current_module(*namespace);

                for statement in statements {
                    self.process_global_statement(statement)?;
                }

                self.replace_current_module(parent_module);
            }
            _ => {}
        }

        Ok(())
    }

    fn define_global_value(&mut self, name: &str, value_type: TypeHandle, is_mutable: bool) -> crate::Result<Register> {
        let path = self.current_module_info().path().child(name);
        let pointer_type = self.get_pointer_type(value_type, PointerSemantics::for_symbol(is_mutable));
        let register = Register::new_global(path.to_string(), pointer_type);

        self.current_namespace_info_mut().define(
            name,
            Symbol::new(SymbolKind::Value(Value::Constant(Constant::Indirect {
                pointee_type: value_type,
                pointer: Box::new(Constant::Register(register.clone())),
            }))),
        )?;

        Ok(register)
    }

    pub fn complete_fill_phase(&mut self) -> crate::Result<()> {
        self.type_registry.calculate_type_properties(&self.target)?;

        self.package.complete_fill_phase();

        Ok(())
    }

    pub fn use_external_value(&mut self, value: &Value) {
        if !self.package.use_external_value(value) {
            return;
        }

        if let &Value::Constant(Constant::Type(handle)) = value {
            if let TypeRepr::Structure { members, .. } = self.type_repr(handle) {
                let inner_external_types: Vec<TypeHandle> = members
                    .iter()
                    .flat_map(|member| self.get_inner_external_types(member.member_type))
                    .collect();
                for inner_type in inner_external_types {
                    self.use_external_value(&Value::Constant(Constant::Type(inner_type)));
                }
            }
        }
        else {
            for inner_type in self.get_inner_external_types(value.get_type()) {
                self.use_external_value(&Value::Constant(Constant::Type(inner_type)));
            }
        }
    }

    fn get_inner_external_types(&self, handle: TypeHandle) -> Vec<TypeHandle> {
        match *self.type_repr(handle) {
            TypeRepr::Structure { is_external: true, .. } |
            TypeRepr::OpaqueStructure { is_external: true, .. } => {
                vec![handle]
            }
            TypeRepr::Pointer { pointee_type, .. } => {
                self.get_inner_external_types(pointee_type)
            }
            TypeRepr::Array { item_type, .. } => {
                self.get_inner_external_types(item_type)
            }
            TypeRepr::Tuple { ref item_types, .. } => {
                item_types
                    .iter()
                    .flat_map(|&item_type| self.get_inner_external_types(item_type))
                    .collect()
            }
            TypeRepr::Function { ref signature } => {
                signature.parameter_types()
                    .iter()
                    .flat_map(|&parameter_type| self.get_inner_external_types(parameter_type))
                    .chain(self.get_inner_external_types(signature.return_type()))
                    .collect()
            }
            _ => Vec::new()
        }
    }
}
