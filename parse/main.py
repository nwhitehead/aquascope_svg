import argparse
from lark import Lark

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
EOL: NEWLINE+
UNESCAPED_LABEL: CNAME
ESCAPED_LABEL: "`" /[^`]+/ "`"
DIGITS: /[\d]+/

label: UNESCAPED_LABEL | ESCAPED_LABEL

start: [EOL] step*

step: "# " TEXT EOL location*

location: "## " TEXT EOL ( region* | def* )

region: "### " TEXT EOL def*

def: label ":" value EOL

destination: label ("." DIGITS)* borrow
borrow: "'"*

value: SIGNED_NUMBER
| "[" value ("," value)* "]" -> array_value
| "(" value ("," value)* ")" -> tuple_value
| "'" /[^']/ "'" -> char_value
| label "{" (label ":" value ("," label ":" value)*)? "}" -> struct_value
| "ptr" "(" destination ")" -> ptr_value
| "*" -> invalid_value

""")

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
