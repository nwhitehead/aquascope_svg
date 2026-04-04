//! Rust data structures matching Kaya diagram format for parsing

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    Number(f64),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Char(char),
    Struct(NamedStruct),
    Pointer(Ptr),
    Invalid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NamedStruct {
    pub name: String,
    pub fields: Vec<(String, Value)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ptr {
    pub name: String,
    pub selectors: Vec<u32>,
    pub borrow: usize,
    pub help: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Def {
    pub label: String,
    pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Region {
    pub name: String,
    pub definitions: Vec<Def>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    pub name: String,
    pub regions: Vec<Region>,
    pub definitions: Vec<Def>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Step {
    pub label: String,
    pub locations: Vec<Location>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Program(pub Vec<Step>);
