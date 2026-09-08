use std::collections::HashMap;

const MAX_DEPTH: usize = 64;
const MAX_STRING_LEN: usize = 1_048_576;
const MAX_ARRAY_LEN: usize = 1_048_576;
const MAX_STRUCT_FIELDS: usize = 1024;
const MAX_RECURSION: usize = 256;

#[derive(Debug, Clone, PartialEq)]
pub enum SerializationError {
    DepthLimitExceeded { depth: usize, limit: usize },
    AllocationLimitExceeded { size: usize, limit: usize },
    InvalidTag(u8),
    TruncatedInput { expected: usize, available: usize },
    InvalidUtf8(String),
    UnsupportedType(String),
    RecursionLimitExceeded,
    MalformedData(String),
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DepthLimitExceeded { depth, limit } => {
                write!(f, "depth limit exceeded: {} > {}", depth, limit)
            }
            Self::AllocationLimitExceeded { size, limit } => {
                write!(f, "allocation limit exceeded: {} > {}", size, limit)
            }
            Self::InvalidTag(tag) => write!(f, "invalid type tag: {}", tag),
            Self::TruncatedInput { expected, available } => {
                write!(f, "truncated input: need {} bytes, have {}", expected, available)
            }
            Self::InvalidUtf8(e) => write!(f, "invalid UTF-8: {}", e),
            Self::UnsupportedType(name) => write!(f, "unsupported type: {}", name),
            Self::RecursionLimitExceeded => write!(f, "recursion limit exceeded"),
            Self::MalformedData(msg) => write!(f, "malformed data: {}", msg),
        }
    }
}

impl std::error::Error for SerializationError {}

#[derive(Debug, Clone)]
pub enum AxiomValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
    Array(Vec<AxiomValue>),
    Struct(Vec<AxiomValue>),
    Maybe(Option<Box<AxiomValue>>),
}

impl PartialEq for AxiomValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a.to_bits() == b.to_bits(),
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Unit, Self::Unit) => true,
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Struct(a), Self::Struct(b)) => a == b,
            (Self::Maybe(a), Self::Maybe(b)) => a == b,
            _ => false,
        }
    }
}

impl AxiomValue {
    pub fn type_tag(&self) -> &'static str {
        match self {
            Self::Integer(_) => "Int",
            Self::Float(_) => "Float",
            Self::String(_) => "String",
            Self::Bool(_) => "Bool",
            Self::Unit => "()",
            Self::Array(_) => "Array",
            Self::Struct(_) => "Struct",
            Self::Maybe(Some(_)) => "Maybe(Some)",
            Self::Maybe(None) => "Maybe(None)",
        }
    }

    pub fn is_transmissible(&self) -> bool {
        match self {
            Self::Integer(_) | Self::Float(_) | Self::String(_) | Self::Bool(_) | Self::Unit => true,
            Self::Array(elements) => elements.iter().all(|e| e.is_transmissible()),
            Self::Struct(fields) => fields.iter().all(|f| f.is_transmissible()),
            Self::Maybe(Some(inner)) => inner.is_transmissible(),
            Self::Maybe(None) => true,
        }
    }
}

impl std::fmt::Display for AxiomValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{}", n),
            Self::Float(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Unit => write!(f, "()"),
            Self::Array(elements) => {
                let items: Vec<String> = elements.iter().map(|e| e.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Self::Struct(fields) => {
                let items: Vec<String> = fields.iter().map(|e| e.to_string()).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Self::Maybe(Some(val)) => write!(f, "Some({})", val),
            Self::Maybe(None) => write!(f, "None"),
        }
    }
}

pub struct Serializer {
    buf: Vec<u8>,
    depth: usize,
    max_depth: usize,
}

impl Serializer {
    pub fn new() -> Self {
        Self::with_limits(MAX_DEPTH)
    }

    pub fn with_limits(max_depth: usize) -> Self {
        Serializer {
            buf: Vec::new(),
            depth: 0,
            max_depth,
        }
    }

    pub fn serialize(value: &AxiomValue) -> Result<Vec<u8>, SerializationError> {
        let mut serializer = Self::new();
        serializer.serialize_value(value)?;
        Ok(serializer.buf)
    }

    pub fn serialize_value(&mut self, value: &AxiomValue) -> Result<(), SerializationError> {
        if self.depth > self.max_depth {
            return Err(SerializationError::DepthLimitExceeded {
                depth: self.depth,
                limit: self.max_depth,
            });
        }

        match value {
            AxiomValue::Integer(n) => {
                self.buf.push(0);
                self.buf.extend_from_slice(&n.to_le_bytes());
            }
            AxiomValue::Float(n) => {
                self.buf.push(1);
                self.buf.extend_from_slice(&n.to_le_bytes());
            }
            AxiomValue::Bool(b) => {
                self.buf.push(2);
                self.buf.push(if *b { 1 } else { 0 });
            }
            AxiomValue::String(s) => {
                let bytes = s.as_bytes();
                if bytes.len() > MAX_STRING_LEN {
                    return Err(SerializationError::AllocationLimitExceeded {
                        size: bytes.len(),
                        limit: MAX_STRING_LEN,
                    });
                }
                self.buf.push(3);
                self.buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                self.buf.extend_from_slice(bytes);
            }
            AxiomValue::Unit => {
                self.buf.push(4);
            }
            AxiomValue::Array(elements) => {
                if elements.len() > MAX_ARRAY_LEN {
                    return Err(SerializationError::AllocationLimitExceeded {
                        size: elements.len(),
                        limit: MAX_ARRAY_LEN,
                    });
                }
                self.buf.push(5);
                self.buf.extend_from_slice(&(elements.len() as u32).to_le_bytes());
                self.depth += 1;
                for elem in elements {
                    self.serialize_value(elem)?;
                }
                self.depth -= 1;
            }
            AxiomValue::Maybe(Some(val)) => {
                self.buf.push(6);
                self.depth += 1;
                self.serialize_value(val)?;
                self.depth -= 1;
            }
            AxiomValue::Maybe(None) => {
                self.buf.push(7);
            }
            AxiomValue::Struct(fields) => {
                if fields.len() > MAX_STRUCT_FIELDS {
                    return Err(SerializationError::AllocationLimitExceeded {
                        size: fields.len(),
                        limit: MAX_STRUCT_FIELDS,
                    });
                }
                self.buf.push(8);
                self.buf.extend_from_slice(&(fields.len() as u32).to_le_bytes());
                self.depth += 1;
                for field in fields {
                    self.serialize_value(field)?;
                }
                self.depth -= 1;
            }
        }
        Ok(())
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

pub struct Deserializer<'a> {
    data: &'a [u8],
    pos: usize,
    depth: usize,
    max_depth: usize,
    recursion_count: usize,
}

impl<'a> Deserializer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self::with_limits(data, MAX_DEPTH)
    }

    pub fn with_limits(data: &'a [u8], max_depth: usize) -> Self {
        Deserializer {
            data,
            pos: 0,
            depth: 0,
            max_depth,
            recursion_count: 0,
        }
    }

    pub fn deserialize(data: &[u8]) -> Result<AxiomValue, SerializationError> {
        let mut deserializer = Deserializer::new(data);
        deserializer.deserialize_value()
    }

    fn check_remaining(&self, needed: usize) -> Result<(), SerializationError> {
        if self.pos + needed > self.data.len() {
            return Err(SerializationError::TruncatedInput {
                expected: needed,
                available: self.data.len() - self.pos,
            });
        }
        Ok(())
    }

    fn read_u8(&mut self) -> Result<u8, SerializationError> {
        self.check_remaining(1)?;
        let val = self.data[self.pos];
        self.pos += 1;
        Ok(val)
    }

    fn read_u32(&mut self) -> Result<u32, SerializationError> {
        self.check_remaining(4)?;
        let val = u32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(val)
    }

    fn read_i64(&mut self) -> Result<i64, SerializationError> {
        self.check_remaining(8)?;
        let val = i64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(val)
    }

    fn read_f64(&mut self) -> Result<f64, SerializationError> {
        self.check_remaining(8)?;
        let val = f64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(val)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], SerializationError> {
        self.check_remaining(len)?;
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }

    pub fn deserialize_value(&mut self) -> Result<AxiomValue, SerializationError> {
        if self.depth > self.max_depth {
            return Err(SerializationError::DepthLimitExceeded {
                depth: self.depth,
                limit: self.max_depth,
            });
        }

        self.recursion_count += 1;
        if self.recursion_count > MAX_RECURSION {
            return Err(SerializationError::RecursionLimitExceeded);
        }

        let tag = self.read_u8()?;
        let result = match tag {
            0 => AxiomValue::Integer(self.read_i64()?),
            1 => AxiomValue::Float(self.read_f64()?),
            2 => {
                let b = self.read_u8()?;
                AxiomValue::Bool(b != 0)
            }
            3 => {
                let len = self.read_u32()? as usize;
                if len > MAX_STRING_LEN {
                    return Err(SerializationError::AllocationLimitExceeded {
                        size: len,
                        limit: MAX_STRING_LEN,
                    });
                }
                let bytes = self.read_bytes(len)?;
                let s = String::from_utf8(bytes.to_vec())
                    .map_err(|e| SerializationError::InvalidUtf8(e.to_string()))?;
                AxiomValue::String(s)
            }
            4 => AxiomValue::Unit,
            5 => {
                let count = self.read_u32()? as usize;
                if count > MAX_ARRAY_LEN {
                    return Err(SerializationError::AllocationLimitExceeded {
                        size: count,
                        limit: MAX_ARRAY_LEN,
                    });
                }
                self.depth += 1;
                let mut elements = Vec::with_capacity(count);
                for _ in 0..count {
                    elements.push(self.deserialize_value()?);
                }
                self.depth -= 1;
                AxiomValue::Array(elements)
            }
            6 => {
                self.depth += 1;
                let inner = self.deserialize_value()?;
                self.depth -= 1;
                AxiomValue::Maybe(Some(Box::new(inner)))
            }
            7 => AxiomValue::Maybe(None),
            8 => {
                let count = self.read_u32()? as usize;
                if count > MAX_STRUCT_FIELDS {
                    return Err(SerializationError::AllocationLimitExceeded {
                        size: count,
                        limit: MAX_STRUCT_FIELDS,
                    });
                }
                self.depth += 1;
                let mut fields = Vec::with_capacity(count);
                for _ in 0..count {
                    fields.push(self.deserialize_value()?);
                }
                self.depth -= 1;
                AxiomValue::Struct(fields)
            }
            _ => return Err(SerializationError::InvalidTag(tag)),
        };

        self.recursion_count -= 1;
        Ok(result)
    }

    pub fn is_complete(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }
}

impl From<AxiomValue> for crate::vm::Value {
    fn from(val: AxiomValue) -> Self {
        match val {
            AxiomValue::Integer(n) => crate::vm::Value::Integer(n),
            AxiomValue::Float(n) => crate::vm::Value::Float(n),
            AxiomValue::String(s) => crate::vm::Value::String(s),
            AxiomValue::Bool(b) => crate::vm::Value::Bool(b),
            AxiomValue::Unit => crate::vm::Value::Unit,
            AxiomValue::Array(elements) => {
                crate::vm::Value::Array(elements.into_iter().map(Into::into).collect())
            }
            AxiomValue::Struct(fields) => {
                crate::vm::Value::Struct(fields.into_iter().map(Into::into).collect())
            }
            AxiomValue::Maybe(inner) => {
                crate::vm::Value::Maybe(inner.map(|v| Box::new((*v).into())))
            }
        }
    }
}

impl From<crate::vm::Value> for AxiomValue {
    fn from(val: crate::vm::Value) -> Self {
        match val {
            crate::vm::Value::Integer(n) => AxiomValue::Integer(n),
            crate::vm::Value::Float(n) => AxiomValue::Float(n),
            crate::vm::Value::String(s) => AxiomValue::String(s),
            crate::vm::Value::Bool(b) => AxiomValue::Bool(b),
            crate::vm::Value::Unit => AxiomValue::Unit,
            crate::vm::Value::Array(elements) => {
                AxiomValue::Array(elements.into_iter().map(Into::into).collect())
            }
            crate::vm::Value::Struct(fields) => {
                AxiomValue::Struct(fields.into_iter().map(Into::into).collect())
            }
            crate::vm::Value::Maybe(inner) => {
                AxiomValue::Maybe(inner.map(|v| Box::new((*v).into())))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(value: &AxiomValue) -> AxiomValue {
        let bytes = Serializer::serialize(value).unwrap();
        Deserializer::deserialize(&bytes).unwrap()
    }

    #[test]
    fn test_integer() {
        let val = AxiomValue::Integer(42);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_negative_integer() {
        let val = AxiomValue::Integer(-12345);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_zero() {
        let val = AxiomValue::Integer(0);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_max_integer() {
        let val = AxiomValue::Integer(i64::MAX);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_min_integer() {
        let val = AxiomValue::Integer(i64::MIN);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_float() {
        let val = AxiomValue::Float(3.14159);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_negative_float() {
        let val = AxiomValue::Float(-2.71828);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_float_special_values() {
        let vals = vec![
            AxiomValue::Float(f64::INFINITY),
            AxiomValue::Float(f64::NEG_INFINITY),
            AxiomValue::Float(f64::NAN),
        ];
        for val in &vals {
            let rt = roundtrip(val);
            match (val, &rt) {
                (AxiomValue::Float(a), AxiomValue::Float(b)) => {
                    if a.is_nan() {
                        assert!(b.is_nan());
                    } else {
                        assert_eq!(a, b);
                    }
                }
                _ => panic!("type mismatch"),
            }
        }
    }

    #[test]
    fn test_bool_true() {
        let val = AxiomValue::Bool(true);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_bool_false() {
        let val = AxiomValue::Bool(false);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_string() {
        let val = AxiomValue::String("hello, AXIOM!".to_string());
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_empty_string() {
        let val = AxiomValue::String("".to_string());
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_string_unicode() {
        let val = AxiomValue::String("hello \u{00e9}\u{00e8}\u{00ea}".to_string());
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_unit() {
        let val = AxiomValue::Unit;
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_array_empty() {
        let val = AxiomValue::Array(vec![]);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_array_integers() {
        let val = AxiomValue::Array(vec![
            AxiomValue::Integer(1),
            AxiomValue::Integer(2),
            AxiomValue::Integer(3),
        ]);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_array_mixed() {
        let val = AxiomValue::Array(vec![
            AxiomValue::Integer(42),
            AxiomValue::String("hello".to_string()),
            AxiomValue::Bool(true),
            AxiomValue::Unit,
        ]);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_maybe_some() {
        let val = AxiomValue::Maybe(Some(Box::new(AxiomValue::Integer(42))));
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_maybe_none() {
        let val = AxiomValue::Maybe(None);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_struct() {
        let val = AxiomValue::Struct(vec![
            AxiomValue::Integer(1),
            AxiomValue::String("Alice".to_string()),
        ]);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_nested_struct() {
        let val = AxiomValue::Struct(vec![
            AxiomValue::Integer(1),
            AxiomValue::Struct(vec![
                AxiomValue::String("nested".to_string()),
                AxiomValue::Bool(false),
            ]),
        ]);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_nested_array() {
        let val = AxiomValue::Array(vec![
            AxiomValue::Array(vec![AxiomValue::Integer(1), AxiomValue::Integer(2)]),
            AxiomValue::Array(vec![AxiomValue::Integer(3), AxiomValue::Integer(4)]),
        ]);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_nested_maybe() {
        let val = AxiomValue::Maybe(Some(Box::new(AxiomValue::Maybe(Some(Box::new(
            AxiomValue::Integer(42),
        ))))));
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_complex_nesting() {
        let val = AxiomValue::Struct(vec![
            AxiomValue::Integer(1),
            AxiomValue::Array(vec![
                AxiomValue::Maybe(Some(Box::new(AxiomValue::String("found".to_string())))),
                AxiomValue::Maybe(None),
            ]),
            AxiomValue::Struct(vec![
                AxiomValue::Bool(true),
                AxiomValue::Unit,
            ]),
        ]);
        assert_eq!(roundtrip(&val), val);
    }

    #[test]
    fn test_corrupted_bytes() {
        let result = Deserializer::deserialize(&[0xFF, 0xFF, 0xFF]);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_integer() {
        let result = Deserializer::deserialize(&[0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_string() {
        let result = Deserializer::deserialize(&[3, 0, 0, 0, 255]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_type_tag() {
        let result = Deserializer::deserialize(&[99]);
        assert!(match result {
            Err(SerializationError::InvalidTag(99)) => true,
            _ => false,
        });
    }

    #[test]
    fn test_empty_input() {
        let result = Deserializer::deserialize(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_utf8() {
        let mut data = vec![3];
        let invalid_utf8 = vec![0xFF, 0xFE];
        data.extend_from_slice(&(invalid_utf8.len() as u32).to_le_bytes());
        data.extend_from_slice(&invalid_utf8);
        let result = Deserializer::deserialize(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_depth_limit() {
        let mut serializer = Serializer::with_limits(2);
        let val = AxiomValue::Array(vec![
            AxiomValue::Array(vec![
                AxiomValue::Array(vec![AxiomValue::Integer(1)]),
            ]),
        ]);
        let result = serializer.serialize_value(&val);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_depth_limit() {
        let val = AxiomValue::Array(vec![
            AxiomValue::Array(vec![
                AxiomValue::Array(vec![AxiomValue::Integer(1)]),
            ]),
        ]);
        let bytes = Serializer::serialize(&val).unwrap();
        let mut deserializer = Deserializer::with_limits(&bytes, 2);
        let result = deserializer.deserialize_value();
        assert!(result.is_err());
    }

    #[test]
    fn test_string_length_limit() {
        let mut serializer = Serializer::with_limits(MAX_DEPTH);
        let long_string = AxiomValue::String("x".repeat(MAX_STRING_LEN + 1));
        let result = serializer.serialize_value(&long_string);
        assert!(result.is_err());
    }

    #[test]
    fn test_array_length_limit() {
        let mut serializer = Serializer::with_limits(MAX_DEPTH);
        let long_array = AxiomValue::Array(vec![AxiomValue::Unit; MAX_ARRAY_LEN + 1]);
        let result = serializer.serialize_value(&long_array);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_transmissible() {
        assert!(AxiomValue::Integer(42).is_transmissible());
        assert!(AxiomValue::Float(3.14).is_transmissible());
        assert!(AxiomValue::String("hi".into()).is_transmissible());
        assert!(AxiomValue::Bool(true).is_transmissible());
        assert!(AxiomValue::Unit.is_transmissible());
        assert!(AxiomValue::Array(vec![AxiomValue::Integer(1)]).is_transmissible());
        assert!(AxiomValue::Maybe(None).is_transmissible());
        assert!(AxiomValue::Maybe(Some(Box::new(AxiomValue::Integer(1)))).is_transmissible());
        assert!(AxiomValue::Struct(vec![AxiomValue::Integer(1)]).is_transmissible());
    }

    #[test]
    fn test_serialization_deterministic() {
        let val = AxiomValue::Struct(vec![
            AxiomValue::Integer(42),
            AxiomValue::String("hello".to_string()),
            AxiomValue::Bool(true),
        ]);
        let bytes1 = Serializer::serialize(&val).unwrap();
        let bytes2 = Serializer::serialize(&val).unwrap();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_display_integer() {
        assert_eq!(AxiomValue::Integer(42).to_string(), "42");
    }

    #[test]
    fn test_display_float() {
        assert_eq!(AxiomValue::Float(3.14).to_string(), "3.14");
    }

    #[test]
    fn test_display_string() {
        assert_eq!(AxiomValue::String("hello".into()).to_string(), "hello");
    }

    #[test]
    fn test_display_bool() {
        assert_eq!(AxiomValue::Bool(true).to_string(), "true");
        assert_eq!(AxiomValue::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_display_unit() {
        assert_eq!(AxiomValue::Unit.to_string(), "()");
    }

    #[test]
    fn test_display_array() {
        let val = AxiomValue::Array(vec![AxiomValue::Integer(1), AxiomValue::Integer(2)]);
        assert_eq!(val.to_string(), "[1, 2]");
    }

    #[test]
    fn test_display_struct() {
        let val = AxiomValue::Struct(vec![AxiomValue::Integer(1), AxiomValue::String("hi".into())]);
        assert_eq!(val.to_string(), "{1, hi}");
    }

    #[test]
    fn test_display_maybe_some() {
        let val = AxiomValue::Maybe(Some(Box::new(AxiomValue::Integer(42))));
        assert_eq!(val.to_string(), "Some(42)");
    }

    #[test]
    fn test_display_maybe_none() {
        let val = AxiomValue::Maybe(None);
        assert_eq!(val.to_string(), "None");
    }
}
