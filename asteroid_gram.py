###########################################################################################
# parser for Asteroid
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
###########################################################################################

from asteroid_lex import Lexer
from asteroid_support import reverse_node_list
from asteroid_support import append_node_list
from asteroid_state import state

###########################################################################################
def dbg_print(string):
    #print(string)
    pass

###########################################################################################
# LL(1) lookahead sets

exp_lookahead = [
    'ESCAPE',
    'HASATTACH',
    'INTEGER',
    'REAL',
    'STRING',
    'TRUE',
    'FALSE',
    'NONE',
    'ID',
    '*',
    '-',
    'NOT',
    'LAMBDA',
    'QUOTE',
    '{',
    '(',
    '[']

stmt_lookahead = [
    '.',
    'ATTACH',
    'DETACH',
    'CONSTRUCTOR',
    'FUNCTION',
    'GLOBAL',
    'LET',
    'NONLOCAL',
    'NOOP',
    'PRINT',
    'PRINTLN',
    'READ',
    'REPEAT',
    'WITH',
    'FOR',
    'WHILE',
    'BREAK',
    'IF',
    'RETURN',
    'TRY',
    'ERROR'
    ] + exp_lookahead

###########################################################################################
class Parser:
    
    ###########################################################################################
    def __init__(self):
        self.lexer = Lexer()
        
        # the constructor for the parser initializes the constructors in 
        # the symbol table.
        # NOTE: you need to keep this in sync with the operators you add to the grammar
        # populate the symbol table with predefined behavior for operator symbols
        # binary
        state.symbol_table.enter_sym('__headtail__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__plus__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__minus__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__times__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__divide__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__or__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__and__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__eq__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__ne__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__le__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__lt__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__ge__', ('constructor', ('arity', 2)))
        state.symbol_table.enter_sym('__gt__', ('constructor', ('arity', 2)))
        # unary
        state.symbol_table.enter_sym('__uminus__', ('constructor', ('arity', 1)))
        state.symbol_table.enter_sym('__not__', ('constructor', ('arity', 1)))


    ###########################################################################################
    def parse(self, input):
        self.lexer.input(input)
        return self.prog()

    ###########################################################################################
    # prog:
    #   stmt_list
    def prog(self):
        dbg_print("parsing PROG")
        sl = self.stmt_list()
        if not self.lexer.EOF():
            raise SyntaxError("Syntax Error: expected 'EOF' found '{}'.".format(
                    self.lexer.peek().type))
        else:
            dbg_print("parsing EOF")
        
        return sl


    ###########################################################################################
    # stmt_list
    #   : LOAD STRING stmt_list
    #   | stmt stmt_list
    def stmt_list(self):
        dbg_print("parsing STMT_LIST")

        if self.lexer.peek().type == 'LOAD':
            # expandd the AST from the file into our current AST
            # using a nested parser object
            self.lexer.match('LOAD')
            str_tok = self.lexer.match('STRING')
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            with open(str_tok.value) as f:
                data = f.read()
                fparser = Parser()
                fstmts = fparser.parse(data)
            sl = self.stmt_list()
            return append_node_list('seq', fstmts, sl)

        elif self.lexer.peek().type in stmt_lookahead:
            s = self.stmt()
            sl = self.stmt_list()
            return ('seq', s, sl)

        else:
            return ('nil',)

    ###########################################################################################
    # NOTE: periods are optional at end of sentences but leaving them out can
    #       lead to ambiguities
    # NOTE: reading will try to cast the read entity in the lowest primitive
    #       datatype with integer < real < string
    # NOTE: should we generalize loops allowing patterns instead of loop index variable?
    # NOTE: the dot is also short hand for the 'noop' command
    # NOTE: in ATTACH the primary should evaluate to a function value
    #
    # stmt
    #    : NOOP
    #    | '.'
    #    | FUNCTION ID body_defs END FUNCTION
    #    | CONSTRUCTOR ID WITH ARITY INTEGER '.'?
    #    | ATTACH primary TO ID '.'?
    #    | DETACH FROM ID '.'?
    #    | LET exp '=' value '.'?
    #    | (GLOBAL | NONLOCAL) var_list '.'?
    #    | (PRINT | PRINTLN) exp (TO FILE ID)? '.'?
    #    | READ ID (FROM FILE ID)? '.'?
    #    | WITH pattern_init_list DO stmt_list END WITH
    #    | FOR ID IN exp DO stmt_list END FOR
    #    | WHILE exp DO stmt_list END WHILE
    #    | REPEAT stmt_list UNTIL exp '.'?
    #    | BREAK
    #    | IF exp DO stmt_list (ELIF exp DO stmt_list)* (ELSE stmt_list)? END IF
    #    | RETURN exp? '.'?
    #    | TRY stmt_list (CATCH pattern DO stmt_list)+ END TRY
    #    | ERROR exp '.'?
    #    | call '.'?
    def stmt(self):
        dbg_print("parsing STMT")
        tt = self.lexer.peek().type  # tt - Token Type

        if tt == 'NOOP':
            dbg_print("parsing NOOP")
            self.lexer.match('NOOP')
            return ('noop',)

        elif tt == '.':
            dbg_print("parsing '.'")
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            return ('noop',)

        elif tt == 'FUNCTION':
            dbg_print("parsing FUNCTION")
            self.lexer.match('FUNCTION')
            id_tok = self.lexer.match('ID')
            body_list = self.body_defs()
            self.lexer.match('END')
            self.lexer.match('FUNCTION')
            # functions are values bound to names
            return ('assign',
                    ('id',id_tok.value),
                    ('function', body_list))

        elif tt == 'CONSTRUCTOR':
            dbg_print("parsing CONSTRUCTOR")
            self.lexer.match('CONSTRUCTOR')
            id_tok = self.lexer.match('ID')
            self.lexer.match('WITH')
            self.lexer.match('ARITY')
            int_tok = self.lexer.match('INTEGER')
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            # constructors are values bound to names
            return ('assign',
                    ('id', id_tok.value),
                    ('constructor', ('arity', int_tok.value)))

        elif tt == 'ATTACH':
            dbg_print("parsing ATTACH")
            self.lexer.match('ATTACH')
            if self.lexer.peek().type == 'ID':
                fid_tok = self.lexer.match('ID')
                self.lexer.match('TO')
                cid_tok = self.lexer.match('ID')
                if self.lexer.peek().type == '.':
                    self.lexer.match('.')
                return ('attach',
                        ('fun-id',fid_tok.value),
                        ('constr-id', cid_tok.value))
            else:
                fconst = self.primary()
                self.lexer.match('TO')
                cid_tok = self.lexer.match('ID')
                if self.lexer.peek().type == '.':
                    self.lexer.match('.')
                return ('attach',
                        ('fun-const',fconst),
                        ('constr-id', cid_tok.value))

        elif tt == 'DETACH':
            dbg_print("parsing DETACH")
            self.lexer.match('DETACH')
            self.lexer.match('FROM')
            fid_tok = self.lexer.match('ID')
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            return ('detach', ('id', fid_tok.value))

        elif tt == 'LET':
            dbg_print("parsing LET")
            self.lexer.match('LET')
            p = self.pattern()
            self.lexer.match('=')
            v = self.value()
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            return ('assign', p, v)

        elif tt == 'GLOBAL' or tt == 'NONLOCAL':
            dbg_print("parsing GLOBAL/NONLOCAL")
            self.lexer.next()
            vl = self.var_list()
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            return (tt.lower(), vl)

        elif tt == 'PRINT' or tt == 'PRINTLN':
            dbg_print("parsing PRINT/PRINTLN")
            self.lexer.next()
            e = self.exp()
            if self.lexer.peek().type == 'TO':
                self.lexer.match('TO')
                self.lexer.match('FILE')
                fname_tok = self.lexer.match('ID')
                if self.lexer.peek().type == '.':
                    self.lexer.match('.')
                return (tt.lower(), e, ('file-name', ('id', fname_tok.value)))
            else:
                if self.lexer.peek().type == '.':
                    self.lexer.match('.')
                return (tt.lower(), e, ('file-name', ('nil',)))

        elif tt == 'READ':
            dbg_print("parsing READ")
            self.lexer.match('READ')
            var_tok = self.lexer.match('ID')
            if self.lexer.peek().type == 'FROM':
                self.lexer.match('FROM')
                self.lexer.match('FILE')
                fname_tok = self.lexer.match('ID')
                if self.lexer.peek().type == '.':
                    self.lexer.match('.')
                return ('read', ('id', var_tok.value), ('file-name', ('id', fname_tok.value)))
            else:
                if self.lexer.peek().type == '.':
                    self.lexer.match('.')
                return ('read', ('id', var_tok.value), ('file-name', ('nil',)))

        elif tt == 'WITH':
            dbg_print("parsing WITH")
            self.lexer.match('WITH')
            pl = self.pattern_init_list()
            self.lexer.match('DO')
            sl = self.stmt_list()
            self.lexer.match('END')
            self.lexer.match('WITH')
            return ('with',
                    ('pattern-list', pl),
                    ('stmt-list', sl))

        elif tt == 'FOR':
            dbg_print("parsing FOR")
            self.lexer.match('FOR')
            var_tok = self.lexer.match('ID')
            self.lexer.match('IN')
            e = self.exp()
            self.lexer.match('DO')
            sl = self.stmt_list()
            self.lexer.match('END')
            self.lexer.match('FOR')
            return ('for',
                    ('id', var_tok.value),
                    ('in-exp', e),
                    ('stmt-list', sl))

        elif tt == 'WHILE':
            dbg_print("parsing WHILE")
            self.lexer.match('WHILE')
            e = self.exp()
            self.lexer.match('DO')
            sl = self.stmt_list()
            self.lexer.match('END')
            self.lexer.match('WHILE')
            return ('while',
                    ('cond-exp', e),
                    ('stmt-list', sl))

        elif tt == 'REPEAT':
            dbg_print("parsing REPEAT")
            self.lexer.match('REPEAT')
            sl = self.stmt_list()
            self.lexer.match('UNTIL')
            e = self.exp()
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            return ('repeat',
                    ('stmt-list', sl),
                    ('until-exp', e))

        elif tt == 'BREAK':
            dbg_print("parsing BREAK")
            self.lexer.match('BREAK')
            return ('break',)

        elif tt == 'IF':
            dbg_print("parsing IF")
            self.lexer.match('IF')
            if_exp = self.exp()
            self.lexer.match('DO')
            then_stmts = self.stmt_list()
            elif_list = ('nil',)
            else_stmts = ('nil',)

            while self.lexer.peek().type == 'ELIF':
                dbg_print("parsing ELIF")
                self.lexer.match('ELIF')
                e = self.exp()
                self.lexer.match('DO')
                sl = self.stmt_list()
                elif_list = ('seq',
                             ('elif',
                              ('cond-exp', e),
                              ('stmt-list', sl)),
                             elif_list)

            if self.lexer.peek().type == 'ELSE':
                dbg_print("parsing ELSE")
                self.lexer.match('ELSE')
                else_stmts = self.stmt_list()
            self.lexer.match('END')
            self.lexer.match('IF')
            # NOTE: elif_list is reversed
            return ('if',
                    ('cond-exp', if_exp),
                    ('then-stmt-list', then_stmts),
                    ('elif-list', reverse_node_list('seq', elif_list)),
                    ('else-stmt-list', else_stmts))

        elif tt == 'RETURN':
            dbg_print("parsing RETURN")
            self.lexer.match('RETURN')
            if self.lexer.peek().type in exp_lookahead:
                e = self.exp()
                if self.lexer.peek().type == '.':
                    self.lexer.match('.')
                return ('return', e)
            else:
                if self.lexer.peek().type == '.':
                    self.lexer.match('.')
                return ('return', ('nil',))

        elif tt == 'TRY':
            dbg_print("parsing TRY")
            self.lexer.match('TRY')
            try_block = self.stmt_list()
            catch_list = ('nil',)
            self.lexer.match('CATCH')
            dbg_print("parsing CATCH")
            catch_pattern = self.pattern()
            self.lexer.match('DO')
            catch_stmts = self.stmt_list()
            catch_list = ('seq',
                          ('catch',
                           ('pattern', catch_pattern),
                           ('stmt-list', catch_stmts)),
                          catch_list)

            while lexer.peek().type == 'CATCH':
                dbg_print("parsing CATCH")
                self.lexer.match('CATCH')
                catch_pattern = self.pattern()
                self.lexer.match('DO')
                catch_stmts = self.stmt_list()
                catch_list = ('seq',
                              ('catch',
                               ('pattern', catch_pattern),
                               ('stmt-list', catch_stmts)),
                              catch_list)
            self.lexer.match('END')
            self.lexer.match('TRY')
            # NOTE: catch_list is in reverse!
            return ('try',
                    ('stmt-list', try_block)
                    ('catch-list', reverse_node_list('seq', catch_list)))

        elif tt == 'ERROR':
            dbg_print("parsing ERROR")
            self.lexer.match('ERROR')
            e = self.exp()
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            return ('error', e)

        else:
            v = self.call()
            if self.lexer.peek().type == '.':
                self.lexer.match('.')
            return v

    ###########################################################################################
    # body_defs
    #   : WITH pattern DO stmt_list (ORWITH pattern DO stmt_list)*
    def body_defs(self):
        dbg_print("parsing BODY_DEFS")
        self.lexer.match('WITH')
        p = self.pattern()
        self.lexer.match('DO')
        sl = self.stmt_list()
        body_list = ('seq',
                     ('body',
                      ('pattern', p),
                      ('stmt-list', sl)),
                     ('nil',))

        while self.lexer.peek().type == 'ORWITH':
            self.lexer.match('ORWITH')
            p = self.pattern()
            self.lexer.match('DO')
            sl = self.stmt_list()
            body_list = ('seq',
                         ('body',
                          ('pattern', p),
                          ('stmt-list', sl)),
                         body_list)

        return ('body-list', reverse_node_list('seq', body_list))

    ###########################################################################################
    # var_list
    #   : ID (',' ID)*
    def var_list(self):
        dbg_print("parsing VAR_LIST")
        id_tok = self.lexer.match('ID')
        vlist = ('seq', id_tok.value, ('nil',))

        while self.lexer.peek().type == ',':
            self.lexer.match(',')
            id_tok = self.lexer.match('ID')
            vlist = ('seq', id_tok.value, vlist)

        # NOTE: vlist is reversed
        return reverse_node_list('seq', vlist)

    ###########################################################################################
    # pattern_init_list
    #    : pattern initializer? (',' pattern initializer?)*
    def pattern_init_list(self):
        dbg_print("parsing PATTERN_INIT_LIST")
        p = self.pattern()

        if self.lexer.peek().type == '=':
            ini = self.initializer()
            v = ('seq', ('assign', p, ini), ('nil',))
        else:
            v = ('seq', ('assign', p, ('nil',)), ('nil',))

        while self.lexer.peek().type == ',':
            self.lexer.match(',')
            p = self.pattern()
            if self.lexer.peek().type == '=':
                ini = self.initializer()
                v = ('seq', ('assign', p, ini), v)
            else:
                v = ('seq', ('assign', p, ('nil',)), v)
        
        return reverse_node_list('seq', v)

    ###########################################################################################
    # initializer
    #    : '=' quote_exp
    def initializer(self):
        dbg_print("parsing INITIALIZER")
        self.lexer.match('=')
        v = self.quote_exp() # cannot be a "raw list"
        return v

    ###########################################################################################
    # pattern
    #    : exp
    def pattern(self):
        dbg_print("parsing PATTERN")
        e = self.exp()
        return e

    ###########################################################################################
    # value
    #   : exp
    def value(self):
        dbg_print("parsing VALUE")
        e = self.exp()
        return e

    ###########################################################################################
    # exp
    #    : quote_exp (',' quote_exp)*
    def exp(self):
        dbg_print("parsing LIST_EXP")
        v = self.quote_exp()

        if self.lexer.peek().type == ',':
            vlist = ('list', [v])
            while self.lexer.peek().type == ',':
                self.lexer.match(',')
                e = self.quote_exp()
                vlist[1].append(e)
            return vlist
        else:
            return v

    ###########################################################################################
    # quote_exp
    #    : QUOTE head_tail
    #    | head_tail
    def quote_exp(self):
        if self.lexer.peek().type == 'QUOTE':
            self.lexer.match('QUOTE')
            v = self.head_tail()
            return ('quote', v)
        else:
            v = self.head_tail()
            return v

    ###########################################################################################
    # head_tail
    #    : rel_exp0 ('|' exp)?
    def head_tail(self):
        dbg_print("parsing HEAD_TAIL")
        v = self.rel_exp0()
        if self.lexer.peek().type == '|':
            self.lexer.match('|')
            tail = self.exp()
            op_sym = '__headtail__'
            v = ('apply', 
                 ('id', op_sym),
                 ('apply',
                  ('list', [v, tail]),
                  ('nil',)))
        return v

    ###########################################################################################
    # NOTE: all terms are expressed as apply nodes of their corresponding constructor names
    ###########################################################################################
    # relational operators with their precedence
    # rel_exp0
    #    : rel_exp1 (OR rel_exp1)*
    #
    # rel_exp1
    #    : rel_exp2 (AND rel_exp2)*
    #
    # rel_exp2
    #    : rel_exp3 (('==' | '><' /* not equal */) rel_exp3)*
    #
    # rel_exp3
    #    : arith_exp0 (('LE' | 'LT'  | 'GE' | 'GT') arith_exp0)*
    def rel_exp0(self):
        dbg_print("parsing REL_EXP")
        v = self.rel_exp1()
        while self.lexer.peek().type == 'OR':
            self.lexer.match('OR')
            v2 = self.rel_exp1()
            op_sym = '__or__'
            v = ('apply', 
                 ('id', op_sym),
                 ('apply',
                  ('list', [v, v2]),
                  ('nil',)))
        return v

    def rel_exp1(self):
        v = self.rel_exp2()
        while self.lexer.peek().type == 'AND':
            self.lexer.match('AND')
            v2 = self.rel_exp2()
            op_sym = '__and__'
            v = ('apply', 
                 ('id', op_sym),
                 ('apply',
                  ('list', [v, v2]),
                  ('nil',)))
        return v

    def rel_exp2(self):
        v = self.rel_exp3()
        while self.lexer.peek().type in ['EQ', 'NE']:
            op_tok = self.lexer.peek()
            self.lexer.next()
            v2 = self.rel_exp3()
            op_sym = '__' + op_tok.type.lower() + '__'
            v = ('apply', 
                 ('id', op_sym),
                 ('apply',
                  ('list', [v, v2]),
                  ('nil',)))
        return v

    def rel_exp3(self):
        v = self.arith_exp0()
        while self.lexer.peek().type in ['LE', 'LT', 'GE', 'GT']:
            op_tok = self.lexer.peek()
            self.lexer.next()
            v2 = self.arith_exp0()
            op_sym = '__' + op_tok.type.lower() + '__'
            v = ('apply', 
                 ('id', op_sym),
                 ('apply',
                  ('list', [v, v2]),
                  ('nil',)))
        return v

    ###########################################################################################
    # arithmetic operators with their precedence
    # arith_exp0
    #    : arith_exp1 ((PLUS | MINUS) arith_exp1)*
    #
    # arith_exp1
    #    : conditional ((TIMES | DIVIDE) conditional)*
    def arith_exp0(self):
        dbg_print("parsing ARITH_EXP")
        v = self.arith_exp1()
        while self.lexer.peek().type in ['PLUS', 'MINUS']:
            op_tok = self.lexer.peek()
            self.lexer.next()
            v2 = self.arith_exp1()
            op_sym = '__' + op_tok.type.lower() + '__'
            v = ('apply', 
                 ('id', op_sym),
                 ('apply',
                  ('list', [v, v2]),
                  ('nil',)))
        return v

    def arith_exp1(self):
        v = self.conditional()
        while self.lexer.peek().type in ['TIMES', 'DIVIDE']:
            op_tok = self.lexer.peek()
            self.lexer.next()
            v2 = self.conditional()
            op_sym = '__' + op_tok.type.lower() + '__'
            v = ('apply', 
                 ('id', op_sym),
                 ('apply',
                  ('list', [v, v2]),
                  ('nil',)))
        return v

    ###########################################################################################
    # conditional
    #    : compound
    #        (
    #           (OTHERWISE exp) |
    #           (WHENEVER exp OTHERWISE exp)
    #        )?
    def conditional(self):
        dbg_print("parsing COONDITIONAL")
        v = self.compound()
        tt = self.lexer.peek().type
        if tt  == 'OTHERWISE':
            self.lexer.match('OTHERWISE')
            v2 = self.exp()
            return ('otherwise', v, v2)
        elif tt == 'WHENEVER':
            self.lexer.match('WHENEVER')
            v2 = self.exp()
            self.lexer.match('OTHERWISE')
            v2 = self.exp()
            return ('whenever', v, v2, v3)
        else:
            return v

    ###########################################################################################
    # compound
    #    : call
    #        (
    #           (IS pattern) |
    #           (IN exp) | // exp has to be a list
    #           (TO exp (STEP exp)?) | // list comprehension
    #           (WHERE ID IN exp) |    // list comprehension
    #           ('@' call)+
    #        )?
    def compound(self):
        dbg_print("parsing COMPOUND")
        v = self.call()
        tt = self.lexer.peek().type
        if tt == 'IS':
            self.lexer.match('IS')
            v2 = self.pattern()
            return ('is', v, v2)

        elif tt == 'IN':
            self.lexer.match('IN')
            v2 = self.exp()
            return ('in', v, v2)

        elif tt == 'TO':
            self.lexer.match('TO')
            v2 = self.exp()
            if self.lexer.peek().type == 'STEP':
                self.lexer.match('STEP')
                v3 = self.exp()
                return ('list-to',
                        ('start', v),
                        ('stop', v2),
                        ('step', v3))
            else:
                return ('list-to',
                        ('start', v),
                        ('stop', v2),
                        ('step', ('nil',)))

        elif tt == 'WHERE':
            self.lexer.match('WHERE')
            id_tok = self.lexer.match('ID')
            self.lexer.match('IN')
            v2 = self.exp()
            return ('list-where', # list comprehension
                    ('comp-exp', v),
                    ('id', id_tok.value),
                    ('in-exp', v2))

        elif tt == '@':
            self.lexer.match('@')
            ix_val = self.call()
            # place scalar index values in a list for easier processing
            if ix_val[0] == 'list':
                v2 = ('index', ix_val, ('nil',))
            else:
                v2 = ('index', ('list', [ix_val]), ('nil',))

            while self.lexer.peek().type == '@':
                self.lexer.match('@')
                ix_val = self.call()
                # place scalar index values in a list for easier processing
                if ix_val[0] == 'list':
                    v2 = ('index', ix_val, v2)
                else:
                    v2 = ('index', ('list', [ix_val]), v2)
                    

            return ('structure-ix', v, reverse_node_list('index', v2))

        else:
            return v

    ###########################################################################################
    # function/constructor call 
    #
    # call
    #    : primary primary*
    def call(self):
        dbg_print("parsing CALL")
        v = self.primary()
        if self.lexer.peek().type in exp_lookahead:
            v = ('apply', v, ('nil',))
            while self.lexer.peek().type in exp_lookahead:
                v2 = self.primary()
                v = ('apply', v2, v) 
            return reverse_node_list('apply', v)
        else:
            return v

    ###########################################################################################
    # NOTE: in HASATTACH the primary should evaluate to a ID
    # NOTE: in EVAL the primary should evaluate to a string
    # NOTE: EVAL allows the user to patch python code into the interpreter and therefore
    #       is able to create custom extension to the interpreter
    #
    # primary
    #    : INTEGER
    #    | REAL
    #    | STRING
    #    | TRUE
    #    | FALSE
    #    | NONE
    #    | ID
    #    | '*' ID  // "dereference" a variable during pattern matching
    #    | NOT rel_exp0
    #    | MINUS arith_exp0
    #    | HASATTACH primary
    #    | ESCAPE STRING
    #    | '(' exp? ')'
    #    | '[' exp? ']' // list or list access
    #    | '{' exp '}' // dictionary access, should this be just ID/STRING?
    #    | function_const
    def primary(self):
        dbg_print("parsing PRIMARY")
        tt = self.lexer.peek().type

        if tt == 'INTEGER':
            tok = self.lexer.match('INTEGER')
            return ('integer', tok.value)

        elif tt == 'REAL':
            tok = self.lexer.match('REAL')
            return ('real', tok.value)

        elif tt == 'STRING':
            tok = self.lexer.match('STRING')
            return ('string', tok.value)

        elif tt == 'TRUE':
            self.lexer.match('TRUE')
            return ('boolean', 'true')

        elif tt == 'FALSE':
            self.lexer.match('FALSE')
            return ('boolean', 'false')

        elif tt == 'NONE':
           self.lexer.match('NONE')
           return ('none',)

        elif tt == 'ID':
            tok = self.lexer.match('ID')
            return ('id', tok.value)

        elif tt == 'TIMES':
            self.lexer.match('TIMES')
            id_tok = self.lexer.match('ID')
            return ('deref', id_tok.value)

        elif tt == 'NOT':
            self.lexer.match('NOT')
            v = self.rel_exp0()
            v = ('apply', 
                 ('id', '__not__'),
                 ('apply',
                  v,
                  ('nil',)))
            return v

        elif tt == 'MINUS':
            self.lexer.match('MINUS')
            v = self.arith_exp0()
            v = ('apply', 
                 ('id', '__uminus__'),
                 ('apply',
                  v,
                  ('nil',)))
            return v

        elif tt == 'ESCAPE':
            self.lexer.match('ESCAPE')
            str_tok = self.lexer.match('STRING')
            return ('escape', str_tok.value)

        elif tt == '(':
            self.lexer.match('(')
            if self.lexer.peek().type in exp_lookahead:
                v = self.exp()
            else:
                v = ('list', [])
            self.lexer.match(')')
            return v

        elif tt == '[':
            self.lexer.match('[')
            if self.lexer.peek().type in exp_lookahead:
                v = self.exp()
                if v[0] != 'list':
                    v = ('list', [v])
            else:
                v = ('list', [])
            self.lexer.match(']')
            return v

        elif tt == '{':
            self.lexer.match('{')
            v = self.exp()
            self.lexer.match('}')
            return ('dict_access', v)

        elif tt == 'LAMBDA':
            return self.function_const()

        else:
            raise SyntaxError("Syntax Error:{}: at '{}'".format(
                    self.lexer.peek().lineno,
                    self.lexer.peek().value))

    ###########################################################################################
    # function_const
    #    : LAMBDA body_defs 
    def function_const(self):
        dbg_print("parsing FUNCTION_CONST")
        self.lexer.match('LAMBDA')
        body_list = self.body_defs()

        return ('function', body_list)

###########################################################################################
### test the parser
if __name__ == "__main__":

    test = \
'''
let x = y[1]{"foo"}.
'''

    parser = Parser()
    parser.parse(test)


