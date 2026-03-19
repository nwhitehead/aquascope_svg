# STATES Diagram Format

The STATES format is a text format for representing the stack and heap states of
programs at different points of execution. It is designed to work with many
different languages and have enough functionality to work with Rust programs.

## Specification

The STATES format is a text format with the following rules:

* Newlines are significant.
* Blank lines are ignored.
* Whitespace (except for newlines) in general is ignored.
* Comments are C++ style.
* The high level contents are a sequence of zero or more location data.
* Each location data starts with a Markdown-style header beginning with `#` and
  then name.
* A location data consists of zero or more regions.
* A region may be optionally dividied into subregions with Markdown style headers with `###`.
* A region is a Markdown style header beginning with `##` and then a name.
* If a region has subregions then all content of the region must be in
  subregions.
* The contents of a region or subregion are zero or more lvalues.
* An lvalue is a label, `:`, then a value, then end of line.
* Labels can be any alphanumeric text starting with non-numbers, including
  `_-+=!@$%^&()|{}`.
* Labels can optionaly be escaped with backticks to allow arbitrary text inside
  (except backticks).
* Values can be:
    * `*`
    * `[`, a comma separated list of values, `]`
    * `(`, a comma separated list of values, `)`
    * `ptr(`, a destination, `)`
    * an integer number
    * a floating point number
    * `'`, a character, `'`
    * a label, `{`, then a comma separated list of label, `:`, value, `}`
* A destination is a label followed by zero or more copies of `.` and natural
  numbers.
* A destination can also be followed by zero or more `'`.

## Example

This is a small example.

```states
# L0
## Stack
### main

# L1
## Stack
### main
x: ptr(H0)
## Heap
H0: 0

# L2
## Stack
### main
x: ptr(H0)
## Heap
H0: 1

# L3
## Stack
### main
x: ptr(H0)
y: ptr(H0)
## Heap
H0: 1

# L4
## Stack
### main
```

Here is another example.
```states
# L0
## Stack
### main

# L1
## Stack
### main
v: ptr(H0.0)
## Heap
H0: [1, 2, 3]

# L2
## Stack
### main
v: ptr(H0.0)
y: ptr(H0.0)
## Heap
H0: [1, 2, 3]

# L3
## Stack
### main
v: ptr(H0.0)
y: *
## Heap
H0: [1, 2, 3, 0]

# L4
## Stack
### main
v: ptr(H0.0)
y: *
## Heap
H0: [1, 2, 3, 0]
```
