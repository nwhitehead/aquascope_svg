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

## Basic Examples

## More Examples

## Setting Up

Here is a Kaya diagram showing memory:

```kaya
# L1
## Stack
x: 5
y: 7
z: ptr(x)
p: ptr(H0)
## Heap
H0: (42, ptr(z))
```

And some python:

```python
print(f"What is 4?")
for i in range(10):
    print(i * i)
```

Yes
