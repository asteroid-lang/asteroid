#!/usr/bin/env python3
###########################################################################################
# Asteroid toplevel driver for the compiler
#
# (c) University of Rhode Island
###########################################################################################

import cProfile
import sys
import os
from compiler.compile import compile
from compiler.gen import gen_function_list, gen_dispatch
from asteroid.version import VERSION

def display_help():
    print("** Asteroid Compiler Version {} **".format(VERSION))
    print("(c) University of Rhode Island")
    print("usage: asteroid [-<switch>] <input file>")
    print("")
    print("command line flags:")
    print(" -t    AST dump")
    print(" -v    version")
    print(" -w    disable tree walk")
    print(" -p    disable prologue")
    print(" -h    display help")
    print(" -e    show full exceptions")

def main():
    # defaults for the flags - when the flag is set on the command line
    # it simply toggles the default value in this table.
    flags = {
        '-s' : False, # symbol table dump flag
        '-t' : False, # AST dump flag
        '-v' : False, # version flag
        '-w' : True,  # tree walk flag
        '-z' : False, # generate pstats flag
        '-p' : True,  # prologue flag
        '-h' : False, # display help flag
        '-r' : True,  # redundant pattern dectector
        '-e' : False, # show full exceptions
    }

    flag_names = list(flags.keys())

    for fl in sys.argv:
        if fl[0] != '-':
            continue
        elif fl not in flag_names:
            print("unknown flag {}".format(fl))
            sys.exit(0)
        flags[fl] = not flags[fl]

    if flags['-h'] or len(sys.argv) == 1:
        display_help()
        sys.exit(0)

    if flags['-v']:
        print("** Asteroid Compiler Version {} **".format(VERSION))
        sys.exit(0)

    input_file = sys.argv[-1]

    if not os.path.isfile(input_file):
        print("unknown file {}".format(input_file))
        sys.exit(0)

    f = open(input_file, 'r')
    input_stream = f.read()
    f.close()

    # execute compiler
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

    compiled_code = compile(input_stream=input_stream,
                   input_name = input_file,
                   tree_dump=flags['-t'],
                   do_walk=flags['-w'],
                   exceptions=flags['-e'],
                   prologue=flags['-p'])

    flist_code = gen_function_list()
    dispatch_code = gen_dispatch()

    end_code = "except Exception as e:\n"
    end_code += "   module, lineno = state.lineinfo\n"
    end_code += "   print('Error: {}: {}: {}'.format(module, lineno, e))\n"

    # assemble the code
    code = begin_code
    code += flist_code
    code += dispatch_code
    code += compiled_code
    code += end_code

    print(code)


# run the compiler
if __name__ == "__main__":
    main()
