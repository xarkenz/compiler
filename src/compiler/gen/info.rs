use super::*;

use crate::sema::*;

use std::fmt;

fn quote_identifier_if_needed(mut identifier: String) -> String {
    let needs_quotes = identifier.contains(|ch| !matches!(ch,
        '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_' | '.' | '$'
    ));

    if needs_quotes {
        identifier.insert(0, '"');
        identifier.push('"');
    }

    identifier
}

#[derive(Clone, PartialEq, Debug)]
pub enum IntegerValue {
    Signed8(i8),
    Unsigned8(u8),
    Signed16(i16),
    Unsigned16(u16),
    Signed32(i32),
    Unsigned32(u32),
    Signed64(i64),
    Unsigned64(u64),
}

impl IntegerValue {
    pub fn new(raw: i128, type_info: &TypeInfo) -> Option<Self> {
        match type_info {
            TypeInfo::Integer { size: 1, signed: true } => Some(Self::Signed8(raw as i8)),
            TypeInfo::Integer { size: 1, signed: false } => Some(Self::Unsigned8(raw as u8)),
            TypeInfo::Integer { size: 2, signed: true } => Some(Self::Signed16(raw as i16)),
            TypeInfo::Integer { size: 2, signed: false } => Some(Self::Unsigned16(raw as u16)),
            TypeInfo::Integer { size: 4, signed: true } => Some(Self::Signed32(raw as i32)),
            TypeInfo::Integer { size: 4, signed: false } => Some(Self::Unsigned32(raw as u32)),
            TypeInfo::Integer { size: 8, signed: true } => Some(Self::Signed64(raw as i64)),
            TypeInfo::Integer { size: 8, signed: false } => Some(Self::Unsigned64(raw as u64)),
            _ => None
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Signed8(..) | Self::Unsigned8(..) => 1,
            Self::Signed16(..) | Self::Unsigned16(..) => 2,
            Self::Signed32(..) | Self::Unsigned32(..) => 4,
            Self::Signed64(..) | Self::Unsigned64(..) => 8,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Self::Signed8(..) | Self::Signed16(..) | Self::Signed32(..) | Self::Signed64(..) => true,
            Self::Unsigned8(..) | Self::Unsigned16(..) | Self::Unsigned32(..) | Self::Unsigned64(..) => false,
        }
    }

    pub fn expanded_value(&self) -> i128 {
        match self {
            &IntegerValue::Signed8(value) => value as i128,
            &IntegerValue::Unsigned8(value) => value as i128,
            &IntegerValue::Signed16(value) => value as i128,
            &IntegerValue::Unsigned16(value) => value as i128,
            &IntegerValue::Signed32(value) => value as i128,
            &IntegerValue::Unsigned32(value) => value as i128,
            &IntegerValue::Signed64(value) => value as i128,
            &IntegerValue::Unsigned64(value) => value as i128,
        }
    }

    pub fn value_type(&self) -> TypeHandle {
        match self {
            Self::Signed8(..) => TypeHandle::I8,
            Self::Unsigned8(..) => TypeHandle::U8,
            Self::Signed16(..) => TypeHandle::I16,
            Self::Unsigned16(..) => TypeHandle::U16,
            Self::Signed32(..) => TypeHandle::I32,
            Self::Unsigned32(..) => TypeHandle::U32,
            Self::Signed64(..) => TypeHandle::I64,
            Self::Unsigned64(..) => TypeHandle::U64,
        }
    }
}

impl fmt::Display for IntegerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signed8(value) => write!(f, "{value}"),
            Self::Unsigned8(value) => write!(f, "{value}"),
            Self::Signed16(value) => write!(f, "{value}"),
            Self::Unsigned16(value) => write!(f, "{value}"),
            Self::Signed32(value) => write!(f, "{value}"),
            Self::Unsigned32(value) => write!(f, "{value}"),
            Self::Signed64(value) => write!(f, "{value}"),
            Self::Unsigned64(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Register {
    identifier: String,
    value_type: TypeHandle,
    is_global: bool,
}

impl Register {
    pub fn new_global(identifier: String, value_type: TypeHandle) -> Self {
        Self {
            identifier: quote_identifier_if_needed(identifier),
            value_type,
            is_global: true,
        }
    }

    pub fn new_local(identifier: String, value_type: TypeHandle) -> Self {
        Self {
            identifier: quote_identifier_if_needed(identifier),
            value_type,
            is_global: false,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn value_type(&self) -> TypeHandle {
        self.value_type
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }
    
    pub fn get_llvm_syntax(&self) -> String {
        if self.is_global() {
            format!("@{}", self.identifier())
        }
        else {
            format!("%{}", self.identifier())
        }
    }
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.identifier().partial_cmp(other.identifier())
    }
}

impl Ord for Register {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Constant {
    Undefined(TypeHandle),
    Poison(TypeHandle),
    ZeroInitializer(TypeHandle),
    NullPointer(TypeHandle),
    Boolean(bool),
    Integer(IntegerValue),
    String {
        array_type: TypeHandle,
        value: token::StringValue,
    },
    Array {
        array_type: TypeHandle,
        items: Vec<Constant>,
    },
    Structure {
        struct_type: TypeHandle,
        members: Vec<Constant>,
    },
    Register(Register),
    Indirect {
        pointee_type: TypeHandle,
        pointer: Box<Constant>,
    },
    BitwiseCast {
        result_type: TypeHandle,
        value: Box<Constant>,
    },
    GetElementPointer {
        result_type: TypeHandle,
        aggregate_type: TypeHandle,
        pointer: Box<Constant>,
        indices: Vec<Constant>,
    },
    Scope(Scope),
}

impl Constant {
    pub fn get_type(&self) -> TypeHandle {
        match *self {
            Self::Undefined(value_type) => value_type,
            Self::Poison(value_type) => value_type,
            Self::ZeroInitializer(value_type) => value_type,
            Self::NullPointer(value_type) => value_type,
            Self::Boolean(..) => TypeHandle::BOOL,
            Self::Integer(ref integer) => integer.value_type(),
            Self::String { array_type, .. } => array_type,
            Self::Array { array_type, .. } => array_type,
            Self::Structure { struct_type, .. } => struct_type,
            Self::Register(ref register) => register.value_type(),
            Self::Indirect { pointee_type, .. } => pointee_type,
            Self::BitwiseCast { result_type, .. } => result_type,
            Self::GetElementPointer { result_type, .. } => result_type,
            Self::Scope(..) => TypeHandle::META,
        }
    }

    pub fn get_llvm_syntax(&self, registry: &TypeRegistry) -> String {
        match self {
            Self::Undefined(..) => "undef".into(),
            Self::Poison(..) => "poison".into(),
            Self::ZeroInitializer(..) => "zeroinitializer".into(),
            Self::NullPointer(..) => "null".into(),
            Self::Boolean(value) => format!("{value}"),
            Self::Integer(value) => format!("{value}"),
            Self::String { value, .. } => format!("{value}"),
            Self::Array { items, .. } => {
                let mut items_iter = items.iter();
                if let Some(item) = items_iter.next() {
                    let mut syntax = String::from("[ ");
                    syntax.push_str(registry.get_llvm_syntax(item.get_type()));
                    syntax.push(' ');
                    syntax.push_str(&item.get_llvm_syntax(registry));
                    for item in items_iter {
                        syntax.push_str(", ");
                        syntax.push_str(registry.get_llvm_syntax(item.get_type()));
                        syntax.push(' ');
                        syntax.push_str(&item.get_llvm_syntax(registry));
                    }
                    syntax.push_str(" ]");
                    syntax
                }
                else {
                    "[]".into()
                }
            },
            Self::Structure { members, .. } => {
                let mut members_iter = members.iter();
                if let Some(member) = members_iter.next() {
                    let mut syntax = String::from("{ ");
                    syntax.push_str(registry.get_llvm_syntax(member.get_type()));
                    syntax.push(' ');
                    syntax.push_str(&member.get_llvm_syntax(registry));
                    for member in members_iter {
                        syntax.push_str(", ");
                        syntax.push_str(registry.get_llvm_syntax(member.get_type()));
                        syntax.push(' ');
                        syntax.push_str(&member.get_llvm_syntax(registry));
                    }
                    syntax.push_str(" }");
                    syntax
                }
                else {
                    "{}".into()
                }
            },
            Self::Register(register) => register.get_llvm_syntax(),
            Self::Indirect { pointer, .. } => format!("<ERROR indirect constant: {}>", pointer.get_llvm_syntax(registry)),
            Self::BitwiseCast { value, result_type: to_type } => {
                let value_type = value.get_type();
                let value_syntax = value.get_llvm_syntax(registry);
                format!(
                    "bitcast ({} {value_syntax} to {})",
                    registry.get_llvm_syntax(value_type),
                    registry.get_llvm_syntax(*to_type),
                )
            },
            Self::GetElementPointer { aggregate_type, pointer, indices, .. } => {
                let mut syntax = format!(
                    "getelementptr inbounds ({}, {} {}",
                    registry.get_llvm_syntax(*aggregate_type),
                    registry.get_llvm_syntax(pointer.get_type()),
                    pointer.get_llvm_syntax(registry),
                );
                for index in indices {
                    syntax.push_str(", ");
                    syntax.push_str(registry.get_llvm_syntax(index.get_type()));
                    syntax.push(' ');
                    syntax.push_str(&index.get_llvm_syntax(registry));
                }
                syntax.push(')');
                syntax
            },
            Self::Scope(..) => "<ERROR scope constant>".into(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Never,
    Break,
    Continue,
    Void,
    Constant(Constant),
    Register(Register),
    Indirect {
        pointer: Box<Value>,
        pointee_type: TypeHandle,
    },
    BoundFunction {
        self_value: Box<Value>,
        function_register: Register,
    },
}

impl Value {
    pub fn get_type(&self) -> TypeHandle {
        match *self {
            Self::Never | Self::Break | Self::Continue => TypeHandle::NEVER,
            Self::Void => TypeHandle::VOID,
            Self::Constant(ref constant) => constant.get_type(),
            Self::Register(ref register) => register.value_type(),
            Self::Indirect { pointee_type, .. } => pointee_type,
            Self::BoundFunction { ref function_register, .. } => function_register.value_type(),
        }
    }

    pub fn into_mutable_lvalue(self, registry: &TypeRegistry) -> crate::Result<(Self, TypeHandle)> {
        match self {
            Self::Indirect { pointer, pointee_type } => {
                if let &TypeInfo::Pointer { semantics: PointerSemantics::Mutable, .. } = registry.get_info(pointer.get_type()) {
                    Ok((*pointer, pointee_type))
                }
                else {
                    Err(Box::new(crate::Error::CannotMutateValue { type_name: registry.get_identifier(pointee_type).into() }))
                }
            },
            _ => {
                Err(Box::new(crate::Error::ExpectedLValue {}))
            }
        }
    }

    pub fn bound_self_value(&self) -> Option<Value> {
        match self {
            Self::BoundFunction { self_value, .. } => Some(self_value.as_ref().clone()),
            _ => None
        }
    }

    pub fn get_llvm_syntax(&self, registry: &TypeRegistry) -> String {
        match self {
            Self::Never | Self::Break | Self::Continue => "<ERROR never value>".into(),
            Self::Void => "<ERROR void value>".into(),
            Self::Constant(constant) => constant.get_llvm_syntax(registry),
            Self::Register(register) => register.get_llvm_syntax(),
            Self::Indirect { pointer, .. } => format!("<ERROR indirect value: {}>", pointer.get_llvm_syntax(registry)),
            Self::BoundFunction { function_register, .. } => function_register.get_llvm_syntax(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Label {
    name: String,
}

impl Label {
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    id: usize,
    name: Option<String>,
}

impl Scope {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn get_member_identifier(&self, member_name: &str) -> String {
        let mut member_identifier = self.name()
            .map_or_else(String::new, |name| format!("{name}::"));
        member_identifier.push_str(member_name);
        
        member_identifier
    }
}

impl PartialEq for Scope {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[derive(Clone, Debug)]
pub struct FunctionSymbol {
    register: Register,
    signature: FunctionSignature,
    is_defined: bool,
}

impl FunctionSymbol {
    pub fn register(&self) -> &Register {
        &self.register
    }

    pub fn signature(&self) -> &FunctionSignature {
        &self.signature
    }

    pub fn is_defined(&self) -> bool {
        self.is_defined
    }

    pub fn value(&self) -> Value {
        Value::Constant(Constant::Register(self.register.clone()))
    }
}

#[derive(Clone, Debug)]
pub struct TypeSymbol {
    type_handle: TypeHandle,
    member_scope: Scope,
}

impl TypeSymbol {
    pub fn type_handle(&self) -> TypeHandle {
        self.type_handle
    }

    pub fn member_scope(&self) -> &Scope {
        &self.member_scope
    }

    pub fn value(&self) -> Value {
        Value::Constant(Constant::Scope(self.member_scope.clone()))
    }
}

#[derive(Clone, Debug)]
pub enum Symbol {
    Global {
        name: String,
        scope: Scope,
        value: Value,
    },
    Local {
        name: String,
        scope: Scope,
        version: usize,
        value: Value,
    },
    Function {
        name: String,
        scope: Scope,
        content: FunctionSymbol,
    },
    Type {
        name: String,
        scope: Scope,
        content: TypeSymbol,
    },
}

impl Symbol {
    pub fn name(&self) -> &str {
        match self {
            Self::Global { name, .. } => name.as_str(),
            Self::Local { name, .. } => name.as_str(),
            Self::Function { name, .. } => name.as_str(),
            Self::Type { name, .. } => name.as_str(),
        }
    }

    pub fn scope(&self) -> &Scope {
        match self {
            Self::Global { scope, .. } => scope,
            Self::Local { scope, .. } => scope,
            Self::Function { scope, .. } => scope,
            Self::Type { scope, .. } => scope,
        }
    }

    pub fn value(&self) -> Value {
        match self {
            Self::Global { value, .. } => value.clone(),
            Self::Local { value, .. } => value.clone(),
            Self::Function { content, .. } => content.value(),
            Self::Type { content, .. } => content.value(),
        }
    }

    pub fn function_value(&self) -> Option<&FunctionSymbol> {
        match self {
            Self::Function { content, .. } => Some(content),
            _ => None
        }
    }

    pub fn type_value(&self) -> Option<&TypeSymbol> {
        match self {
            Self::Type { content, .. } => Some(content),
            _ => None
        }
    }
}

#[derive(Clone, Debug)]
struct SymbolTableNode {
    symbol: Symbol,
    next_node: Option<Box<SymbolTableNode>>,
}

#[derive(Debug)]
pub struct SymbolTable {
    hash_table_buckets: Vec<Option<Box<SymbolTableNode>>>,
    active_scopes: Vec<Scope>,
    next_scope_id: usize,
}

impl SymbolTable {
    pub fn new(capacity: usize) -> Self {
        let mut hash_table_bins = Vec::new();
        hash_table_bins.resize_with(capacity, || None);

        let outermost_scope = Scope {
            id: 0,
            name: None,
        };

        Self {
            hash_table_buckets: hash_table_bins,
            active_scopes: vec![outermost_scope],
            next_scope_id: 1,
        }
    }

    pub fn capacity(&self) -> usize {
        self.hash_table_buckets.len()
    }

    pub fn current_scope(&self) -> &Scope {
        self.active_scopes.last().unwrap()
    }

    pub fn create_inactive_scope(&mut self, name: Option<String>) -> Scope {
        let id = self.next_scope_id;
        self.next_scope_id += 1;

        Scope {
            id,
            name,
        }
    }

    pub fn enter_new_scope(&mut self) {
        let new_scope = self.create_inactive_scope(None);
        self.active_scopes.push(new_scope);
    }

    pub fn enter_scope(&mut self, scope: Scope) {
        self.active_scopes.push(scope);
    }

    pub fn leave_scope(&mut self) {
        if self.active_scopes.len() > 1 {
            self.active_scopes.pop();
        }
        else {
            // This should never occur other than in the case of programmer error
            panic!("attempted to leave outermost scope");
        }
    }

    pub fn scope_is_active(&self, scope: &Scope) -> bool {
        self.active_scopes.contains(scope)
    }

    pub fn find(&self, name: &str, in_scope: Option<&Scope>) -> Option<&Symbol> {
        let index = self.hash_index(name);

        self.find_in_bucket(index, name, in_scope)
    }

    pub fn find_mut(&mut self, name: &str, in_scope: Option<&Scope>) -> Option<&mut Symbol> {
        let index = self.hash_index(name);

        self.find_in_bucket_mut(index, name, in_scope)
    }

    pub fn clear_locals(&mut self) {
        for mut current_node_link in self.hash_table_buckets.iter_mut() {
            // FIXME: preferably could avoid the .as_mut().unwrap() with pattern matching but the borrow checker is weird
            // (As it turns out, the code with pattern matching compiles under the nightly Polonius feature... guess I'm waiting on that then)
            while current_node_link.is_some() {
                if let Symbol::Local { .. } = current_node_link.as_ref().unwrap().symbol {
                    // Remove the node by replacing the link to the current node with the link to the next node
                    let next_node_link = current_node_link.as_mut().unwrap().next_node.take();
                    *current_node_link = next_node_link;
                    // current_node_link already points to the next node, so it doesn't need to be advanced
                }
                else {
                    // Advance current_node_link to the next node
                    current_node_link = &mut current_node_link.as_mut().unwrap().next_node;
                }
            }
        }
    }

    pub fn next_local_symbol_version(&self, name: &str) -> usize {
        let index = self.hash_index(name);

        self.find_local_in_bucket(index, name)
            .map_or(0, |(_, _, version)| version + 1)
    }

    pub fn create_type_symbol(&self, name: String, type_handle: TypeHandle, member_scope: Scope) -> Symbol {
        Symbol::Type {
            name,
            scope: self.current_scope().clone(),
            content: TypeSymbol {
                type_handle,
                member_scope,
            },
        }
    }

    pub fn create_function_symbol(&self, name: String, signature: FunctionSignature, is_defined: bool, registry: &mut TypeRegistry) -> (Symbol, Register) {
        let identifier = self.current_scope().get_member_identifier(&name);
        let function_type = registry.get_function_handle(&signature);
        let register = Register::new_global(identifier, function_type);

        let symbol = Symbol::Function {
            name,
            scope: self.current_scope().clone(),
            content: FunctionSymbol {
                register: register.clone(),
                signature,
                is_defined,
            },
        };

        (symbol, register)
    }

    pub fn create_global_indirect_symbol(&self, name: String, pointee_type: TypeHandle, is_mutable: bool, registry: &mut TypeRegistry) -> (Symbol, Register) {
        let identifier = self.current_scope().get_member_identifier(&name);
        let semantics = PointerSemantics::simple(is_mutable);
        let pointer_type = registry.get_pointer_handle(pointee_type, semantics);
        let pointer = Register::new_global(identifier, pointer_type);

        let symbol = Symbol::Global {
            name,
            scope: self.current_scope().clone(),
            value: Value::Indirect {
                pointer: Box::new(Value::Register(pointer.clone())),
                pointee_type,
            },
        };

        (symbol, pointer)
    }

    pub fn create_local_indirect_symbol(&self, name: String, pointee_type: TypeHandle, is_mutable: bool, registry: &mut TypeRegistry) -> (Symbol, Register) {
        let version = self.next_local_symbol_version(&name);
        let identifier = match version {
            0 => format!("{name}"),
            1.. => format!("{name}-{version}"),
        };
        let semantics = PointerSemantics::simple(is_mutable);
        let pointer_type = registry.get_pointer_handle(pointee_type, semantics);
        let pointer = Register::new_local(identifier, pointer_type);

        let symbol = Symbol::Local {
            name,
            scope: self.current_scope().clone(),
            version,
            value: Value::Indirect {
                pointer: Box::new(Value::Register(pointer.clone())),
                pointee_type,
            },
        };

        (symbol, pointer)
    }

    pub fn create_indirect_local_constant_symbol(&self, name: String, pointee_type: TypeHandle, function_name: &str, registry: &mut TypeRegistry) -> (Symbol, Register) {
        let version = self.next_local_symbol_version(&name);
        let identifier = match version {
            0 => format!("{function_name}.{name}"),
            1.. => format!("{function_name}.{name}-{version}"),
        };
        let pointer_type = registry.get_pointer_handle(pointee_type, PointerSemantics::Immutable);
        let pointer = Register::new_global(identifier, pointer_type);

        let symbol = Symbol::Local {
            name,
            scope: self.current_scope().clone(),
            version,
            value: Value::Constant(Constant::Indirect {
                pointer: Box::new(Constant::Register(pointer.clone())),
                pointee_type,
            }),
        };

        (symbol, pointer)
    }

    pub fn insert(&mut self, symbol: Symbol) {
        let index = self.hash_index(symbol.name());
        
        let root_node = &mut self.hash_table_buckets[index];
        let node_to_insert = SymbolTableNode {
            symbol,
            next_node: root_node.take(),
        };
        *root_node = Some(Box::new(node_to_insert));
    }

    fn hash_index(&self, key: &str) -> usize {
        // https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function#FNV_offset_basis
        const FNV_OFFSET_BASIS: u64 = 0xCBF29CE484222325;
        // Any large prime number will do
        const FNV_PRIME: u64 = 0x100000001B3;

        let mut hash = FNV_OFFSET_BASIS;
        for byte_value in key.bytes() {
            hash ^= byte_value as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        hash as usize % self.capacity()
    }

    fn find_in_bucket(&self, index: usize, name: &str, in_scope: Option<&Scope>) -> Option<&Symbol> {
        let mut next_node = self.hash_table_buckets.get(index)?.as_deref();

        while let Some(current_node) = next_node {
            // FIXME: probably need to do more than just `self.active_scopes.contains()`` to ensure the retrieved symbol is from the *closest* scope
            if current_node.symbol.name() == name && in_scope.map_or_else(
                || self.active_scopes.contains(current_node.symbol.scope()),
                |in_scope| in_scope == current_node.symbol.scope(),
            ) {
                return Some(&current_node.symbol);
            }
            next_node = current_node.next_node.as_deref();
        }

        None
    }

    fn find_in_bucket_mut(&mut self, index: usize, name: &str, in_scope: Option<&Scope>) -> Option<&mut Symbol> {
        let mut next_node = self.hash_table_buckets.get_mut(index)?.as_deref_mut();

        while let Some(current_node) = next_node {
            if current_node.symbol.name() == name && in_scope.map_or_else(
                || self.active_scopes.contains(current_node.symbol.scope()),
                |in_scope| in_scope == current_node.symbol.scope(),
            ) {
                return Some(&mut current_node.symbol);
            }
            next_node = current_node.next_node.as_deref_mut();
        }

        None
    }

    fn find_local_in_bucket(&self, index: usize, name: &str) -> Option<(&Value, &Scope, usize)> {
        let mut next_node = self.hash_table_buckets.get(index)?.as_deref();

        while let Some(current_node) = next_node {
            if let Symbol::Local { name: symbol_name, value, scope, version } = &current_node.symbol {
                if symbol_name == name {
                    return Some((value, scope, *version));
                }
            }
            next_node = current_node.next_node.as_deref();
        }

        None
    }
}