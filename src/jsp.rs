use std::collections::HashMap;
use std::fmt;
use std::vec::Vec;

#[derive(PartialEq, Debug)]
pub enum JsonValue {
    Int(String),
    Float(String),
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
            JsonValue::Array(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|x| { x.to_string() })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            JsonValue::Object(h) => write!(
                f,
                "{{{}}}",
                h.iter()
                    .map(|x| { format!("\"{}\": {}", x.0, x.1) })
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}
