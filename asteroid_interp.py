###########################################################################################
# Asteroid interpreter
#
# (c) 2018-2020 - Lutz Hamel, University of Rhode Island
###########################################################################################

import sys
from asteroid_frontend import Parser
from asteroid_state import state
from asteroid_walk import walk
from asteroid_walk import ThrowValue
from asteroid_walk import ReturnValue
from asteroid_support import dump_AST
from asteroid_support import term2string
from asteroid_version import VERSION

# TODO: adjust the defaults
def interp(input_stream, tree_dump=False, do_walk=True, symtab_dump=False, exceptions=False, version=False):

    if version:
        print("** Asteroid Version {} **".format(VERSION))

    try:
        # initialize state
        state.initialize()

        # build the AST
        parser = Parser()
        state.AST = parser.parse(input_stream)

        # walk the AST
        if tree_dump:
            dump_AST(state.AST)
        if do_walk:
            walk(state.AST)
        if symtab_dump:
            state.symbol_table.dump()

    except ThrowValue as throw_val:
        # handle exceptions using the standard Error constructor
        module, lineno = state.lineinfo
        if throw_val.value[0] == 'apply' and throw_val.value[1][1] == 'Error':
            (APPLY, (ID, id), (APPLY, error_obj, rest)) = throw_val.value
            print("Error: {}: {}: {}".format(module, lineno, term2string(error_obj)))
        else:
            print("Error: {}: {}: unhandled Asteroid exception: {}"
                  .format(module, lineno, term2string(throw_val.value)))

        sys.exit(1)

    except ReturnValue as inst:
        module, lineno = state.lineinfo
        print("Error: {}: {}: return statement used outside a function environment"
              .format(module, lineno))
        sys.exit(1)

    except Exception as e:
        if exceptions: # rethrow the exception so that you can see the full backtrace
            if symtab_dump:
                state.symbol_table.dump()
            raise e
        else:
            module, lineno = state.lineinfo
            print("Error: {}: {}: {}".format(module, lineno, e))
            sys.exit(1)

    except  KeyboardInterrupt as e:
            print("Error: keyboard interrupt")
            sys.exit(1)

    except  BaseException as e:
            print("Error: {}".format(e))
            sys.exit(1)
