/// The universal currency of the VM. A 64-bit tagged value that is either
/// an inline small integer (Smi) or a pointer to a HeapObject.
/// Passed on the stack, stored in registers, constant pools, and object slots.
///
/// Tagging scheme: bit 0 = 0 → Smi (value << 1), bit 0 = 1 → heap pointer (index << 1 | 1)
/// Future: NaN-boxing for doubles, or extend tag bits for null/undefined/boolean
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value(u64);

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_smi() {
            write!(f, "{}", self.as_smi())
        } else {
            write!(f, "HeapObject({})", self.heap_offset())
        }
    }
}

impl Value {
    const TAG_MASK: u64 = 1;

    pub fn from_smi(n: i64) -> Value {
        Value((n << Self::TAG_MASK) as u64)
    }

    pub fn is_smi(self) -> bool {
        self.0 & Self::TAG_MASK == 0
    }

    pub fn as_smi(self) -> i64 {
        (self.0 as i64) >> Self::TAG_MASK
    }

    pub fn from_heap(offset: usize) -> Value {
        Value(((offset << 1) | 1) as u64)
    }

    pub fn is_heap_object(self) -> bool {
        !self.is_smi()
    }
    pub fn heap_offset(self) -> usize {
        (self.0 as usize) >> Self::TAG_MASK
    }
}

impl Default for Value {
    fn default() -> Self {
        Self(Default::default())
    }
}
