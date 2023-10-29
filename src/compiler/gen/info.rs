use super::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Symbol {
    identifier: String,
    format: ValueFormat,
    alignment: usize,
    register: Register,
}

impl Symbol {
    pub fn new(identifier: String, format: ValueFormat, alignment: usize, register: Register) -> Self {
        Self {
            identifier,
            format,
            alignment,
            register,
        }
    }

    pub fn identifier(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn format(&self) -> &ValueFormat {
        &self.format
    }

    pub fn alignment(&self) -> usize {
        self.alignment
    }

    pub fn register(&self) -> &Register {
        &self.register
    }
}

#[derive(Clone, Debug)]
struct SymbolTableNode {
    symbol: Symbol,
    next_node: Option<Box<SymbolTableNode>>,
}

#[derive(Debug)]
pub struct SymbolTable {
    hash_table_bins: Vec<Option<SymbolTableNode>>,
}

impl SymbolTable {
    pub fn new(capacity: usize) -> Self {
        let mut hash_table_bins = Vec::new();
        hash_table_bins.resize_with(capacity, Default::default);
        Self {
            hash_table_bins,
        }
    }

    pub fn capacity(&self) -> usize {
        self.hash_table_bins.len()
    }

    pub fn find(&self, identifier: &str) -> Option<&Symbol> {
        let index = self.hash_index(identifier);

        self.find_in_bin(index, identifier)
    }

    pub fn find_mut(&mut self, identifier: &str) -> Option<&mut Symbol> {
        let index = self.hash_index(identifier);

        self.find_in_bin_mut(index, identifier)
    }

    pub fn insert(&mut self, symbol: Symbol) -> Option<Symbol> {
        let index = self.hash_index(symbol.identifier());

        if let Some(existing_symbol) = self.find_in_bin_mut(index, symbol.identifier()) {
            return Some(std::mem::replace(existing_symbol, symbol));
        }
        
        let root_node = self.hash_table_bins.get_mut(index)?;
        let node_to_insert = SymbolTableNode {
            symbol,
            next_node: root_node.take().map(|node| Box::new(node)),
        };
        *root_node = Some(node_to_insert);

        None
    }

    #[allow(arithmetic_overflow)]
    fn hash_index(&self, key: &str) -> usize {
        // https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function#FNV_offset_basis
        const FNV_OFFSET_BASIS: usize = 0xCBF29CE484222325;
        // Any large prime number will do
        const FNV_PRIME: usize = 0x100000001B3;

        let mut hash = FNV_OFFSET_BASIS;
        for char_value in key.chars() {
            hash ^= char_value as usize;
            hash %= self.capacity(); // FIXME: figure out how to disable overflow checking
            hash *= FNV_PRIME;
        }

        hash % self.capacity()
    }

    fn find_in_bin(&self, index: usize, identifier: &str) -> Option<&Symbol> {
        let mut next_node = self.hash_table_bins.get(index)?.as_ref();

        while let Some(current_node) = next_node {
            if current_node.symbol.identifier() == identifier {
                return Some(&current_node.symbol);
            }
            next_node = current_node.next_node.as_deref();
        }

        None
    }

    fn find_in_bin_mut(&mut self, index: usize, identifier: &str) -> Option<&mut Symbol> {
        let mut next_node = self.hash_table_bins.get_mut(index)?.as_mut();

        while let Some(current_node) = next_node {
            if current_node.symbol.identifier() == identifier {
                return Some(&mut current_node.symbol);
            }
            next_node = current_node.next_node.as_deref_mut();
        }

        None
    }
}