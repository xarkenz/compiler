use super::*;
use std::collections::HashMap;

struct TypeEntry {
    path: AbsolutePath,
    repr: TypeRepr,
    namespace: NamespaceHandle,
    alignment: Option<Option<u64>>,
    size: Option<Option<u64>>,
}

pub struct TypeRegistry {
    /// Table of all types in existence.
    type_table: Vec<TypeEntry>,
    path_base_types: HashMap<PathBaseType, TypeHandle>,
    pointer_types: HashMap<(TypeHandle, PointerSemantics), TypeHandle>,
    array_types: HashMap<(TypeHandle, Option<u64>), TypeHandle>,
    tuple_types: HashMap<Box<[TypeHandle]>, TypeHandle>,
    function_types: HashMap<FunctionSignature, TypeHandle>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            type_table: Vec::with_capacity(PRIMITIVE_TYPES.len()),
            path_base_types: HashMap::with_capacity(PRIMITIVE_TYPES.len()),
            pointer_types: HashMap::new(),
            array_types: HashMap::new(),
            tuple_types: HashMap::new(),
            function_types: HashMap::new(),
        }
    }

    pub fn populate_primitive_types<F>(
        &mut self,
        mut create_namespace: F,
        target: &TargetInfo,
    )
    where
        F: FnMut(AbsolutePath) -> NamespaceHandle,
    {
        if !self.type_table.is_empty() {
            panic!("primitive types have already been populated");
        }

        for (registry_index, &(name, ref repr)) in PRIMITIVE_TYPES.iter().enumerate() {
            let base_type = PathBaseType::Primitive(PrimitiveType {
                handle: TypeHandle::new(registry_index),
                name,
            });
            let path = AbsolutePath::at_base_type(Box::new(base_type.clone()));
            let namespace = create_namespace(path.clone());
            let handle = self.create_type(
                path,
                repr.resolve_primitive_type(target.pointer_size()),
                namespace,
                target,
                false,
            );
            self.path_base_types.insert(base_type, handle);
        }
    }

    fn type_entry(&self, handle: TypeHandle) -> &TypeEntry {
        &self.type_table[handle.registry_index()]
    }

    fn type_entry_mut(&mut self, handle: TypeHandle) -> &mut TypeEntry {
        &mut self.type_table[handle.registry_index()]
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

    pub fn type_alignment(&self, handle: TypeHandle) -> Option<u64> {
        self.type_entry(handle).alignment
            .expect("type alignment cannot be known before fill phase is completed")
    }

    pub fn type_size(&self, handle: TypeHandle) -> Option<u64> {
        self.type_entry(handle).size
            .expect("type size cannot be known before fill phase is completed")
    }

    pub fn create_type(&mut self, path: AbsolutePath, repr: TypeRepr, namespace: NamespaceHandle, target: &TargetInfo, fill_phase_complete: bool) -> TypeHandle {
        let handle = TypeHandle::new(self.type_table.len());

        let alignment = fill_phase_complete.then(|| self.calculate_alignment(&repr, target));
        let size = fill_phase_complete.then(|| self.calculate_size(&repr, target));

        self.type_table.push(TypeEntry {
            path,
            repr,
            namespace,
            alignment,
            size,
        });

        handle
    }

    pub fn update_type_repr(&mut self, handle: TypeHandle, repr: TypeRepr, target: &TargetInfo, fill_phase_complete: bool) {
        let alignment = fill_phase_complete.then(|| self.calculate_alignment(&repr, target));
        let size = fill_phase_complete.then(|| self.calculate_size(&repr, target));

        let entry = self.type_entry_mut(handle);
        entry.repr = repr;
        entry.alignment = alignment;
        entry.size = size;
    }

    pub fn path_base_type(&self, base_type: &PathBaseType) -> Option<TypeHandle> {
        self.path_base_types.get(base_type).copied()
    }

    pub fn set_path_base_type(&mut self, base_type: PathBaseType, handle: TypeHandle) {
        self.path_base_types.insert(base_type, handle);
    }

    /// Get `*T` or `*mut T` from `T` and the pointer semantics.
    pub fn get_pointer_type<F>(
        &mut self,
        pointee_type: TypeHandle,
        semantics: PointerSemantics,
        create_namespace: F,
        target: &TargetInfo,
        fill_phase_complete: bool,
    ) -> TypeHandle
    where
        F: FnOnce(AbsolutePath) -> NamespaceHandle,
    {
        if let Some(handle) = self.pointer_types.get(&(pointee_type, semantics)) {
            *handle
        }
        else {
            let repr = TypeRepr::Pointer {
                pointee_type,
                semantics,
            };
            let path = AbsolutePath::at_base_type(Box::new(PathBaseType::Pointer {
                pointee_type: self.type_path(pointee_type).clone(),
                semantics,
            }));

            let namespace = create_namespace(path.clone());
            let handle = self.create_type(path, repr, namespace, target, fill_phase_complete);
            self.pointer_types.insert((pointee_type, semantics), handle);
            handle
        }
    }

    /// Get `[T; N]` or `[T]` from `T` and an optional array length.
    pub fn get_array_type<F>(
        &mut self,
        item_type: TypeHandle,
        length: Option<u64>,
        create_namespace: F,
        target: &TargetInfo,
        fill_phase_complete: bool,
    ) -> TypeHandle
    where
        F: FnOnce(AbsolutePath) -> NamespaceHandle,
    {
        if let Some(handle) = self.array_types.get(&(item_type, length)) {
            *handle
        }
        else {
            let repr = TypeRepr::Array {
                item_type,
                length,
            };
            let path = AbsolutePath::at_base_type(Box::new(PathBaseType::Array {
                item_type: self.type_path(item_type).clone(),
                length,
            }));

            let namespace = create_namespace(path.clone());
            let handle = self.create_type(path, repr, namespace, target, fill_phase_complete);
            self.array_types.insert((item_type, length), handle);
            handle
        }
    }

    /// Get a tuple type from its item types.
    pub fn get_tuple_type<F>(
        &mut self,
        item_types: &[TypeHandle],
        create_namespace: F,
        target: &TargetInfo,
        fill_phase_complete: bool,
    ) -> TypeHandle
    where
        F: FnOnce(AbsolutePath) -> NamespaceHandle,
    {
        if let Some(handle) = self.tuple_types.get(item_types) {
            *handle
        }
        else {
            let repr = TypeRepr::Tuple {
                item_types: item_types.into(),
            };
            let path = AbsolutePath::at_base_type(Box::new(PathBaseType::Tuple {
                item_types: item_types
                    .iter()
                    .map(|&item_type| self.type_path(item_type).clone())
                    .collect(),
            }));

            let namespace = create_namespace(path.clone());
            let handle = self.create_type(path, repr, namespace, target, fill_phase_complete);
            self.tuple_types.insert(item_types.into(), handle);
            handle
        }
    }

    /// Get a function type from its signature.
    pub fn get_function_type<F>(
        &mut self,
        signature: &FunctionSignature,
        create_namespace: F,
        target: &TargetInfo,
        fill_phase_complete: bool,
    ) -> TypeHandle
    where
        F: FnOnce(AbsolutePath) -> NamespaceHandle,
    {
        if let Some(handle) = self.function_types.get(signature) {
            *handle
        }
        else {
            let repr = TypeRepr::Function {
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

            let namespace = create_namespace(path.clone());
            let handle = self.create_type(path, repr, namespace, target, fill_phase_complete);
            self.function_types.insert(signature.clone(), handle);
            handle
        }
    }

    pub fn try_implicit_conversion(&self, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> Option<Conversion> {
        Conversion::try_implicit(self, from_type, to_type, from_mutable)
    }

    pub fn try_explicit_conversion(&self, from_type: TypeHandle, to_type: TypeHandle, from_mutable: bool) -> Option<Conversion> {
        Conversion::try_explicit(self, from_type, to_type, from_mutable)
    }

    pub fn calculate_type_properties(&mut self, target: &TargetInfo) -> crate::Result<()> {
        let mut dependency_stack = Vec::new();
        for registry_index in 0 .. self.type_table.len() {
            self.calculate_properties_for_type(
                TypeHandle::new(registry_index),
                target,
                true,
                true,
                &mut dependency_stack,
            )?;
            if !dependency_stack.is_empty() {
                panic!("dependencies were handled incorrectly while calculating properties for type {registry_index}");
            }
        }

        Ok(())
    }

    fn calculate_properties_for_type(
        &mut self,
        handle: TypeHandle,
        target: &TargetInfo,
        get_alignment: bool,
        get_size: bool,
        dependency_stack: &mut Vec<TypeHandle>,
    ) -> crate::Result<()> {
        let entry = self.type_entry(handle);
        let get_alignment = get_alignment && entry.alignment.is_none();
        let get_size = get_size && entry.size.is_none();

        if !get_alignment && !get_size {
            return Ok(());
        }

        if dependency_stack.contains(&handle) {
            return Err(Box::new(crate::Error::new(
                None,
                crate::ErrorKind::RecursiveTypeDefinition {
                    type_name: self.type_path(handle).to_string(),
                },
            )));
        }
        dependency_stack.push(handle);

        let repr = self.type_repr(handle).clone();
        match repr {
            TypeRepr::Unresolved => {
                panic!("{handle:?} is still unresolved at the end of the fill phase");
            }
            TypeRepr::Pointer { pointee_type, .. } => {
                self.calculate_properties_for_type(
                    pointee_type,
                    target,
                    false,
                    false,
                    dependency_stack,
                )?;
            }
            TypeRepr::Array { item_type, length } => {
                self.calculate_properties_for_type(
                    item_type,
                    target,
                    get_alignment,
                    get_size && length.is_some(),
                    dependency_stack,
                )?;
            }
            TypeRepr::Tuple { ref item_types } => {
                for &item_type in item_types {
                    self.calculate_properties_for_type(
                        item_type,
                        target,
                        get_alignment || get_size,
                        get_size,
                        dependency_stack,
                    )?;
                }
            }
            TypeRepr::Structure { ref members, .. } => {
                for member in members {
                    self.calculate_properties_for_type(
                        member.member_type,
                        target,
                        get_alignment || get_size,
                        get_size,
                        dependency_stack,
                    )?;
                }
            }
            _ => {}
        }

        dependency_stack.pop();

        if get_alignment {
            let alignment = self.calculate_alignment(&repr, target);
            self.type_entry_mut(handle).alignment = Some(alignment);
        }
        if get_size {
            let size = self.calculate_size(&repr, target);
            self.type_entry_mut(handle).size = Some(size);
        }

        Ok(())
    }

    fn calculate_alignment(&self, repr: &TypeRepr, target: &TargetInfo) -> Option<u64> {
        match *repr {
            TypeRepr::Unresolved => None,
            TypeRepr::Meta => None,
            TypeRepr::Never => None,
            TypeRepr::Void => None,
            TypeRepr::Boolean => Some(1),
            TypeRepr::Integer { size, .. } => Some(size),
            TypeRepr::PointerSizedInteger { .. } => panic!("unresolved pointer sized integer"),
            TypeRepr::Float32 => Some(4),
            TypeRepr::Float64 => Some(8),
            TypeRepr::Pointer { .. } => Some(target.pointer_size()),
            TypeRepr::Function { .. } => Some(target.pointer_size()),
            TypeRepr::Array { item_type, .. } => self.type_alignment(item_type),
            TypeRepr::Tuple { ref item_types } => item_types
                .iter()
                .map(|&item_type| self.type_alignment(item_type))
                .max()
                .unwrap_or(Some(1)),
            TypeRepr::Structure { ref members, .. } => members
                .iter()
                .map(|member| self.type_alignment(member.member_type))
                .max()
                .unwrap_or(Some(1)),
            TypeRepr::OpaqueStructure { .. } => None,
        }
    }

    fn calculate_size(&self, repr: &TypeRepr, target: &TargetInfo) -> Option<u64> {
        match *repr {
            TypeRepr::Unresolved => None,
            TypeRepr::Meta => None,
            TypeRepr::Never => Some(0),
            TypeRepr::Void => Some(0),
            TypeRepr::Boolean => Some(1),
            TypeRepr::Integer { size, .. } => Some(size),
            TypeRepr::PointerSizedInteger { .. } => panic!("unresolved pointer sized integer"),
            TypeRepr::Float32 => Some(4),
            TypeRepr::Float64 => Some(8),
            TypeRepr::Pointer { .. } => Some(target.pointer_size()),
            TypeRepr::Function { .. } => Some(target.pointer_size()),
            TypeRepr::Array { item_type, length } => {
                let length = length?;
                let item_size = self.type_size(item_type)?;
                Some(length * item_size)
            }
            TypeRepr::Tuple { ref item_types } => {
                self.calculate_structure_size(item_types
                    .iter()
                    .cloned())
            }
            TypeRepr::Structure { ref members, .. } => {
                self.calculate_structure_size(members
                    .iter()
                    .map(|member| member.member_type))
            }
            TypeRepr::OpaqueStructure { .. } => None,
        }
    }

    fn calculate_structure_size(&self, member_types: impl IntoIterator<Item = TypeHandle>) -> Option<u64> {
        let mut current_size = 0;
        let mut max_alignment = 1;

        for member_type in member_types {
            let alignment = self.type_alignment(member_type).unwrap_or(0);
            max_alignment = max_alignment.max(alignment);

            // Calculate padding
            let intermediate_size = current_size + alignment - 1;
            let padded_size = intermediate_size - intermediate_size % alignment;
            current_size = padded_size + self.type_size(member_type)?;
        }

        // Pad for the largest member alignment
        let intermediate_size = current_size + max_alignment - 1;
        let padded_size = intermediate_size - intermediate_size % max_alignment;

        Some(padded_size)
    }

    pub fn finish_package(&mut self) {
        for type_entry in &mut self.type_table {
            type_entry.repr.set_external(true);
        }
    }
}
