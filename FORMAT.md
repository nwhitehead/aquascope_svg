# Diagram Format

This is notes about a format for representing the JSON output of aquascope in a
human readable and editable way. The idea is to migrate from verbose JSON to
this human readable format for workflow for publishing rust diagrams.

## Examples

The diagrams should be in fenced code blocks, so no markdown styles are needed
(but can be used). Should be easy to parse, unambiguous.

First try:
```
L1:
# Stack
## main
s0: v | ptr(h0)
n | ptr(h0)
# Heap
h0: [1, 2, 3]
5
---
L2:
# Stack
## main
v | ptr(h0)
n | *
# Heap
h0: [1, 2, 3, 0]
5
---
L3: undefined behavior
# Stack
## main
v | ptr(h0)
n | ***
# Heap
h0: [1, 2, 3, 0]
5
```

Some features:
* Allows empty stack frames (will be annotated as empty)
* Pointers can go anywhere to anywhere that is labelled
* Labels can be inside values

Removing bars from stack, let's just use labels.

Removing special `---` syntax, just use headers.

```
# L0
## Stack
### main

# L1
## Stack
### main
v: ptr(h0)
n: ptr(h0)
## Heap ##
h0: [1, 2, 3]
h1: 5
## Arena ##
h2: [0, 0, 0]
h3: ptr(h2)

# L2
## Stack
### main
v: ptr(h0)
n: *
## Heap ##
h0: [1, h1: 2, 3, 0]
5

# L3
undefined behavior
## Stack
### main
v: ptr(h0)
tpl: (1, 4, ptr(v))
n: ***
## Heap ##
h0: [1, 2, 3, 0]
h1: 5
```

Syntax rules:
* Lines that start with `#` start new steps
* If we are in `#` and not in any deeper headers, then text becomes annotations for current step
* Lines that start with `##` begin memory regions (stacks and heaps)
* If the line ends with `##` then it is nameless, otherwise labels are shown as names in diagram
* Subheads `###` are rendered as lables below enclosing `##` memory region (can also be nameless or named)
* Labels are text followed by `:`
* Values can be:
    * Number
    * `[` optionally labelled values, `,` separated, `]`
    * `(` optionally labelled values, `,` separated, `)`
    * `*` to indicate invalid
    * `***` to indicate invalid and accessed
    * `ptr(` label `)`, label can be to any memory region in current step

What about structs?


