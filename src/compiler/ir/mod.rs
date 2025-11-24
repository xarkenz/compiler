use std::collections::HashSet;
use std::path::Path;
use crate::ir::value::{Constant, LocalRegister, GlobalRegister};
use crate::sema::TypeHandle;

pub mod value;
pub mod instr;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GlobalVariableKind {
    Constant,
    AnonymousConstant,
    Mutable,
}

pub struct ExternalGlobalVariable {
    register: GlobalRegister,
    kind: GlobalVariableKind,
    value_type: TypeHandle,
}

impl ExternalGlobalVariable {
    pub fn new(register: GlobalRegister, kind: GlobalVariableKind, value_type: TypeHandle) -> Self {
        Self {
            register,
            kind,
            value_type,
        }
    }

    pub fn register(&self) -> &GlobalRegister {
        &self.register
    }

    pub fn kind(&self) -> GlobalVariableKind {
        self.kind
    }

    pub fn value_type(&self) -> TypeHandle {
        self.value_type
    }
}

pub struct ExternalFunction {
    register: GlobalRegister,
}

impl ExternalFunction {
    pub fn new(register: GlobalRegister) -> Self {
        Self {
            register,
        }
    }

    pub fn register(&self) -> &GlobalRegister {
        &self.register
    }
}

pub struct GlobalVariable {
    register: GlobalRegister,
    kind: GlobalVariableKind,
    value: Constant,
}

impl GlobalVariable {
    pub fn new(register: GlobalRegister, kind: GlobalVariableKind, value: Constant) -> Self {
        Self {
            register,
            kind,
            value,
        }
    }

    pub fn register(&self) -> &GlobalRegister {
        &self.register
    }

    pub fn kind(&self) -> GlobalVariableKind {
        self.kind
    }

    pub fn value(&self) -> &Constant {
        &self.value
    }
}

pub struct FunctionDefinition {
    register: GlobalRegister,
    return_type: TypeHandle,
    parameter_registers: Vec<LocalRegister>,
    is_variadic: bool,
    blocks: Vec<instr::BasicBlock>,
}

impl FunctionDefinition {
    pub fn new(
        register: GlobalRegister,
        return_type: TypeHandle,
        is_variadic: bool,
    ) -> Self {
        Self {
            register,
            return_type,
            parameter_registers: Vec::new(),
            is_variadic,
            blocks: Vec::new(),
        }
    }

    pub fn register(&self) -> &GlobalRegister {
        &self.register
    }

    pub fn return_type(&self) -> TypeHandle {
        self.return_type
    }

    pub fn parameter_registers(&self) -> &[LocalRegister] {
        &self.parameter_registers
    }

    pub fn add_parameter_register(&mut self, register: LocalRegister) {
        self.parameter_registers.push(register);
    }

    pub fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    pub fn blocks(&self) -> &[instr::BasicBlock] {
        &self.blocks
    }

    pub fn add_block(&mut self, block: instr::BasicBlock) {
        self.blocks.push(block);
    }
}

pub struct CompilationUnit {
    main_path: Box<Path>,
    type_declarations: HashSet<TypeHandle>,
    external_global_variables: Vec<ExternalGlobalVariable>,
    external_functions: Vec<ExternalFunction>,
    global_variables: Vec<GlobalVariable>,
    function_definitions: Vec<FunctionDefinition>,
}

impl CompilationUnit {
    pub fn new(main_path: impl Into<Box<Path>>) -> Self {
        Self {
            main_path: main_path.into(),
            type_declarations: HashSet::new(),
            external_global_variables: Vec::new(),
            external_functions: Vec::new(),
            global_variables: Vec::new(),
            function_definitions: Vec::new(),
        }
    }

    pub fn main_path(&self) -> &Path {
        &self.main_path
    }

    pub fn type_declarations(&self) -> &HashSet<TypeHandle> {
        &self.type_declarations
    }

    pub fn add_type_declaration(&mut self, type_handle: TypeHandle) -> bool {
        self.type_declarations.insert(type_handle)
    }

    pub fn external_global_variables(&self) -> &[ExternalGlobalVariable] {
        &self.external_global_variables
    }

    pub fn add_external_global_variable(&mut self, variable: ExternalGlobalVariable) {
        self.external_global_variables.push(variable);
    }

    pub fn external_functions(&self) -> &[ExternalFunction] {
        &self.external_functions
    }

    pub fn add_external_function(&mut self, function: ExternalFunction) {
        self.external_functions.push(function);
    }

    pub fn global_variables(&self) -> &[GlobalVariable] {
        &self.global_variables
    }

    pub fn add_global_variable(&mut self, global_variable: GlobalVariable) {
        self.global_variables.push(global_variable);
    }

    pub fn function_definitions(&self) -> &[FunctionDefinition] {
        &self.function_definitions
    }

    pub fn add_function_definition(&mut self, function: FunctionDefinition) {
        self.function_definitions.push(function);
    }
}
