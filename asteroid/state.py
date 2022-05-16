###########################################################################################
# Asteroid State Object
#
# (c) University of Rhode Island
###########################################################################################

from asteroid.symtab import SymTab

class State:
    def __init__(self):
        self.initialize()

    def initialize(self,module="<input>"):
        self.symbol_table = SymTab()
        self.modules = [] # loaded modules
        self.AST = None
        self.ignore_pattern = 0 # used to evaluate pattern expressions
        self.constraint_lvl = 0 # used to evaluate constraint-only patterns
        self.cond_warning = False # used to indicate if conditional subsumption
                                  # warning has been displayed
        self.eval_redundancy = True
        self.lineinfo = (module, 1) # tuple: module, lineno
        # stack of 3-tuples for stack trace of function
        # calls: (module,lineno,function name)
        self.trace_stack = [(module,1,"<toplevel>")]

state = State()

def warning(str):
    module, lineno = state.lineinfo
    print("Warning: {}: {}: {}".format(module, lineno, str))

def dump_trace():
    if len(state.trace_stack) == 1:
        return
    else:
        print("traceback (most recent call last):")
        for i in range(0,len(state.trace_stack)):
            (module,lineno,fname) = state.trace_stack[i]
            print("{}: {}: calling {}".format(module,lineno,fname))
