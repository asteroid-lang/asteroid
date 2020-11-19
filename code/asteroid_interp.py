###########################################################################################
# Asteroid interpreter
#
# (c) Lutz Hamel, University of Rhode Island
###########################################################################################

import sys
from pathlib import Path
from asteroid_frontend import Parser
from asteroid_state import state
from asteroid_walk import walk
from asteroid_walk import ThrowValue
from asteroid_walk import ReturnValue
from asteroid_support import dump_AST
from asteroid_support import term2string
from asteroid_version import VERSION

# the prologue file is expected to be in the 'modules' folder
prologue_name = 'prologue.ast'

# TODO: adjust the defaults
def interp(input_stream,
           input_name = "<input>",
           tree_dump=False,
           do_walk=True,
           symtab_dump=False,
           exceptions=False,
           version=False,
           prologue=True):

    if version:
        print("** Asteroid Version {} **".format(VERSION))
        sys.exit(0)

    try:
        # initialize state
        state.initialize()

        # read in prologue
        if prologue:
            # load the prologue file
            prologue_file_base = '/modules/' + prologue_name

            if Path(sys.path[0] + prologue_file_base).is_file():
                prologue_file = sys.path[0] + prologue_file_base
                #lhh
                #print("path[0]:"+prologue_file)
            elif Path(sys.path[1] + prologue_file_base).is_file():
                prologue_file = sys.path[1] + prologue_file_base
                #lhh
                #print("path[1]:"+prologue_file)
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
