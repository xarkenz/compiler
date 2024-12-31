use super::*;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModuleHandle(NonZeroUsize);

impl ModuleHandle {
    pub const ROOT: ModuleHandle = ModuleHandle::new(0);

    pub const fn new(registry_index: usize) -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(registry_index + 1) })
    }

    pub const fn registry_index(self) -> usize {
        self.0.get() - 1
    }
}

#[derive(Clone, Debug)]
pub struct ModuleInfo {
    identifier: String,
    super_module: Option<ModuleHandle>,
    module_bindings: HashMap<String, ModuleHandle>,
    type_bindings: HashMap<String, TypeHandle>,
    symbols: HashMap<String, GlobalSymbol>
}

#[derive(Clone, Debug)]
pub struct GlobalSymbol {
    pub value: Value,
    pub is_defined: bool,
}

impl ModuleInfo {
    pub fn new(identifier: String, super_module: Option<ModuleHandle>) -> Self {
        Self {
            identifier,
            super_module,
            module_bindings: HashMap::new(),
            type_bindings: HashMap::new(),
            symbols: HashMap::new(),
        }
    }
    
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    
    pub fn super_module(&self) -> Option<ModuleHandle> {
        self.super_module
    }

    pub fn create_member_identifier(&self, member_name: &str) -> String {
        if self.identifier().is_empty() {
            member_name.into()
        }
        else {
            format!("{}::{member_name}", self.identifier())
        }
    }
    
    pub fn module_binding(&self, name: &str) -> Option<ModuleHandle> {
        self.module_bindings.get(name).copied()
    }
    
    pub fn bind_module(&mut self, name: String, handle: ModuleHandle) -> crate::Result<()> {
        if self.module_bindings.contains_key(&name) || self.type_bindings.contains_key(&name) {
            Err(Box::new(crate::Error::TypeSymbolConflict { name: name.clone() }))
        }
        else {
            self.module_bindings.insert(name, handle);
            Ok(())
        }
    }
    
    pub fn type_binding(&self, name: &str) -> Option<TypeHandle> {
        self.type_bindings.get(name).copied()
    }
    
    pub fn bind_type(&mut self, name: String, handle: TypeHandle) -> crate::Result<()> {
        if self.module_bindings.contains_key(&name) || self.type_bindings.contains_key(&name) {
            Err(Box::new(crate::Error::TypeSymbolConflict { name: name.clone() }))
        }
        else {
            self.type_bindings.insert(name, handle);
            Ok(())
        }
    }
    
    pub fn find_symbol(&self, name: &str) -> Option<&GlobalSymbol> {
        self.symbols.get(name)
    }
    
    pub fn insert_symbol(&mut self, name: String, symbol: GlobalSymbol) {
        self.symbols.insert(name, symbol);
    }
}
