from lark import Lark

l = Lark(open('grammar.lark').read())

def main():
    tree = l.parse("""
# L0
## Stack
### main
x: 5

# L1
## Stack
### main
x: 5
y: 2
## Heap
H0: 5

# L2
## Stack
### main
## Heap

# L3
## Stack
### main
## Heap

# L4
## Stack
### main
""")
    print(tree.pretty())

if __name__ == "__main__":
    main()
