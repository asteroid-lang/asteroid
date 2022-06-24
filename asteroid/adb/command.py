import re

class Token:
    def __init__(self,type,value):
        self.type = type
        self.value = value

    def __str__(self):
        return 'Token({},{})'.format(self.type,self.value)

class DebuggerLexer:
    def __init__(self, input_string):
        self.token_specs = [
        #   type:          value:
            ('STRING',     r'"(\\"|\\.|[^"])*"'),
            
            ('STEP',        r'\bstep\b|\bs\b'),
            ('EVAL',        r'\beval\b'),
            ('CONTINUE',    r'\bcontinue\b|\bcont\b|\bc\b'),
            ('NEXT',        r'\bnext\b|\bn\b'),
            ('BREAK',       r'\bbreak\b|\bb\b'),
            ('DELETE',      r'\bdelete\b|\bdel\b|\bd\b'),
            ('BANG',        r'!'),
            ('MACRO',       r'\bmacro\b'),
            ('LIST',        r'\blist\b|\bl\b'),
            ('LONGLIST',    r'\blonglist\b|\bll\b'),
            ('QUIT',        r'\bquit\b|\bq\b'),
            ('EXPLICIT',    r'\bexplicit\b|\be\b'),
            ('UNEXPLICIT',  r'\bunexplicit\b|\bu\b'),
            ('HELP',        r'\bh\b|\bhelp\b'),
            ('UP',          r'\bup\b'),

            ('IF',          r'\bif\b'),

            ('NUM',         r'[+-]?([0-9]*[.])?[0-9]+'),
            ('EQUAL',       r'='),
            ('SEMI',        r';'),
            ('COMMA',       r','),
            ('LPAREN',      r'\('),
            ('RPAREN',      r'\)'),

            ('NAME',        r'[a-zA-Z_\$][a-zA-Z0-9_\$]*'),
            ('WHITESPACE',  r'[ \t\n]+'),
            ('UNKNOWN',     r'.')
        ]

        # used for sanity checking in lexer.
        self.token_types = set(type for (type,_) in self.token_specs)
        self.tokens = self.tokenize(input_string)
        # the following is always valid because we will always have
        # at least the EOF token on the tokens list.
        self.curr_token_ix = 0

    def pointer(self):
        return self.tokens[self.curr_token_ix]

    def next(self):
        if not self.end_of_file():
            self.curr_token_ix += 1
        return self.pointer()

    def match(self, token_type):
        if token_type == self.pointer().type:
            tk = self.pointer()
            self.next()
            return tk
        elif token_type not in self.token_types:
            raise ValueError("unknown token type '{}'".format(token_type))
        else:
            raise ValueError('unexpected token {} while parsing, expected {}'
                              .format(self.pointer().type, token_type))

    def end_of_file(self):
        if self.pointer().type == 'EOF':
            return True
        else:
            return False

    def tokenize(self, code):
        tokens = []
        re_list = ['(?P<{}>{})'.format(type,re) for (type,re) in self.token_specs]
        combined_re = '|'.join(re_list)
        match_object_list = list(re.finditer(combined_re, code))
        for mo in match_object_list:
            type = mo.lastgroup
            value = mo.group()

            if type == 'WHITESPACE':
                continue #ignore
            elif type == 'STRING':
                tokens.append(Token('STRING', value.encode('utf-8')
                                                   .decode('unicode-escape')[1:-1]))
            elif type == 'UNKNOWN':
                raise ValueError("unexpected character '{}'".format(value))
            else: 
                tokens.append(Token(type, value))
        tokens.append(Token('EOF', '\eof'))
        return tokens

class DebuggerParser:
    def __init__(self):
        self.dlx = None

    def parse(self, input_string):
        self.dlx = DebuggerLexer(input_string)
        return self.line()

    def line(self):
        if self.dlx.pointer().type == 'MACRO':
            return self.macro()
        else:
            cmds = [self.command()]
            while self.dlx.pointer().type == 'SEMI':
                self.dlx.match('SEMI')
                if self.dlx.pointer().type != 'EOF':
                    cmds += [self.command()]
                else:
                    break
            return ('LINE', cmds)
    
    def macro(self):
        self.dlx.match('MACRO')

        if self.dlx.pointer().type == 'EOF':
            return ('LINE', [('MACRO',)])

        else:
            name = self.dlx.match('NAME')
            self.dlx.match('EQUAL')

            l = self.line()

            return ('LINE', [('MACRO', name.value, l[1])] )

    def eval_cmd(self):
        self.dlx.match("EVAL")
        self.dlx.match('LPAREN')
        code = self.dlx.match('STRING')
        self.dlx.match('RPAREN')
        
        return ('EVAL', code.value)
    
    def break_cmd(self):
        self.dlx.match('BREAK')
        nums = []
        conds = []
        first = True

        while self.dlx.pointer().type == 'NUM':
            if first:
                first = False
            else:
                self.dlx.match('COMMA')
            nums.append(self.dlx.match('NUM').value)

            if self.dlx.pointer().type == 'IF':
                self.dlx.match('IF')
                self.dlx.match('EVAL')
                self.dlx.match('LPAREN')
                code = self.dlx.match('STRING')
                self.dlx.match('RPAREN')
                conds.append(code.value)
            else:
                conds.append(None)
        return ('BREAK', list(map(int, nums)), conds)

    def delete_cmd(self):
        self.dlx.match('DELETE')
        nums = [self.dlx.match('NUM').value]
        
        while self.dlx.pointer().type == 'NUM':
            nums.append(self.dlx.match('NUM').value)
        
        return ('DELETE', list(map(int, nums)))


    def help_cmd(self):
        self.dlx.match('HELP')
        
        n = None
        if self.dlx.pointer().type != 'EOF':
            n = self.dlx.pointer().value
            self.dlx.next()
        
        return ('HELP', n)
    
    def command(self):
        match(self.dlx.pointer().type):
            case 'EVAL':    return self.eval_cmd()
            case 'BREAK':   return self.break_cmd()
            case 'DELETE':  return self.delete_cmd()
            case 'HELP':    return self.help_cmd()

            case 'BANG' | 'LONGLIST' | 'LIST' | 'QUIT' | 'EXPLICIT' | 'UNEXPLICIT' | \
                 'STEP' | 'CONTINUE' | 'NEXT' | 'UP':
                t = self.dlx.pointer().type
                self.dlx.match(t)
                return (t,)

            case 'NAME':
                n = self.dlx.match('NAME').value
                return ('NAME', n)

            case 'EOF':
                return []

            case _:
                raise ValueError("Unknown command: {}".format(
                    str(self.dlx.pointer().value)
                ))