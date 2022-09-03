#!/usr/bin/env python3
###########################################################################################
# Asteroid toplevel driver
#
# (c) University of Rhode Island
###########################################################################################

import cProfile
import sys
import os
from asteroid.interp import interp
from asteroid.repl import repl
from asteroid.version import VERSION
from asteroid.adb import adb

def display_help():
    print("** Asteroid Version {} **".format(VERSION))
    print("(c) University of Rhode Island")
    print("usage: asteroid [-<switch>] <input file>")
    print("")
    print("command line flags:")
    print(" -s          enable symbol table dump")
    print(" -t          AST dump")
    print(" -v          version")
    print(" -w          disable tree walk")
    print(" -z          generate pstats")
    print(" -p          disable prologue")
    print(" -h          display help")
    print(" -r          disable redundant pattern detector")
    print(" -e          show Python exceptions")
    print(" -F          functional mode")
    print(" -g, --adb   run program through debugger")

def main():
    # defaults for the flags - when the flag is set on the command line
    # it simply toggles the default value in this table.
    flags = {
        '-s' : False,  # symbol table dump flag
        '-t' : False,  # AST dump flag
        '-v' : False,  # version flag
        '-w' : True,   # tree walk flag
        '-z' : False,  # generate pstats flag
        '-p' : True,   # prologue flag
        '-h' : False,  # display help flag
        '-r' : True,   # redundant pattern dectector
        '-e' : False,  # show full exceptions
        '-F' : False,  # functional mode
        '--adb': False, # debugger flag 
        '-g': False    # Short debugger flag
    }

    flag_names = list(flags.keys())

    if len(sys.argv) == 1:
        repl(redundancy=flags['-r'],
             prologue=flags['-p'],
             functional_mode=flags['-F'])
        sys.exit(0)

    for fl in sys.argv:
        if fl[0] != '-':
            continue
        elif fl not in flag_names:
            print("unknown flag {}".format(fl))
            sys.exit(0)
        flags[fl] = not flags[fl]

    if flags['-h']:
        display_help()
        sys.exit(0)

    if flags['-v']:
        print("** Asteroid Version {} **".format(VERSION))
        sys.exit(0)

    input_file = sys.argv[-1]

    if input_file[0] == '-':
        if flags['--adb']:
            print("Please provide a file to debug")
            sys.exit(1)
        repl(redundancy=flags['-r'],
             prologue=flags['-p'],
             functional_mode=flags['-F'])
        sys.exit(0)

    if not os.path.isfile(input_file):
        print("unknown file {}".format(input_file))
        sys.exit(0)

    if flags['--adb'] or flags['-g']:
        # Create a new debugger
        db = adb.ADB()

        # Set the debugger's internal interpretation options
        db.interp_options = {
            'redundancy': flags['-r'],
            'prologue': flags['-p'],
            'functional_mode': flags['-F'],
            'exceptions': flags['-e'],
        }

        db.run(input_file)
        sys.exit(0)
    
    f = open(input_file, 'r')
    input_stream = f.read()
    f.close()

    # execute interpreter
    interp_object = \
    '''interp(program=input_stream,
           program_name = input_file,
           tree_dump=flags['-t'],
           do_walk=flags['-w'],
           symtab_dump=flags['-s'],
           exceptions=flags['-e'],
           redundancy=flags['-r'],
           prologue=flags['-p'],
           functional_mode=flags['-F'])'''

    if flags['-z']:
        # generates pstats into the file 'pstats'
        # see https://docs.python.org/3/library/profile.html
        cProfile.run(interp_object, 'pstats')
    else:
        exec(interp_object)

# for manual testing purposes
if __name__ == "__main__":
    main()
