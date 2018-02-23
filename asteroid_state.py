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

state = State()


