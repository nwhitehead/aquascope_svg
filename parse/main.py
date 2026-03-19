from lark import Lark

l = Lark('''start: WORD "," WORD "!"

            %import common.WORD   // imports from terminal library
            %ignore " "           // Disregard spaces in text
         ''')

def main():
    print( l.parse("Hello, World!") )

if __name__ == "__main__":
    main()
