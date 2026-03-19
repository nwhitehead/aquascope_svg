import argparse
from pprint import pprint
from lark import Lark, Transformer

l = Lark(r"""

%import common.WS_INLINE
%import common.NEWLINE
%import common.C_COMMENT
%import common.CPP_COMMENT
%import common.FLOAT
%import common.INT

// Ignore inline whitespace (but keep newlines)
%ignore WS_INLINE

// Allow C and C++ style comments
%ignore C_COMMENT
%ignore CPP_COMMENT

%import common.CNAME

TEXT: /[^\n]+/
_EOL: NEWLINE+
UNESCAPED_LABEL: CNAME
ESCAPED_LABEL: "`" /[^`]+/ "`"
DIGITS: /[\d]+/
unsigned_number: FLOAT -> float | INT -> int
number: ["+"|"-"] unsigned_number

label: UNESCAPED_LABEL | ESCAPED_LABEL

start: [_EOL] step*

step: "# " TEXT _EOL location*

location: "## " TEXT _EOL ( region* | defln_* )

region: "### " TEXT _EOL defln_*

def_ : label ":" value
?defln_: def_ _EOL

destination: label ("." DIGITS)* borrow
borrow: "'"*

?value:
| number
| "[" value ("," value)* "]" -> array_value
| "(" value ("," value)* ")" -> tuple_value
| "'" /[^']/ "'" -> char_value
| label "{" (def_ ("," def_)*)? "}" -> struct_value
| "ptr" "(" destination ")" -> ptr_value
| "*" -> invalid_value

""")

class NamedStruct:
    def __init__(self, name, data):
        self.name = name
        self.data = data
    def __repr__(self):
        inner = ", ".join([f"{n}: {repr(v)}" for (n, v) in self.data])
        return f'{self.name }{{{inner}}}'

class Ptr:
    def __init__(self, name, selectors, borrow):
        self.name = name
        self.selectors = selectors
        self.borrow = borrow
    def __repr__(self):
        access = "".join([f".{str(x)}" for x in self.selectors])
        borrows = "'" * self.borrow
        return f'ptr({self.name}{access}{borrows})'

class MyTransformer(Transformer):
    def float(self, x):
        return float(x[0])
    def int(self, x):
        return int(x[0])
    def number(self, n):
        return n[0]
    def label(self, n):
        return n[0]
    def def_(self, n):
        return [n[0][0], n[1]]
    start = list
    step = list
    location = list
    region = list
    array_value = list
    tuple_value = tuple
    TEXT = str
    UNESCAPED_LABEL = str
    ESCAPED_LABEL = str
    def DIGITS(self, n):
        return int(n)
    def char_value(self, n):
        return str(n[0])
    def invalid_value(self, n):
        return '*'
    def struct_value(self, n):
        return NamedStruct(n[0], n[1:])
    def destination(self, n):
        borrows = 0
        while len(n) > 1 and n[-1] == 'B':
            borrows += 1
            del n[-1]
        return Ptr(n[0], n[1:], borrows)
    def ptr_value(self, n):
        return n[0]
    def borrow(self, n):
        return 'B'

def main():
    tree = l.parse("""
# L0
## Stack
### main
x: 'h'

# L1
## Stack
### main
x: [5.2, 1]
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
H0: ptr( H0.1.10.0' )

# L4
## Stack
### main
""")
    tree = MyTransformer().transform(tree)
    pprint(tree)
    # pprint(tree, indent=2, width=80)
    ns = NamedStruct('foo', [('bar', 5), ('x', '*')])
    print(ns)
    ptr = Ptr('H', [0, 1], 2)
    print(ptr)

if __name__ == "__main__":
    main()
