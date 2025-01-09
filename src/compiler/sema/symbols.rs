use super::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ContainerHandle {
    Type(TypeHandle),
    Module(ModuleHandle),
}

impl ContainerHandle {
    pub fn as_type(self) -> Option<TypeHandle> {
        match self {
            Self::Type(handle) => Some(handle),
            _ => None
        }
    }

    pub fn as_module(self) -> Option<ModuleHandle> {
        match self {
            Self::Module(handle) => Some(handle),
            _ => None
        }
    }
}

impl From<TypeHandle> for ContainerHandle {
    fn from(handle: TypeHandle) -> Self {
        Self::Type(handle)
    }
}

impl From<ModuleHandle> for ContainerHandle {
    fn from(handle: ModuleHandle) -> Self {
        Self::Module(handle)
    }
}

#[derive(Clone, Debug)]
pub struct SymbolTable {
    symbols: HashMap<String, Value>,
    unresolved_names: HashSet<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            unresolved_names: HashSet::new(),
        }
    }
    
    pub fn has_symbol(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
    
    pub fn is_unresolved(&self, name: &str) -> bool {
        self.unresolved_names.contains(name)
    }
    
    pub fn find(&self, name: &str) -> Option<&Value> {
        self.symbols.get(name)
    }
    
    pub fn find_container(&self, name: &str) -> Option<ContainerHandle> {
        self.find(name).and_then(Value::as_container)
    }

    pub fn find_type(&self, name: &str) -> Option<TypeHandle> {
        self.find_container(name).and_then(ContainerHandle::as_type)
    }
    
    pub fn find_module(&self, name: &str) -> Option<ModuleHandle> {
        self.find_container(name).and_then(ContainerHandle::as_module)
    }
    
    /// No collision checking.
    pub fn define(&mut self, name: String, value: Value) {
        self.unresolved_names.remove(&name);
        self.symbols.insert(name, value);
    }
    
    /// No collision checking.
    pub fn declare(&mut self, name: String, value: Value) {
        if !self.has_symbol(&name) {
            self.unresolved_names.insert(name.clone());
            self.symbols.insert(name, value);
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}
