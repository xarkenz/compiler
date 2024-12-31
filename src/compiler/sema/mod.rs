use std::collections::HashMap;
use std::num::NonZeroUsize;
use crate::ast::TypeNode;

mod values;
mod types;
mod modules;
mod local_ctx;

pub use values::*;
pub use types::*;
pub use modules::*;
pub use local_ctx::*;

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
    /// Find a primitive type by name.
    primitive_types: HashMap<String, TypeHandle>,
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
    implementation: HashMap<String, GlobalSymbol>,
    alignment: usize,
    size: Option<usize>,
    llvm_syntax: String,
}

impl GlobalContext {
    pub fn new() -> Self {
        let mut context = Self {
            pointer_size: size_of::<*const ()>(),
            module_registry: vec![ModuleInfo::new(String::new(), None)],
            current_module: ModuleHandle::ROOT,
            type_registry: Vec::with_capacity(PRIMITIVE_TYPES.len()),
            current_self_type: None,
            primitive_types: HashMap::new(),
            pointer_types: HashMap::new(),
            array_types: HashMap::new(),
            function_types: HashMap::new(),
        };
        
        for &(ref info, identifier) in PRIMITIVE_TYPES {
            let handle = context.create_type(info.clone(), identifier.into());
            context.primitive_types.insert(identifier.into(), handle);
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
    
    pub fn set_self_type(&mut self, self_type: Option<TypeHandle>) {
        self.current_self_type = self_type;
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

    pub fn type_alignment(&self, handle: TypeHandle) -> usize {
        self.type_entry(handle).alignment
    }

    pub fn type_size(&self, handle: TypeHandle) -> Option<usize> {
        self.type_entry(handle).size
    }
    
    pub fn type_llvm_syntax(&self, handle: TypeHandle) -> &str {
        &self.type_entry(handle).llvm_syntax
    }
    
    pub fn enter_module(&mut self, name: String) -> crate::Result<()> {
        let new_module_info = ModuleInfo::new(name.clone(), Some(self.current_module()));
        let new_module = ModuleHandle::new(self.module_registry.len());
        self.module_registry.push(new_module_info);
        
        self.current_module_info_mut().bind_module(name, new_module)?;
        self.current_module = new_module;
        
        Ok(())
    }
    
    pub fn leave_module(&mut self) -> crate::Result<()> {
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
    
    pub fn leave_implement_block(&mut self) {
        if self.current_self_type().is_none() {
            panic!("attempted to leave inactive implement block");
        }
        self.current_self_type = None;
    }

    pub fn declare_symbol(&mut self, name: String, value: Value) -> crate::Result<()> {
        if let Some(symbol) = self.find_conflicting_symbol(&name) {
            if !self.types_are_equivalent(symbol.value.get_type(), value.get_type()) {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: name.clone() }));
            }
        }
        else {
            self.insert_symbol(name, value, false);
        }
        
        Ok(())
    }

    pub fn define_symbol(&mut self, name: String, value: Value) -> crate::Result<()> {
        if let Some(symbol) = self.find_conflicting_symbol(&name) {
            if symbol.is_defined || !self.types_are_equivalent(symbol.value.get_type(), value.get_type()) {
                return Err(Box::new(crate::Error::GlobalSymbolConflict { name: name.clone() }));
            }
        }

        self.insert_symbol(name, value, true);

        Ok(())
    }
    
    /// "Conflicting" i.e. the symbol would conflict if a new symbol of the same name were
    /// to be defined in the current scope. The current `implement`
    /// block is searched if applicable, otherwise the current module is searched.
    fn find_conflicting_symbol(&self, name: &str) -> Option<&GlobalSymbol> {
        if let Some(self_type) = self.current_self_type() {
            self.type_entry(self_type).implementation.get(name)
        }
        else {
            self.current_module_info().find_symbol(name)
        }
    }
    
    fn insert_symbol(&mut self, name: String, value: Value, is_defined: bool) {
        let symbol = GlobalSymbol {
            value,
            is_defined,
        };
        
        if let Some(self_type) = self.current_self_type() {
            self.type_entry_mut(self_type).implementation.insert(name, symbol);
        }
        else {
            self.current_module_info_mut().insert_symbol(name, symbol);
        }
    }

    pub fn get_type_handle_from_node(&mut self, type_node: &TypeNode) -> crate::Result<TypeHandle> {
        match type_node {
            TypeNode::Path { names } => {
                let (type_name, module_names) = names.split_last().unwrap();
                let mut module = self.current_module();
                for module_name in module_names {
                    let module_info = self.module_info(module);
                    if let Some(found_module) = module_info.module_binding(module_name) {
                        module = found_module;
                    }
                    else {
                        return Err(Box::new(crate::Error::UndefinedModule { name: module_info.create_member_identifier(module_name) }));
                    }
                }
                let module_info = self.module_info(module);
                module_info.type_binding(type_name)
                    .ok_or_else(|| Box::new(crate::Error::UnknownType { type_name: module_info.create_member_identifier(type_name) }))
            }
            TypeNode::Pointer { pointee_type, semantics } => {
                let pointee_type = self.get_type_handle_from_node(pointee_type)?;
                Ok(self.get_pointer_type(pointee_type, *semantics))
            }
            TypeNode::Array { item_type, length } => {
                let item_type = self.get_type_handle_from_node(item_type)?;
                let length = match length {
                    Some(node) => Some(node.as_array_length()
                        .ok_or_else(|| Box::new(crate::Error::NonConstantArrayLength {}))?),
                    None => None,
                };
                Ok(self.get_array_type(item_type, length))
            }
            TypeNode::Function { parameter_types, is_variadic, return_type } => {
                let parameter_types: Box<[_]> = parameter_types.iter()
                    .map(|type_node| self.get_type_handle_from_node(type_node))
                    .collect::<Result<_, _>>()?;
                let return_type = self.get_type_handle_from_node(return_type)?;
                let signature = FunctionSignature::new(return_type, parameter_types, *is_variadic);
                Ok(self.get_function_type(&signature))
            }
            TypeNode::SelfType => {
                self.current_self_type.ok_or_else(|| Box::new(crate::Error::SelfOutsideImplement {}))
            }
        }
    }
    
    pub fn get_named_type(&mut self, module: ModuleHandle, name: &str) -> crate::Result<TypeHandle> {
        if let Some(handle) = self.module_info(module).type_binding(name) {
            Ok(handle)
        }
        else {
            let info = TypeInfo::Undefined {
                name: name.into(),
            };
            let identifier = self.module_info_mut(module).create_member_identifier(name);
            
            let handle = self.create_type(info, identifier.into());
            self.module_info_mut(module).bind_type(name.into(), handle)?;
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
        if let Some(handle) = self.function_types.get(&signature) {
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
            implementation: HashMap::new(),
            alignment,
            size,
            llvm_syntax,
        };

        let registry_index = self.type_registry.len();
        self.type_registry.push(entry);
        TypeHandle::new(registry_index)
    }

    fn calculate_alignment(&self, info: &TypeInfo) -> usize {
        match *info {
            TypeInfo::Never => 0,
            TypeInfo::Void => 1,
            TypeInfo::Boolean => 1,
            TypeInfo::Integer { size, .. } => size,
            TypeInfo::Pointer { .. } => self.pointer_size,
            TypeInfo::Function { .. } => self.pointer_size,
            TypeInfo::Array { item_type, .. } => self.type_alignment(item_type),
            TypeInfo::Structure { ref members, .. } => members.iter()
                .map(|member| self.type_alignment(member.member_type))
                .max().unwrap_or(1),
            TypeInfo::Undefined { .. } => 0,
            TypeInfo::Alias { target } => self.type_alignment(target),
        }
    }
    
    fn calculate_size(&self, info: &TypeInfo) -> Option<usize> {
        match *info {
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
                let mut max_alignment = 0;

                for member in members {
                    let alignment = self.type_alignment(member.member_type);
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
            TypeInfo::Alias { target } => Some(self.type_alignment(target)),
        }
    }
    
    fn generate_type_llvm_syntax(&self, identifier: &str, info: &TypeInfo) -> String {
        match *info {
            TypeInfo::Never => "void".into(),
            TypeInfo::Void => "void".into(),
            TypeInfo::Boolean => "i1".into(),
            TypeInfo::Integer { size, .. } => format!("i{}", size * 8),
            TypeInfo::Pointer { pointee_type, .. } => {
                format!("{}*", self.type_llvm_syntax(pointee_type))
            }
            TypeInfo::Array { item_type, length } => match length {
                Some(length) => format!("[{} x {}]", length, self.type_llvm_syntax(item_type)),
                None => self.type_llvm_syntax(item_type).into(),
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
            TypeInfo::Alias { target } => self.type_llvm_syntax(target).into(),
        }
    }
    
    pub fn types_are_equivalent(&self, lhs_type: TypeHandle, rhs_type: TypeHandle) -> bool {
        // TODO
        lhs_type == rhs_type
    }

    pub fn can_coerce_type(&self, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> bool {
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
