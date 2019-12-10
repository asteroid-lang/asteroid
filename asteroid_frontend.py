###########################################################################################
# front end for Asteroid
#
# (c) Lutz Hamel, University of Rhode Island
###########################################################################################

import sys
from pathlib import Path, PurePath
from asteroid_lex import Lexer
from asteroid_state import state

###########################################################################################
# this is used to compute the filename extensions of the modules
asteroid_file_suffix = ".ast"

###########################################################################################
def dbg_print(string):
    #print(string)
    pass

###########################################################################################
# LL(1) lookahead sets

exp_lookahead_no_ops = [
    'INTEGER',
    'REAL',
    'STRING',
    'TRUE',
    'FALSE',
    'NONE',
    'ID',
    '{',
    '[',
    '(',
    ]

exp_lookahead = exp_lookahead_no_ops + [
                 'ESCAPE',
                 #'TIMES',
                 'MINUS',
                 'NOT',
                 'LAMBDA',
                 'QUOTE',
                 ]

stmt_lookahead = [
    '.',
    'ATTACH',
    'BREAK',
    'CLASS',
    'CONSTRUCTOR',
    'DETACH',
    'FOR',
    'FUNCTION',
    'GLOBAL',
    'IF',
    'LET',
    'NONLOCAL',
    'NOOP',
    'REPEAT',
    'RETURN',
    'THROW'
    'TRY',
    'WHILE',
    'WITH',
    ] + exp_lookahead

class_stmt_lookahead = [
    '.',
    'DATA',
    'FUNCTION',
    'NOOP',
    ]

###########################################################################################
class Parser:

    def __init__(self, filename="<input>"):
        self.lexer = Lexer()
        state.lineinfo = (filename,0)

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
            raise SyntaxError("Syntax Error: expected 'EOF' found '{}'." \
                              .format(self.lexer.peek().type))
        else:
            dbg_print("parsing EOF")
        return sl

    ###########################################################################################
    # stmt_list
    #   : stmt stmt_list
    #   | empty
    def stmt_list(self):
        dbg_print("parsing STMT_LIST")

        if self.lexer.peek().type in stmt_lookahead:
            lineinfo = ('lineinfo', state.lineinfo)
            s = self.stmt()
            (LIST, sl) = self.stmt_list()
            return ('list', [lineinfo] + [s] +  sl)

        else:
            return ('list', [])

    ###########################################################################################
    # NOTE: periods are optional at end of sentences but leaving them out can
    #       lead to ambiguities
    # NOTE: the dot is also short hand for the 'noop' command
    # NOTE: in ATTACH the primary should evaluate to a function constructor
    #
    # stmt
    #    : NOOP
    #    | '.'
    #    | LOAD STRING '.'?
    #    | GLOBAL id_list '.'?
    #    | NONLOCAL id_list '.'?
    #    | function_def
    #    | CLASS ID WITH class_stmt_list END CLASS
    #    | CONSTRUCTOR ID WITH ARITY INTEGER '.'?
    #    | ATTACH primary TO ID '.'?
    #    | DETACH FROM ID '.'?
    #    | LET pattern '=' exp '.'?
    #    | FOR pattern IN exp DO stmt_list END FOR
    #    | WHILE exp DO stmt_list END WHILE
    #    | REPEAT (DO?) stmt_list UNTIL exp '.'?
    #    | BREAK
    #    | IF exp DO stmt_list (ELIF exp DO stmt_list)* (ELSE (DO?) stmt_list)? END IF
    #    | RETURN exp? '.'?
    #    | TRY stmt_list (CATCH pattern DO stmt_list)+ END TRY
    #    | THROW exp '.'?
    #    | call '.'?
    def stmt(self):
        dbg_print("parsing STMT")
        tt = self.lexer.peek().type  # tt - Token Type

        if tt in ['NOOP', '.']:
            return self.noop_stmt()

        elif tt == 'LOAD':
            # expand the AST from the file into our current AST
            # using a nested parser object
            self.lexer.match('LOAD')
            str_tok = self.lexer.match('STRING')
            self.lexer.match_optional('.')

            raw_pp = PurePath(str_tok.value)
            module_name = raw_pp.stem

            # if module is on the list of modules then we have loaded
            # it already -- ignore -- continue parsing the program file
            if module_name in state.modules:
                # lhh
                # print("Ignoring module {}".format(module_name))
                return self.stmt_list()

            # search for module file:
            # 0. raw module name - could be an absolute path
            # 1. search in current directory (path[1])
            # 2. search in directory where Asteroid is installed (path[0])
            # 3. search in subdirectory where Asteroid was started
            # lhh: does this work on all OS's?
            search_list = []
            search_list.append(str_tok.value)
            search_list.append(str_tok.value + asteroid_file_suffix)
            search_list.append(sys.path[1] + '/' + module_name + asteroid_file_suffix)
            search_list.append(sys.path[0] + '/modules/' + module_name + asteroid_file_suffix)
            search_list.append('modules/' + module_name + asteroid_file_suffix)

            file_found = False

            for ix in range(len(search_list)):
                ast_module_file = search_list[ix]
                #lhh
                #print("AST module: {}".format(ast_module_file))
                ast_module_path = Path(ast_module_file)
                if ast_module_path.is_file():
                    file_found = True
                    break

            if not file_found:
                raise ValueError("Asteroid module {} not found"
                                 .format(str_tok.value))

            #lhh
            #print("opening module {}".format(ast_module_file))

            old_lineinfo = state.lineinfo

            with open(ast_module_file) as f:
                state.modules.append(module_name)
                data = f.read()
                fparser = Parser(module_name)
                (STMT_LIST, fstmts) = fparser.parse(data)

            state.lineinfo = old_lineinfo
            (LIST, sl) = self.stmt_list()
            return ('list', fstmts + sl)

        elif tt == 'GLOBAL':
            dbg_print("parsing GLOBAL")
            self.lexer.match('GLOBAL')
            id_list = self.id_list()
            self.lexer.match_optional('.')
            return ('global', id_list)

        elif tt == 'NONLOCAL':
            dbg_print("parsing NONLOCAL")
            self.lexer.match('NONLOCAL')
            id_list = self.id_list()
            self.lexer.match_optional('.')
            return ('nonlocal', id_list)

        elif tt == 'FUNCTION':
            return self.function_def()

        elif tt == 'CLASS':
            dbg_print("parsing CLASS")
            self.lexer.match('CLASS')
            id_tok = self.lexer.match('ID')
            self.lexer.match('WITH')
            stmts = self.class_stmt_list()
            self.lexer.match('END')
            self.lexer.match('CLASS')
            return ('class-def',
                    ('id', id_tok.value),
                    ('stmt-list', stmts))

        elif tt == 'CONSTRUCTOR':
            dbg_print("parsing CONSTRUCTOR")
            self.lexer.match('CONSTRUCTOR')
            id_tok = self.lexer.match('ID')
            self.lexer.match('WITH')
            self.lexer.match('ARITY')
            int_tok = self.lexer.match('INTEGER')
            self.lexer.match_optional('.')
            # constructors are values bound to names
            return ('unify',
                    ('id', id_tok.value),
                    ('constructor', ('arity', int_tok.value)))

        elif tt == 'ATTACH':
            dbg_print("parsing ATTACH")
            self.lexer.match('ATTACH')
            fexp = self.primary()
            self.lexer.match('TO')
            cid_tok = self.lexer.match('ID')
            self.lexer.match_optional('.')
            return ('attach',
                    ('fun-exp', fexp),
                    ('constr-id', cid_tok.value))

        elif tt == 'DETACH':
            dbg_print("parsing DETACH")
            self.lexer.match('DETACH')
            self.lexer.match('FROM')
            fid_tok = self.lexer.match('ID')
            self.lexer.match_optional('.')
            return ('detach', ('id', fid_tok.value))

        elif tt == 'LET':
            dbg_print("parsing LET")
            self.lexer.match('LET')
            p = self.pattern()
            self.lexer.match('=')
            v = self.exp()
            self.lexer.match_optional('.')
            return ('unify', p, v)

        elif tt == 'FOR':
            dbg_print("parsing FOR")
            self.lexer.match('FOR')
            e = self.exp()
            if e[0] != 'in':
                raise ValueError("syntax error: expected 'in' expression in for loop")
            self.lexer.match('DO')
            sl = self.stmt_list()
            self.lexer.match('END')
            self.lexer.match('FOR')
            return ('for',
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
            if self.lexer.peek().type == 'DO':
                self.lexer.match('DO')
            sl = self.stmt_list()
            self.lexer.match('UNTIL')
            e = self.exp()
            self.lexer.match_optional('.')
            return ('repeat',
                    ('stmt-list', sl),
                    ('until-exp', e))

        elif tt == 'BREAK':
            dbg_print("parsing BREAK")
            self.lexer.match('BREAK')
            return ('break',)

        elif tt == 'IF':
            # if statements are coded as a list of ('if-clause', condition, stmts)
            if_list = []

            dbg_print("parsing IF")
            self.lexer.match('IF')
            cond = self.exp()
            self.lexer.match('DO')
            stmts = self.stmt_list()
            if_list.append(('if-clause', ('cond', cond), ('stmt-list', stmts)))

            while self.lexer.peek().type == 'ELIF':
                dbg_print("parsing ELIF")
                self.lexer.match('ELIF')
                cond = self.exp()
                self.lexer.match('DO')
                stmts = self.stmt_list()
                if_list.append(('if-clause', ('cond', cond), ('stmt-list', stmts)))

            if self.lexer.peek().type == 'ELSE':
                dbg_print("parsing ELSE")
                self.lexer.match('ELSE')
                self.lexer.match_optional('DO')
                stmts = self.stmt_list()
                # make the else look like another elif with the condition set to 'true'
                if_list.append(('if-clause', ('cond', ('boolean', True)), ('stmt-list', stmts)))

            self.lexer.match('END')
            self.lexer.match('IF')
            return ('if', ('list', if_list))


        elif tt == 'RETURN':
            dbg_print("parsing RETURN")
            self.lexer.match('RETURN')
            if self.lexer.peek().type in exp_lookahead:
                e = self.exp()
                self.lexer.match_optional('.')
                return ('return', e)
            else:
                self.lexer.match_optional('.')
                return ('return', ('none', None))

        elif tt == 'TRY':
            dbg_print("parsing TRY")

            # the catch list is a list of ('catch', pattern, stmts)
            catch_list = []

            self.lexer.match('TRY')
            try_stmts = self.stmt_list()
            self.lexer.match('CATCH')
            dbg_print("parsing CATCH")
            pattern = self.pattern()
            self.lexer.match('DO')
            stmts = self.stmt_list()
            catch_list.append(('catch', ('pattern', pattern), ('stmt-list', stmts)))

            while self.lexer.peek().type == 'CATCH':
                dbg_print("parsing CATCH")
                self.lexer.match('CATCH')
                pattern = self.pattern()
                self.lexer.match('DO')
                stmts = self.stmt_list()
                catch_list.append(('catch',('pattern', pattern), ('stmt-list', stmts)))

            self.lexer.match('END')
            self.lexer.match('TRY')

            return ('try',
                    ('stmt-list', try_stmts),
                    ('catch-list', ('list', catch_list)))

        elif tt == 'THROW':
            dbg_print("parsing THROW")
            self.lexer.match('THROW')
            e = self.exp()
            self.lexer.match_optional('.')
            return ('throw', e)

        else:
            v = self.call()
            self.lexer.match_optional('.')
            return v

    ###########################################################################################
    # function_def
    #  : FUNCTION ID body_defs END FUNCTION
    def function_def(self):
        dbg_print("parsing FUNCTION_DEF")
        self.lexer.match('FUNCTION')
        id_tok = self.lexer.match('ID')
        body_list = self.body_defs()
        self.lexer.match('END')
        self.lexer.match('FUNCTION')
        # functions are values bound to names
        return ('unify',
                ('id',id_tok.value),
                ('function', body_list))

    ###########################################################################################
    # class_stmt
    #  : function_def
    #  | DATA ID '.'?
    #  | noop_stmt
    def class_stmt(self):
        dbg_print("parsing CLASS_STMT")
        tt = self.lexer.peek().type  # tt - Token Type

        if tt == 'FUNCTION':
            return self.function_def()

        elif tt == 'DATA':
            self.lexer.match('DATA')
            id_tok = self.lexer.match('ID')
            self.lexer.match_optional('.')
            return ('data', ('id', id_tok.value))
        else:
            return self.noop_stmt()

    ###########################################################################################
    # class_stmt_list
    #   : class_stmt class_stmt_list
    #   | empty
    def class_stmt_list(self):
        dbg_print("parsing CLASS_STMT_LIST")

        if self.lexer.peek().type in class_stmt_lookahead:
            lineinfo = ('lineinfo', state.lineinfo)
            s = self.class_stmt()
            (LIST, sl) = self.class_stmt_list()
            return ('list', [lineinfo] + [s] +  sl)
        else:
            return ('list', [])

    ###########################################################################################
    # noop_stmt
    #  : NOOP '.'?
    #  | '.'
    def noop_stmt(self):
        dbg_print("parsing NOOP_STMT")
        if self.lexer.peek().type == 'NOOP':
            self.lexer.match('NOOP')
            self.lexer.match_optional('.')
        else:
            self.lexer.match('.')
        return ('noop',)

    ###########################################################################################
    # id_list
    #   : ID (',' ID)*
    def id_list(self):
        dbg_print("parsing ID_LIST")

        id_list = []

        id_tok = self.lexer.match('ID')
        id_list.append(('id', id_tok.value))
        while self.lexer.peek().type == ',':
            self.lexer.match(',')
            id_tok = self.lexer.match('ID')
            id_list.append(('id', id_tok.value))
        return ('list', id_list)

    ###########################################################################################
    # body_defs
    #   : WITH pattern DO stmt_list (ORWITH pattern DO stmt_list)*
    def body_defs(self):
        dbg_print("parsing BODY_DEFS")

        # a list of ('body', pattern, stmts) pairs
        body_list = []

        self.lexer.match('WITH')
        p = self.pattern()
        self.lexer.match('DO')
        sl = self.stmt_list()
        body_list.append(('body', ('pattern', p), ('stmt-list', sl)))

        while self.lexer.peek().type == 'ORWITH':
            self.lexer.match('ORWITH')
            p = self.pattern()
            self.lexer.match('DO')
            sl = self.stmt_list()
            body_list.append(('body', ('pattern', p), ('stmt-list', sl)))

        return ('body-list', ('list', body_list))

    ###########################################################################################
    # pattern
    #    : exp
    def pattern(self):
        dbg_print("parsing PATTERN")
        e = self.exp()
        return e

    ###########################################################################################
    # exp
    #    : quote_exp (',' quote_exp?)*
    #
    # NOTE: trailing comma means single element list!
    # NOTE: raw-list nodes are list nodes that were constructed with just the comma constructor
    #       they should work just like list nodes in the context of interpretation
    #
    def exp(self):
        dbg_print("parsing EXP")

        v = self.quote_exp()

        if self.lexer.peek().type == ',':
            raw_list = [v]
            while self.lexer.peek().type == ',':
                self.lexer.match(',')
                if self.lexer.peek().type in exp_lookahead:
                    e = self.quote_exp()
                    raw_list.append(e)
            return ('raw-list', raw_list)

        else:
            return v

    ###########################################################################################
    # quote_exp
    #    : QUOTE head_tail
    #    | head_tail
    def quote_exp(self):
        dbg_print("parsing QUOTE_EXP")

        if self.lexer.peek().type == 'QUOTE':
            self.lexer.match('QUOTE')
            v = self.head_tail()
            return ('quote', v)
        else:
            v = self.head_tail()
            return v

    ###########################################################################################
    # head_tail
    #    : compound ('|' exp)?
    #
    # NOTE: * as a value this operator will construct a list from the semantic values of
    #         head and tail
    #       * as a pattern this operator will be unified with a list such that head will
    #         unify with the first element of the list and tail with the remaining list
    # NOTE: this is a list constructor and therefore should never appear in the semantic
    #       processing, use walk to expand the list before processing it.
    def head_tail(self):
        dbg_print("parsing HEAD_TAIL")

        v = self.compound()

        if self.lexer.peek().type == '|':
            self.lexer.match('|')
            head = v
            tail = self.exp()
            v = ('head-tail', head, tail)

        return v

    ###########################################################################################
    # compound
    #    : rel_exp0
    #        (
    #           (IS pattern) |
    #           (IN exp) | // exp has to be a list
    #           (TO exp (STEP exp)?) | // list comprehension
    #           (WHERE pattern IN exp)     // list comprehension
    #        )?
    def compound(self):
        dbg_print("parsing COMPOUND")

        v = self.rel_exp0()

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
                return ('raw-to-list',
                        ('start', v),
                        ('stop', v2),
                        ('step', v3))
            else:
                return ('raw-to-list',
                        ('start', v),
                        ('stop', v2),
                        ('step', ('integer', '1')))

        elif tt == 'WHERE':
            self.lexer.match('WHERE')
            in_exp = self.exp()
            if in_exp[0] != 'in':
                raise ValueError("syntax error: expected 'in' got '{}'".format(in_exp[0]))
#            self.lexer.match('IN')
#            v2 = self.exp()
            return ('raw-where-list', # list comprehension
                    ('comp-exp', v),
                    ('in-exp', in_exp))

        else:
            return v

    ###########################################################################################
    # NOTE: all terms are expressed as apply-list nodes of their corresponding constructor names
    ###########################################################################################
    # relational operators with their precedence
    # rel_exp0
    #    : rel_exp1 (OR rel_exp1)*
    #
    # rel_exp1
    #    : rel_exp2 (AND rel_exp2)*
    #
    # rel_exp2
    #    : rel_exp3 (('==' | '=/=' /* not equal */) rel_exp3)*
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
            v = ('apply-list', ('list',[('id', op_sym), ('list', [v, v2])]))
        return v

    def rel_exp1(self):
        v = self.rel_exp2()
        while self.lexer.peek().type == 'AND':
            self.lexer.match('AND')
            v2 = self.rel_exp2()
            op_sym = '__and__'
            v = ('apply-list', ('list',[('id', op_sym), ('list', [v, v2])]))
        return v

    def rel_exp2(self):
        v = self.rel_exp3()
        while self.lexer.peek().type in ['EQ', 'NE']:
            op_tok = self.lexer.peek()
            self.lexer.next()
            v2 = self.rel_exp3()
            op_sym = '__' + op_tok.type.lower() + '__'
            v = ('apply-list', ('list',[('id', op_sym), ('list', [v, v2])]))
        return v

    def rel_exp3(self):
        v = self.arith_exp0()
        while self.lexer.peek().type in ['LE', 'LT', 'GE', 'GT']:
            op_tok = self.lexer.peek()
            self.lexer.next()
            v2 = self.arith_exp0()
            op_sym = '__' + op_tok.type.lower() + '__'
            v = ('apply-list', ('list',[('id', op_sym), ('list', [v, v2])]))
        return v

    ###########################################################################################
    # arithmetic operators with their precedence
    # arith_exp0
    #    : arith_exp1 ((PLUS | MINUS) arith_exp1)*
    #
    # arith_exp1
    #    : call ((TIMES | DIVIDE) call)*
    def arith_exp0(self):
        dbg_print("parsing ARITH_EXP")
        v = self.arith_exp1()
        while self.lexer.peek().type in ['PLUS', 'MINUS']:
            op_tok = self.lexer.peek()
            self.lexer.next()
            v2 = self.arith_exp1()
            op_sym = '__' + op_tok.type.lower() + '__'
            v = ('apply-list', ('list',[('id', op_sym), ('list', [v, v2])]))
        return v

    def arith_exp1(self):
        v = self.call()
        while self.lexer.peek().type in ['TIMES', 'DIVIDE']:
            op_tok = self.lexer.peek()
            self.lexer.next()
            v2 = self.call()
            op_sym = '__' + op_tok.type.lower() + '__'
            v = ('apply-list', ('list',[('id', op_sym), ('list', [v, v2])]))
        return v

    ###########################################################################################
    # conditional
    #    : call
    #        (
    #           (OTHERWISE exp) |
    #           (IF exp (ELSE exp)?) # expression level if-else
    #        )?
    def conditional(self):
        dbg_print("parsing CONDITIONAL")

        v = self.call()

        tt = self.lexer.peek().type
        if tt  == 'OTHERWISE':
            self.lexer.match('OTHERWISE')
            v2 = self.exp()
            return ('otherwise', v, v2)

        elif tt == 'IF':
            self.lexer.match('IF')
            v2 = self.exp()
            self.lexer.match('ELSE')
            v3 = self.exp()
            return ('if-exp', v2, v, v3) # mapping it into standard if-then-else format

        else:
            return v

    ###########################################################################################
    # function/constructor call
    #
    # call
    #    : index index*
    def call(self):
        dbg_print("parsing CALL")

        v = self.index()

        if self.lexer.peek().type in exp_lookahead_no_ops:
            apply_list = [v]
            while self.lexer.peek().type in exp_lookahead_no_ops:
                v2 = self.index()
                apply_list.append(v2)
            return ('apply-list', ('list', apply_list))
        else:
            return v

    ###########################################################################################
    # index
    #    : primary ('@' primary)*
    def index(self):
        dbg_print("parsing INDEX")
        v = self.primary()

        if self.lexer.peek().type == '@':
            self.lexer.match('@')
            ix_val = self.primary()
            index_list = [('index', ix_val)]

            while self.lexer.peek().type == '@':
                self.lexer.match('@')
                ix_val = self.primary()
                index_list.append(('index', ix_val))

            return ('structure-ix', v, ('index-list', ('list', index_list)))

        else:
            return v

    ###########################################################################################
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
    #    | NOT primary
    #    | MINUS primary
    #    | ESCAPE STRING
    #    | '(' exp ')'  // see notes below on exp vs list
    #    | '[' exp? ']' // list or list access
    #    | '{' exp '}'  // exp should only produce integer and string typed expressions
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
            return ('boolean', True)

        elif tt == 'FALSE':
            self.lexer.match('FALSE')
            return ('boolean', False)

        elif tt == 'NONE':
           self.lexer.match('NONE')
           return ('none', None)

        elif tt == 'ID':
            tok = self.lexer.match('ID')
            return ('id', tok.value)

        elif tt == 'TIMES':
            self.lexer.match('TIMES')
            id_tok = self.lexer.match('ID')
            return ('deref', id_tok.value)

        elif tt == 'NOT':
            self.lexer.match('NOT')
            v = self.primary()
            return ('apply-list', ('list', [('id', '__not__'), v]))

        elif tt == 'MINUS':
            self.lexer.match('MINUS')
            v = self.primary()
            return ('apply-list', ('list', [('id', '__uminus__'), v]))

        elif tt == 'ESCAPE':
            self.lexer.match('ESCAPE')
            str_tok = self.lexer.match('STRING')
            return ('escape', str_tok.value)

        # TODO: need to look at list constructors for to-list, where-list and head-tail
        elif tt == '(':
            # Parenthesized expressions have the following meaning:
            #       (A)    means a parenthesized value A
            #       (A,)   means a list with a single value A
            #       (A, B) means a list with values A and B
            #       ()     NOT ALLOWED
            # NOTE: the ',' is handled in exp
            self.lexer.match('(')
            v = self.exp() # list or value
            self.lexer.match(')')
            if v[0] == 'raw-list':
                v = ('list', v[1])
            elif v[0] == 'raw-to-list':
                v = ('to-list', v[1])
            elif v[0] == 'raw-where-list':
                v = ('where-list', v[1])
            elif v[0] == 'raw-head-tail':
                v = ('head-tail', v[1])
            return v

        elif tt == '[':
            self.lexer.match('[')
            if self.lexer.peek().type in exp_lookahead:
                v = self.exp()
                if v[0] == 'raw-list': # we are putting brackets around a raw-list, turn it into a list
                    v = ('list', v[1])
                elif v[0] == 'raw-to-list':
                    v = ('to-list', v[1])
                elif v[0] == 'raw-where-list':
                    v = ('where-list', v[1])
                elif v[0] == 'raw-head-tail':
                    v = ('head-tail', v[1])
                else:
                    # turn contents into a list (possibly nested lists)
                    v = ('list', [v])
            elif self.lexer.peek().type == ']':
                v = ('list', [])
            else:
                raise SyntaxError("Syntax Error:{}: at '{}'".format(
                        self.lexer.peek().lineno,
                        self.lexer.peek().value))
            self.lexer.match(']')
            return v

        elif tt == '{':
            self.lexer.match('{')
            v = self.exp()
            self.lexer.match('}')
            return ('dict-access', v)

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

    test = 'load "io". print "Hello World!"'
    parser.parse(test)
