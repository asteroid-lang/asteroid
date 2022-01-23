###########################################################################################
# Asteroid interpreter
#
# (c) Lutz Hamel, University of Rhode Island
###########################################################################################

import os
import sys

from asteroid.globals import *
from asteroid.support import *
from asteroid.frontend import Parser
from asteroid.state import state
from asteroid.walk import walk

# the prologue file is expected to be in the 'modules' folder
prologue_name = 'prologue.ast'

# TODO: adjust the defaults
def interp(input_stream,
           input_name = "<input>",
           tree_dump=False,
           do_walk=True,
           symtab_dump=False,
           exceptions=False,
           redundancy=True,
           prologue=True):
    try:
        # initialize state
        state.initialize()

        #lhh
        #print("path[0]: {}".format(sys.path[0]))
        #print("path[1]: {}".format(sys.path[1]))

        # initialize "check for useless clauses" flag
        state.eval_redundancy = redundancy

        # read in prologue
        if prologue:

            prologue_file = ''
            prologue_file_base = os.path.join('modules', prologue_name)
            module_path = os.path.join(os.path.split(os.path.abspath(__file__))[0], prologue_file_base)
            working_path = os.path.join(os.getcwd(), prologue_file_base)

            if os.path.isfile(module_path):
                prologue_file = module_path
            elif os.path.isfile(working_path):
                prologue_file = working_path
            else:
                raise ValueError("Asteroid prologue '{}' not found"
                                .format(prologue_file_base))

            with open(prologue_file) as f:
                state.modules.append(prologue_name)
                data = f.read()
                pparser = Parser(prologue_name)
                (LIST, pstmts) = pparser.parse(data)

        # build the AST
        parser = Parser(input_name)
        (LIST, istmts) = parser.parse(input_stream)
        if prologue:
            state.AST = ('list', pstmts + istmts)
        else:
            state.AST = ('list', istmts)

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

    except RedundantPatternFound as e:
        print("Error:  {}".format(e))
        sys.exit(1)

    except NonLinearPatternError as e:
        print("Error:  {}".format(e))
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
