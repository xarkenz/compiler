use crate::ir::value::{BlockLabel, LocalRegister, Value};
use crate::sema::ConversionOperation;

pub enum Instruction {
    Negate {
        result: LocalRegister,
        operand: Value,
    },
    Add {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    Subtract {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    Multiply {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    Divide {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    Remainder {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    ShiftLeft {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    ShiftRight {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    Not {
        result: LocalRegister,
        operand: Value,
    },
    And {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    Or {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    Xor {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    ExtractValue {
        result: LocalRegister,
        aggregate: Value,
        indices: Box<[Value]>,
    },
    InsertValue {
        result: LocalRegister,
        aggregate: Value,
        value: Value,
        indices: Box<[Value]>,
    },
    StackAllocate {
        result: LocalRegister,
    },
    Load {
        result: LocalRegister,
        pointer: Value,
    },
    Store {
        value: Value,
        pointer: Value,
    },
    GetElementPointer {
        result: LocalRegister,
        pointer: Value,
        indices: Box<[Value]>,
    },
    Convert {
        operation: ConversionOperation,
        result: LocalRegister,
        value: Value,
    },
    CompareEqual {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    CompareNotEqual {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    CompareLessThan {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    CompareLessEqual {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    CompareGreaterThan {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    CompareGreaterEqual {
        result: LocalRegister,
        lhs: Value,
        rhs: Value,
    },
    Call {
        result: Option<LocalRegister>,
        callee: Value,
        arguments: Box<[Value]>,
    },
}

pub struct PhiInstruction {
    pub result: LocalRegister,
    pub inputs: Box<[(Value, BlockLabel)]>,
}

pub enum TerminatorInstruction {
    Return {
        value: Value,
    },
    Branch {
        to_label: BlockLabel,
    },
    ConditionalBranch {
        condition: Value,
        consequent_label: BlockLabel,
        alternative_label: BlockLabel,
    },
    Unreachable,
}

pub struct BasicBlock {
    label: BlockLabel,
    phis: Vec<PhiInstruction>,
    body: Vec<Instruction>,
    terminator: TerminatorInstruction,
}

impl BasicBlock {
    pub fn new(label: BlockLabel) -> Self {
        Self {
            label,
            phis: Vec::new(),
            body: Vec::new(),
            terminator: TerminatorInstruction::Unreachable,
        }
    }

    pub fn label(&self) -> &BlockLabel {
        &self.label
    }

    pub fn phis(&self) -> &[PhiInstruction] {
        &self.phis
    }

    pub fn add_phi(&mut self, phi: PhiInstruction) {
        self.phis.push(phi);
    }

    pub fn body(&self) -> &[Instruction] {
        &self.body
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.body.push(instruction);
    }

    pub fn terminator(&self) -> &TerminatorInstruction {
        &self.terminator
    }

    pub fn set_terminator(&mut self, terminator: TerminatorInstruction) {
        self.terminator = terminator;
    }
}
