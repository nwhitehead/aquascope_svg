from lark import Lark

l = Lark(open('grammar.lark').read())

def main():
    print( l.parse("""
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
""") )

if __name__ == "__main__":
    main()
