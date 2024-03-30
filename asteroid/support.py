###########################################################################################
# Asteroid support code
#
# (c) University of Rhode Island
###########################################################################################

import re
import sys

from asteroid.state import state

###########################################################################################
_temp_prefix = "__AST__TEMP"
_temp_postfix = "__"
_tempval = 0

def gettemp():
    global _tempval
    new_name = _temp_prefix + str(_tempval) + _temp_postfix
    _tempval += 1
    return new_name

###########################################################################################
__dump_level = sys.maxsize  # during debugging you can set this to limit tree dump size

def set_AST_dump_level(n):
    __dump_level = n

def dump_AST(node):
    '''
    this function will print any AST that follows the

         (TYPE [, child1, child2,...])

    tuple format for tree nodes.
    '''
    _dump_AST(node)
    print('')

def _dump_AST(node, level=0):
    if level > __dump_level:
        return
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
    else:
        return 'none'

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
def to_python_list(asteroid_list):
    '''
    convert a list from Asteroid list or tuple AST format into a standard Python list.
    ex. ('list',[('integer',1),('integer',2)])   =>   [1,2]
    '''
    output = []
    
    for (_type,val) in asteroid_list[1]:
        output.append(val)
        
    return output

###########################################################################################
# term2string is an unparsing function mapping AST snippets representing **values** into 
# a Python string
#
# DO NOT attempt to make this into a general AST unparser!

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
    #lhh
    #print(term)
    if TYPE == 'string':
        val = term[1]
        output_str = ""
        match_object_list = list(re.finditer(combined_re, val, re.DOTALL))
        for mo in match_object_list:
            type = mo.lastgroup
            value = mo.group()
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

    elif TYPE in ['integer', 'real']:
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
         (MEMBER_NAMES, (LIST, member_names)),
         (OBJECT_MEMORY, (LIST, object_memory))) = term

        # if __str__ function exists for this object use it
        if '__str__' in member_names:
            slot_ix = member_names.index('__str__')
            str_fval = object_memory[slot_ix]
            # calling a __str__ member function
            import asteroid.walk
            (STRING, obj_str) = asteroid.walk.handle_call(term, # object reference
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

    else:
        return '('+TYPE+'...)'

def get_tail_term(sym, term, attrs):
    # sym was None: print error message and return None
    if not sym:
        print("error: variable {} not found".format('@'.join(sym + attrs)))
        return None
    
    # Loop over all of the attributes, get the type and check if the attribute is an integer or number
    for a in attrs:
        TYPE = term[0]
        is_index = a.lstrip('-+').isnumeric()
        is_member = not a.isnumeric()
        
        # Indexing a list or tuple: if the index is at least 0, set the term to the value of that index.
        # Otherwise, print an error message and return None
        if TYPE in ['list', 'tuple'] and is_index:
            val = term[1]
            if 0 <= int(a) < len(val):
                term = val[int(a)]
            else:
                print("error: invalid index value {}".format(a))
                return None
        # Indexing a string: if the index is at least 0, set the term to a string equal to a single character.
        # Otherwise, error and return None
        elif TYPE in ['string'] and is_index:
            val = term[1]
            if 0 <= int(a) < len(val):
                term = ('string', val[int(a)])
            else:
                print("error: invalid index value {}".format(a))
                return None
        # Accessing structure member: if the member exists, set the term to the object data of that member
        # Otherwise, error and return None
        elif TYPE in ['object'] and is_member:
            (OBJECT,
            (STRUCT_ID, (ID, struct_id)),
            (MEMBER_NAMES, (LIST, member_names)),
            (OBJECT_MEMORY, (LIST, object_memory))) = term
            if a in member_names:
                member_ix = member_names.index(a)
                term = object_memory[member_ix]
            else:
                print("error: instance of struct {} has no member '{}'".format(struct_id, a))
                return None
        
        # Accessing member on list, tuple, or string, or indexing a structure, or attempting to access attributes on any type that
        # does not include those types: error and return None.
        elif TYPE in ['list', 'tuple', 'string'] and is_member:
            print("error: instance of {} has no data members".format(TYPE))
            return None
        elif TYPE in ['object'] and is_index:
            print("error: structures are not indexable")
            return None
        elif TYPE not in ['list', 'tuple', 'string', 'object']:
            print("error: instance of {} has no accessible attributes".format(TYPE))
            return None
        
        # Used for error handling: attach the next attribute to the existing symbol
        sym += "@{}".format(a)
    return term

def term2verbose(term, indent_size=0, initial_indent=True):
    TYPE = term[0]
    # Type does not contain nested data: call term2string and return the value
    if TYPE not in ['list', 'tuple', 'object']:
        return term2string(term)
    
    curr_indent, next_indent = ' '*4*indent_size, ' '*4*(indent_size+1)
    term_string = curr_indent if initial_indent else ''
    
    # Type is an object: build string with struct name, and data members and corresponding types on separate lines
    # Make sure indent is not applied twice
    if TYPE == 'object':
        (OBJECT,
        (STRUCT_ID, (ID, struct_id)),
        (MEMBER_NAMES, (LIST, member_names)),
        (OBJECT_MEMORY, (LIST, object_memory))) = term
        
        term_string += (struct_id + '(\n')
        l = len(member_names)
        for ix in range(l):
            term_string += "{}{}: {}".format(next_indent,
                                           member_names[ix],
                                           term2verbose(object_memory[ix], indent_size=indent_size+1, initial_indent=False))
            term_string += ',\n' if ix < l-1 else '\n'
        term_string += curr_indent + ')'
    # Type is list or tuple: build string with brackets, and stored data on each new line, making sure to indent new data when not nested.
    else:
        val = term[1]
        term_string += '[\n' if TYPE == 'list' else '(\n'
        l = len(val)
        for i in range(l):
            if val[i][0] not in ['list', 'tuple', 'object']:
                term_string += next_indent
            term_string += term2verbose(val[i], indent_size=indent_size+1)
            if i != l-1:
                term_string += ','
            if l > 1 or TYPE == 'list':
                term_string += '\n'
        if l == 1 and TYPE == 'tuple':
            term_string += ',\n'
        term_string += curr_indent + (']' if TYPE == 'list' else ')')
    return term_string

def find_function(sym, val, fname, func_name):
    # If the function has already been found, check if it is in the correct file
    if sym == func_name and val[0] == 'function-val':
        if _check_lineinfo_function(val, fname): return True
    # If we are looking at a struct, check if the function is one of its members and if it lies in the correct file
    if val[0] == 'struct':
        (STRUCT,
        (MEMBER_NAMES, (LIST, member_names)),
        (STRUCT_MEMORY, (LIST, struct_memory))) = val
        if func_name in member_names and struct_memory[member_names.index(func_name)][0] == 'function-val':
            if _check_lineinfo_function(struct_memory[member_names.index(func_name)], fname): return True
    # If we are looking at a loaded module, recurse on everything that is not another module and return True if the function was found
    if val[0] == 'module':
        (MODULE,
        (ID, id),
        (SCOPE, scope)) = val
        for s in scope[0]:
            for (new_sym, new_val) in s.items():
                if new_val[0] != 'module' and find_function(new_sym, new_val, fname, func_name):
                    return True
    return False
    
def _check_lineinfo_function(val, fname):
    # Extract the lineinfo field and compare it to the provided filename
    (FUNCTION_VAL,
    (BODY_LIST,
    (LIST, body_list)),
    (scopes, globals, global_scope)) = val
    (LINEINFO,
    (filename, lineno)) = body_list[0]
    return filename == fname
