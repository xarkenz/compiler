use super::*;

#[derive(Clone, Debug)]
pub struct GlobalSymbol {
    pub value: Value,
    pub is_defined: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ContainerHandle {
    Module(ModuleHandle),
    Type(TypeHandle),
}

impl ContainerHandle {
    pub fn container_binding(self, name: &str, context: &GlobalContext) -> Option<ContainerHandle> {
        match self {
            ContainerHandle::Module(handle) => {
                context.module_info(handle).container_binding(name)
            }
            ContainerHandle::Type(..) => None,
        }
    }

    pub fn module_binding(self, name: &str, context: &GlobalContext) -> Option<ModuleHandle> {
        match self {
            ContainerHandle::Module(handle) => {
                context.module_info(handle).module_binding(name)
            }
            ContainerHandle::Type(..) => None,
        }
    }

    pub fn type_binding(self, name: &str, context: &GlobalContext) -> Option<TypeHandle> {
        match self {
            ContainerHandle::Module(handle) => {
                context.module_info(handle).type_binding(name)
            }
            ContainerHandle::Type(..) => None,
        }
    }

    pub fn find_symbol<'a>(self, name: &str, context: &'a GlobalContext) -> Option<&'a GlobalSymbol> {
        match self {
            ContainerHandle::Module(handle) => {
                context.module_info(handle).find_symbol(name)
            }
            ContainerHandle::Type(handle) => {
                context.find_type_implementation_symbol(handle, name)
            }
        }
    }
}

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
    container_bindings: HashMap<String, ContainerHandle>,
    symbols: HashMap<String, GlobalSymbol>
}

impl ModuleInfo {
    pub fn new(identifier: String, super_module: Option<ModuleHandle>) -> Self {
        Self {
            identifier,
            super_module,
            container_bindings: HashMap::new(),
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
    
    pub fn container_binding(&self, name: &str) -> Option<ContainerHandle> {
        self.container_bindings.get(name).copied()
    }
    
    pub fn module_binding(&self, name: &str) -> Option<ModuleHandle> {
        if let Some(ContainerHandle::Module(handle)) = self.container_binding(name) {
            Some(handle)
        }
        else {
            None
        }
    }
    
    pub fn bind_module(&mut self, name: String, handle: ModuleHandle) -> crate::Result<()> {
        if self.container_bindings.contains_key(&name) {
            Err(Box::new(crate::Error::TypeSymbolConflict { name: name.clone() }))
        }
        else {
            self.container_bindings.insert(name, ContainerHandle::Module(handle));
            Ok(())
        }
    }
    
    pub fn type_binding(&self, name: &str) -> Option<TypeHandle> {
        if let Some(ContainerHandle::Type(handle)) = self.container_binding(name) {
            Some(handle)
        }
        else {
            None
        }
    }
    
    pub fn bind_type(&mut self, name: String, handle: TypeHandle) -> crate::Result<()> {
        if self.container_bindings.contains_key(&name) {
            Err(Box::new(crate::Error::TypeSymbolConflict { name: name.clone() }))
        }
        else {
            self.container_bindings.insert(name, ContainerHandle::Type(handle));
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
