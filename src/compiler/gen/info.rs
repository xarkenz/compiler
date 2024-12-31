use crate::sema::*;

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

    pub fn create_function_symbol(&self, name: String, signature: FunctionSignature, is_defined: bool, registry: &mut GlobalContext) -> (Symbol, Register) {
        let identifier = self.current_scope().get_member_identifier(&name);
        let function_type = registry.get_function_type(&signature);
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

    pub fn create_global_indirect_symbol(&self, name: String, pointee_type: TypeHandle, is_mutable: bool, registry: &mut GlobalContext) -> (Symbol, Register) {
        let identifier = self.current_scope().get_member_identifier(&name);
        let semantics = PointerSemantics::simple(is_mutable);
        let pointer_type = registry.get_pointer_type(pointee_type, semantics);
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

    pub fn create_local_indirect_symbol(&self, name: String, pointee_type: TypeHandle, is_mutable: bool, registry: &mut GlobalContext) -> (Symbol, Register) {
        let version = self.next_local_symbol_version(&name);
        let identifier = match version {
            0 => format!("{name}"),
            1.. => format!("{name}-{version}"),
        };
        let semantics = PointerSemantics::simple(is_mutable);
        let pointer_type = registry.get_pointer_type(pointee_type, semantics);
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

    pub fn create_indirect_local_constant_symbol(&self, name: String, pointee_type: TypeHandle, function_name: &str, registry: &mut GlobalContext) -> (Symbol, Register) {
        let version = self.next_local_symbol_version(&name);
        let identifier = match version {
            0 => format!("{function_name}.{name}"),
            1.. => format!("{function_name}.{name}-{version}"),
        };
        let pointer_type = registry.get_pointer_type(pointee_type, PointerSemantics::Immutable);
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