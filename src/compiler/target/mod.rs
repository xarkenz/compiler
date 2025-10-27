#[derive(Debug)]
pub struct TargetInfo {
    /// The size of a pointer in bytes.
    pointer_size: u64,
    // TODO: more
}

impl TargetInfo {
    pub fn new(pointer_size: u64) -> Self {
        Self {
            pointer_size,
        }
    }

    pub fn pointer_size(&self) -> u64 {
        self.pointer_size
    }
}
