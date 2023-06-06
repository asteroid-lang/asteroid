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
        self.mainmodule = None
        self.symbol_table = SymTab()
        self.AST = None
        self.eval_redundancy = True
        self.warning = True # switch for general warnings
        self.lineinfo = (module, 1) # tuple: module, lineno
        # stack of 3-tuples for stack trace of function
        # calls: (module,lineno,function name)
        self.trace_stack = [(module,1,"<toplevel>")]
        # if an exception occurs then error_trace will point to
        # it.  an exception handler is responsible for clearing
        # this.
        self.error_trace = None
        self.debugger = None

state = State()

def warning(str):
    if state.warning:
        module, lineno = state.lineinfo
        print("Warning: {}: {}: {}".format(module, lineno, str))

def dump_trace():
    if state.error_trace:
        _dump_trace(state.error_trace)
        return        
    else:
        _dump_trace(state.trace_stack)
        return

def _dump_trace(trace):
    print("traceback (most recent call last):")
    for i in range(0,len(trace)):
        (module,lineno,fname) = trace[i]
        print("{}: {}: calling {}".format(module,lineno,fname))
