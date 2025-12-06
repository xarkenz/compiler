use super::*;
use std::collections::HashMap;
use crate::ir::FunctionDefinition;
use crate::ir::instr::{BasicBlock, Instruction, PhiInstruction, TerminatorInstruction};
use crate::ir::value::BlockLabel;

#[derive(Clone)]
pub struct BreakScope {
    label: BlockLabel,
    expected_type: TypeHandle,
    breaks: Vec<(Value, BlockLabel)>,
}

impl BreakScope {
    pub fn new(label: BlockLabel, expected_type: TypeHandle) -> Self {
        Self {
            label,
            expected_type,
            breaks: Vec::new(),
        }
    }

    pub fn label(&self) -> &BlockLabel {
        &self.label
    }

    pub fn expected_type(&self) -> TypeHandle {
        self.expected_type
    }

    pub fn breaks(&self) -> &[(Value, BlockLabel)] {
        &self.breaks
    }

    pub fn register_break(&mut self, value: Value, from_label: BlockLabel) {
        self.breaks.push((value, from_label));
    }
}

pub struct LocalContext {
    function: FunctionDefinition,
    function_path: AbsolutePath,
    current_block: BasicBlock,
    break_scope_stack: Vec<BreakScope>,
    continue_scope_stack: Vec<BlockLabel>,
    symbol_versions: HashMap<Box<str>, usize>,
    scope_stack: Vec<HashMap<Box<str>, Value>>,
    next_anonymous_register_id: usize,
    next_basic_block_id: usize,
}

impl LocalContext {
    pub fn new(function: FunctionDefinition, function_path: AbsolutePath) -> Self {
        Self {
            function,
            function_path,
            current_block: BasicBlock::new(BlockLabel::new(b".block.0".as_slice().into())),
            break_scope_stack: Vec::new(),
            continue_scope_stack: Vec::new(),
            symbol_versions: HashMap::new(),
            scope_stack: vec![HashMap::new()],
            next_anonymous_register_id: 0,
            next_basic_block_id: 1,
        }
    }

    pub fn function(&self) -> &FunctionDefinition {
        &self.function
    }

    pub fn function_mut(&mut self) -> &mut FunctionDefinition {
        &mut self.function
    }

    pub fn function_path(&self) -> &AbsolutePath {
        &self.function_path
    }

    pub fn return_type(&self) -> TypeHandle {
        self.function.return_type()
    }

    pub fn current_block(&self) -> &BasicBlock {
        &self.current_block
    }

    pub fn current_block_mut(&mut self) -> &mut BasicBlock {
        &mut self.current_block
    }

    pub fn add_phi(&mut self, phi: PhiInstruction) {
        self.current_block.add_phi(phi);
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.current_block.add_instruction(instruction);
    }

    pub fn set_terminator(&mut self, terminator: TerminatorInstruction) {
        self.current_block.set_terminator(terminator);
    }

    pub fn start_new_block(&mut self, label: BlockLabel) -> &BlockLabel {
        let finished_block = std::mem::replace(&mut self.current_block, BasicBlock::new(label));
        self.function.add_block(finished_block);
        self.function.blocks().last().unwrap().label()
    }

    pub fn break_scope(&self) -> Option<&BreakScope> {
        self.break_scope_stack.last()
    }

    pub fn break_scope_mut(&mut self) -> Option<&mut BreakScope> {
        self.break_scope_stack.last_mut()
    }

    pub fn continue_label(&self) -> Option<&BlockLabel> {
        self.continue_scope_stack.last()
    }

    pub fn enter_break_scope(&mut self, label: BlockLabel, expected_type: TypeHandle) {
        self.break_scope_stack.push(BreakScope::new(label, expected_type));
    }

    pub fn exit_break_scope(&mut self) {
        self.break_scope_stack.pop().expect("attempted to exit a nonexistent break scope");
    }

    pub fn enter_continue_scope(&mut self, label: BlockLabel) {
        self.continue_scope_stack.push(label);
    }

    pub fn exit_continue_scope(&mut self) {
        self.continue_scope_stack.pop().expect("attempted to exit a nonexistent continue scope");
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

    pub fn define_indirect_symbol(&mut self, name: Box<str>, pointer_type: TypeHandle, pointee_type: TypeHandle) -> LocalRegister {
        let version = *self.symbol_versions.entry(name.clone())
            .and_modify(|version| *version += 1)
            .or_insert(0);
        let identifier = match version {
            0 => name.as_bytes().into(),
            1.. => format!("{name}-{version}").as_bytes().into(),
        };
        let register = LocalRegister::new(identifier, pointer_type);
        let value = Value::Indirect {
            pointer: Box::new(Value::Register(register.clone())),
            pointee_type,
        };

        self.define_symbol(name, value);

        register
    }

    pub fn define_indirect_constant_symbol(&mut self, name: Box<str>, pointer_type: TypeHandle, pointee_type: TypeHandle) -> GlobalRegister {
        let version = *self.symbol_versions.entry(name.clone())
            .and_modify(|version| *version += 1)
            .or_insert(0);
        let identifier = match version {
            0 => format!("{}.{name}", self.function_path()).as_bytes().into(),
            1.. => format!("{}.{name}-{version}", self.function_path()).as_bytes().into(),
        };
        let register = GlobalRegister::new(identifier, pointer_type);
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

    pub fn new_anonymous_register(&mut self, value_type: TypeHandle) -> LocalRegister {
        let id = self.next_anonymous_register_id;
        self.next_anonymous_register_id += 1;

        LocalRegister::new(id.to_string().as_bytes().into(), value_type)
    }

    pub fn new_block_label(&mut self) -> BlockLabel {
        let id = self.next_basic_block_id;
        self.next_basic_block_id += 1;

        BlockLabel::new(format!(".block.{id}").as_bytes().into())
    }

    pub fn finish(mut self) -> FunctionDefinition {
        self.function.add_block(self.current_block);
        self.function
    }
}
