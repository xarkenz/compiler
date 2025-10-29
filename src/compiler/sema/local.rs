use super::*;

#[derive(Debug)]
pub struct LocalContext {
    function_path: AbsolutePath,
    return_type: TypeHandle,
    break_stack: Vec<Label>,
    continue_stack: Vec<Label>,
    symbol_versions: HashMap<Box<str>, usize>,
    scope_stack: Vec<HashMap<Box<str>, Value>>,
    current_block_label: Label,
    next_anonymous_register_id: usize,
    next_basic_block_id: usize,
}

impl LocalContext {
    pub fn new(function_path: AbsolutePath, return_type: TypeHandle) -> Self {
        Self {
            function_path,
            return_type,
            break_stack: Vec::new(),
            continue_stack: Vec::new(),
            symbol_versions: HashMap::new(),
            scope_stack: vec![HashMap::new()],
            current_block_label: Label::new(".block.0".into()),
            next_anonymous_register_id: 0,
            next_basic_block_id: 1,
        }
    }

    pub fn function_path(&self) -> &AbsolutePath {
        &self.function_path
    }

    pub fn return_type(&self) -> TypeHandle {
        self.return_type
    }

    pub fn break_label(&self) -> Option<&Label> {
        self.break_stack.last()
    }

    pub fn continue_label(&self) -> Option<&Label> {
        self.continue_stack.last()
    }

    pub fn push_break_label(&mut self, label: Label) {
        self.break_stack.push(label);
    }

    pub fn pop_break_label(&mut self) {
        self.break_stack.pop().expect("attempted to pop from empty break stack");
    }

    pub fn push_continue_label(&mut self, label: Label) {
        self.continue_stack.push(label);
    }

    pub fn pop_continue_label(&mut self) {
        self.continue_stack.pop().expect("attempted to pop from empty continue stack");
    }

    pub fn enter_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scope_stack.pop();
        self.scope_stack.last().expect("attempted to exit the root local scope");
    }

    pub fn find_symbol(&self, name: &str) -> Option<&Value> {
        self.scope_stack.iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }

    pub fn define_indirect_symbol(&mut self, name: Box<str>, pointer_type: TypeHandle, pointee_type: TypeHandle) -> Register {
        let version = *self.symbol_versions.entry(name.clone())
            .and_modify(|version| *version += 1)
            .or_insert(0);
        let identifier = match version {
            0 => format!("{name}"),
            1.. => format!("{name}-{version}"),
        };
        let register = Register::new_local(identifier, pointer_type);
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(register.clone())),
            pointee_type,
        };

        self.define_symbol(name, value);

        register
    }

    pub fn define_indirect_constant_symbol(&mut self, name: Box<str>, pointer_type: TypeHandle, pointee_type: TypeHandle) -> Register {
        let version = *self.symbol_versions.entry(name.clone())
            .and_modify(|version| *version += 1)
            .or_insert(0);
        let identifier = match version {
            0 => format!("{}.{name}", self.function_path()),
            1.. => format!("{}.{name}-{version}", self.function_path()),
        };
        let register = Register::new_global(identifier, pointer_type);
        let value = Value::Constant(Constant::Indirect {
            pointer: Box::new(Constant::Register(register.clone())),
            pointee_type,
        });

        self.define_symbol(name, value);

        register
    }

    pub fn define_symbol(&mut self, name: Box<str>, value: Value) {
        self.scope_stack.last_mut().unwrap().insert(name, value);
    }

    pub fn new_anonymous_register(&mut self, value_type: TypeHandle) -> Register {
        let id = self.next_anonymous_register_id;
        self.next_anonymous_register_id += 1;

        Register::new_local(id.to_string(), value_type)
    }

    pub fn new_block_label(&mut self) -> Label {
        let id = self.next_basic_block_id;
        self.next_basic_block_id += 1;

        Label::new(format!(".block.{id}"))
    }

    pub fn current_block_label(&self) -> &Label {
        &self.current_block_label
    }

    pub fn set_current_block_label(&mut self, label: Label) {
        self.current_block_label = label;
    }
}
