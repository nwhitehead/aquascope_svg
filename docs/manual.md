# Kaya Manual

Kaya is a human-readable text format that uses some Markdown conventions and
encodes program state at various points. In this manual we'll explain how
Kaya works and give examples to show how you might use it. At the end we'll
also cover adding Kaya support to your Markdown renderer.

## Overview

The 

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
