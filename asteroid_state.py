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
        self.AST = None

        # populate the symbol table with predefined behavior for operator symbols
        self.symbol_table.enter_sym('__plus__', ('constructor', ('arity', 2)))
        self.symbol_table.enter_sym('__minus__', ('constructor', ('arity', 2)))


state = State()


