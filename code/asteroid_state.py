###########################################################################################
# Asteroid State Object
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
###########################################################################################

from asteroid_symtab import SymTab

class State:
    def __init__(self):
        self.initialize()

    def initialize(self):
        self.symbol_table = SymTab()
        self.modules = [] # loaded modules
        self.AST = None
        self.ignore_quote = False # used to evaluate quoted expressions
        self.lineinfo = ("", 0) # tuple: module, lineno

        #lhh
        #print("initializing the symbol table")
        # initialize the constructor symbols for our builtin operators
        # in the symbol table.
        #
        # NOTE: you need to keep this in sync with the operators you add
        # to the grammar and populate the symbol table with predefined
        # behavior for operator symbols
        #
        # binary
        self.symbol_table.enter_sym('__plus__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__minus__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__times__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__divide__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__or__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__and__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__eq__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__ne__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__le__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__lt__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__ge__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__gt__', ('constructor', ('arity', 2)))
        # unary
        self.symbol_table.enter_sym('__uminus__', ('constructor', ('arity', 1)))
        self.symbol_table.enter_sym('__not__', ('constructor', ('arity', 1)))

state = State()
