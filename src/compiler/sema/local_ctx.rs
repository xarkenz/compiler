use super::*;

#[derive(Debug)]
pub struct LocalContext {
    function_name: String,
    return_type: TypeHandle,
    break_stack: Vec<Label>,
    continue_stack: Vec<Label>,
    symbol_versions: HashMap<String, usize>,
    scopes: Vec<HashMap<String, Value>>,
}

impl LocalContext {
    pub fn new(function_name: String, return_type: TypeHandle) -> Self {
        Self {
            function_name,
            return_type,
            break_stack: Vec::new(),
            continue_stack: Vec::new(),
            symbol_versions: HashMap::new(),
            scopes: vec![HashMap::new()],
        }
    }
}
