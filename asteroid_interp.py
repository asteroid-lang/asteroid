#!/usr/bin/env python
###########################################################################################
# Asteroid interpreter
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
###########################################################################################

import sys
from pprint import pprint
from argparse import ArgumentParser
from asteroid_parser import Parser
from asteroid_state import state
from asteroid_walk import walk
from asteroid_walk import ThrowValue
from asteroid_support import dump_AST

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
            (APPLY, (ID, id), (APPLY, (STRING, error_string), rest)) = throw_val.value
            print("Error: {}".format(error_string))
        else:
            print("Error: unhandled Asteroid exception:")
            pprint(throw_val)
        
        sys.exit(1)

    except Exception as e:
        if exceptions: # rethrow the exception so that you can see the full backtrace
            if symtab_dump:
                state.symbol_table.dump()
            raise e
        else: 
            print("Error: {}".format(e))
            sys.exit(1)

if __name__ == "__main__":
    # parse command line args
    aparser = ArgumentParser()
    aparser.add_argument('-t','--tree', action='store_true', help='AST dump flag')
    aparser.add_argument('-w','--no_walk', action='store_false', help='walk flag')
    aparser.add_argument('-s','--symtab', action='store_true', help='symbol table dump flag')
    aparser.add_argument('input', metavar='input_file', help='Asteroid input file')

    args = vars(aparser.parse_args())

    f = open(args['input'], 'r')
    input_stream = f.read()
    f.close()

    # execute interpreter
    interp(input_stream=input_stream, 
           tree_dump=args['t'],
           do_walk=args['w'],
           symtab_dump=args['s'])
