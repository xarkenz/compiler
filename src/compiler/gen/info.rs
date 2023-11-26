use super::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Scope {
    id: usize,
}

impl Scope {
    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Symbol {
    identifier: String,
    value: Value,
    scope: Scope,
    version: usize,
}

impl Symbol {
    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn version(&self) -> usize {
        self.version
    }

    pub fn format(&self) -> ValueFormat {
        self.value.format()
    }
}

#[derive(Clone, Debug)]
struct SymbolTableNode {
    symbol: Symbol,
    next_node: Option<Box<SymbolTableNode>>,
}

#[derive(Debug)]
pub struct SymbolTable {
    is_global: bool,
    hash_table_bins: Vec<Option<SymbolTableNode>>,
    active_scopes: Vec<Scope>,
    next_scope_id: usize,
}

impl SymbolTable {
    pub fn new(capacity: usize, is_global: bool) -> Self {
        let mut hash_table_bins = Vec::new();
        hash_table_bins.resize_with(capacity, Default::default);
        Self {
            is_global,
            hash_table_bins,
            active_scopes: vec![Scope { id: 0 }],
            next_scope_id: 1,
        }
    }

    pub fn is_global(&self) -> bool {
        self.is_global
    }

    pub fn capacity(&self) -> usize {
        self.hash_table_bins.len()
    }

    pub fn clear(&mut self) {
        for root_node in self.hash_table_bins.iter_mut() {
            *root_node = None;
        }
    }

    pub fn current_scope(&self) -> &Scope {
        self.active_scopes.last().unwrap()
    }

    pub fn enter_scope(&mut self) {
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        self.active_scopes.push(Scope { id });
    }

    pub fn leave_scope(&mut self) {
        if self.active_scopes.len() > 1 {
            self.active_scopes.pop();
        }
        else {
            panic!("attempted to leave outermost scope");
        }
    }

    pub fn scope_is_active(&self, scope: &Scope) -> bool {
        self.active_scopes.contains(scope)
    }

    pub fn find(&self, identifier: &str) -> Option<&Symbol> {
        let index = self.hash_index(identifier);

        self.find_in_bin(index, identifier, true)
    }

    pub fn find_mut(&mut self, identifier: &str) -> Option<&mut Symbol> {
        let index = self.hash_index(identifier);

        self.find_in_bin_mut(index, identifier, true)
    }

    pub fn next_symbol_version(&self, identifier: &str) -> usize {
        let index = self.hash_index(identifier);

        self.find_in_bin(index, identifier, false)
            .map_or(0, |symbol| symbol.version() + 1)
    }

    pub fn create_register_symbol(&self, identifier: String, format: ValueFormat) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_symbol_version(&identifier);
        let qualified_name = if version == 0 {
            identifier.clone()
        } else {
            format!("{identifier}-{version}")
        };
        let register = Register {
            name: qualified_name,
            format,
            is_global: self.is_global(),
        };
        let symbol = Symbol {
            identifier,
            value: Value::Register(register.clone()),
            scope,
            version,
        };

        (symbol, register)
    }

    pub fn create_indirect_symbol(&self, identifier: String, loaded_format: ValueFormat) -> (Symbol, Register) {
        let scope = self.current_scope().clone();
        let version = self.next_symbol_version(&identifier);
        let qualified_name = if version == 0 {
            identifier.clone()
        } else {
            format!("{identifier}-{version}")
        };
        let pointer = Register {
            name: qualified_name,
            format: loaded_format.clone().into_pointer(),
            is_global: self.is_global(),
        };
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(pointer.clone())),
            loaded_format,
        };
        let symbol = Symbol {
            identifier,
            value,
            scope,
            version,
        };

        (symbol, pointer)
    }

    pub fn insert(&mut self, symbol: Symbol) {
        let index = self.hash_index(symbol.identifier());
        
        let root_node = &mut self.hash_table_bins[index];
        let node_to_insert = SymbolTableNode {
            symbol,
            next_node: root_node.take().map(|node| Box::new(node)),
        };
        *root_node = Some(node_to_insert);
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

    fn find_in_bin(&self, index: usize, identifier: &str, check_scope: bool) -> Option<&Symbol> {
        let mut next_node = self.hash_table_bins.get(index)?.as_ref();

        while let Some(current_node) = next_node {
            let is_in_scope = !check_scope || self.active_scopes.contains(current_node.symbol.scope());
            if current_node.symbol.identifier() == identifier && is_in_scope {
                return Some(&current_node.symbol);
            }
            next_node = current_node.next_node.as_deref();
        }

        None
    }

    fn find_in_bin_mut(&mut self, index: usize, identifier: &str, check_scope: bool) -> Option<&mut Symbol> {
        let mut next_node = self.hash_table_bins.get_mut(index)?.as_mut();

        while let Some(current_node) = next_node {
            // I would use self.scope_is_active() here, but that requires an immutable borrow of *self, not just self.active_scopes
            let is_in_scope = !check_scope || self.active_scopes.contains(current_node.symbol.scope());
            if current_node.symbol.identifier() == identifier && is_in_scope {
                return Some(&mut current_node.symbol);
            }
            next_node = current_node.next_node.as_deref_mut();
        }

        None
    }
}