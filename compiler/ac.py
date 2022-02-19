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
from asteroid.version import VERSION

def display_help():
    print("** Asteroid Compiler Version {} **".format(VERSION))
    print("(c) University of Rhode Island")
    print("usage: asteroid [-<switch>] <input file>")
    print("")
    print("command line flags:")
    print(" -t    AST dump")
    print(" -v    version")
    print(" -p    disable prologue")
    print(" -h    display help")
    print(" -e    show full exceptions")

def main():
    # defaults for the flags - when the flag is set on the command line
    # it simply toggles the default value in this table.
    flags = {
        '-t' : False, # AST dump flag
        '-v' : False, # version flag
        '-p' : True,  # prologue flag
        '-h' : False, # display help flag
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
    code = compile(input_stream=input_stream,
                   input_name = input_file,
                   tree_dump=flags['-t'],
                   exceptions=flags['-e'],
                   prologue=flags['-p'])

    print(code)


# run the compiler
if __name__ == "__main__":
    main()
