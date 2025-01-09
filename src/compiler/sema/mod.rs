use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use crate::ast::{Node, PathSegment, TypeNode};

mod values;
mod types;
mod modules;
mod local_ctx;
mod symbols;

pub use values::*;
pub use types::*;
pub use modules::*;
pub use local_ctx::*;
pub use symbols::*;
use crate::token::Literal;

pub struct GlobalContext {
    /// The size of a pointer in bytes.
    pointer_size: usize,
    /// Table of all modules in existence.
    module_registry: Vec<ModuleInfo>,
    /// The module currently being analyzed.
    current_module: ModuleHandle,
    /// Table of all types in existence.
    type_registry: Vec<TypeEntry>,
    /// The type `Self` currently represents, or `None` if not in an `implement` block.
    current_self_type: Option<TypeHandle>,
    /// Find `*T`, `*mut T`, or `*own T`.
    pointer_types: HashMap<(TypeHandle, PointerSemantics), TypeHandle>,
    /// Find `[T; N]` or `[T]`.
    array_types: HashMap<(TypeHandle, Option<usize>), TypeHandle>,
    /// Find a function type by signature.
    function_types: HashMap<FunctionSignature, TypeHandle>,
}

struct TypeEntry {
    identifier: String,
    info: TypeInfo,
    alignment: Option<usize>,
    size: Option<usize>,
    llvm_syntax: String,
    symbol_table: SymbolTable,
}

impl GlobalContext {
    pub fn new() -> Self {
        let mut context = Self {
            pointer_size: size_of::<*const ()>(),
            module_registry: vec![ModuleInfo::new(String::new(), None)],
            current_module: ModuleHandle::ROOT,
            type_registry: Vec::with_capacity(PRIMITIVE_TYPES.len()),
            current_self_type: None,
            pointer_types: HashMap::new(),
            array_types: HashMap::new(),
            function_types: HashMap::new(),
        };

        for &(name, ref info) in PRIMITIVE_TYPES {
            context.create_type(info.clone(), name.into());
        }

        context
    }

    pub fn pointer_size(&self) -> usize {
        self.pointer_size
    }

    pub fn current_module(&self) -> ModuleHandle {
        self.current_module
    }

    pub fn module_info(&self, handle: ModuleHandle) -> &ModuleInfo {
        &self.module_registry[handle.registry_index()]
    }

    pub fn module_info_mut(&mut self, handle: ModuleHandle) -> &mut ModuleInfo {
        &mut self.module_registry[handle.registry_index()]
    }

    pub fn current_module_info(&self) -> &ModuleInfo {
        self.module_info(self.current_module())
    }

    pub fn current_module_info_mut(&mut self) -> &mut ModuleInfo {
        self.module_info_mut(self.current_module())
    }
    
    pub fn current_self_type(&self) -> Option<TypeHandle> {
        self.current_self_type
    }
    
    fn type_entry(&self, handle: TypeHandle) -> &TypeEntry {
        &self.type_registry[handle.registry_index()]
    }

    fn type_entry_mut(&mut self, handle: TypeHandle) -> &mut TypeEntry {
        &mut self.type_registry[handle.registry_index()]
    }
    
    pub fn type_identifier(&self, handle: TypeHandle) -> &str {
        &self.type_entry(handle).identifier
    }

    pub fn type_info(&self, handle: TypeHandle) -> &TypeInfo {
        &self.type_entry(handle).info
    }
    
    pub fn type_symbol_table(&self, handle: TypeHandle) -> &SymbolTable {
        &self.type_entry(handle).symbol_table
    }

    pub fn type_symbol_table_mut(&mut self, handle: TypeHandle) -> &mut SymbolTable {
        &mut self.type_entry_mut(handle).symbol_table
    }

    pub fn type_alignment(&self, handle: TypeHandle) -> Option<usize> {
        self.type_entry(handle).alignment
    }

    pub fn type_size(&self, handle: TypeHandle) -> Option<usize> {
        self.type_entry(handle).size
    }
    
    pub fn type_llvm_syntax(&self, handle: TypeHandle) -> &str {
        &self.type_entry(handle).llvm_syntax
    }
    
    pub fn get_canonical_type(&self, mut handle: TypeHandle) -> TypeHandle {
        while let &TypeInfo::Alias { target } = self.type_info(handle) {
            handle = target;
        }
        
        handle
    }

    pub fn symbol_table(&self, container: ContainerHandle) -> &SymbolTable {
        match container {
            ContainerHandle::Type(handle) => self.type_symbol_table(handle),
            ContainerHandle::Module(handle) => self.module_info(handle).symbol_table(),
        }
    }

    pub fn symbol_table_mut(&mut self, container: ContainerHandle) -> &mut SymbolTable {
        match container {
            ContainerHandle::Type(handle) => self.type_symbol_table_mut(handle),
            ContainerHandle::Module(handle) => self.module_info_mut(handle).symbol_table_mut(),
        }
    }
    
    pub fn container_identifier(&self, container: ContainerHandle) -> &str {
        match container {
            ContainerHandle::Type(handle) => self.type_identifier(handle),
            ContainerHandle::Module(handle) => self.module_info(handle).identifier(),
        }
    }
    
    pub fn current_container(&self) -> ContainerHandle {
        if let Some(self_type) = self.current_self_type() {
            self_type.into()
        }
        else {
            self.current_module().into()
        }
    }
    
    pub fn current_symbol_table(&self) -> &SymbolTable {
        if let Some(self_type) = self.current_self_type() {
            self.type_symbol_table(self_type)
        }
        else {
            self.current_module_info().symbol_table()
        }
    }
    
    pub fn current_symbol_table_mut(&mut self) -> &mut SymbolTable {
        if let Some(self_type) = self.current_self_type() {
            self.type_symbol_table_mut(self_type)
        }
        else {
            self.current_module_info_mut().symbol_table_mut()
        }
    }

    pub fn enter_module(&mut self, name: String) -> crate::Result<()> {
        let new_module_info = ModuleInfo::new(
            self.current_module_info().create_member_identifier(&name),
            Some(self.current_module()),
        );
        let new_module = ModuleHandle::new(self.module_registry.len());
        self.module_registry.push(new_module_info);

        self.define_symbol(name, new_module.into())?;
        self.current_module = new_module;

        Ok(())
    }

    pub fn exit_module(&mut self) -> crate::Result<()> {
        self.current_module = self.current_module_info().super_module()
            .expect("attempted to leave root module");

        Ok(())
    }

    pub fn enter_implement_block(&mut self, self_type: TypeHandle) {
        if self.current_self_type().is_some() {
            panic!("attempted to enter nested implement block");
        }
        self.current_self_type = Some(self_type);
    }

    pub fn exit_implement_block(&mut self) {
        if self.current_self_type().is_none() {
            panic!("attempted to leave inactive implement block");
        }
        self.current_self_type = None;
    }

    pub fn define_symbol(&mut self, name: String, value: Value) -> crate::Result<()> {
        if let Some(existing_value) = self.current_symbol_table().find(&name) {
            if !self.current_symbol_table().is_unresolved(&name)
                || !self.types_are_equivalent(existing_value.get_type(), value.get_type())
            {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: name.clone() }));
            }
        }

        self.current_symbol_table_mut().define(name, value);

        Ok(())
    }

    pub fn declare_symbol(&mut self, name: String, value: Value) -> crate::Result<()> {
        if let Some(existing_value) = self.current_symbol_table().find(&name) {
            if !self.types_are_equivalent(existing_value.get_type(), value.get_type()) {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: name.clone() }));
            }
        }
        else {
            self.current_symbol_table_mut().declare(name, value);
        }

        Ok(())
    }

    pub fn create_member_identifier(&self, container: ContainerHandle, member_name: &str) -> String {
        match container {
            ContainerHandle::Type(handle) => {
                format!("{}::{member_name}", self.type_identifier(handle))
            }
            ContainerHandle::Module(handle) => {
                self.module_info(handle).create_member_identifier(member_name)
            }
        }
    }
    
    fn get_super_module(&self, container: ContainerHandle) -> crate::Result<ModuleHandle> {
        container.as_module()
            .and_then(|module| self.module_info(module).super_module())
            .ok_or_else(|| Box::new(crate::Error::UndefinedModule {
                name: self.create_member_identifier(container, "super"),
            }))
    }
    
    pub fn interpret_path(&mut self, segments: &[PathSegment], allow_impl_items: bool) -> crate::Result<(Option<String>, Value)> {
        let (final_segment, leading_segments) = segments.split_last().expect("path is empty");
        
        let mut current_container = ContainerHandle::Module(self.current_module());
        for segment in leading_segments {
            match segment {
                PathSegment::Name(name) => {
                    let Some(container) = self.symbol_table(current_container).find_container(name) else {
                        return Err(Box::new(crate::Error::UndefinedModule {
                            name: self.create_member_identifier(current_container, name),
                        }))
                    };
                    current_container = container;
                }
                PathSegment::RootModule => {
                    current_container = ModuleHandle::ROOT.into()
                }
                PathSegment::SuperModule => {
                    current_container = self.get_super_module(current_container)?.into();
                }
                PathSegment::SelfModule => {
                    let ContainerHandle::Module(..) = current_container else {
                        return Err(Box::new(crate::Error::UndefinedModule {
                            name: self.create_member_identifier(current_container, "module"),
                        }));
                    };
                }
                PathSegment::SelfType => {
                    let Some(self_type) = self.current_self_type() else {
                        return Err(Box::new(crate::Error::SelfOutsideImplement {}));
                    };
                    current_container = self_type.into();
                }
                PathSegment::PrimitiveType(primitive_type) => {
                    current_container = primitive_type.handle.into();
                }
                PathSegment::Type(type_node) => {
                    current_container = self.interpret_type_node(type_node)?.into();
                }
            }
            
            if !allow_impl_items && matches!(current_container, ContainerHandle::Type(..)) {
                return Err(Box::new(crate::Error::UndefinedModule {
                    name: self.container_identifier(current_container).to_owned(),
                }))
            }
        }
        
        match final_segment {
            PathSegment::Name(name) => {
                self.symbol_table(current_container).find(name)
                    .map(|value| (Some(name.clone()), value.clone()))
                    .ok_or_else(|| Box::new(crate::Error::UndefinedSymbol {
                        name: self.create_member_identifier(current_container, name),
                    }))
            }
            PathSegment::RootModule => {
                Ok((None, ModuleHandle::ROOT.into()))
            }
            PathSegment::SuperModule => {
                let super_module = self.get_super_module(current_container)?;
                let super_name = self.module_info(super_module).identifier().to_owned();
                Ok((Some(super_name), super_module.into()))
            }
            PathSegment::SelfModule => {
                let Some(module) = current_container.as_module() else {
                    return Err(Box::new(crate::Error::UndefinedModule {
                        name: self.create_member_identifier(current_container, "module"),
                    }));
                };
                let module_name = self.module_info(module).identifier().to_owned();
                Ok((Some(module_name), current_container.into()))
            }
            PathSegment::SelfType => {
                self.current_self_type()
                    .map(|self_type| (None, self_type.into()))
                    .ok_or_else(|| Box::new(crate::Error::SelfOutsideImplement {}))
            }
            PathSegment::PrimitiveType(primitive_type) => {
                Ok((None, primitive_type.handle.into()))
            }
            PathSegment::Type(type_node) => {
                Ok((None, self.interpret_type_node(type_node)?.into()))
            }
        }
    }

    pub fn interpret_type_node(&mut self, type_node: &TypeNode) -> crate::Result<TypeHandle> {
        match type_node {
            TypeNode::Path { segments } => {
                self.interpret_path(segments, true)?.1.as_type()
                    .ok_or_else(|| Box::new(crate::Error::UnknownType {
                        type_name: PathSegment::path_to_string(&segments),
                    }))
            }
            TypeNode::Pointer { pointee_type, semantics } => {
                let pointee_type = self.interpret_type_node(pointee_type)?;
                Ok(self.get_pointer_type(pointee_type, *semantics))
            }
            TypeNode::Array { item_type, length } => {
                let item_type = self.interpret_type_node(item_type)?;
                let length = match length {
                    Some(node) => Some(node.as_array_length()
                        .ok_or_else(|| Box::new(crate::Error::NonConstantArrayLength {}))?),
                    None => None,
                };
                Ok(self.get_array_type(item_type, length))
            }
            TypeNode::Function { parameter_types, is_variadic, return_type } => {
                let parameter_types: Box<[_]> = Result::from_iter(parameter_types.iter()
                    .map(|type_node| self.interpret_type_node(type_node)))?;
                let return_type = self.interpret_type_node(return_type)?;
                let signature = FunctionSignature::new(return_type, parameter_types, *is_variadic);
                Ok(self.get_function_type(&signature))
            }
        }
    }
    
    pub fn interpret_node_as_type(&mut self, node: &Node) -> crate::Result<TypeHandle> {
        match node {
            Node::Type(type_node) => {
                self.interpret_type_node(type_node)
            }
            Node::Path { segments } => {
                self.interpret_path(segments, true)?.1.as_type()
                    .ok_or_else(|| Box::new(crate::Error::UnknownType {
                        type_name: PathSegment::path_to_string(&segments),
                    }))
            }
            Node::Literal(Literal::Name(name)) => {
                self.get_named_type(self.current_module().into(), name)
            }
            Node::Literal(Literal::PrimitiveType(primitive_type)) => {
                Ok(primitive_type.handle)
            }
            _ => {
                Err(Box::new(crate::Error::UnexpectedExpression {}))
            }
        }
    }
    
    pub fn get_named_type(&mut self, container: ContainerHandle, name: &str) -> crate::Result<TypeHandle> {
        if let Some(handle) = self.symbol_table(container).find_type(name) {
            Ok(handle)
        }
        else if self.symbol_table(container).has_symbol(name) {
            Err(Box::new(crate::Error::TypeSymbolConflict {
                name: self.create_member_identifier(container, name),
            }))
        }
        else {
            let info = TypeInfo::Undefined {
                name: name.into(),
            };
            let identifier = self.create_member_identifier(container, name);

            let handle = self.create_type(info, identifier);
            self.symbol_table_mut(container).declare(name.into(), handle.into());
            
            Ok(handle)
        }
    }
    
    pub fn define_named_type(&mut self, container: ContainerHandle, name: &str, new_info: TypeInfo) -> crate::Result<TypeHandle> {
        if let Some(handle) = self.symbol_table(container).find_type(name) {
            if !self.symbol_table(container).is_unresolved(name) {
                return Err(Box::new(crate::Error::TypeSymbolConflict {
                    name: self.type_identifier(handle).into(),
                }));
            };
            
            let new_identifier = self.create_member_identifier(container, name);
            let new_alignment = self.calculate_alignment(&new_info);
            let new_size = self.calculate_size(&new_info);
            let new_llvm_syntax = self.generate_type_llvm_syntax(&new_identifier, &new_info);
            
            let entry = self.type_entry_mut(handle);
            entry.identifier = new_identifier;
            entry.info = new_info;
            entry.alignment = new_alignment;
            entry.size = new_size;
            entry.llvm_syntax = new_llvm_syntax;
            
            Ok(handle)
        }
        else {
            let identifier = self.create_member_identifier(container, name);
            
            let handle = self.create_type(new_info, identifier);
            self.symbol_table_mut(container).define(name.into(), handle.into());
            
            Ok(handle)
        }
    }
    
    pub fn get_pointer_type(&mut self, pointee_type: TypeHandle, semantics: PointerSemantics) -> TypeHandle {
        if let Some(handle) = self.pointer_types.get(&(pointee_type, semantics)) {
            *handle
        }
        else {
            let pointee_identifier = self.type_identifier(pointee_type);
            let identifier = match semantics {
                PointerSemantics::Immutable => format!("*{pointee_identifier}"),
                PointerSemantics::Mutable => format!("*mut {pointee_identifier}"),
                PointerSemantics::Owned => format!("*own {pointee_identifier}"),
            };
            let info = TypeInfo::Pointer {
                pointee_type,
                semantics,
            };

            let handle = self.create_type(info, identifier);
            self.pointer_types.insert((pointee_type, semantics), handle);
            handle
        }
    }
    
    pub fn get_array_type(&mut self, item_type: TypeHandle, length: Option<usize>) -> TypeHandle {
        if let Some(handle) = self.array_types.get(&(item_type, length)) {
            *handle
        }
        else {
            let item_identifier = self.type_identifier(item_type);
            let identifier = match length {
                Some(length) => format!("[{item_identifier}; {length}]"),
                None => format!("[{item_identifier}]"),
            };
            let info = TypeInfo::Array {
                item_type,
                length,
            };

            let handle = self.create_type(info, identifier);
            self.array_types.insert((item_type, length), handle);
            handle
        }
    }
    
    pub fn get_function_type(&mut self, signature: &FunctionSignature) -> TypeHandle {
        if let Some(handle) = self.function_types.get(signature) {
            *handle
        }
        else {
            let mut identifier = String::from("function(");
            let mut parameter_types_iter = signature.parameter_types().iter();
            if let Some(&parameter_type) = parameter_types_iter.next() {
                identifier.push_str(self.type_identifier(parameter_type));
                for &parameter_type in parameter_types_iter {
                    identifier.push_str(", ");
                    identifier.push_str(self.type_identifier(parameter_type));
                }
                if signature.is_variadic() {
                    identifier.push_str(", ..");
                }
            }
            else if signature.is_variadic() {
                identifier.push_str("..");
            }
            identifier.push_str(") -> ");
            identifier.push_str(self.type_identifier(signature.return_type()));
            let info = TypeInfo::Function {
                signature: signature.clone(),
            };
            let handle = self.create_type(info, identifier);
            self.function_types.insert(signature.clone(), handle);
            handle
        }
    }

    fn create_type(&mut self, info: TypeInfo, identifier: String) -> TypeHandle {
        let alignment = self.calculate_alignment(&info);
        let size = self.calculate_size(&info);
        let llvm_syntax = self.generate_type_llvm_syntax(&identifier, &info);
        let entry = TypeEntry {
            identifier,
            info,
            alignment,
            size,
            llvm_syntax,
            symbol_table: SymbolTable::new(),
        };

        let registry_index = self.type_registry.len();
        self.type_registry.push(entry);
        TypeHandle::new(registry_index)
    }

    fn calculate_alignment(&self, info: &TypeInfo) -> Option<usize> {
        match *info {
            TypeInfo::Meta => None,
            TypeInfo::Never => None,
            TypeInfo::Void => None,
            TypeInfo::Boolean => Some(1),
            TypeInfo::Integer { size, .. } => Some(size),
            TypeInfo::Pointer { .. } => Some(self.pointer_size),
            TypeInfo::Function { .. } => Some(self.pointer_size),
            TypeInfo::Array { item_type, .. } => self.type_alignment(item_type),
            TypeInfo::Structure { ref members, .. } => members.iter()
                .map(|member| self.type_alignment(member.member_type))
                .max().unwrap_or(Some(1)),
            TypeInfo::Undefined { .. } => None,
            TypeInfo::Alias { target } => self.type_alignment(target),
        }
    }
    
    fn calculate_size(&self, info: &TypeInfo) -> Option<usize> {
        match *info {
            TypeInfo::Meta => None,
            TypeInfo::Never => Some(0),
            TypeInfo::Void => Some(0),
            TypeInfo::Boolean => Some(1),
            TypeInfo::Integer { size, .. } => Some(size),
            TypeInfo::Pointer { .. } => Some(self.pointer_size),
            TypeInfo::Function { .. } => Some(self.pointer_size),
            TypeInfo::Array { item_type, length } => {
                let length = length?;
                let item_size = self.type_size(item_type)?;
                Some(length * item_size)
            }
            TypeInfo::Structure { ref members, .. } => {
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
            TypeInfo::Undefined { .. } => None,
            TypeInfo::Alias { target } => self.type_size(target),
        }
    }
    
    fn generate_type_llvm_syntax(&self, identifier: &str, info: &TypeInfo) -> String {
        match *info {
            TypeInfo::Meta => "<ERROR meta type>".to_owned(),
            TypeInfo::Never => "void".to_owned(),
            TypeInfo::Void => "void".to_owned(),
            TypeInfo::Boolean => "i1".to_owned(),
            TypeInfo::Integer { size, .. } => format!("i{}", size * 8),
            TypeInfo::Pointer { pointee_type, .. } => {
                match self.type_llvm_syntax(pointee_type) {
                    "void" => "{}*".to_owned(),
                    pointee_syntax => format!("{pointee_syntax}*")
                }
            }
            TypeInfo::Array { item_type, length } => match length {
                Some(length) => format!("[{} x {}]", length, self.type_llvm_syntax(item_type)),
                None => self.type_llvm_syntax(item_type).to_owned(),
            }
            TypeInfo::Structure { .. } | TypeInfo::Undefined { .. } => {
                format!("%\"type.{identifier}\"")
            },
            TypeInfo::Function { ref signature, .. } => {
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
            TypeInfo::Alias { target } => self.type_llvm_syntax(target).to_owned(),
        }
    }

    pub fn types_are_equivalent(&self, lhs_type: TypeHandle, rhs_type: TypeHandle) -> bool {
        self.get_canonical_type(lhs_type) == self.get_canonical_type(rhs_type)
    }

    pub fn can_coerce_type(&self, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> bool {
        let from_type = self.get_canonical_type(from_type);
        let to_type = self.get_canonical_type(to_type);
        from_type == to_type || match (self.type_info(from_type), self.type_info(to_type)) {
            (
                &TypeInfo::Pointer { pointee_type: from_pointee, semantics: from_semantics },
                &TypeInfo::Pointer { pointee_type: to_pointee, semantics: to_semantics },
            ) => {
                // Not sure if this is right... needs testing
                use PointerSemantics::*;
                match (from_semantics, to_semantics) {
                    (Immutable, Immutable) => self.can_coerce_type(from_pointee, to_pointee, false),
                    (Immutable, _) => false,
                    (Mutable, Immutable) => self.can_coerce_type(from_pointee, to_pointee, true),
                    (Mutable, Mutable) => self.can_coerce_type(from_pointee, to_pointee, true),
                    (Mutable, _) => false,
                    (Owned, Immutable) => self.can_coerce_type(from_pointee, to_pointee, from_mutable),
                    (Owned, Mutable) => from_mutable && self.can_coerce_type(from_pointee, to_pointee, from_mutable),
                    (Owned, Owned) => self.can_coerce_type(from_pointee, to_pointee, from_mutable),
                }
            },
            (
                &TypeInfo::Array { item_type: from_item, length: Some(from_length) },
                &TypeInfo::Array { item_type: to_item, length: Some(to_length) },
            ) => {
                from_length == to_length && self.can_coerce_type(from_item, to_item, from_mutable)
            },
            (
                &TypeInfo::Array { item_type: from_item, length: _ },
                &TypeInfo::Array { item_type: to_item, length: None },
            ) => {
                self.can_coerce_type(from_item, to_item, from_mutable)
            },
            (
                TypeInfo::Function { signature: from_signature },
                TypeInfo::Function { signature: to_signature },
            ) => {
                from_signature.is_variadic() == to_signature.is_variadic()
                    && self.can_coerce_type(from_signature.return_type(), to_signature.return_type(), false)
                    && from_signature.parameter_types().len() == to_signature.parameter_types().len()
                    && std::iter::zip(from_signature.parameter_types(), to_signature.parameter_types())
                        .all(|(&from_param, &to_param)| {
                            self.can_coerce_type(from_param, to_param, false)
                        })
            },
            (TypeInfo::Void, _) | (_, TypeInfo::Void) => true,
            _ => false
        }
    }
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self::new()
    }
}
