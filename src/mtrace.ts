export interface CharPos {
  line: number;
  column: number;
}

export interface FilenameIndex {
  private_use_as_methods_instead: number;
}

export interface CharRange {
  start: CharPos;
  end: CharPos;
  filename: FilenameIndex;
}

export type MPathSegment =
  | { type: "Field"; value: number }
  | { type: "Index"; value: number }
  | { type: "Subslice"; value: [number, number] };

export type MMemorySegment =
  | { type: "Stack"; value: { frame: number; local: string } }
  | { type: "Heap"; value: { index: number } };

export interface MPath {
  segment: MMemorySegment;
  parts: MPathSegment[];
}

export type MHeapAllocKind =
  | { type: "String"; value: { len: bigint } }
  | { type: "Vec"; value: { len: bigint } }
  | { type: "Box" };

export type Abbreviated<T> =
  | { type: "All"; value: T[] }
  | { type: "Only"; value: [T[], T] };

export type MValue =
  | { type: "Bool"; value: boolean }
  | { type: "Char"; value: number }
  | { type: "Uint"; value: bigint }
  | { type: "Int"; value: bigint }
  | { type: "Float"; value: number }
  | { type: "Tuple"; value: MValue[] }
  | { type: "Array"; value: Abbreviated<MValue> }
  | {
      type: "Adt";
      value: {
        name: string;
        variant: string | null;
        fields: [string, MValue][];
        alloc_kind: MHeapAllocKind | null;
      };
    }
  | { type: "Pointer"; value: { path: MPath; range: bigint | null } }
  | { type: "Unallocated"; value: { alloc_id: number | null } };

export interface MLocal {
  name: string;
  value: MValue;
  moved_paths: MPathSegment[][];
}

export interface MFrame {
  name: string;
  body_span: CharRange;
  location: CharRange;
  locals: MLocal[];
}

export interface MStack {
  frames: MFrame[];
}

export interface MHeap {
  locations: MValue[];
}

export interface MStep {
  stack: MStack;
  heap: MHeap;
}

export type MUndefinedBehavior =
  | { type: "PointerUseAfterFree"; value: { alloc_id: number } }
  | { type: "Other"; value: string };

export type MResult =
  | { type: "Success" }
  | { type: "Error"; value: MUndefinedBehavior };

export interface MTrace {
  steps: MStep[];
  result: MResult;
}
