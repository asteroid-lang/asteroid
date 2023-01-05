###########################################################################################
# Lexer for Asteroid
#
# (c) University of Rhode Island
###########################################################################################

import re

from asteroid.state import state, warning
from asteroid.globals import ExpectationError

# table that specifies the token value and type for keywords
keywords = {
#   value:          type:
    'and'           : 'AND',
    'as'            : 'AS',
    'assert'        : 'ASSERT',
    'bind'          : 'BIND',
    'break'         : 'BREAK',
    'catch'         : 'CATCH',
    'data'          : 'DATA',
    'do'            : 'DO',
    'elif'          : 'ELIF',
    'else'          : 'ELSE',
    'end'           : 'END',
    'escape'        : 'ESCAPE',
    'eval'          : 'EVAL',
    'for'           : 'FOR',
    'from'          : 'FROM',
    'function'      : 'FUNCTION',
    'global'        : 'GLOBAL',
    'if'            : 'IF',
    'in'            : 'IN',
    'is'            : 'IS',
    'lambda'        : 'LAMBDA',
    'let'           : 'LET',
    'load'          : 'LOAD',
    'loop'          : 'LOOP',
    'not'           : 'NOT',
    'or'            : 'OR',
    'orwith'        : 'ORWITH',
    'pattern'       : 'PATTERN',
    'repeat'        : 'REPEAT',
    'return'        : 'RETURN',
    'step'        : 'STEP',
    'structure'     : 'STRUCTURE',
    'system'        : 'SYSTEM',
    'throw'         : 'THROW',
    'to'            : 'TO',
    'try'           : 'TRY',
    'until'         : 'UNTIL',
    'while'         : 'WHILE',
    'with'          : 'WITH',
    # constants
    'none'          : 'NONE',
    'true'          : 'TRUE',
    'false'         : 'FALSE',
    }

# this table defines tokens whose value is defined by a
# regular expression.
token_specs = [
#   value:                                        display:      type:
    (r'[0-9]+([.][0-9]+)?((e|E)(\+|\-)?[0-9]+)?', "number",     'NUMBER'),
    # the following is very fragil, re's are tried in order, therefore we
    # try matching things in the following order:
    #   1. everything that is not a double quote or a backslash
    #   2. an escaped double quote
    #   3 then a backslash
    (r'"([^"\\]|\\"|\\)*"',                       "string",     'STRING'),
    (r'(--.*)|(\#.*)',                            "comment",    'COMMENT'),
    (r'[a-zA-Z_][a-zA-Z_0-9]*',                   "variable",   'ID'),
    (r'\n',                                       "\n",         'NEWLINE'),
    (r'[ \t]+',                                   "space/tab",  'WHITESPACE'),
    (r'\%[a-zA-Z_][a-zA-Z_0-9]*',                 "%type",      'TYPEMATCH'),
    (r'\+',                                       "'+'",        'PLUS'),
    (r'-',                                        "'-'",        'MINUS'),
    (r'\*',                                       "'*'",        'TIMES'),
    (r'/',                                        "'/'",        'DIVIDE'),
    (r'==',                                       "'=='",       'EQ'),
    (r'=/=',                                      "'=/='",      'NE'),
    (r'<=',                                       "'<='",       'LE'),
    (r'<',                                        "'<'",        'LT'),
    (r'>=',                                       "'>='",       'GE'),
    (r'>',                                        "'>'",        'GT'),
    (r'@',                                        "'@'",        'AT'),
    (r'\%\[',                                     "'%['",       'LCONSTRAINT'),
    (r'\]\%',                                     "']%'",       'RCONSTRAINT'),
    (r'\(',                                       "'('",        'LPAREN'),
    (r'\)',                                       "')'",        'RPAREN'),
    (r'\[',                                       "'['",        'LBRACKET'),
    (r'\]',                                       "']'",        'RBRACKET'),
    (r':',                                        "':'",        'COLON'),
    (r'\|',                                       "'|'",        'BAR'),
    (r'\.',                                       "'.'",        'DOT'),
    (r',',                                        "','",        'COMMA'),
    (r'=',                                        "'='",        'ASSIGN'),
    # this is the catch-all pattern, it has to be
    # here do that we can report illegal characters
    # in the input.
    (r'.',                                        "unknown",    'MISMATCH'),
]

# this table specifies token types that are used in the tokenizer
# but are not defined in the above tables.
implicit_token_types = [
    'INTEGER',
    'REAL',
]

# A table that given a token type will return the expected value
# a reverse token specification: given a type compute the value
token_values = {}

def init_token_values():
    global token_values

    # grab tokens from token_specs table
    token_values.update({'INTEGER':'integer value'})
    token_values.update({'REAL':'real value'})
    token_values.update({'EOF':'EOF'})
    for (_,display,type) in token_specs:
        token_values.update({type:display})

    # grab tokens from keywords table
    for k in keywords:
        token_values.update({keywords[k]:"'"+k+"'"})

def token_lookup(type):
    return token_values[type]

class Token:
    def __init__(self,type,value,module,lineno):
        self.type = type
        self.value = value
        self.module = module
        self.lineno = lineno

    def __str__(self):
        return '({},{},{},{})'.format(self.type,self.value,self.module,self.lineno)

def tokenize(code):
    # output token list
    tokens = []
    # state/line info
    (module, line_num) = state.lineinfo
    # here we create a list of named patterns from the token_specs table
    # the name of the pattern is the token type
    named_re_list = ['(?P<{}>{})'.format(type,re) for (re,_,type) in token_specs]
    # create one giant re that describes the token structure of the whole
    # language. we 'or' together all the re's on the named_re_list
    combined_re = '|'.join(named_re_list)
    # generate a list of match objects. the group name of a match
    # is the token type.
    match_object_list = list(re.finditer(combined_re, code))
    for mo in match_object_list:
        # get the token type and value from
        # the match object
        type = mo.lastgroup
        value = mo.group()
        # some special processing of tokens
        if type == 'NUMBER':
            if '.' in value or 'e' in value or 'E' in value:
                type = 'REAL'
                value = float(value)
            else:
                type = 'INTEGER'
                value = int(value)
        elif type == 'ID':
            # IDs and keywords share the same
            # here we replace the ID type with
            # the appropriate token type given
            # keyword value, if not a keyword the
            # code defaults to the ID token type
            type = keywords.get(value,'ID')
        elif type == 'TYPEMATCH':
            if value[1:] == 'if':
                warning("'%if' has been deprecated, please replace with 'if'")
                type = 'IF'
                value = value[1:]
            else:
                type = 'TYPEMATCH'
                value = value[1:]
        elif type == 'STRING':
            #lhh
            #print(value)
            if value[-2] == '\\' and value[-3] != '\\':
                raise ValueError("bad escape sequence '\\\"' in string")
            lines = value.count('\n')
            (module, lineno) = state.lineinfo
            line_num += lines
            state.lineinfo = (module, line_num)
            value = value[1:-1].replace('\\"','"') # strip the quotes
        elif type == 'NEWLINE':
            line_num += 1
            state.lineinfo = (module,line_num)
            continue
        elif type == 'COMMENT':
            continue
        elif type == 'WHITESPACE':
            continue
        elif type == 'MISMATCH':
            raise ValueError("unexpected character '{}'".format(value))
        # put the token onto the tokens list
        tokens.append(Token(type, value, module, line_num))
    # always append an EOF token so we never run out of tokens
    # in the lexer.
    tokens.append(Token('EOF', '', module, line_num))
    return tokens

def dbg_print(string):
    #print(string)
    pass

# convenient interface to the token stream
class Lexer:
    def __init__(self):
        self.tokens = None
        self.curr_token_ix = None
        self.curr_token = None
        # keep a set of all possible token types in our lexer
        # this let's us weed out bad match calls very easily
        self.token_types = \
            set(type for (_,_,type) in token_specs) | \
            set(list(keywords.values())) | \
            set(implicit_token_types)
        # init token value lookup
        init_token_values()

    def input(self, input_string):
        self.tokens = tokenize(input_string)
        # the following is always valid because we will always have
        # at least the EOF token on the tokens list.
        self.curr_token_ix = 0
        self.curr_token = self.tokens[self.curr_token_ix]
        state.lineinfo = (self.curr_token.module, self.curr_token.lineno)

    def peek(self):
        return self.curr_token

    def next(self):
        dbg_print('skipping {}'.format(self.curr_token.type))
        if self.curr_token.type != 'EOF':
            self.curr_token_ix += 1
            self.curr_token = self.tokens[self.curr_token_ix]
            state.lineinfo = (self.curr_token.module, self.curr_token.lineno)


    def EOF(self):
        if self.curr_token.type == 'EOF':
            return True
        else:
            return False

    def match(self, token_type):
        if token_type not in self.token_types:
            raise ValueError("unknown token type '{}'".format(token_type))
        elif token_type != self.curr_token.type:
            raise ExpectationError(found=token_lookup(self.curr_token.type),
                                   expected=token_lookup(token_type))
        else:
            dbg_print('matching {}'.format(token_type))
            ct = self.curr_token
            self.next()
            return ct

    def match_optional(self, token_type):
        if token_type == self.curr_token.type:
            return self.match(token_type)
        else:
            return None

# test lexer
if __name__ == "__main__":
    from sys import stdin
    lexer = Lexer()
    lexer.input(stdin.read())
    while not lexer.EOF():
        tok = lexer.peek()
        print(tok)
        lexer.next()
