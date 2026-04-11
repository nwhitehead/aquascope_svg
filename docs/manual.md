<!--

If you just want to read the manual you can click "Kaya" in the upper-left
corner of the page. Right now you are reading the source Markdown for the
manual in the Markdown editor. Feel free to edit things here and see the
changes on the right.

-->

# Kaya Manual

Kaya is a human-readable text format that uses some Markdown conventions and
encodes program state at different points. In this manual we'll explain how Kaya
works and give examples to show how you might use it. At the end we'll also
cover adding Kaya support to your Markdown renderer.

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

## Basic Examples

Here is a simple Rust program, with location comments `L0` through `L4` marking
locations at runtime that we will illustrate with diagrams.

```rust
fn main()                  /* L0 */ {
  let mut x = Box::new(0); /* L1 */
  *x += 1;                 /* L2 */
  let y = x;               /* L3 */
}                          /* L4 */
```

```python
print(f"What is 4?")
for i in range(10):
    print(i * i)
```
## More Examples

## Setting Up

