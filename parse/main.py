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

location: "## " TEXT _EOL ( region* | def* )

region: "### " TEXT _EOL def*

def: label ":" value _EOL

destination: label ("." DIGITS)* borrow
borrow: "'"*

?value:
| number
| "[" value ("," value)* "]" -> array_value
| "(" value ("," value)* ")" -> tuple_value
| "'" /[^']/ "'" -> char_value
| label "{" (label ":" value ("," label ":" value)*)? "}" -> struct_value
| "ptr" "(" destination ")" -> ptr_value
| "*" -> invalid_value

""")

class MyTransformer(Transformer):
    def float(self, x):
        return float(x[0])
    def int(self, x):
        return int(x[0])
    def number(self, n):
        return n[0]
    def label(self, n):
        return n[0]
    array_value = list
    tuple_value = tuple
    def char_value(self, n):
        return n[0]

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
H0: ptr( H0.1.0.0' )

# L4
## Stack
### main
""")
    tree = MyTransformer().transform(tree)
    print(tree.pretty())
    # pprint(tree, indent=2, width=80)

if __name__ == "__main__":
    main()
