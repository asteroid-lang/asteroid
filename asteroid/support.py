###########################################################################################
# Asteroid support code
#
# (c) Lutz Hamel, University of Rhode Island
###########################################################################################

import re

from asteroid.state import state

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
    nomatch = False

    if isinstance(expected, list):
        if input not in expected:
            nomatch = True
    elif input != expected:
        nomatch = True

    if nomatch:
        raise ValueError(
            "Internal Error: pattern assert failed: expected '{}' but got '{}'".
            format(expected, input))

###########################################################################################
def promote(type1, type2):
    '''
    type promotion table for builtin primitive types.  this table implements the
    type hierarchies

                 boolean < integer < real < string
                 list < string
                 tuple < string
                 none
    '''

    if type1 == 'string' and type2 in['string','real','integer','list','tuple','boolean']:
        return 'string'
    if type2 == 'string' and type1 in['string','real','integer','list','tuple','boolean']:
        return 'string'
    elif type1 == 'real' and type2 in ['real', 'integer', 'boolean']:
        return 'real'
    elif type2 == 'real' and type1 in ['real', 'integer', 'boolean']:
        return 'real'
    elif type1 == 'integer' and type2 in ['integer', 'boolean']:
        return 'integer'
    elif type2 == 'integer' and type1 in ['integer', 'boolean']:
        return 'integer'
    elif type1 == 'boolean' and type2 == 'boolean':
        return 'boolean'
    elif type1 == 'list' and type2 == 'list':
        return 'list'
    elif type1 == 'tuple' and type2 == 'tuple':
        return 'tuple'
    elif type1 == 'none' and type2 == 'none':
        return 'none'
    else:
        if type1 == type2:
            raise ValueError("binary operation on type '{}' not supported".format(type1))
        else:
            raise ValueError("type '{}' and type '{}' are incompatible".format(type1, type2))

###########################################################################################
# Asteroid uses truth values similar to Python's Pythonic truth values:
#
# Any object can be tested for truth value, for use in an if or while condition or as
# operand of the Boolean operations.
#
# The following values are considered false:
#
#     none
#     false
#     zero of the numeric types: 0, 0.0.
#     the empty string
#     any empty list: (,), [].
#
#  All other values are considered true, in particular any object is considered
#  to be a true value.
#
def map2boolean(value):

    if value[0] == 'none':
        return ('boolean', False)

    elif value[0] == 'boolean':
        return value

    elif value[0] in  ['integer', 'real', 'list', 'tuple', 'string']:
        return ('boolean', bool(value[1]))

    elif value[0] in ['object', 'function-val', 'pattern']:
        return ('boolean', True)

    else:
        raise ValueError("unsupported type '{}' as truth value".format(value[0]))

###########################################################################################
def data_only(memory):
    '''
    filter an object memory and return a memory with only data values.
    '''
    data_memory = list()

    for item in memory:
        if item[0] != 'function-val':
            data_memory.append(item)

    return data_memory

###########################################################################################
def data_ix_list(memory):
    '''
    compute the set of indexes that point to data members in the object memory
    '''
    ix_list = list()
    for i in range(0,len(memory)):
        if (memory[i])[0] != 'function-val':
            ix_list.append(i)

    return ix_list

###########################################################################################
# the following two tables are used in format processing of strings
# in term2string to support "\\", "\n", "\t", and "\"" characters.
named_re_list = [
    r'(?P<BS2>\\\\)',
    r'(?P<NL>\\n)',
    r'(?P<TAB>\\t)',
    r'(?P<DQ>\\")',
    r'(?P<ANYTHING>.)',
]
combined_re = '|'.join(named_re_list)

def term2string(term):
    TYPE = term[0]

    if TYPE == 'string':
        val = term[1]
        output_str = ""
        match_object_list = list(re.finditer(combined_re, val, re.DOTALL))
        for mo in match_object_list:
            type = mo.lastgroup
            value = mo.group()
            #lhh
            #print(type)
            if type == 'BS2':
                output_str += "\\"
            elif type == 'NL':
                output_str += chr(10)
            elif type == 'TAB':
                output_str += chr(9)
            elif type == 'DQ':
                output_str += '"'
            elif type == 'ANYTHING':
                output_str += value
            else:
                raise ValueError("internal error: unknown match group {} in term2string"
                                 .format(type))
        return output_str

    elif TYPE in ['id', 'integer', 'real']:
        val = term[1]
        return str(val)

    elif TYPE in ['boolean', 'none']:
        val = term[1]
        return str(val).lower()

    elif TYPE in ['list', 'tuple']:
        val = term[1]
        term_string = '[' if TYPE == 'list' else '('
        l = len(val)
        for i in range(l):
            term_string += term2string(val[i])
            if i != l-1:
                term_string += ','
        if l == 1 and TYPE == 'tuple': # proper 1-tuple notation
            term_string += ','
        term_string += ']' if TYPE == 'list' else ')'
        return term_string

    elif TYPE == 'object':
        (OBJECT,
         (STRUCT_ID, (ID, struct_id)),
         (OBJECT_MEMORY, (LIST, object_memory))) = term

        struct_val = state.symbol_table.lookup_sym(struct_id)

        (STRUCT,
            (MEMBER_NAMES, (LIST, member_names)),
            (STRUCT_MEMORY, (LIST, struct_memory))) = struct_val

        # if __str__ function exists for this object use it
        if '__str__' in member_names:
            slot_ix = member_names.index('__str__')
            str_fval = object_memory[slot_ix]
            # calling a __str__ member function
            import walk
            # for some reason we have to reinitialize unify again
            if walk.debugging:
                walk.unify = walk.debug_unify
            else:
                walk.unify = walk.__unify
            (STRING, obj_str) = walk.handle_call(term, # object reference
                                    str_fval,
                                    ('none', None),
                                    'member function __str__')
            return obj_str
        else:
            data_memory = data_only(object_memory)
            term_string = struct_id + '('
            for ix in range(0, len(data_memory)):
                term_string += term2string(data_memory[ix])
                term_string += ',' if ix != len(data_memory)-1 else ''
            term_string += ')'
            return term_string

    elif TYPE in ['function-val', 'member-function-val']:
        return '(function ...)'

    elif TYPE == 'apply':
        (APPLY, f, args) = term
        operator_table = {
            '__uminus__'    : ("-", True),
            '__uplus__'     : ("+", True),
            '__not__'       : ("not ", True),
            '__plus__'      : ("+", False),
            '__minus__'     : ("-", False),
            '__times__'     : ("*", False),
            '__divide__'    : ("/", False),
            '__or__'        : (" or ", False),
            '__and__'       : (" and ", False),
            '__eq__'        : (" == ", False),
            '__ne__'        : (" != ", False),
            '__le__'        : (" <= ", False),
            '__lt__'        : (" < ", False),
            '__ge__'        : (" >= ", False),
            '__gt__'        : (" > ", False)
        }
        if isinstance(f[1], str) and f[1] in operator_table:
            (op_symbol, is_unary) = operator_table[f[1]]
            if is_unary:
                term_string = "{}({})".format(
                    op_symbol, term2string(args)
                )
            else:
                (_, arglist) = args
                term_string = "{}{}{}".format(
                    term2string(arglist[0]), op_symbol, term2string(arglist[1])
                )
        else:
            term_string = term2string(f)
            term_string += term2string(args)

        return term_string

    elif TYPE == 'pattern':
        # we are printing out a term - just ignore the pattern operator
        val = term[1]
        return term2string(val)

    elif TYPE == 'nil':
        return ''

    elif TYPE == 'head-tail':           # Handle a head-tail pattern
        length = head_tail_length(term)
        term_string = "["
        for ix in range(1,length):

            #update output text with each entry in head-tail list
            term_string += term2string(term[1])

            #step down the head-tail tree
            term = term[2]

            #Insert head-tail entry delimiter
            term_string+= "|"

            #Catch the last entry
            if (ix == (length-1)):
                term_string += term2string(term)

        #Put the head-tail list delimiter on the end and then return
        term_string += "]"
        return term_string

    elif TYPE == 'raw-head-tail':
        (RHT, e1, e2) = term
        walked_e1 = term2string(e1)
        walked_e2 = term2string(e2)

        return walked_e1 + "|" + walked_e2

    elif TYPE == 'named-pattern':       # Handle a named pattern
        (NAMED_PATTERN,ID,pattern) = term

        if ID[0] == 'index':
            (INDEX, i1, i2) = ID
            term_string = term2string(i1) + "@ " + term2string(i2) + ":"
        else:
            term_string = ID[1] + ':'

        return term_string + term2string(pattern)

    elif TYPE == 'typematch':           # Handle a type pattern
        (TYPECLASS,cType) = term
        term_string = '%'
        term_string += cType
        return term_string

    elif TYPE == 'constraint':          # Handle a constraint-only pattern
        (CONSTRAINT,ptrn,bl) = term
        if bl[0] != 'nil':
            raise ValueError("term2string: binding term lists not supported")
        term_string = '%['
        term_string += term2string(ptrn)
        term_string += ']%'
        return term_string

    elif TYPE == 'deref':               # Handle a first-class pattern
        (DEREF, d_exp, binding_list) = term
        if binding_list[0] != 'nil':
            raise ValueError("internal error: binding lists in term2string not supported")
        term_string = "*" + term2string(d_exp)

        #Get the actual pattern from the symbol table
        # NOTE: THIS BREAKS WHEN A PATTERN IS NOT DEFINED IN SCOPE
        # TODO: FIX THIS
        # \/\/\/\/\/\/\/\/\/
        #term_string += term2string(state.symbol_table.lookup_sym(pName))
        return term_string

    elif TYPE == 'if-exp':              # Handle a conditional pattern
        (IF_EXP,EXPRESSION,value,NULL) = term
        # conditions can be arbitrary computations not structural terms
        # therefore we don't print the whole tree.
        return term2string(value)+' if (condition...)'

    elif TYPE == 'index':
        (INDEX, base, ix) = term
        return term2string(base) + "@ " + term2string(ix)

    elif TYPE == 'foreign':
        return "(foreign ...)"

    elif TYPE == 'is':
        (IS, e1, e2) = term

        return term2string(e1) + " is " + term2string(e2)

    elif TYPE == 'in':
        (IN, e1, e2) = term

        return term2string(e1) + " in " + term2string(e2)

    elif TYPE == 'to-list':
        (TO_LIST, (START, start), (STOP, stop), (STRIDE, stride)) = term
        return "[" + term2string(start) + " to " + term2string(stop) + ", stride: " + term2string(stride) + "]"

    elif TYPE == 'function-exp':
        return ('function exp...')

    elif TYPE == 'function-val':
        return ('function val...')

    elif TYPE == 'struct':
        return ('struct...')
    else:
        raise ValueError(
            "unknown type '{}' in term2string"
            .format(TYPE))

##############################################################################################
# Function head_tail_length determines the lenth of a head-tail node by walking to the end.
# The length is then returned from this function as in integer.
# Example Input : [h1|h2|h3|tail]
#         Output: 3
# The output is 3 because the input heal-tail pattern has 3 heads.
#
def head_tail_length( node ):

    # Counter to hold length of head-tail node
    ctr = 1

    # Try to walk down the tree, incrementing the counter
    # return the counter when we fail
    while(1):
        try:
            node = node[2]
        except:
            return ctr
        else:
            ctr += 1

##############################################################################################
