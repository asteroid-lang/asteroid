##################################################################
# Asteroid support code
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
##################################################################
# this function will print any AST that follows the
#
#      (TYPE [, child1, child2,...])
#
# tuple format for tree nodes.

def dump_AST(node):
    _dump_AST(node)
    print('')

def _dump_AST(node, level=0):
    
    if isinstance(node, tuple):
        _indent(level)
        nchildren = len(node) - 1

        print("(%s" % node[0], end='')
        
        if nchildren > 0:
            print(" ", end='')
        
        for c in range(nchildren):
            _dump_AST(node[c+1], level+1)
            if c != nchildren-1:
                print(' ', end='')
        
        print(")", end='')
    else:
        print("%s" % str(node), end='')

def _indent(level):
    print('')
    for i in range(level):
        print('  |',end='')


##################################################################
def assert_match(input, expected):
    if input != expected:
        raise ValueError("Pattern match failed: expected '{}' but got '{}'".format(expected, input))

##################################################################


