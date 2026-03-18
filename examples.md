```rust
fn main() {
    let mut x = 1;
    let y = f(x);
    x += 1;
}
fn f(x: i32) -> i32 {
    x + 1
}
```

```states
# L0 {pos="1"}
## Stack
### main

# L1 {pos="2"}
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
