use crate::value::Value;

pub struct Heap {
    bytes: Vec<u8>,
}

#[repr(u8)]
pub enum InstanceType {
    String = 1,
}

// TODO: abstract write and read value without hardcoded function.
impl Heap {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn alloc_string(&mut self, s: &str) -> Value {
        let offset = self.bytes.len();

        self.bytes.push(InstanceType::String as u8);
        let len = s.len() as u32;
        self.bytes.extend_from_slice(&len.to_le_bytes());
        self.bytes.extend_from_slice(s.as_bytes());

        Value::from_heap(offset)
    }

    pub fn read_string(&self, val: Value) -> &str {
        let mut offset = val.heap_offset();
        // skip instance type
        offset += 1;
        let str_len = self.bytes[offset..offset + 4].try_into().unwrap();
        let str_len = u32::from_le_bytes(str_len) as usize;
        offset += 4;
        str::from_utf8(&self.bytes[offset..offset + str_len]).unwrap()
    }
}
