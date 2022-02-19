###########################################################################################
# Asteroid compiler
#
# (c) University of Rhode Island
###########################################################################################

import os
import sys

from asteroid.globals import *
from asteroid.support import *
from asteroid.state import state
from compiler.frontend import Parser
from compiler.gen import walk as generate_code, gen_function_list, gen_dispatch
from asteroid.version import VERSION

# the prologue file is expected to be in the 'modules' folder
prologue_name = 'prologue.ast'

# TODO: adjust the defaults
def compile(input_stream,
           input_name = "<input>",
           tree_dump=False,
           exceptions=False,
           prologue=True):
    try:
        #lhh
        #print("path[0]: {}".format(sys.path[0]))
        #print("path[1]: {}".format(sys.path[1]))

        # read in prologue
        if prologue:
            prologue_file_base = os.path.join('../asteroid/modules', prologue_name)
            module_path = os.path.join(os.path.split(os.path.abspath(__file__))[0], prologue_file_base)
            working_path = os.path.join(os.getcwd(), prologue_file_base)

            #print(module_path)
            #print(working_path)

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

        begin_code = "### Asteroid Compiler Version {} ###\n\n".format(VERSION)
        begin_code += "from avm.avm import *\n"
        begin_code += "import avm.avm\n" # in order to support __retval__ properly
        begin_code += "from asteroid.globals import *\n"
        begin_code += "from asteroid.support import *\n"
        begin_code += "from asteroid.state import state\n"
        begin_code += "\n"
        begin_code += "__retval__ = ('none', None)\n"
        begin_code += "\n"
        begin_code += "try:\n"

        compiled_code = generate_code(state.AST)
        flist_code = gen_function_list()
        dispatch_code = gen_dispatch()

        end_code = "except Exception as e:\n"
        end_code += "   module, lineno = state.lineinfo\n"
        end_code += "   print('Error: {}: {}: {}'.format(module, lineno, e))\n"

        # assemble the code
        code = ""
        code += begin_code
        code += flist_code
        code += dispatch_code
        code += compiled_code
        code += end_code

        return code

    except Exception as e:
        if exceptions: # rethrow the exception so that you can see the full backtrace
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
