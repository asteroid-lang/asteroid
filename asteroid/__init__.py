#!/usr/bin/env python3
###########################################################################################
# Asteroid toplevel driver
#
# (c) University of Rhode Island
###########################################################################################

import cProfile
import sys
import os
import os.path
from asteroid.interp import interp
from asteroid.repl import repl
from asteroid.version import VERSION
from asteroid.mad import MAD

def display_help():
    print("Asteroid {}".format(VERSION))
    print("(c) University of Rhode Island")
    print("usage: asteroid [<switch>] <input file>")
    print("")
    print("command line switches:")
    print(" -d             run program through debugger")
    print(" -e             show Python exceptions")
    print(" -F             functional mode")
    print(" -h, --help     display help")
    print(" -p             disable prologue")
    print(" -r             disable redundant pattern detector")
    print(" -s             enable symbol table dump")
    print(" -t             AST dump")
    print(" -v, --version  version")
    print(" -w             disable tree walk")
    print(" -W             disable warnings")
    print(" -z             generate pstats")

def main():
    # defaults for the switches or flags - when the flag is set on the command line
    # it simply toggles the default value in this table.
    flags = {
        '-d' : False,    # Short debugger flag
        '-e' : False,  # show full exceptions
        '-F' : False,  # functional mode
        '--help' : False,  # display help flag
        '-h' : False,  # display help flag
        '-p' : True,   # prologue flag
        '-r' : True,   # redundant pattern dectector
        '-s' : False,  # symbol table dump flag
        '-t' : False,  # AST dump flag
        '--version' : False,  # version flag
        '-v' : False,  # version flag
        '-w' : True,   # tree walk flag
        '-W' : True,   # warnings
        '-z' : False,  # generate pstats flag
    }

    flag_names = list(flags.keys())
    argv_ix = 1
    asteroid_ext = '.ast'

    # logic of command line parameters
    # - process all flags to Asteroid interpreter
    # - determine if we need to respond to special flags and exit
    # - determine if we are starting in interactive mode or not
    #  if not in interactive mode execute the asteroid script

    # process all flags to interpreter
    while argv_ix < len(sys.argv):
        fl = sys.argv[argv_ix]
        if fl[0] == '-':
            if fl not in flag_names:
                raise ValueError("unknown flag {}".format(fl))
            else:
                flags[fl] = not flags[fl]
            argv_ix += 1
        else:
            break

    # determine if we need to respond to special flags and exit
    if flags['--help'] or flags['-h']:
        display_help()
        sys.exit(0)

    if flags['--version'] or flags['-v']:
        print("Asteroid {}".format(VERSION))
        sys.exit(0)

    debug_flag = flags['-d']

    # determine if we are starting in interactive mode or not
    # Note: first non-switch argument has to be an Asteroid source file
    if len(sys.argv) == argv_ix:
        input_file = ''
    else:
        input_file = sys.argv[argv_ix]
    (input_root,input_ext) = os.path.splitext(input_file)
    if input_file and input_ext != asteroid_ext:
        print("error: file '{}' is not an Asteroid source file".format(input_file))
        sys.exit(1)

    # run the REPL
    if input_ext == '' and not debug_flag:
        repl(redundancy=flags['-r'],
             prologue=flags['-p'],
             functional_mode=flags['-F'])
        sys.exit(0)

    # we have an input file, check if it exists
    if input_file and not os.path.isfile(input_file):
        print("error: unknown file '{}'".format(input_file))
        sys.exit(1)

    if debug_flag:
        # Create a new debugger
        db = MAD()
    else:
        db = None

    # execute the interpreter
    f = open(input_file, 'r')
    input_stream = f.read()
    f.close()

    # execute interpreter
    interp_object = \
    '''interp(program=input_stream,
           program_name = input_file,
           exceptions=flags['-e'],
           functional_mode=flags['-F'],
           prologue=flags['-p'],
           redundancy=flags['-r'],
           symtab_dump=flags['-s'],
           tree_dump=flags['-t'],
           do_walk=flags['-w'],
           warnings=flags['-W'],
           debugger=db
           )'''

    if flags['-z']:
        # generates pstats into the file 'pstats'
        # see https://docs.python.org/3/library/profile.html
        cProfile.runctx(interp_object, globals(), locals(), filename='pstats')
    else:
        exec(interp_object)

# for manual testing purposes
if __name__ == "__main__":
    main()
