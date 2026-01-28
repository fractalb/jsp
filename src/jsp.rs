use std::collections::HashMap;
use std::fmt;
use std::vec::Vec;

#[derive(PartialEq, Debug)]
pub enum JsonValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonValue::Int(x) => write!(f, "{x}"),
            JsonValue::Float(x) => write!(f, "{x}"),
            JsonValue::Bool(x) => write!(f, "{x}"),
            JsonValue::String(x) => write!(f, "\"{x}\""),
            JsonValue::Null => write!(f, "null"),
            JsonValue::Array(v) => {
                write!(f, "[")?;
                if v.len() > 0 {
                    write!(f, "{} ", v[0])?;
                    for i in &v[1..] {
                        write!(f, ", {}", i)?;
                    }
                }
                write!(f, "]")
            }
            JsonValue::Object(h) => {
                write!(f, "{{")?;
                let mut i = 0;
                for (k, v) in h {
                    if i == 0 {
                        write!(f, "\"{}\" : {}", k, v)?;
                        i = 1;
                        continue;
                    }
                    write!(f, ", \"{}\" : {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}
