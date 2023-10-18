use super::ConstantValue;

#[derive(Debug)]
struct SymbolTableEntry {
    name: String,
    value: ConstantValue,
    next: Box<SymbolTableEntry>,
}