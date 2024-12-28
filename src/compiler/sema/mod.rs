use std::collections::HashMap;
use std::num::NonZeroUsize;
use crate::ast::TypeNode;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PointerSemantics {
    Immutable,
    Mutable,
    Owned,
}

impl PointerSemantics {
    pub fn simple(is_mutable: bool) -> Self {
        if is_mutable {
            Self::Mutable
        }
        else {
            Self::Immutable
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructureMember {
    pub name: String,
    pub value_type: TypeHandle,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FunctionSignature {
    return_type: TypeHandle,
    parameter_types: Box<[TypeHandle]>,
    is_variadic: bool,
}

impl FunctionSignature {
    pub fn new(return_type: TypeHandle, parameter_types: Box<[TypeHandle]>, is_variadic: bool) -> Self {
        Self {
            return_type,
            parameter_types,
            is_variadic,
        }
    }

    pub fn return_type(&self) -> TypeHandle {
        self.return_type
    }

    pub fn parameter_types(&self) -> &[TypeHandle] {
        &self.parameter_types
    }

    pub fn is_variadic(&self) -> bool {
        self.is_variadic
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TypeInfo {
    Meta,
    Never,
    Void,
    Boolean,
    Integer {
        size: usize,
        signed: bool,
    },
    Pointer {
        pointee_type: TypeHandle,
        semantics: PointerSemantics,
    },
    Array {
        item_type: TypeHandle,
        length: Option<usize>,
    },
    Structure {
        name: String,
        members: Box<[StructureMember]>,
    },
    Function {
        signature: FunctionSignature,
    },
    Opaque {
        name: String,
    },
    Alias {
        target: TypeHandle,
    },
}

/// Order of elements is important! If anything is changed here, `TypeHandle::*` may need to be
/// changed as well.
const BUILTIN_TYPES: &[(TypeInfo, Option<&str>)] = &[
    (TypeInfo::Meta, None),
    (TypeInfo::Never, Some("never")),
    (TypeInfo::Void, Some("void")),
    (TypeInfo::Boolean, Some("bool")),
    (TypeInfo::Integer { size: 1, signed: true }, Some("i8")),
    (TypeInfo::Integer { size: 1, signed: false }, Some("u8")),
    (TypeInfo::Integer { size: 2, signed: true }, Some("i16")),
    (TypeInfo::Integer { size: 2, signed: false }, Some("u16")),
    (TypeInfo::Integer { size: 4, signed: true }, Some("i32")),
    (TypeInfo::Integer { size: 4, signed: false }, Some("u32")),
    (TypeInfo::Integer { size: 8, signed: true }, Some("i64")),
    (TypeInfo::Integer { size: 8, signed: false }, Some("u64")),
];

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct TypeHandle(NonZeroUsize);

impl TypeHandle {
    pub const META: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(1) });
    pub const NEVER: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(2) });
    pub const VOID: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(3) });
    pub const BOOL: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(4) });
    pub const I8: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(5) });
    pub const U8: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(6) });
    pub const I16: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(7) });
    pub const U16: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(8) });
    pub const I32: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(9) });
    pub const U32: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(10) });
    pub const I64: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(11) });
    pub const U64: TypeHandle = TypeHandle(unsafe { NonZeroUsize::new_unchecked(12) });
    
    pub fn get_identifier(self, registry: &TypeRegistry) -> &str {
        registry.get_identifier(self)
    }
    
    pub fn get_llvm_syntax(self, registry: &TypeRegistry) -> &str {
        registry.get_llvm_syntax(self)
    }
}

struct TypeEntry {
    identifier: String,
    info: TypeInfo,
    alignment: usize,
    size: Option<usize>,
    llvm_syntax: String,
}

pub struct TypeRegistry {
    pointer_size: usize,
    registered_types: Vec<TypeEntry>,
    /// Get the type `Self` currently represents.
    self_type: Option<TypeHandle>,
    /// Find a type by name.
    named_types: HashMap<String, TypeHandle>,
    /// Find `*T`, `*mut T`, or `*own T`.
    pointer_types: HashMap<(TypeHandle, PointerSemantics), TypeHandle>,
    /// Find `[T; N]` or `[T]`.
    array_types: HashMap<(TypeHandle, Option<usize>), TypeHandle>,
    /// Find a function type by signature.
    function_types: HashMap<FunctionSignature, TypeHandle>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            pointer_size: size_of::<*const ()>(),
            registered_types: Vec::with_capacity(BUILTIN_TYPES.len()),
            self_type: None,
            named_types: HashMap::new(),
            pointer_types: HashMap::new(),
            array_types: HashMap::new(),
            function_types: HashMap::new(),
        };
        for (info, identifier) in BUILTIN_TYPES {
            let handle = registry.create_handle(identifier.unwrap_or("").into(), info.clone());
            if let &Some(name) = identifier {
                registry.named_types.insert(name.into(), handle);
            }
        }
        registry
    }
    
    pub fn self_type(&self) -> Option<TypeHandle> {
        self.self_type
    }
    
    pub fn set_self_type(&mut self, self_type: Option<TypeHandle>) {
        self.self_type = self_type;
    }
    
    fn get_entry(&self, handle: TypeHandle) -> &TypeEntry {
        // Subtract 1 because the first created handle has value 1 but represents index 0
        &self.registered_types[handle.0.get() - 1]
    }
    
    pub fn get_identifier(&self, handle: TypeHandle) -> &str {
        &self.get_entry(handle).identifier
    }

    pub fn get_info(&self, handle: TypeHandle) -> &TypeInfo {
        &self.get_entry(handle).info
    }
    
    pub fn get_function_info(&self, handle: TypeHandle) -> Option<&FunctionSignature> {
        match &self.get_entry(handle).info {
            TypeInfo::Function { signature } => Some(signature),
            _ => None,
        }
    }

    pub fn get_alignment(&self, handle: TypeHandle) -> usize {
        self.get_entry(handle).alignment
    }

    pub fn get_size(&self, handle: TypeHandle) -> Option<usize> {
        self.get_entry(handle).size
    }
    
    pub fn get_llvm_syntax(&self, handle: TypeHandle) -> &str {
        &self.get_entry(handle).llvm_syntax
    }
    
    fn create_handle(&mut self, identifier: String, info: TypeInfo) -> TypeHandle {
        let alignment = self.calculate_alignment(&info);
        let size = self.calculate_size(&info);
        let llvm_syntax = self.generate_llvm_syntax(&identifier, &info);
        let entry = TypeEntry {
            identifier,
            info,
            alignment,
            size,
            llvm_syntax,
        };
        self.registered_types.push(entry);
        TypeHandle(NonZeroUsize::new(self.registered_types.len()).unwrap())
    }

    pub fn get_handle(&mut self, type_node: &TypeNode) -> crate::Result<TypeHandle> {
        match type_node {
            TypeNode::Named { name } => {
                Ok(self.get_named_handle(name))
            }
            TypeNode::Pointer { pointee_type, semantics } => {
                let pointee_type = self.get_handle(pointee_type)?;
                Ok(self.get_pointer_handle(pointee_type, *semantics))
            }
            TypeNode::Array { item_type, length } => {
                let item_type = self.get_handle(item_type)?;
                let length = match length {
                    Some(node) => Some(node.as_array_length()
                        .ok_or_else(|| Box::new(crate::Error::NonConstantArrayLength {}))?),
                    None => None,
                };
                Ok(self.get_array_handle(item_type, length))
            }
            TypeNode::Function { parameter_types, is_variadic, return_type } => {
                let parameter_types: Box<[_]> = parameter_types.iter()
                    .map(|type_node| self.get_handle(type_node))
                    .collect::<Result<_, _>>()?;
                let return_type = self.get_handle(return_type)?;
                let signature = FunctionSignature::new(return_type, parameter_types, *is_variadic);
                Ok(self.get_function_handle(&signature))
            }
            TypeNode::SelfType => {
                self.self_type.ok_or_else(|| Box::new(crate::Error::SelfOutsideImplement {}))
            }
        }
    }
    
    pub fn get_named_handle(&mut self, name: &str) -> TypeHandle {
        if let Some(handle) = self.named_types.get(name) {
            *handle
        }
        else {
            let info = TypeInfo::Opaque {
                name: name.into(),
            };
            let handle = self.create_handle(name.into(), info);
            self.named_types.insert(name.into(), handle);
            handle
        }
    }
    
    pub fn get_pointer_handle(&mut self, pointee_type: TypeHandle, semantics: PointerSemantics) -> TypeHandle {
        if let Some(handle) = self.pointer_types.get(&(pointee_type, semantics)) {
            *handle
        }
        else {
            let pointee_identifier = self.get_identifier(pointee_type);
            let identifier = match semantics {
                PointerSemantics::Immutable => format!("*{pointee_identifier}"),
                PointerSemantics::Mutable => format!("*mut {pointee_identifier}"),
                PointerSemantics::Owned => format!("*own {pointee_identifier}"),
            };
            let info = TypeInfo::Pointer {
                pointee_type,
                semantics,
            };
            let handle = self.create_handle(identifier, info);
            self.pointer_types.insert((pointee_type, semantics), handle);
            handle
        }
    }
    
    pub fn get_array_handle(&mut self, item_type: TypeHandle, length: Option<usize>) -> TypeHandle {
        if let Some(handle) = self.array_types.get(&(item_type, length)) {
            *handle
        }
        else {
            let item_identifier = self.get_identifier(item_type);
            let identifier = match length {
                Some(length) => format!("[{item_identifier}; {length}]"),
                None => format!("[{item_identifier}]"),
            };
            let info = TypeInfo::Array {
                item_type,
                length,
            };
            let handle = self.create_handle(identifier, info);
            self.array_types.insert((item_type, length), handle);
            handle
        }
    }
    
    pub fn get_function_handle(&mut self, signature: &FunctionSignature) -> TypeHandle {
        if let Some(handle) = self.function_types.get(&signature) {
            *handle
        }
        else {
            let mut identifier = String::from("function(");
            let mut parameter_types_iter = signature.parameter_types().iter();
            if let Some(&parameter_type) = parameter_types_iter.next() {
                identifier.push_str(self.get_identifier(parameter_type));
                for &parameter_type in parameter_types_iter {
                    identifier.push_str(", ");
                    identifier.push_str(self.get_identifier(parameter_type));
                }
                if signature.is_variadic() {
                    identifier.push_str(", ..");
                }
            }
            else if signature.is_variadic() {
                identifier.push_str("..");
            }
            identifier.push_str(") -> ");
            identifier.push_str(self.get_identifier(signature.return_type()));
            let info = TypeInfo::Function {
                signature: signature.clone(),
            };
            let handle = self.create_handle(identifier, info);
            self.function_types.insert(signature.clone(), handle);
            handle
        }
    }

    pub fn try_get_handle(&self, type_node: &TypeNode) -> Option<TypeHandle> {
        match type_node {
            TypeNode::Named { name } => {
                self.named_types.get(name).cloned()
            }
            TypeNode::Pointer { pointee_type, semantics } => {
                let pointee_type = self.try_get_handle(pointee_type)?;
                self.pointer_types.get(&(pointee_type, *semantics)).cloned()
            }
            TypeNode::Array { item_type, length } => {
                let item_type = self.try_get_handle(item_type)?;
                let length = match length {
                    Some(node) => Some(node.as_array_length()?),
                    None => None,
                };
                self.array_types.get(&(item_type, length)).cloned()
            }
            TypeNode::Function { parameter_types, is_variadic, return_type } => {
                let parameter_types: Box<[_]> = parameter_types.iter()
                    .map(|type_node| self.try_get_handle(type_node))
                    .collect::<Option<_>>()?;
                let return_type = self.try_get_handle(return_type)?;
                let signature = FunctionSignature::new(return_type, parameter_types, *is_variadic);
                self.function_types.get(&signature).cloned()
            }
            TypeNode::SelfType => {
                self.self_type
            }
        }
    }

    pub fn calculate_alignment(&self, info: &TypeInfo) -> usize {
        match *info {
            TypeInfo::Meta => 0,
            TypeInfo::Never => 0,
            TypeInfo::Void => 1,
            TypeInfo::Boolean => 1,
            TypeInfo::Integer { size, .. } => size,
            TypeInfo::Pointer { .. } => self.pointer_size,
            TypeInfo::Function { .. } => self.pointer_size,
            TypeInfo::Array { item_type, .. } => self.get_alignment(item_type),
            TypeInfo::Structure { ref members, .. } => members.iter()
                .map(|member| self.get_alignment(member.value_type))
                .max().unwrap_or(1),
            TypeInfo::Opaque { .. } => 0,
            TypeInfo::Alias { target } => self.get_alignment(target),
        }
    }
    
    pub fn calculate_size(&self, info: &TypeInfo) -> Option<usize> {
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
                let item_size = self.get_size(item_type)?;
                Some(length * item_size)
            }
            TypeInfo::Structure { ref members, .. } => {
                let mut current_size = 0;
                let mut max_alignment = 0;

                for member in members {
                    let alignment = self.get_alignment(member.value_type);
                    max_alignment = max_alignment.max(alignment);

                    // Calculate padding
                    let intermediate_size = current_size + alignment - 1;
                    let padded_size = intermediate_size - intermediate_size % alignment;
                    current_size = padded_size + self.get_size(member.value_type)?;
                }

                // Pad for the largest member alignment
                let intermediate_size = current_size + max_alignment - 1;
                let padded_size = intermediate_size - intermediate_size % max_alignment;

                Some(padded_size)
            }
            TypeInfo::Opaque { .. } => None,
            TypeInfo::Alias { target } => Some(self.get_alignment(target)),
        }
    }
    
    pub fn generate_llvm_syntax(&self, identifier: &str, info: &TypeInfo) -> String {
        match *info {
            TypeInfo::Meta => "<META>".into(),
            TypeInfo::Never => "void".into(),
            TypeInfo::Void => "void".into(),
            TypeInfo::Boolean => "i1".into(),
            TypeInfo::Integer { size, .. } => format!("i{}", size * 8),
            TypeInfo::Pointer { pointee_type, .. } => {
                format!("{}*", self.get_llvm_syntax(pointee_type))
            }
            TypeInfo::Array { item_type, length } => match length {
                Some(length) => format!("[{} x {}]", length, self.get_llvm_syntax(item_type)),
                None => self.get_llvm_syntax(item_type).into(),
            }
            TypeInfo::Structure { .. } | TypeInfo::Opaque { .. } => {
                format!("%\"type.{identifier}\"")
            },
            TypeInfo::Function { ref signature, .. } => {
                let mut syntax = format!("{}(", self.get_llvm_syntax(signature.return_type()));
                let mut parameters_iter = signature.parameter_types().iter();
                if let Some(&parameter) = parameters_iter.next() {
                    syntax.push_str(self.get_llvm_syntax(parameter));
                    for &parameter in parameters_iter {
                        syntax.push_str(", ");
                        syntax.push_str(self.get_llvm_syntax(parameter));
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
            TypeInfo::Alias { target } => self.get_llvm_syntax(target).into(),
        }
    }

    pub fn can_coerce(&self, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> bool {
        from_type == to_type || match (self.get_info(from_type), self.get_info(to_type)) {
            (
                &TypeInfo::Pointer { pointee_type: from_pointee, semantics: from_semantics },
                &TypeInfo::Pointer { pointee_type: to_pointee, semantics: to_semantics },
            ) => {
                // Not sure if this is right... needs testing
                use PointerSemantics::*;
                match (from_semantics, to_semantics) {
                    (Immutable, Immutable) => self.can_coerce(from_pointee, to_pointee, false),
                    (Immutable, _) => false,
                    (Mutable, Immutable) => self.can_coerce(from_pointee, to_pointee, true),
                    (Mutable, Mutable) => self.can_coerce(from_pointee, to_pointee, true),
                    (Mutable, _) => false,
                    (Owned, Immutable) => self.can_coerce(from_pointee, to_pointee, from_mutable),
                    (Owned, Mutable) => from_mutable && self.can_coerce(from_pointee, to_pointee, from_mutable),
                    (Owned, Owned) => self.can_coerce(from_pointee, to_pointee, from_mutable),
                }
            },
            (
                &TypeInfo::Array { item_type: from_item, length: Some(from_length) },
                &TypeInfo::Array { item_type: to_item, length: Some(to_length) },
            ) => {
                from_length == to_length && self.can_coerce(from_item, to_item, from_mutable)
            },
            (
                &TypeInfo::Array { item_type: from_item, length: _ },
                &TypeInfo::Array { item_type: to_item, length: None },
            ) => {
                self.can_coerce(from_item, to_item, from_mutable)
            },
            (
                TypeInfo::Function { signature: from_signature },
                TypeInfo::Function { signature: to_signature },
            ) => {
                from_signature.is_variadic() == to_signature.is_variadic()
                    && self.can_coerce(from_signature.return_type(), to_signature.return_type(), false)
                    && from_signature.parameter_types().len() == to_signature.parameter_types().len()
                    && std::iter::zip(from_signature.parameter_types(), to_signature.parameter_types())
                        .all(|(&from_param, &to_param)| {
                            self.can_coerce(from_param, to_param, false)
                        })
            },
            (TypeInfo::Void, _) | (_, TypeInfo::Void) => true,
            _ => false
        }
    }
}
