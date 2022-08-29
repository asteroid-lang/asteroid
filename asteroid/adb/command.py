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
            
            ('NEXT',        r'\bnext\b|\bn\b'),
            ('STEP',        r'\bstep\b|\bs\b'),
            ('CONTINUE',    r'\bcontinue\b|\bcont\b|\bc\b'),
            ('RETURN',      r'\breturn\b|\bret\b|\br\b'),
            ('UNTIL',       r'\buntil\b|\bu\b'),

            ('EVAL',        r'\beval\b'),
            ('BANG',        r'!'),

            ('BREAK',       r'\bbreak\b|\bb\b'),
            ('DELETE',      r'\bdelete\b|\bdel\b|\bd\b'),

            ('MACRO',       r'\bmacro\b'),
            ('LIST',        r'\blist\b|\bl\b'),
            ('LONGLIST',    r'\blonglist\b|\bll\b'),

            ('RETVAL',      r'\b__retval__\b|\b_\b'),
            
            ('EXPLICIT',    r'\bexplicit\b|\be\b'),
            ('ON',          r'\bon\b'),
            ('OFF',         r'\boff\b'),

            ('HELP',        r'\bh\b|\bhelp\b'),
            ('WHERE',       r'\bwhere\b|\bw\b'),
            ('UP',          r'\<'),
            ('DOWN',        r'\>'),

            ('IF',          r'\bif\b'),

            ('NUM',         r'[+-]?([0-9]*[.])?[0-9]+'),
            ('EQUAL',       r'='),
            ('SEMI',        r';'),
            ('COMMA',       r','),
            ('LPAREN',      r'\('),
            ('RPAREN',      r'\)'),

            ('QUIT',        r'\bquit\b|\bq\b'),

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
            while self.dlx.pointer().type in ['SEMI']:
                self.dlx.match('SEMI')

                if self.dlx.pointer().type == 'EOF':
                    break

                cmds += [self.command()]

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

            if not first:
                if self.dlx.pointer().type == 'COMMA':
                    self.dlx.match('COMMA')
                    continue
                elif self.dlx.pointer().type == 'EOF':
                    self.dlx.match('EOF')
                    break

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

    def explicit_cmd(self):
        self.dlx.match('EXPLICIT')

        set_exp = None
        if self.dlx.pointer().type in ['ON', 'OFF']:
            set_exp = (self.dlx.pointer().type == 'ON')
            self.dlx.next()
        
        return ('EXPLICIT', set_exp)

    def until_cmd(self):
        self.dlx.match('UNTIL')
        
        n = None
        if self.dlx.pointer().type == 'NUM':
            n = self.dlx.pointer().value
            self.dlx.next()
        
        return ('UNTIL', n)

    def command(self):
        t = self.dlx.pointer().type
        
        if t ==  'EVAL':     return self.eval_cmd()
        elif t ==  'BREAK':    return self.break_cmd()
        elif t ==  'DELETE':   return self.delete_cmd()
        elif t ==  'HELP':     return self.help_cmd()
        elif t ==  'EXPLICIT': return self.explicit_cmd()
        elif t ==  'UNTIL':    return self.until_cmd()
        
        elif t in ['BANG', 'LONGLIST', 'LIST', 'QUIT', 'RETURN', 'RETVAL',
             'STEP', 'CONTINUE', 'NEXT', 'UP', 'DOWN', 'WHERE']:
            t = self.dlx.pointer().type
            self.dlx.match(t)
            return (t,)
        
        elif t ==  'NAME':
            n = self.dlx.match('NAME').value
            return ('NAME', n)
        
        elif t in  ['EOF', 'SEMI']:
            return ('NOOP',)
        
        else:
            raise ValueError("Unknown command: {}".format(
                str(self.dlx.pointer().value)
            ))