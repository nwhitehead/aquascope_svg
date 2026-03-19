import argparse
from pprint import pprint
from lark import Lark, Transformer

l = Lark(r"""

%import common.WS_INLINE
%import common.NEWLINE
%import common.C_COMMENT
%import common.CPP_COMMENT
%import common.SIGNED_NUMBER

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

label: UNESCAPED_LABEL | ESCAPED_LABEL

start: [_EOL] step*

step: "# " TEXT _EOL location*

location: "## " TEXT _EOL ( region* | def* )

region: "### " TEXT _EOL def*

def: label ":" value _EOL

destination: label ("." DIGITS)* borrow
borrow: "'"*

?value:
| SIGNED_NUMBER -> number
| "[" value ("," value)* "]" -> array_value
| "(" value ("," value)* ")" -> tuple_value
| "'" /[^']/ "'" -> char_value
| label "{" (label ":" value ("," label ":" value)*)? "}" -> struct_value
| "ptr" "(" destination ")" -> ptr_value
| "*" -> invalid_value

""")

class MyTransformer(Transformer):
    start = list

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
    print(tree)
    # tree = MyTransformer().transform(tree)
    # pprint(tree, indent=2, width=80)

if __name__ == "__main__":
    main()
