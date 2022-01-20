###########################################################################################
# Asteroid State Object
#
# (c) Lutz Hamel, University of Rhode Island
###########################################################################################

from asteroid.symtab import SymTab

class State:
    def __init__(self):
        self.initialize()

    def initialize(self):
        self.symbol_table = SymTab()
        self.modules = [] # loaded modules
        self.AST = None
        self.ignore_quote = False # used to evaluate quoted expressions
        self.constraint_lvl = 0 # used to evaluate constraint-only patterns
        self.cond_warning = False # used to indicate if conditional subsumption
                                  # warning has been displayed
        self.eval_redundancy = True
        self.lineinfo = ("<input>", 1) # tuple: module, lineno

state = State()

def warning(str):
    module, lineno = state.lineinfo
    print("Warning: {}: {}: {}".format(module, lineno, str))
