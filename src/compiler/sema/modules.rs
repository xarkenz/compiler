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
    symbol_table: SymbolTable,
}

impl ModuleInfo {
    pub fn new(identifier: String, super_module: Option<ModuleHandle>) -> Self {
        Self {
            identifier,
            super_module,
            symbol_table: SymbolTable::new(),
        }
    }
    
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    
    pub fn super_module(&self) -> Option<ModuleHandle> {
        self.super_module
    }
    
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
    
    pub fn symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    pub fn create_member_identifier(&self, member_name: &str) -> String {
        if self.identifier().is_empty() {
            member_name.to_owned()
        }
        else {
            format!("{}::{member_name}", self.identifier())
        }
    }
}
