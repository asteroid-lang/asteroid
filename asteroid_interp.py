#!/usr/bin/env python
###########################################################################################
# Asteroid interpreter
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
###########################################################################################

from argparse import ArgumentParser
from asteroid_gram import Parser
from asteroid_state import state
#from asteroid_interp_walk import walk
from asteroid_support import dump_AST

def interp(input_stream):

    # initialize the state object
    state.initialize()

    # build the AST
    parser = Parser()
    state.AST = parser.parse(input_stream)

    # walk the AST
    dump_AST(state.AST)
    #walk(state.AST)

if __name__ == "__main__":
    # parse command line args
    aparser = ArgumentParser()
    aparser.add_argument('input')

    args = vars(aparser.parse_args())

    f = open(args['input'], 'r')
    input_stream = f.read()
    f.close()

    # execute interpreter
    interp(input_stream=input_stream)
