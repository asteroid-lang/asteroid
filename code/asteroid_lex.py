###########################################################################################
# Lexer for Asteroid
#
# (c) Lutz Hamel, University of Rhode Island
###########################################################################################

# NOTE: we copied the lexer from ply (version 3.11) so that
# we have no installation dependencies other than python 3.x
import lex
from lex import LexToken
from asteroid_state import state

reserved = {
    'and'           : 'AND',
    'assert'        : 'ASSERT',
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
    'nonlocal'      : 'NONLOCAL',
    'not'           : 'NOT',
    'or'            : 'OR',
    'orwith'        : 'ORWITH',
    'otherwise'     : 'OTHERWISE',
    'pattern'       : 'PATTERN',
    'repeat'        : 'REPEAT',
    'return'        : 'RETURN',
    'step'          : 'STEP',
    'structure'     : 'STRUCTURE',
    'throw'         : 'THROW',
    'to'            : 'TO',
    'trait'         : 'TRAIT',
    'try'           : 'TRY',
    'until'         : 'UNTIL',
    'while'         : 'WHILE',
    'with'          : 'WITH',
    # constants
    'none'          : 'NONE',
    'true'          : 'TRUE',
    'false'         : 'FALSE',
    }

literals = [':','.',',','=','(',')','[',']','|','@']

tokens = [
          'PLUS',
          'MINUS',
          'TIMES',
          'DIVIDE',
          'EQ',
          'NE',
          'LE',
          'LT',
          'GE',
          'GT',
          'INTEGER',
          'REAL',
          'STRING',
          'ID',
          'QUOTE',
          'TYPECLASS',
          'CMATCH',
          ] + list(reserved.values())

t_PLUS    = r'\+'
t_MINUS   = r'-'
t_TIMES   = r'\*'
t_DIVIDE  = r'/'
t_EQ      = r'=='
t_NE      = r'=/='
t_LE      = r'<='
t_LT      = r'<'
t_GE      = r'>='
t_GT      = r'>'

t_QUOTE   = r'\''

t_ignore = ' \t'

def t_TYPECLASS(t):
    r'\%[a-zA-Z_][a-zA-Z_0-9]*'
    # check for typeclass keywords
    # for the values get rid of the preceeding '%'
    if t.value[1:] == 'if':
        t.type = 'CMATCH'
        t.value = t.value[1:]
        return t
    else:
        t.type = 'TYPECLASS'
        t.value = t.value[1:]
        return t

def t_ID(t):
    r'[a-zA-Z_][a-zA-Z_0-9]*'
    # Check for reserved words
    t.type = reserved.get(t.value,'ID')
    return t

# TODO: scientific notation for real numbers
def t_NUMBER(t):
    r'([0-9]*[.])?[0-9]+'
    if '.' in t.value:
        t.type = 'REAL'
        t.value = float(t.value)
    else:
        t.type = 'INTEGER'
        t.value = int(t.value)
    return t

def t_STRING(t):
    r'\"[^\"]*\"'
    lines = t.value.count('\n')
    (module, lineno) = state.lineinfo
    state.lineinfo = (module, lineno + lines)
    t.value = t.value[1:-1] # strip the quotes
    from pprint import pprint
    return t

def t_COMMENT(t):
    # hash comment is only here to support the shebang for linux
    # so that Asteroid scripts can run as "executables"
    r'--.* | \#.*'
    pass

def t_NEWLINE(t):
    r'\n+'
    (module, lineno) = state.lineinfo
    state.lineinfo = (module, lineno + len(t.value))

def t_error(t):
    raise ValueError("illegal character {}".format(t.value[0]))

def dbg_print(string):
    #print(string)
    pass

class Lexer:

    def __init__(self):
        self.plylexer = lex.lex(debug=0)
        self.curr_token = None

    def make_eof_token(self):
        if not self.curr_token:
            t = LexToken()
            t.type = 'EOF'
            t.value = ''
            self.curr_token = t

    def input(self, input_string):
        self.plylexer.input(input_string)
        self.curr_token = self.plylexer.token()
        self.make_eof_token()

    def peek(self):
        return self.curr_token

    def next(self):
        dbg_print('skipping {}'.format(self.curr_token.type))
        self.curr_token = self.plylexer.token()
        self.make_eof_token()

    def EOF(self):
        if self.curr_token.type == 'EOF':
            return True
        else:
            return False

    def match(self, token_type):
        if token_type not in tokens+literals:
            raise ValueError("unknown token type: '{}'.".format(token_type))
        elif token_type != self.curr_token.type:
            raise ValueError("expected '{}' found '{}'."
                             .format(token_type, self.curr_token.type))
        else:
            dbg_print('matching {}'.format(token_type))
            ct = self.curr_token
            self.curr_token = self.plylexer.token()
            self.make_eof_token()
            return ct

    def match_optional(self, token_type):
        if token_type == self.curr_token.type:
            return self.match(token_type)
        else:
            return None

# test lexer
if __name__ == "__main__":

    lexer = Lexer()

    data = 'let x = y@{"foo"}.'
    lexer.input(data)

    while not lexer.EOF():
        tok = lexer.peek()
        print(tok)
        lexer.next()
