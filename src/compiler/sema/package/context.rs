use super::*;

pub struct PackageContext {
    info: Rc<PackageInfo>,
    current_module: NamespaceHandle,
    current_self_type: Option<TypeHandle>,
    source_paths: Vec<PathBuf>,
    parse_queue: VecDeque<SimplePath>,
    fill_phase_complete: bool,
}

impl PackageContext {
    pub fn new(info: Rc<PackageInfo>, package_root_module: NamespaceHandle) -> Self {
        let main_module_path = SimplePath::empty().into_child(info.name());
        Self {
            info,
            current_module: package_root_module,
            current_self_type: None,
            source_paths: Vec::new(),
            parse_queue: VecDeque::from([main_module_path]),
            fill_phase_complete: false,
        }
    }

    pub fn info(&self) -> &PackageInfo {
        &self.info
    }

    pub fn source_paths(&self) -> &[PathBuf] {
        &self.source_paths
    }

    /// The namespace of the module currently being analyzed.
    pub fn current_module(&self) -> NamespaceHandle {
        self.current_module
    }

    pub fn replace_current_module(&mut self, module: NamespaceHandle) -> NamespaceHandle {
        std::mem::replace(&mut self.current_module, module)
    }

    /// The type `Self` currently represents, or `None` if not analyzing an `implement` block or
    /// `struct` definition.
    pub fn current_self_type(&self) -> Option<TypeHandle> {
        self.current_self_type
    }

    pub fn set_self_type(&mut self, self_type: TypeHandle) {
        if self.current_self_type.is_some() {
            panic!("'Self' type should not have been set");
        }

        self.current_self_type = Some(self_type);
    }

    pub fn unset_self_type(&mut self) -> TypeHandle {
        self.current_self_type.take()
            .expect("'Self' type should have been set")
    }

    /// Flag for whether the fill phase has been completed. If so, all type properties are known.
    pub fn fill_phase_complete(&self) -> bool {
        self.fill_phase_complete
    }

    pub fn complete_fill_phase(&mut self) {
        self.fill_phase_complete = true;
    }

    pub fn get_next_module_to_parse(&mut self) -> Option<SimplePath> {
        self.parse_queue.pop_front()
    }

    pub fn queue_module_file(&mut self, module_path: SimplePath) {
        self.parse_queue.push_back(module_path);
    }

    pub fn register_source_path(&mut self, path: PathBuf) -> Option<usize> {
        if self.source_paths.contains(&path) {
            None
        }
        else {
            let source_id = self.source_paths.len();
            self.source_paths.push(path);
            Some(source_id)
        }
    }

    pub fn get_file_path_for_module(&self, module_path: &SimplePath) -> PathBuf {
        let parent_module_path = module_path.parent()
            .expect("module path should have at least 1 segment");

        if parent_module_path.is_empty() {
            self.info.main_path().to_path_buf()
        }
        else {
            let mut file_path = self.info
                .main_path()
                .parent()
                .expect("package.main_path should have a parent")
                .join("modules");
            file_path.push(parent_module_path
                .segments()[1..] // Skip package name
                .join(std::path::MAIN_SEPARATOR_STR));
            file_path.push(format!("{}.cupr", module_path.tail_name().unwrap()));
            file_path
        }
    }
}
