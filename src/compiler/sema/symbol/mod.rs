use super::*;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SimplePath {
    segments: Vec<Box<str>>,
}

impl SimplePath {
    pub fn new(segments: Vec<Box<str>>) -> Self {
        Self {
            segments,
        }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn segments(&self) -> &[Box<str>] {
        &self.segments
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn parent(&self) -> Option<Self> {
        self.segments().split_last().map(|(_, segments)| {
            Self::new(segments.to_vec())
        })
    }

    pub fn child(&self, name: impl Into<Box<str>>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(name.into());
        Self::new(segments)
    }

    pub fn into_child(mut self, name: impl Into<Box<str>>) -> Self {
        self.segments.push(name.into());
        self
    }

    pub fn tail_name(&self) -> Option<&str> {
        self.segments().last().map(Box::as_ref)
    }
}

impl Default for SimplePath {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::fmt::Display for SimplePath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.segments().join("::"))
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PathBaseType {
    Primitive(PrimitiveType),
    Pointer {
        pointee_type: AbsolutePath,
        semantics: PointerSemantics,
    },
    Array {
        item_type: AbsolutePath,
        length: Option<u64>,
    },
    Tuple {
        item_types: Box<[AbsolutePath]>
    },
    Function {
        parameter_types: Box<[AbsolutePath]>,
        is_variadic: bool,
        return_type: AbsolutePath,
    },
}

impl std::fmt::Display for PathBaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(primitive) => {
                write!(f, "{primitive}")
            }
            Self::Pointer { pointee_type, semantics } => match semantics {
                PointerSemantics::Immutable => write!(f, "*{pointee_type}"),
                PointerSemantics::Mutable => write!(f, "*mut {pointee_type}"),
            }
            Self::Array { item_type, length: Some(length) } => {
                write!(f, "[{item_type}; {length}]")
            }
            Self::Array { item_type, length: _none } => {
                write!(f, "[{item_type}]")
            }
            Self::Tuple { item_types } => match item_types.as_ref() {
                [] => write!(f, "()"),
                [item_type] => write!(f, "({item_type},)"),
                [item_type, item_types @ ..] => {
                    write!(f, "({item_type}")?;
                    for item_type in item_types {
                        write!(f, ", {item_type}")?;
                    }
                    write!(f, ")")
                }
            }
            Self::Function { parameter_types, is_variadic, return_type } => {
                write!(f, "function(")?;
                let mut parameter_types_iter = parameter_types.iter();
                if let Some(parameter_type) = parameter_types_iter.next() {
                    write!(f, "{parameter_type}")?;
                    for parameter_type in parameter_types_iter {
                        write!(f, ", {parameter_type}")?;
                    }
                    if *is_variadic {
                        write!(f, ", ..")?;
                    }
                }
                else if *is_variadic {
                    write!(f, "..")?;
                }
                write!(f, ") -> {return_type}")
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AbsolutePath {
    base_type: Option<Box<PathBaseType>>,
    simple: SimplePath,
}

impl AbsolutePath {
    pub fn new(base_type: Option<Box<PathBaseType>>, simple: SimplePath) -> Self {
        Self {
            base_type,
            simple,
        }
    }

    pub fn from_root(simple: SimplePath) -> Self {
        Self::new(None, simple)
    }

    pub fn from_base_type(base_type: Box<PathBaseType>, simple: SimplePath) -> Self {
        Self::new(Some(base_type), simple)
    }

    pub fn at_root() -> Self {
        Self::from_root(SimplePath::empty())
    }

    pub fn at_base_type(base_type: Box<PathBaseType>) -> Self {
        Self::from_base_type(base_type, SimplePath::empty())
    }

    pub fn base_type(&self) -> Option<&PathBaseType> {
        self.base_type.as_deref()
    }

    pub fn simple(&self) -> &SimplePath {
        &self.simple
    }

    pub fn parent(&self) -> Option<Self> {
        self.simple().parent().map(|simple| {
            Self::new(
                self.base_type().map(|base_type| Box::new(base_type.clone())),
                simple,
            )
        })
    }

    pub fn child(&self, name: impl Into<Box<str>>) -> Self {
        Self::new(
            self.base_type().map(|base_type| Box::new(base_type.clone())),
            self.simple().child(name),
        )
    }

    pub fn into_child(mut self, name: impl Into<Box<str>>) -> Self {
        self.simple = self.simple.into_child(name);
        self
    }

    pub fn tail_name(&self) -> Option<&str> {
        self.simple().tail_name()
    }
}

impl std::fmt::Display for AbsolutePath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(base_type) = self.base_type() {
            if self.simple().is_empty() {
                write!(f, "{base_type}")
            }
            else {
                write!(f, "<{base_type}>::{}", self.simple())
            }
        }
        else {
            if self.simple().is_empty() {
                write!(f, "::module")
            }
            else {
                write!(f, "::{}", self.simple())
            }
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NamespaceHandle(NonZeroUsize);

impl NamespaceHandle {
    pub const ROOT: Self = Self::new(0);

    pub const fn new(registry_index: usize) -> Self {
        Self(unsafe { NonZeroUsize::new_unchecked(registry_index + 1) })
    }

    pub const fn registry_index(self) -> usize {
        self.0.get() - 1
    }

    pub fn info(self, context: &GlobalContext) -> &NamespaceInfo {
        context.namespace_info(self)
    }

    pub fn info_mut(self, context: &mut GlobalContext) -> &mut NamespaceInfo {
        context.namespace_info_mut(self)
    }
}

#[derive(Clone, Debug)]
pub struct NamespaceInfo {
    path: AbsolutePath,
    symbols: HashMap<Box<str>, Symbol>,
    glob_imports: Vec<AbsolutePath>,
}

impl NamespaceInfo {
    pub fn new(path: AbsolutePath) -> Self {
        Self {
            path,
            symbols: HashMap::new(),
            glob_imports: Vec::new(),
        }
    }

    pub fn path(&self) -> &AbsolutePath {
        &self.path
    }

    pub fn glob_imports(&self) -> &[AbsolutePath] {
        &self.glob_imports
    }

    pub fn find(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }

    pub fn define(&mut self, name: &str, symbol: Symbol) -> crate::Result<()> {
        match self.symbols.insert(name.into(), symbol) {
            Some(..) => Err(Box::new(crate::Error::GlobalSymbolConflict {
                namespace: self.path().to_string(),
                name: name.to_owned(),
            })),
            None => Ok(()),
        }
    }

    pub fn add_glob_import(&mut self, path: impl Into<AbsolutePath>) {
        self.glob_imports.push(path.into());
    }
}

#[derive(Clone, Debug)]
pub enum Symbol {
    /// The symbol is defined by a direct import (e.g. `path::to::symbol`). The path to the source
    /// symbol is given.
    Alias(AbsolutePath),
    /// The symbol is defined by a module declaration. The namespace handle for the module is
    /// given.
    Module(NamespaceHandle),
    /// The symbol is defined by a structure type declaration. The type handle corresponding to
    /// the declared structure type is given.
    Type(TypeHandle),
    /// The symbol is defined by a `let` statement or `function` declaration. The declared function
    /// or value is given.
    Value(Value),
}
