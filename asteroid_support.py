###########################################################################################
# Asteroid support code
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
###########################################################################################

from asteroid_state import state

#########################################################################
def len_seq(seq_list):

    if seq_list[0] == 'nil':
        return 0

    elif seq_list[0] == 'seq':
        # unpack the seq node
        (SEQ, p1, p2) = seq_list

        return 1 + len_seq(p2)

    else:
            raise ValueError("unknown node type: {}".format(seq_list[0]))

###########################################################################################
def reverse_node_list(node_type, node_list):
    ''' 
    shallow reversal of a nil terminated node_type list
    assumes the structure of node_type node: (node_type, element, next)
    NOTE: the list needs to be ('nil',) terminated
    '''
    
    new_list = ('nil',)

    e = node_list
    while(e[0] != 'nil'):
        new_list = (node_type, e[1], new_list)
        e = e[2]

    return new_list

###########################################################################################
def append_node_list(node_type, list1, list2):
    '''
    append list2 to list1.  assume 'nil' terminated lists of node_type
    NOTE: there is a more efficient way of doing this by iterating...
    '''
    
    if list1[0] == 'nil':
        return list2

    else:
        return (node_type, 
                list1[1], 
                append_node_list(node_type,
                                 list1[2],
                                 list2))

###########################################################################################
def dump_AST(node):
    '''
    this function will print any AST that follows the
    
         (TYPE [, child1, child2,...])
    
    tuple format for tree nodes.
    '''
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

    elif isinstance(node, list):
        _indent(level)
        print("[", end='')

        nchildren = len(node)

        if nchildren > 0:
            print(" ", end='')
        
        for c in range(nchildren):
            _dump_AST(node[c], level+1)
            if c != nchildren-1:
                print(' ', end='')
        
        print("]", end='')

    else:
        print("%s" % str(node), end='')

def _indent(level):
    print('')
    for i in range(level):
        print('  |',end='')


###########################################################################################
def assert_match(input, expected):
    if input != expected:
        raise ValueError(
            "Pattern match failed: expected '{}' but got '{}'".format(
                expected, input))

###########################################################################################
def unify(term, pattern):
    '''
    unify term and pattern recursively and return the unifier.
    this unification allows for the same variable to appear
    multiple times in the unifier.  the user of this
    function must take appropriate actions if this happens.

    we assume that both the term and the pattern are made up of tuple
    nodes:

             (<type>, children*)

    leaf nodes must be nullary constructors.
    '''
    #print("term {}\npattern {}\n\n".format(term, pattern))

    if isinstance(term, list) or isinstance(pattern, list):
        if not(isinstance(term, list) and isinstance(pattern, list)):
            raise ValueError("Pattern match failed: term and pattern do not agree on list constructor")

        elif len(term) != len(pattern):
            raise ValueError("Pattern match failed: term and pattern lists are not the same length")

        else:
            unifier = []
            for i in range(len(term)):
                unifier += unify(term[i], pattern[i])
            return unifier

    elif pattern[0] == 'deref':  # ('deref', id)
        sym = pattern[1]
        p = state.symtab.lookup_sym(sym)
        return unify(term,p)

    elif pattern[0] == 'id': # variable in pattern add to unifier
        sym = pattern[1]
        if sym == '_': # anonymous variable - ignore unifier
            return []
        else:
            unifier = (pattern, term)
            return [unifier]

    elif pattern[0] == 'juxta': # list access -- cannot pattern match on function calls!
        (TYPE, val) = pattern[1]
        if TYPE != 'id':
            raise ValueError("expected list name in access expression")
        else:
            unifier = (pattern, term)
            return [unifier]

    elif term[0] == 'id': # variable in term not allowed
        raise ValueError(
            "Pattern match failed: variable {} in term not allowed".format(
                term[1]))

    elif len(term) != len(pattern): # nodes are not of same the arity
        raise ValueError(
            "Pattern match failed: nodes {} and {} are not of the same arity".format(
                term[0], pattern[0]))

    elif term[0] != pattern[0]:  # nodes are not the same
        raise ValueError(
            "Pattern match failed: nodes {} and {} are not the same".format(
                term[0], pattern[0]))

    else:
        unifier = []
        for i in range(1,len(term)):
            unifier += unify(term[i], pattern[i])
        return unifier
    
###########################################################################################
def promote(type1, type2):
    '''
    type promotion table for builtin primitive types
    '''
    
    if type1 == 'string' and type2 in['string', 'real', 'integer']:
        return 'string'
    if type2 == 'string' and type1 in['string', 'real', 'integer']:
        return 'string'
    elif type1 == 'real' and type2 in ['real', 'integer']:
        return 'real'
    elif type2 == 'real' and type1 in ['real', 'integer']:
        return 'real'
    elif type1 == 'integer' and type2 == 'integer':
        return 'integer'
    else:
        raise ValueError("type {} and type {} are incompatible".format(type1, type2))

###########################################################################################


