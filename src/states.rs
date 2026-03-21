//! Rust data structure matching STATES diagram format for parsing

#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Char(char),
    Struct(NamedStruct),
    Pointer(Ptr),
    Invalid,
}

#[derive(Debug, Clone)]
pub struct NamedStruct {
    pub name: String,
    pub fields: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
pub struct Ptr {
    pub name: String,
    pub selectors: Vec<u32>,
    pub borrow: usize,
    pub help: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Def {
    pub label: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    pub definitions: Vec<Def>,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub name: String,
    pub regions: Vec<Region>,
    pub definitions: Vec<Def>,
}

#[derive(Debug, Clone)]
pub struct Step {
    pub label: String,
    pub locations: Vec<Location>,
}

#[derive(Debug, Clone)]
pub struct Program(pub Vec<Step>);
