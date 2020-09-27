use super::prelude::*;
use super::dehydrated::set_union;
use std::collections::HashSet;
use std::hash::Hash;

type Data = HashSet<Value>;

#[derive(Clone, Debug)]
enum Value {
    Unknown,
    Int(i64),
    Bool(bool),
    Float(f64)
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Unknown, Value::Unknown) => true,
            (Value::Int(i1), Value::Int(i2)) => i1 == i2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Float(f1), Value::Float(f2)) => f1 == f2,
            _ => false
        }
    }
}

impl Eq for Value {}
