use super::*;
use std::rc::Rc;
use crate::package::PackageInfo;

pub struct NamespaceRegistry {
    /// Table of all namespaces in existence.
    namespace_table: Vec<NamespaceInfo>,
}

impl NamespaceRegistry {
    pub fn new() -> Self {
        Self {
            namespace_table: vec![NamespaceInfo::new(AbsolutePath::at_root())],
        }
    }

    pub fn namespace_info(&self, handle: NamespaceHandle) -> &NamespaceInfo {
        &self.namespace_table[handle.registry_index()]
    }

    pub fn namespace_info_mut(&mut self, handle: NamespaceHandle) -> &mut NamespaceInfo {
        &mut self.namespace_table[handle.registry_index()]
    }

    pub fn create_namespace(&mut self, path: AbsolutePath) -> NamespaceHandle {
        let handle = NamespaceHandle::new(self.namespace_table.len());

        self.namespace_table.push(NamespaceInfo::new(path));

        handle
    }

    pub fn create_package_context(&mut self, info: Rc<PackageInfo>) -> PackageContext {
        let package_root_module = self.create_namespace(AbsolutePath::from_root(
            SimplePath::empty().into_child(info.name()),
        ));

        self.namespace_info_mut(NamespaceHandle::GLOBAL_ROOT)
            .define(info.name(), Symbol::new(SymbolKind::Module(package_root_module)))
            .expect("global root module should not have conflicts");

        PackageContext::new(info, package_root_module)
    }

    pub fn finish_package(&mut self) {
        for namespace_info in &mut self.namespace_table {
            for symbol in namespace_info.symbols.values_mut() {
                symbol.set_external(true);
            }
        }
    }
}

impl Default for NamespaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
