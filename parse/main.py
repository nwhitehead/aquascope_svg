from lark import Lark

l = Lark(open('grammar.lark').read())

def main():
    tree = l.parse("""
# L0
## Stack
### main
x: 'h'

# L1
## Stack
### main
x: [5, 1]
y: (2, 1)
## Heap
H0: 5

# L2
## Stack
### main
## Heap

# L3
## Stack
### main
x: foo{i: 3, j: `bar`{}}
y: *
## Heap
H0: ptr( H0.1.0.0' )

# L4
## Stack
### main
""")
    print(tree.pretty())

if __name__ == "__main__":
    main()
