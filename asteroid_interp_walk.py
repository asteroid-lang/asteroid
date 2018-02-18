#########################################################################
# A tree walker to interpret Asteroid programs
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
#########################################################################

from asteroid_state import state
from asteroid_support import assert_match
from asteroid_support import unify
from asteroid_support import promote
from pprint import pprint

#########################################################################
__retval__ = None  # return value register for escaped code

#########################################################################
# Use the exception mechanism to return values from function calls

class ReturnValue(Exception):
    
    def __init__(self, value):
        self.value = value
    
    def __str__(self):
        return(repr(self.value))

#########################################################################
def eval_actual_args(args):

    return walk(args)

#########################################################################
def declare_formal_args(unifiers):
    # unfiers is of the format: [ (pattern, term), (pattern, term),...]

    for u in unifiers:
        pattern, term = u
        (ID, sym) = pattern
        if ID != 'id':
            raise ValueError("no pattern match possible in function call")
        state.symbol_table.enter_sym(sym, term)

#########################################################################
# handle list index expressions as rvals
def handle_list_ix(list_val, ix):

    (VAL_LIST, ll) = list_val
    (IX_TYPE, ixs) = ix

    if VAL_LIST != 'list':
        raise ValueError(
            "expected list node but got {} node".format(
               VAL_LIST))

    if IX_TYPE == 'list': # then ixs is a list of indexes
        new_l = [] # construct a list of return values
        for i in ixs:
            (IX_EXP_TYPE, ival) = walk(i)
            if IX_EXP_TYPE != 'integer':
                raise ValueError("list index is not an integer")
            else:
                ix_val = int(ival)
                new_l.append(ll[ix_val])

        if len(new_l) == 1: # return scalar value
            return new_l[0]
        else:
            return ('list', new_l)

    else:
        raise ValueError("index op {} not yet implemented".format(ix[0]))

#########################################################################
# recursively walk through the contents of a list together with the
# index expression and find the element to assign to
#
# NOTE: the key here is that list names in Python are treated as references,
# that is, even though we are working outside the symbol table, the 
# symbol table holds a reference to the list we are updating so writing
# to the list here will update the list in the symbol table.
# the list acts like memory associated with the list name
def assign_to_list(list_val, ix, value):

    (JUXTA, ix_exp, rest_ix) = ix
    assert_match(JUXTA, 'juxta')

    # evaluate ix_exp and use it to update list element
    (LIST, ix_val_list) = walk(ix_exp)

    if LIST != 'list':
        raise ValueError("unknown index expression")

    if len(ix_val_list) != 1:
        raise ValueError("list slicing not supported on assignment")

    (TYPE, ix_val) = ix_val_list[0]

    if TYPE != 'integer':
        raise ValueError("non-integer list index expression")
    else:
        ix_val = int(ix_val)

    if rest_ix[0] == 'nil': # assign to list element
        list_val[ix_val] = value

    else: # keep recursing
        nested_list = list_val[ix_val]
        (TYPE, val) = nested_list
        if TYPE != 'list':
            raise ValueError("list and index expression do not match")
        else:
            assign_to_list(val, rest_ix, value)
        
#########################################################################
# handle list index expressions as lvals -- compute the list lval from
# sym and ix and assign to it the value
def handle_list_ix_lval(sym, ix, value):
    
    sym_list_val = state.symbol_table.lookup_sym(sym)

    (TYPE, val) = sym_list_val

    if TYPE != 'list':
        raise ValueError("{} is not of type list".format(sym))

    assign_to_list(val, ix, value)

#########################################################################
def handle_call(fval, actual_arglist):
    
    if fval[0] != 'function':
        raise ValueError("not a function in call")

    actual_val_args = eval_actual_args(actual_arglist)   # evaluate actuals in current symtab
    body_list = fval[1]   # get the list of function bodies - nil terminated seq list

    # iterate over the bodies to find one that unifies with the actual parameters
    (BODY_LIST, body_list_ix) = body_list
    unified = False

    while body_list_ix[0] != 'nil':
        (SEQ, body, next) = body_list_ix

        (BODY, 
         (PATTERN, p),
         (STMT_LIST, stmts)) = body

        try:
            unifiers = unify(actual_val_args, p)
            unified = True
        except:
            unifiers = []
            unified = False

        if unified:
            break
        else:
            body_list_ix = next

    if not unified:
        ValueError("none of the function bodies unified with actual parameters")

    # dynamic scoping for functions!!!
    state.symbol_table.push_scope()
    declare_formal_args(unifiers)

    # execute the function
    try:
        walk(stmts)         
    except ReturnValue as val:
        return_value = val.value
    else:
        return_value = ('none',) # need that in case function has no return statement

    # return to the original scope
    state.symbol_table.pop_scope()

    return return_value

#########################################################################
# node functions
#########################################################################
def attach_stmt(node):

    (ATTACH, f, (CONSTR_ID, sym)) = node
    assert_match(ATTACH, 'attach')
    assert_match(CONSTR_ID, 'constr-id')

    if f[0] == 'fun-id':
        fval = state.symbol_table.lookup_sym(f[1])
    elif f[0] == 'fun-const':
        fval = f[1]

    if fval[0] != 'function':
        raise ValueError("{} is not a function".format(f[1]))
    else:
        state.symbol_table.attach_to_sym(sym, fval)

#########################################################################
def assign_stmt(node):

    (ASSIGN, pattern, exp) = node
    assert_match(ASSIGN, 'assign')
    
    term = walk(exp)
    unifiers = unify(term, pattern)

    # TODO: check for repeated names in the unfiers
    # TODO: deal with non-local variables

    # walk the unifiers and bind name-value pairs into the symtab
    for unifier in unifiers:

        #lhh
        #print("assign unifier: {}".format(unifier))

        lval, value = unifier

        if lval[0] == 'id':
            state.symbol_table.enter_sym(lval[1], value)

        elif lval[0] == 'juxta':
            (JUXTA, (ID, sym), ix) = lval
            handle_list_ix_lval(sym, ix, value)

        else:
            raise ValueError("unknown unifier type {}".format(lval[0]))

#########################################################################
def get_stmt(node):

    (GET, name) = node
    assert_match(GET, 'get')

    s = input("Value for " + name + '? ')
    
    try:
        value = int(s)
    except ValueError:
        raise ValueError("expected an integer value for " + name)
    
    state.symbol_table.update_sym(name, ('scalar', value))

#########################################################################
def print_stmt(node):

    # TODO: deal with files and structures/lists

    (PRINT, exp, f) = node
    assert_match(PRINT, 'print')
    
    value = walk(exp)
    print("{}".format(value))

#########################################################################
def call_stmt(node):

    (CALLSTMT, name, actual_args) = node
    assert_match(CALLSTMT, 'callstmt')

    handle_call(name, actual_args)

#########################################################################
def return_stmt(node):

    (RETURN, e) = node
    assert_match(RETURN, 'return')

    if e[0] == 'nil': # no return value
        raise ReturnValue(('none',))

    else:
        raise ReturnValue(walk(e))

#########################################################################
def while_stmt(node):

    (WHILE, cond, body) = node
    assert_match(WHILE, 'while')
    
    value = walk(cond)
    while value != 0:
        walk(body)
        value = walk(cond)

#########################################################################
def if_stmt(node):
    
    try: # try the if-then pattern
        (IF, cond, then_stmt, (NIL,)) = node
        assert_match(IF, 'if')
        assert_match(NIL, 'nil')

    except ValueError: # if-then pattern didn't match
        (IF, cond, then_stmt, else_stmt) = node
        assert_match(IF, 'if')
        
        value = walk(cond)
        
        if value != 0:
            walk(then_stmt)
        else:
            walk(else_stmt)

    else: # if-then pattern matched
        value = walk(cond)
        if value != 0:
            walk(then_stmt)

#########################################################################
def block_stmt(node):
    
    (BLOCK, stmt_list) = node
    assert_match(BLOCK, 'block')
    
    state.symbol_table.push_scope()
    walk(stmt_list)
    state.symbol_table.pop_scope()

#########################################################################
def plus_exp(node):
    
    (PLUS,c1,c2) = node
    assert_match(PLUS, '__plus__')
    
    v1 = walk(c1)
    v2 = walk(c2)

    fval = state.symbol_table.lookup_sym('__plus__')
    
    if fval[0] == 'constructor':
        return ('__plus__', v1, v2)

    elif fval[0] == 'function':
        arglist = ('list', [v1, v2])
        v = walk(('juxta',
                  fval,
                  ('juxta',
                   arglist,
                   ('nil',))))
        return v

    else:
        raise ValueError("{} not implemented in __plus__".format(fval[0]))

#########################################################################
def minus_exp(node):
    
    (MINUS,c1,c2) = node
    assert_match(MINUS, '-')
    
    v1 = walk(c1)
    v2 = walk(c2)
    
    return v1 - v2

#########################################################################
def times_exp(node):
    
    (TIMES,c1,c2) = node
    assert_match(TIMES, '*')
    
    v1 = walk(c1)
    v2 = walk(c2)
    
    return v1 * v2

#########################################################################
def divide_exp(node):
    
    (DIVIDE,c1,c2) = node
    assert_match(DIVIDE, '/')
    
    v1 = walk(c1)
    v2 = walk(c2)
    
    return v1 // v2

#########################################################################
def eq_exp(node):
    
    (EQ,c1,c2) = node
    assert_match(EQ, '==')
    
    v1 = walk(c1)
    v2 = walk(c2)
    
    return 1 if v1 == v2 else 0

#########################################################################
def le_exp(node):
    
    (LE,c1,c2) = node
    assert_match(LE, '<=')
    
    v1 = walk(c1)
    v2 = walk(c2)
    
    return 1 if v1 <= v2 else 0

#########################################################################
def juxta_exp(node):
    # could be a call: fval fargs
    # could be a list access: x [0]

    #lhh
    #print("node: {}".format(node))

    (JUXTA, val, args) = node
    assert_match(JUXTA, 'juxta')

    if args[0] == 'nil':
        return walk(val)

    # look at the semantics of val
    v = walk(val)

    if v[0] == 'function': # execute a function call
        # if it is a function call then the args node is another
        # 'juxta' node
        (JUXTA, parms, rest) = args
        assert_match(JUXTA, 'juxta')
        return walk(('juxta', handle_call(v, parms), rest))

    elif v[0] == 'constructor': # return the structure
        (ID, constr_sym) = val
        assert_match(ID, 'id') # name of the constructor
        (JUXTA, parms, rest) = args
        assert_match(JUXTA, 'juxta')

        # constructor juxta nodes come in 2 flavors:
        # 1) (juxta, parms, nil) -- single call
        if rest[0] == 'nil':  
            return ('juxta', 
                    ('id', constr_sym),
                    ('juxta', walk(parms), rest))

        # 2) (juxta, e1, (juxta, e2, rest)) -- cascade
        else:
            return ('juxta', 
                    ('id', constr_sym),
                    walk(args))

    elif v[0] == 'list': # handle list indexing/slicing
        # if it is a list then the args node is another
        # 'juxta' node for indexing the list
        (JUXTA, ix, rest) = args
        assert_match(JUXTA, 'juxta')
        return walk(('juxta', handle_list_ix(v, ix), rest))

    else: # not yet implemented
        raise ValueError("'juxta' not implemented for {}".format(v[0]))

#########################################################################
def uminus_exp(node):
    
    (UMINUS, exp) = node
    assert_match(UMINUS, 'uminus')
    
    val = walk(exp)
    return - val

#########################################################################
def not_exp(node):
    
    (NOT, exp) = node
    assert_match(NOT, 'not')
    
    val = walk(exp)
    return 0 if val != 0 else 1

#########################################################################
def list_exp(node):

    (LIST, inlist) = node
    assert_match(LIST, 'list')

    outlist =[]

    for e in inlist:
        outlist.append(walk(e))

    return ('list', outlist)

#########################################################################
def escape_exp(node):

    (ESCAPE, s) = node
    assert_match(ESCAPE, 'escape')

    global __retval__
    __retval__ = ('none',)

    exec(s)

    return __retval__

#########################################################################
# walk
#########################################################################
def walk(node):
    # node format: (TYPE, [child1[, child2[, ...]]])
    type = node[0]
    
    if type in dispatch_dict:
        node_function = dispatch_dict[type]
        return node_function(node)
    else:
        raise ValueError("walk: unknown tree node type: " + type)

# a dictionary to associate tree nodes with node functions
dispatch_dict = {
    # statements
    'attach'  : attach_stmt,
    'assign'  : assign_stmt,
    'get'     : get_stmt,
    'print'   : print_stmt,
    'callstmt': call_stmt,
    'return'  : return_stmt,
    'while'   : while_stmt,
    'if'      : if_stmt,
    'block'   : block_stmt,

    # expressions
    'list'    : list_exp,
    'seq'     : lambda node : ('seq', walk(node[1]), walk(node[2])),
    'none'    : lambda node : node,
    'nil'     : lambda node : node,
    'function': lambda node : node, # looks like a constant
    'constructor' : lambda node : node, # looks like a constant
    'string'  : lambda node : node,
    'integer' : lambda node : node,
    'real'    : lambda node : node,
    'id'      : lambda node : state.symbol_table.lookup_sym(node[1]),
    'juxta'   : juxta_exp,
    'escape'  : escape_exp,
    'quote'   : lambda node : node[1],

    # built-in operators
    '__plus__'    : plus_exp,
    '__minus__'   : minus_exp,
    '*'       : times_exp,
    '/'       : divide_exp,
    '=='      : eq_exp,
    '<='      : le_exp,
    'uminus'  : uminus_exp,
    'not'     : not_exp
}


