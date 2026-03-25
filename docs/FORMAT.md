# Kaya Diagram Format

The Kaya format is a text format for representing the stack and heap states of
programs at different points of execution. It is designed to work with many
different languages and have enough functionality to work with Rust programs.

## Specification

The Kaya format is a text format with the following rules:

* Newlines are significant.
* Blank lines are ignored.
* Whitespace (except for newlines) in general is ignored.
* Comments can be C or C++ style.
* The high level contents is a sequence of zero or more steps.

The structure generall follows Markdown with up to 3 levels of nesting.
* Each step starts with a Markdown-style header beginning with `#` and then a
  name. (For Aquascope these are labelled `L0` and up).
* Steps may contain zero or more locations.
* A location starts with a Markdown style header beginning with `##` and then an
  optional name. (Location names might be `Stack` or `Heap`).
* Location contents consist of zero or more regions, or zero or more
  definitions.
* A region starts with a Markdown style header beginning with `###` and then an
  optional name. (Regions are used for stack to divide frames, example region
  name is `main`).
* Region contents consist of zero or more definitions.
* If a location has regions then all content of the location must be in
  regions (cannot mix definitions and regions inside a location).

The  contents are lists of definitions.

* A definition is a label, `:`, then a value, then end of line.
* Labels can be any alphanumeric/digits/underscore text starting with a
  non-digit.
* Labels can optionaly be escaped with backticks to allow arbitrary text inside
  (except backticks).
* Values can be:
    * `*` (representing an invalid value or NULL pointer)
    * `[`, a comma separated list of values, `]`
    * `(`, a comma separated list of values, `)`
    * `ptr(`, a destination, `)`, followed by optional "help" of the form `.`,
      then identifier, repeated any number of times
    * an integer number
    * a floating point number
    * `'`, a character, `'`
    * a label, `{`, then a comma separated list of label, `:`, value, `}`
      (representing struct values)
* A destination is a label followed by zero or more copies of `.` and natural
  numbers (indicating paths within values).
* A destination can also be followed by zero or more `'` (to disambiguate
  shadowed names).

## Destinations

Pointers always point to labels within their step. Beyond that restriction,
pointers can point to any location or region within the step. The destination
label of the pointer must match up with a label in the definitions of one of the
locations or regions within the step.

If a destination label is just a label then it points to the value in the
definition corresponding to that label. If the value itself is a compound value
then it is also possible to point inside the structure. The destination can be
followed by a sequence of numbers representing index values into compound
structures. For example, if `H0` has an array value `[8, 9, 10]`, then
destination `H0` indicates the value `[8, 9, 10]`, `H0.0` indicates the value
`8`, `H0.1` indicates the value `9`, and `H0.2` indicates the value `10`.

Note that index values are always numbers, even for struct values. Structs
contain an internal list of fields. The index value counts starting from 0 into
this list of field values.

If necessary a pointer value can refer to itself. This is legal in the diagram.
Just use the name in the definition of the pointer for the destination.

## Shadowing

Within a step there may be more than one instance of a label. The normal way
this happens is if a later stack variable shadows the definition of an earlier
variable of the same name. Both instances will appear in the `Stack` location
with the same labels.

Pointer destinations can refer to either instance in the step by using the `'`
suffix. The convention is that pointer destinations always resolve in order from
first to last as ordered in the `kaya` description. So without any `'` suffixes
a pointer will always refer to the first instance of that label no matter where
it appears in relation to the location of the pointer value. A destination with
one `'` will always refer to the second instance of the label and so on.

## Help

The "help" part of `ptr` is for layout and drawing. Let's you pick properties of
the pointer arrow.

```
.sn .se .sw .ss         Start at north side, east side, ...

.dn .de .dw .ds         End destination on north side, east side, ...

.straight .arc .fluid   Strategies to draw the arrow.
.magnet .grid           Default is .fluid.

.svlight                Set gravity of the source, from very light
.slight                 to very heavy. Higher gravity influences the
.smedium                arrow more.
.sheavy
.svheavy

.dvlight ...            Set gravity of the destination.

.vl .vlight             Set gravity of both source and destination.
.l .light
.m .medium
.h .heavy
.vh .vheavy
```

## Example

This is a small example.

```kaya
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

```kaya
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
