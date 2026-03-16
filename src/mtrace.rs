use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharPos {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilenameIndex {
    #[serde(rename = "private_use_as_methods_instead")]
    pub private_use_as_methods_instead: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharRange {
    pub start: CharPos,
    pub end: CharPos,
    pub filename: FilenameIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MPathSegment {
    Field { value: u32 },
    Index { value: u32 },
    Subslice { value: (u32, u32) },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MMemorySegment {
    Stack { value: MStackValue },
    Heap { value: MHeapValue },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MStackValue {
    pub frame: u32,
    pub local: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MHeapValue {
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MPath {
    pub segment: MMemorySegment,
    pub parts: Vec<MPathSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MHeapAllocKind {
    String { value: MHeapAllocString },
    Vec { value: MHeapAllocVec },
    Box,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MHeapAllocString {
    pub len: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MHeapAllocVec {
    pub len: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AbbreviatedMValue {
    All { value: Vec<MValue> },
    Only { value: (Vec<MValue>, Box<MValue>) },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MValue {
    Bool { value: bool },
    Char { value: u32 },
    Uint { value: u64 },
    Int { value: i64 },
    Float { value: f64 },
    Tuple { value: Vec<MValue> },
    Array { value: AbbreviatedMValue },
    Adt { value: MValueAdt },
    Pointer { value: MValuePointer },
    Unallocated { value: MValueUnallocated },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MValueAdt {
    pub name: String,
    pub variant: Option<String>,
    pub fields: Vec<(String, MValue)>,
    #[serde(rename = "alloc_kind")]
    pub alloc_kind: Option<MHeapAllocKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MValuePointer {
    pub path: MPath,
    pub range: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MValueUnallocated {
    #[serde(rename = "alloc_id")]
    pub alloc_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLocal {
    pub name: String,
    pub value: MValue,
    #[serde(rename = "moved_paths")]
    pub moved_paths: Vec<Vec<MPathSegment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFrame {
    pub name: String,
    #[serde(rename = "body_span")]
    pub body_span: CharRange,
    pub location: CharRange,
    pub locals: Vec<MLocal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MStack {
    pub frames: Vec<MFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MHeap {
    pub locations: Vec<MValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MStep {
    pub stack: MStack,
    pub heap: MHeap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MUndefinedBehavior {
    PointerUseAfterFree {
        value: MUndefinedBehaviorPointerUseAfterFree,
    },
    Other {
        value: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MUndefinedBehaviorPointerUseAfterFree {
    #[serde(rename = "alloc_id")]
    pub alloc_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MResult {
    Success,
    Error { value: MUndefinedBehavior },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MTrace {
    pub code: String,
    pub steps: Vec<MStep>,
    pub result: MResult,
}
