<!--

If you just want to read the manual you can click "Kaya" in the upper-left
corner of the page. Right now you are reading the source Markdown for the
manual in the Markdown editor. Feel free to edit things here and see the
changes on the right.

-->

# Kaya Manual

Kaya is a human-readable text format that uses some Markdown conventions and
encodes program state at different points. In this manual we'll explain how Kaya
works and give examples to show how you might use it.

## Overview

Kaya is inspired by the
[Aquascope](https://github.com/cognitive-engineering-lab/aquascope) project. The
Aquascope project includes many different tools for compiling and visualizing
Rust programs (even incorrect Rust programs). One of these tools produces
diagrams similar to Kaya. The goal of the Kaya project is to formalize the
diagram part of Aquascope in a way that is generally useful outside of the
Aquascope toolset while still being compatible with the Aquascope tools.

The normal kind of diagram you would use Kaya to generate will generally
include the following things:
* One or more snapshots at different program states
* Illustration of the stack with names and values
* Illustration of the heap with values
* Arrows drawn between values on the stack and heap representing pointers or references

You might use the diagrams in things like tutorials or other explanatory texts
desribing various programming concepts. The diagrams are also well suited to
programming exercises and quiz problems inside of learning resources. The format
is designed to be relatively simple to automatically generate so the diagrams
can also be used within systems doing program analysis to communicate results of
the analysis. Finally, Kaya diagrams are amenable to generation by LLMs for
general communication purposes so can be part of agentic program analysis or
synthesis workflows.

## Pseudocode Examples

These examples are Kaya diagrams for pseudocode programs that illustrate
different fundamental concepts in programming.

### Example: _Simple Stack Values_

Here is a simple program with location comments `L0` through `L3` marking
locations at runtime that we will illustrate in the diagram.

```text
main():      /* L0 */
  x = 1      /* L1 */
  y = x      /* L2 */
  x += 1     /* L3 */
```

Here is the Kaya diagram showing the state of the program at `L0` through `L3`.

```kaya
# L0
## Stack
### main

# L1
## Stack
### main
x: 1

# L2
## Stack
### main
x: 1
y: 1

# L3
## Stack
### main
x: 2
y: 1
```

#### Concept

The main thing being illustrated here is how `x` and `y` are independent. The
initial value of `y` is copied from `x`, but when `x` is then later updated it
does not affect `y`.

### Example: _Aliased Heap Value_

Here is a simple program with location comments `L0` through `L3` marking
locations at runtime that we will illustrate in the diagram.

```text
main():      /* L0 */
  x = new(1) /* L1 */
  y = x      /* L2 */
  *x += 1    /* L3 */
```

Here is the Kaya diagram showing the state of the program at `L0` through `L3`.

```kaya
# L0
## Stack
### main

# L1
## Stack
### main
x: ptr(H0)
## Heap
H0: 1

# L2
## Stack
### main
x: ptr(H0)
y: ptr(H0)
## Heap
H0: 1

# L3
## Stack
### main
x: ptr(H0)
y: ptr(H0)
## Heap
H0: 2
```

#### Concept

The main thing being illustrated here is how `x` and `y` are both pointers to
the same heap value. When `x` is used to update the value both pointers will
then see the new value.

### Example: _Linked List_

Here is a program with location comments `L0` through `L2` marking locations at
runtime that we will illustrate in the diagram. The program constructs a linked
list where each node is a tuple of two values where the first is the data in the
linked list and the second is a pointer to the next node or an invalid pointer
if there are no more nodes in the list.

```text
main():
    x = new((1, (2, (3, *)))) /* L0 */
    y = x[1]                  /* L1 */
    x = y                     /* L2 */
```

Here is the Kaya diagram showing the state of the program at `L0` through `L2`.

```kaya
# L0
## Stack
### main
x: ptr(H0)
## Heap
H0: (1, ptr(H1))
H1: (2, ptr(H2))
H2: (3, *)

# L1
## Stack
### main
x: ptr(H0)
y: ptr(H1)
## Heap
H0: (1, ptr(H1))
H1: (2, ptr(H2))
H2: (3, *)

# L2
## Stack
### main
x: ptr(H1)
## Heap
H0: (1, ptr(H1))
H1: (2, ptr(H2))
H2: (3, *)
```

### Example: _Stack Pointers_

```text
main():     /* L0 */
    x = 1   /* L1 */
    y = &x  /* L2 */
    *y = 2  /* L3 */
```

```kaya
# L0
## Stack
### main

# L1
## Stack
### main
x: 1

# L2
## Stack
### main
x: 1
y: ptr(x)

# L3
## Stack
### main
x: 2
y: ptr(x)

```


## Rust Examples

These examples are Kaya diagrams for simple Rust programs that illustrate
different fundamental concepts in programming.

### Example: _Simple Stack Values_

Here is a simple Rust program with location comments `L0` through `L4` marking
locations at runtime that we will illustrate in the diagram.

```rust
fn main()        /* L0 */ {
  let mut x = 1; /* L1 */
  let y = x;     /* L2 */
  x += 1;        /* L3 */
}                /* L4 */
```

Here is the Kaya diagram showing the state of the program at `L0` through `L4`.

```kaya
# L0
## Stack
### main

# L1
## Stack
### main
x: 1

# L2
## Stack
### main
x: 1
y: 1

# L3
## Stack
### main
x: 2
y: 1

# L4
## Stack
### main
```

#### Concept

The main thing being illustrated here is how `x` and `y` are independent. The
initial value of `y` is copied from `x`, but when `x` is then later updated it
does not affect `y`.

### Example: _One Boxed Heap Value_

Here is a simple Rust program with location comments `L0` through `L3` marking
locations at runtime that we will illustrate in the diagram.

```rust
fn main()                  /* L0 */ {
  let mut x = Box::new(0); /* L1 */
  *x += 1;                 /* L2 */
}                          /* L3 */
```

Here is the Kaya diagram showing the state of the program at `L0` through `L3`.

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
```

#### Concept

The main thing being illustrated here is how `x` is a pointer value that points
to a value on the heap that can be manipulated by following the pointer.

### Example: _Pointer Chain_

```rust
fn main()                              /* L0 */ {
    let mut x = Box::new(Box::new(2)); /* L1 */
    **x += 1;                          /* L2 */
}                                      /* L3 */
```

```kaya
# L0
## Stack
### main

# L1
## Stack
### main
x: ptr(H0)
## Heap
H0: ptr(H1)
##
H1: 2

# L2
## Stack
### main
x: ptr(H0)
## Heap
H0: ptr(H1)
##
H1: 3

# L3
## Stack
### main
```

In the Kaya diagram source above the multiple heap values are split into two
separate columns. The first column is labelled "Heap", the second is unlabelled.
This is just to arrange the values in the heap in a way that shows off the
chained arrows.

```text
# L2
## Stack
### main
x: ptr(H0)
## Heap
H0: ptr(H1)
##    // <--- this splits and starts new nameless heap column
H1: 3
```

The diagram can also be done showing both heap values in a single heap by
removing the `##` line.

```kaya
# L2
## Stack
### main
x: ptr(H0)
## Heap
H0: ptr(H1)
H1: 3
```

#### Concept

This example illustrates nested pointers and nested dereferencing. It also shows
how to use multiple heap columns to arrange heap values visually.

### Example: _Aliased Heap Value_

Here is a simple *but wrong* Rust program with location comments `L0` through `L4` marking
locations at runtime that we will illustrate in the diagram.

```rust
fn main()                  /* L0 */ {
  let mut x = Box::new(0); /* L1 */
  let y = x;               /* L2 */
  *x += 1;                 /* L3 */ // REJECTED: value used after move
}                          /* L4 */
```

Because the compiler rejects the above program there is no correct diagram
showing the execution of the program.

Here is the Kaya diagram showing what the programmer _incorrectly thinks_ will
be the state of the program at `L0` through `L4`.

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
y: ptr(H0)
## Heap
H0: 0

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

The programmer thinks that the line `let y = x` will create a new pointer to the
single heap value and that both `x` and `y` will see any updates to the value.

In reality the line `let y = x` *moves* the heap value to `y`. Once that has
happened there is no way to use `x`.

Here is the Kaya diagram showing what might happen if the Rust compiler
let the code execute:

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
x: *
y: ptr(H0)
## Heap
H0: 0
```

In this sequence there is no `L3` because the operation at point `L2` is a
pointer access through an invalid pointer, e.g. a `segfault`.

#### Concept

The main thing being illustrated here is how moving a value invalidates the
previous binding to the value. The Kaya diagrams can show what the programmer
might expect to happen, or what could happen if the Rust compiler allowed access
to invalidated values. In actuality the compiler rejects the program so it
cannot execute.

## More Examples

Here is the example from the `README.md` in the repository:

```kaya
/* 
Demo Kaya diagram

This demo is designed to demonstrate a few features of Kaya in a somewhat
realistic way without being overwhelming.
*/

# L1
## Stack
### main
x: ptr(H0)
y: ptr(H1)
## Heap
H0: 1
H1: (4, ptr(H2))
##
H2: (2, *)

# L2
## Stack
### main
x: ptr(H0)
y: *
z: ptr(H2).ds
## Heap
H0: 1
H1: (4, ptr(H2))
##
H2: (2, *)
```

Here is the more advanced example from the repository:

```kaya
/* 
Demo2 Kaya diagram

This demo is designed to demonstrate as many features of Kaya as possible in one
diagram.

* Steps
* Stack values
* Stack regions
* Heap values
* Multiple nameless heaps
* Multiple named heaps
* Pointers to stack
* Pointers to heap
* Pointers to contents of struct/tuple
* Pointers to contents of array
* Pointers to deep contents
* Pointers inside struct
* Pointers inside array
* Invalid pointers
* Char values
* Number values
* Looped pointers

*/

# L0
## Stack
### main
x: 5
y: [67, ptr(x)]
### foo
m: ptr(H0)
d: ptr(H1)
l: ptr(H2)
u: (1000, 1001, ptr(H1.0).ds)
z: [1, [2, [3, 4]]]
## Heap
H0: ['H', 'e', 'l', 'l', 'o', ptr(H100)]
H1: person{ id: 1000 }
H2: [5, ptr(H3)]
H3: [4, *]
## Arena
H100: [67, 5]
end: ptr(end)

```
