###########################################################################################
# Asteroid interpreter
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
###########################################################################################

import sys
from pprint import pprint
from asteroid_parser import Parser
from asteroid_state import state
from asteroid_walk import walk
from asteroid_walk import ThrowValue
from asteroid_walk import ReturnValue
from asteroid_support import dump_AST
from asteroid_support import term2string

# TODO: adjust the defaults
def interp(input_stream, tree_dump=False, do_walk=True, symtab_dump=False, exceptions=False):

    # initialize the state object
    state.initialize()

    # build the AST
    parser = Parser()
    state.AST = parser.parse(input_stream)

    try:
        # walk the AST
        if tree_dump:
            dump_AST(state.AST)
        if do_walk:
            walk(state.AST)
        if symtab_dump:
            state.symbol_table.dump()

    except ThrowValue as throw_val:
        # handle exceptions using the standard Error constructor
        if throw_val.value[0] == 'apply' and throw_val.value[1][1] == 'Error':
            (APPLY, (ID, id), (APPLY, error_obj, rest)) = throw_val.value
            print("Error: {}".format(term2string(error_obj)))
        else:
            print("Unhandled Asteroid exception: {}".format(term2string(throw_val.value)))
        
        sys.exit(1)

    except ReturnValue as inst:
        print("Error: return statement used outside a function environment")
        sys.exit(1)

    except Exception as e:
        if exceptions: # rethrow the exception so that you can see the full backtrace
            if symtab_dump:
                state.symbol_table.dump()
            raise e
        else: 
            print("Error: {}".format(e))
            sys.exit(1)

