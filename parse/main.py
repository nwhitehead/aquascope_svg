from lark import Lark

l = Lark(open('test_grammar.lark').read())

def main():
    print( l.parse("Hello, World!") )

if __name__ == "__main__":
    main()
