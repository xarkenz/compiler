use super::*;

#[derive(Debug)]
pub struct LocalContext {
    function_identifier: String,
    return_type: TypeHandle,
    break_stack: Vec<Label>,
    continue_stack: Vec<Label>,
    symbol_versions: HashMap<String, usize>,
    scope_stack: Vec<HashMap<String, Value>>,
    next_anonymous_register_id: usize,
    next_basic_block_id: usize,
}

impl LocalContext {
    pub fn new(function_identifier: String, return_type: TypeHandle) -> Self {
        Self {
            function_identifier,
            return_type,
            break_stack: Vec::new(),
            continue_stack: Vec::new(),
            symbol_versions: HashMap::new(),
            scope_stack: vec![HashMap::new()],
            next_anonymous_register_id: 0,
            next_basic_block_id: 0,
        }
    }

    pub fn function_identifier(&self) -> &str {
        &self.function_identifier
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

    pub fn define_indirect_symbol(&mut self, name: String, pointer_type: TypeHandle, pointee_type: TypeHandle) -> Register {
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

    pub fn define_indirect_constant_symbol(&mut self, name: String, pointer_type: TypeHandle, pointee_type: TypeHandle) -> Register {
        let version = *self.symbol_versions.entry(name.clone())
            .and_modify(|version| *version += 1)
            .or_insert(0);
        let identifier = match version {
            0 => format!("{}.{name}", self.function_identifier()),
            1.. => format!("{}.{name}-{version}", self.function_identifier()),
        };
        let register = Register::new_global(identifier, pointer_type);
        let value = Value::Constant(Constant::Indirect {
            pointer: Box::new(Constant::Register(register.clone())),
            pointee_type,
        });

        self.define_symbol(name, value);

        register
    }

    pub fn define_symbol(&mut self, name: String, value: Value) {
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
}
