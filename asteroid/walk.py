#########################################################################
# A tree walker to interpret Asteroid programs
#
# (c) University of Rhode Island
#########################################################################

from copy import deepcopy,copy
from re import match as re_match
from math import isclose

from asteroid.globals import *
from asteroid.support import *
from asteroid.state import state, warning

#########################################################################
# this dictionary maps list member function names to function
# implementations given in the Asteroid prologue.
# see 'prologue.ast' for details
list_member_functions = dict()

#########################################################################
# this dictionary maps string member function names to function
# implementations given in the Asteroid prologue.
# see 'prologue.ast' for details
string_member_functions = dict()

#########################################################################
__retval__ = None  # return value register for escaped code

#########################################################################
# return values for function computed by the last expression executed
# in the context of a function.  note that we consider global code
# to be part of the 'top-level' function
function_return_value = [None]

###########################################################################################
def unify(term, pattern, unifying = True ):
    '''
    unify term and pattern recursively and return the unifier.

    we assume that both the term and the pattern are made up of AST tuple
    nodes:

             (<type>, children*)

    NOTE: if the pattern looks like an lval then it is treated like an lval, e.g.
            let a@[0] = a@[0].
          stores the value stored at a@[0] into lval a@[0].
    NOTE: Default argument unifying is set to be true. If we are unifying, then we are
          evaluating unification/matching between a pattern and a term. If we are not
          unifying, then we are evaluating subsumption between two patterns for the
          purpose of detecting redundant/useless pattern clauses in functions.
    NOTE: The tests (if/elif clauses) are ordered:
            1. Python instance level matching
            2. Special match cases for terms and patterns
            3. AST structural matching
    '''

    ################################
    ### 1. Python instance level matching
    # NOTE: in the first rules where we test instances we are comparing
    # Python level values, if they don't match exactly then we have
    # a pattern match fail.
    if isinstance(term, str): 
        # apply regular expression match
        # Note: a pattern needs to match the whole term.
        if isinstance(pattern, str) and re_match("^"+pattern+"$", term):
            return [] # return empty unifier
        else:
            raise PatternMatchFailed(
                "regular expression '{}' did not match '{}'"
                .format(pattern, term))
                
    elif isinstance(term, (int, float, bool)):
        if term == pattern:
            return [] # return an empty unifier
        else:
            raise PatternMatchFailed(
                "'{}' is not the same as '{}'"
                .format(term, pattern))

    elif isinstance(term, list) or isinstance(pattern, list):
        if not(isinstance(term, list)) or not(isinstance(pattern, list)):
            raise PatternMatchFailed(
                "term and pattern do not agree on list/tuple constructor")
        elif len(term) != len(pattern):
            raise PatternMatchFailed(
                "term and pattern lists/tuples are not the same length")
        else:
            # Make our unifier(s)
            unifier = []
            for i in range(len(term)):
                unifier += unify(term[i], pattern[i], unifying)
            # Ensure we have no non-linear patterns
            check_repeated_symbols(unifier)
            return unifier
    
    #################################
    ### 2. Special match cases for terms and patterns
    # Asteroid value level matching
    elif (not unifying) and (pattern[0] == 'deref' or term[0] == 'deref'):
        # since first-class patterns are almost always conditional
        # patterns and conditional patterns are not supported by
        # the redundancy test we punt here and save computation time.
        raise PatternMatchFailed(
            "first-class patterns not supported.")

    elif (not unifying) and (pattern[0] == 'typematch' or term[0] == 'typematch'):
        raise PatternMatchFailed(
                "typematch patterns not supported")

    elif (not unifying) and (pattern[0] == 'if-exp' or term[0] == 'if-exp'):
        # conditional patterns are not supported in redundancy
        # test because they can involved arbitraty computations
        # which cannot be tested.
        raise PatternMatchFailed(
            "conditional patterns not supported.")

    elif (not unifying) and (pattern[0] in ['head-tail', 'raw-head-tail'] or 
             term[0] in ['head-tail', 'raw-head-tail']):
        raise PatternMatchFailed(
            "head-tail operator not supported")

    elif (not unifying) and pattern[0] == 'apply' and term[0] == 'apply':
        fp = pattern[1]
        ft = term[1]
        if fp[0] != 'id' or ft[0] != 'id':
            raise ValueError("pattern subsumption not supported")
        # unpack the apply structures
        (APPLY, (ID, t_id), t_arg) = term
        (APPLY, (ID, p_id), p_arg) = pattern
        # only constructors are allowed in patterns
        type = state.symbol_table.lookup_sym(t_id,strict=False)
        if not type or type[0] != 'struct':
            raise ValueError(
                    "illegal pattern: function or operator '{}' not supported"
                    .format(t_id))
        type = state.symbol_table.lookup_sym(p_id,strict=False)
        if not type or type[0] != 'struct':
            raise ValueError(
                    "illegal pattern: function or operator '{}' not supported"
                    .format(p_id))
        # make sure apply id's match
        if t_id != p_id:
            raise PatternMatchFailed(
                "term '{}' does not match pattern '{}'"
                .format(t_id, p_id))
        # unify the args
        return unify(t_arg, p_arg, unifying)

    elif term[0] in (unify_not_allowed - {'function-val', 'foreign'}):
        # NOTE: functions/foreign are allowed in terms as long as they are matched
        # by a variable in the pattern - anything else will fail
        raise PatternMatchFailed(
            "term of type '{}' not allowed in pattern matching"
            .format(term[0]))

    elif pattern[0] in unify_not_allowed:
        raise PatternMatchFailed(
            "pattern of type '{}' not allowed in pattern matching"
            .format(pattern[0]))

    elif pattern[0] == 'id': # variable in pattern add to unifier
        if pattern[1] == '_': # anonymous variable - ignore unifier
            return []
        else:
            id_val = state.symbol_table.lookup_sym(pattern[1],strict=False)
            if id_val and id_val[0] == 'pattern':
                warning("you are overwriting a pattern stored in '{}'".format(pattern[1]))
            unifier = (pattern, term)
            return [unifier]

    elif pattern[0] == 'index': # index act like a var
        unifier = (pattern, term)
        return [unifier]

    elif pattern[0] == 'none':
        if term[0] == 'none':
            return []
        else:
            raise PatternMatchFailed(
                    "expected 'none' got '{}'"
                    .format(term[0]))

    elif pattern[0] == 'if-exp':
        (IF_EXP, cond_exp, patexp, else_exp) = pattern
        if else_exp[0] != 'null':
            raise PatternMatchFailed(
                    "conditional patterns do not support 'else' clauses")
        # evaluate the conditional expression in the
        # context of the unifiers of the pattern before the 
        # if clause and only expose all
        # unifiers if conditional was successful
        state.symbol_table.push_scope({})
        declare_unifiers(unify(term, patexp, unifying))
        bool_val = walk(cond_exp)
        if bool_val[0] != 'boolean':
            raise ValueError("found '{}' expected 'boolean' in conditional pattern"
                             .format(bool_val[0]))
        # copy unifiers out of the temporary scope of the
        # if expression.
        unifiers = state.symbol_table.get_curr_scope(option="unifiers")
        state.symbol_table.pop_scope()
        if bool_val[1]:
            return unifiers
        else:
            raise PatternMatchFailed(
                    "conditional pattern match failed")

    elif pattern[0] == 'typematch':
        typematch_kind = pattern[1]
        if typematch_kind in ['string','real','integer','list','tuple','boolean','none']:
            if typematch_kind == term[0]:
                return []
            else:
                raise PatternMatchFailed(
                    "expected a value of type '{}' got a value of type '{}'"
                    .format(typematch_kind, term[0]))
        elif typematch_kind == 'function':
            # matching function and member function values
            if term[0] in ['function-val','member-function-val']:
                return []
            else:
                raise PatternMatchFailed(
                    "expected a value of type '{}' got a value of type '{}'"
                    .format(typematch_kind, term[0]))
        elif typematch_kind == 'pattern':
            # any kind of structure can be a pattern, and variables
            # see globals.py for a definition of 'patterns'
            if term[0] in patterns:
                return []
            else:
                raise PatternMatchFailed(
                        "expected a value of type '{}' got a value of type '{}'"
                        .format(typematch_kind, term[0]))
        elif term[0] == 'object': # then typematch_kind has to be a structure type
            if state.symbol_table.lookup_sym(typematch_kind)[0] != 'struct':
                raise PatternMatchFailed( 
                        "'{}' is not a type"
                        .format(typematch_kind) )
            (OBJECT,
                (STRUCT_ID, (ID, struct_id)),
                (MEMBER_NAMES, LIST),
                (OBJECT_MEMORY, LIST)) = term
            if struct_id == typematch_kind:
                return []
            else:
                raise PatternMatchFailed(
                    "expected a value of type '{}' got a value of type '{}'"
                    .format(typematch_kind, struct_id))
        else:
            if state.symbol_table.lookup_sym(typematch_kind)[0] != 'struct':
                raise PatternMatchFailed( "'{}' is not a type".format(typematch_kind) )
            else:
                raise PatternMatchFailed(
                    "expected a value of type '{}' got a value of type '{}'"
                    .format(typematch_kind, term[0]))

    elif pattern[0] == 'pattern':
        if term[0] == 'pattern':
            # term is a first-class pattern, ie, a pattern value
            # we are not allowed to match on that.
            raise PatternMatchFailed(
                "cannot pattern-match patterns")
        else:
            # treat the pattern value as a pattern and continue matching.
            return unify(term, pattern[1], unifying)

    elif pattern[0] == 'deref':  # ('deref', v, bl)
        # v can be an AST representing any computation
        # that produces a pattern.
        p = walk(pattern[1])
        if pattern[2][0] != 'nil': # we have a binding term list
            if p[1][0] != 'scope':
                raise ValueError(
                    "binding term lists only supported for constraint patterns")
            else: 
                # construct a new constraint pattern with the binding term list in place
                p = ('pattern',
                        ('scope',
                            p[1][1],
                            pattern[2]))
        return unify(term,p,unifying)

    elif pattern[0] == 'scope':
        p = pattern[1]
        bl = pattern[2] # binding term list
        # constraint patterns are evaluated in their own scope
        try:
            state.symbol_table.push_scope({})
            unifier = unify(term,p)
            state.symbol_table.pop_scope()
        except PatternMatchFailed as e:
            state.symbol_table.pop_scope()
            # rethrow exception so that pattern match failure is properly propagated
            raise e 
        # process binding list
        if bl[0] == 'nil':
            return [] #Return an empty unifier
        else:
            # binding list is non-empty, map variables
            # that appear in the binding list into
            # the returned unifier
            new_unifier = []
            for u in unifier:
                ((ID,x),exp) = u
                for bt in bl[1]:
                    (BINDING_TERM, (ID,y), new_id) = bt
                    if x == y:
                        new_unifier += [(new_id,exp)]
            return new_unifier

    elif pattern[0] == 'object' and term[0] == 'object':
        # this can happen when we dereference a variable pointing
        # to an object as a pattern, e.g.
        #    let o = A(1,2). -- A is a structure with 2 data members
        #    let *o = o.
        (OBJECT, (STRUCT_ID, (ID, pid)), pml, (OBJECT_MEMORY, (LIST, pl))) = pattern
        (OBJECT, (STRUCT_ID, (ID, tid)), tml, (OBJECT_MEMORY, (LIST, tl))) = term
        if pid != tid:
            raise PatternMatchFailed(
                "pattern type '{}' and term type '{}' do not agree"
                .format(pid,tid))
        unifiers = []
        # we only pattern match on data members
        tl_data_members = data_only(tl)
        pl_data_members = data_only(pl)
        if len(tl_data_members) != len(pl_data_members):
            raise ValueError("internal error: not the same number of data members for objects")
        for i in range(len(pl_data_members)):
            unifiers += unify(tl_data_members[i], pl_data_members[i])
        return unifiers

    elif pattern[0] == 'apply' and term[0] == 'object':
        # in patterns constructor functions match objects, e.g.
        #   let A(x,y) = A(1,2)
        # only constructors are allowed in patterns
        f = pattern[1]
        if f[0] == 'index':
            # scope qualified pattern name
            (APPLY,
              (INDEX,
                (ID, modname),
                (ID, apply_id)),
              arg) = pattern
        else:
            modname = None
            (APPLY,
              (ID, apply_id),
              arg) = pattern
        # unpack term
        (OBJECT,
         (STRUCT_ID, (ID, struct_id)),
         (MEMBER_NAMES, (LIST, member_names)),
         (OBJECT_MEMORY, (LIST, obj_memory))) = term
        if modname:
            orig_config = set_module_env(modname)        
        type = state.symbol_table.lookup_sym(apply_id,strict=False)
        if modname:
            set_config(orig_config)
        if not type or type[0] != 'struct':
            raise ValueError("illegal pattern, '{}' is not a type".format(apply_id))
        if struct_id != apply_id:
            raise PatternMatchFailed("expected type '{}' got type '{}'"
                .format(apply_id, struct_id))
        # retrieve argument list to constructor
        # Note: we want it to be a list so we can compare it to the object memory
        if arg[0] == 'tuple':
            arg_list = arg[1]
        else:
            arg_list = [arg]
        # only pattern match on object data members
        data_list = data_only(obj_memory)
        return unify(data_list, arg_list, unifying)

    elif pattern[0] in ['head-tail', 'raw-head-tail']:
        (HEAD_TAIL, pattern_head, pattern_tail) = pattern
        if term[0] != 'list':
            raise PatternMatchFailed(
                "head-tail operator expected a list got a value of type '{}'"
                .format(term[0]))
        (LIST, list_val) = term
        if not len(list_val):
            raise PatternMatchFailed(
                "head-tail operator expected a non-empty list")
        list_head = list_val[0]
        list_tail = ('list', list_val[1:])
        unifier = []
        unifier += unify(list_head, pattern_head, unifying)
        unifier += unify(list_tail, pattern_tail, unifying)
        check_repeated_symbols(unifier) #Ensure we have no non-linear patterns
        return unifier
 
    #################################
    ### 3. AST structural matching
    elif pattern[0] != term[0]:  # node types are not the same
        raise PatternMatchFailed(
            "nodes '{}' and '{}' are not the same"
            .format(term[0], pattern[0]))

    elif len(pattern) != len(term): # nodes are not of same the arity
        raise PatternMatchFailed(
            "nodes '{}' and '{}'' are not of the same arity"
            .format(term[0], pattern[0]))

    else: # unify AST children nodes
        unifier = []
        for i in range(1,len(term)):
            unifier += unify(term[i], pattern[i], unifying)
        return unifier

#########################################################################
def eval_actual_args(args):

    return walk(args)

#########################################################################
def declare_formal_args(unifiers):
    # unfiers is of the format: [ (pattern, term), (pattern, term),...]

    for u in unifiers:
        (pattern, term) = u
        (ID, sym) = pattern # in unifiers the pattern is always a variable
        assert_match(ID,'id')
        if sym == 'this':
            raise ValueError("'this' is a reserved word")
        state.symbol_table.enter_sym(sym, term)

#########################################################################
# Evaluates a set of unifiers for the presence of repeated variable
# names within a pattern. Repeated variables names within the same pattern
# are what is called a non-linear pattern, which is not currently supported
# by Asteroid.
# This function will raise a NonLinearPatternError exception when a non-linear
# pattern has been recognized.
# Otherwise, this function returns control to the caller after finishing.
def check_repeated_symbols( unifiers ):

    symbols = {} # Will hold all previously seen unifiers(term-pattern) as (key-value)
    skip_unifier = False #Determines if we want to eval the current unifier pair

    # For each pair of unifiers
    for unifier in unifiers:

        # Unpack the pattern-term pair
        (pattern, term) = unifier

        # If the pattern side is an ID node, unpack it
        # Else we skip this unifier
        if pattern[0] == 'id':
            (ID, sym) = pattern
        else:
            skip_unifier = True

        # If we are not skipping this turn, check to see if we have seen this
        # variable before.
        if skip_unifier:
            skip_unifier = False

        elif sym in symbols: # We have found a non-linear pattern
            raise NonLinearPatternError(
            "multiple instances of '{}' found within pattern.".format(sym))

        else: # Else we have never seen this before so we record it.
            symbols[sym] = term

#########################################################################
# we are indexing into the memory of either a list/tuple/string or an
# object to read the memory.
#
# NOTE: when indexed with a scalar it will return a single value,
# that value of course could be a list etc.  When index with a list
# then it will return a list of values. Therefore:
#       a@1 =/= a@[1]
# the value on the left of the inequality is a single value, the
# value on the right is a singleton list.
def read_at_ix(structure_val, ix):

    # find the actual memory we need to access
    # list: return the actual list
    if structure_val[0] in ['list', 'tuple', 'string']:
        if structure_val[0] == 'list' \
        and ix[0] == 'id' \
        and ix[1] in list_member_functions:
            # we are looking at the function name of a list member
            # function - find the implementation and return it.
            impl_name = list_member_functions[ix[1]]
            # remember the object reference.
            return ('member-function-val',
                    structure_val,
                    state.symbol_table.lookup_sym(impl_name))
        elif structure_val[0] == 'string' \
        and ix[0] == 'id' \
        and ix[1] in string_member_functions:
            # we are looking at the function name of a string member
            # function - find the implementation and return it.
            impl_name = string_member_functions[ix[1]]
            # remember the object reference.
            return ('member-function-val',
                    structure_val,
                    state.symbol_table.lookup_sym(impl_name))
        else:
            # get a reference to the memory
            memory = structure_val[1]
            # compute the index
            ix_val = walk(ix)

    # for objects we access the object memory
    elif structure_val[0] == 'object':
        (OBJECT,
         (STRUCT_ID, (ID, struct_id)),
         (MEMBER_NAMES, (LIST, member_names)),
         (OBJECT_MEMORY, (LIST, memory))) = structure_val
        if ix[0] == 'id' and ix[1] in member_names:
            ix_val = ('integer', member_names.index(ix[1]))
        else:
            raise ValueError("{} is not a member of type {}"
                .format(term2string(ix),struct_id))
            #ix_val = walk(ix)

    elif structure_val[0] == 'module':
        (MODULE, id, (CLOSURE, closure)) = structure_val
        config = state.symbol_table.get_config()
        state.symbol_table.set_config(closure)
        val = walk(ix)
        state.symbol_table.set_config(config)
        return val

    elif structure_val[0] == 'pattern':
        # simple patterns are just structures - skip the pattern operator
        return read_at_ix(structure_val[1], ix)

    else:
        raise ValueError("term '{}' is not indexable"
                         .format(term2string(structure_val)))

    # index into memory and get value(s)
    if ix_val[0] == 'integer':
        if structure_val[0] == 'string':
            return ('string', memory[ix_val[1]])
        elif structure_val[0] == 'object' \
        and memory[ix_val[1]][0] == 'function-val':
            # remember the object reference.
            return ('member-function-val',
                    structure_val,
                    memory[ix_val[1]])
        else:
            return memory[ix_val[1]]

    elif ix_val[0] == 'list':
        if len(ix_val[1]) == 0:
            raise ValueError("index list is empty")

        return_memory = []
        for i in ix_val[1]:
            (IX_EXP_TYPE, ix_exp) = i

            if IX_EXP_TYPE == 'integer':
                return_memory.append(memory[ix_exp])
            else:
                raise ValueError("unsupported list index")

        if structure_val[0] == 'string':
            return ('string',"".join(return_memory))
        else:
            return ('list', return_memory)

    else:
        raise ValueError("index op '{}' not supported".format(ix_val[0]))

#########################################################################
# we are indexing into the memory of either a list or an object to
# write into the memory.
def store_at_ix(structure_val, ix, value):
    # find the actual memory we need to access
    # for lists it is just the python list
    if structure_val[0] == 'list':
        memory = structure_val[1]
        # compute the index
        ix_val = walk(ix)

    # for objects we access the object memory
    elif structure_val[0] == 'object':
        (OBJECT,
         (STRUCT_ID, (ID, struct_id)),
         (MEMBER_NAMES, (LIST, member_names)),
         (OBJECT_MEMORY, (LIST, memory))) = structure_val
        if ix[0] == 'id' and ix[1] in member_names:
            ix_val = ('integer', member_names.index(ix[1]))
        else:
            ix_val = walk(ix)

    elif structure_val[0] == 'module':
        (MODULE, id, (CLOSURE, closure)) = structure_val
        config = state.symbol_table.get_config()
        state.symbol_table.set_config(closure)
        # execute an assignment in the context of the module
        walk(('unify', ix, value))
        state.symbol_table.set_config(config)
        return

    elif structure_val[0] == 'pattern':
        # simple patterns are just structures - skip the pattern operator
        store_at_ix(structure_val[1], ix, value)
        return

    else:
        raise ValueError("term '{}' is not a mutable structure"
                         .format(term2string(structure_val)))

    # Next, we do the actual memory storage operation

    # If it's just an integer, index into that location and
    # set the value
    if ix_val[0] == 'integer':
        memory[ix_val[1]] = value
        return

    # otherwhise, if the index is a list
    elif ix_val[0] == 'list':

        # Make sure the rval is a list
        if value[0] != 'list':
            raise ValueError('pattern slicing needs values to be a list')
        elif value[0] == 'list' and (len(ix_val[1]) != len(value[1])):
            raise ValueError('pattern slicing needs indexes and values of equal length')

        # Get the l/rval
        (LIST, lval) = ix_val
        (LIST_r, rval) = value

        # For each index
        for i in range(len(lval)):
            # Get the memory location of the lval and set it to the
            # corresponding rval value
            (INTEGER, location) = lval[i]
            memory[location] = rval[i]
        return

    else:
        raise ValueError("index op '{}' in patterns not supported"
                         .format(ix_val[0]))

#########################################################################
# set module environment
def set_module_env(modname):
    module_val = state.symbol_table.lookup_sym(modname)
    if module_val[0] != 'module':
        raise ValueError("{} is not a module".format(modname))
    (MODULE, id, (CLOSURE, closure)) = module_val
    config = state.symbol_table.get_config()
    state.symbol_table.set_config(closure)
    return config

#########################################################################
def set_config(config):
    state.symbol_table.set_config(config)

#########################################################################
# implementations for builtin operators and functions. these operators 
# and functions do not need/are not allowed to have a function 
# local scope.  therefore they are implemented here as builtins as 
# part of the interpreter proper.  for other builtins that do 
# not have this restriction see the prologue.

def handle_builtins(node):
    (APPLY, (ID, opname), args) = node
    assert_match(APPLY, 'apply')
    assert_match(ID, 'id')

    # deal with binary operators
    if opname in binary_operators:
        (TUPLE, [a,b])= args

        if opname == '__plus__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real', 'list', 'string']:
                if val_b[0] in ['integer', 'real', 'list', 'string']:
                    if val_a[0]==val_b[0]:
                        return (val_a[0], val_a[1] + val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} + {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} + {}' expected '{} + {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '+'".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '+'".format(val_a[0]))
        elif opname == '__minus__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real']:
                if val_b[0] in ['integer', 'real']:
                    if val_a[0]==val_b[0]:
                        return (val_a[0], val_a[1] - val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} - {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} - {}' expected '{} - {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '-'".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '-'".format(val_a[0]))
        elif opname == '__times__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real']:
                if val_b[0] in ['integer', 'real']:
                    if val_a[0]==val_b[0]:
                        return (val_a[0], val_a[1] * val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} * {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} * {}' expected '{} * {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '*'".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '*'".format(val_a[0]))
        elif opname == '__divide__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real']:
                if val_b[0] in ['integer', 'real']:
                    if val_a[0]==val_b[0]:
                        if val_a[0] == 'integer':
                            return ('integer', val_a[1] // val_b[1])
                        elif val_a[0] == 'real':
                            return ('real', val_a[1] / val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} / {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} / {}' expected '{} / {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '/'".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '/'".format(val_a[0]))
        elif opname == '__or__':
            # short circuit evaluation
            val_a = walk(a)
            if val_a[0] == 'boolean':
                if val_a[1] == True:
                    return ('boolean', True)
            else:
                raise ValueError(
                    "found '{} expected 'boolean and boolean'"
                    .format(val_a[0]))
            val_b = walk(b)
            if val_b[0] == 'boolean':
                return ('boolean', val_a[1] or val_b[1])
            else:
                raise ValueError(
                    "found '{} and {}' expected 'boolean and boolean'"
                    .format(val_a[0],val_b[0]))
        elif opname == '__and__':
            # short circuit evaluation
            val_a = walk(a)
            if val_a[0] == 'boolean':
                if val_a[1] == False:
                    return ('boolean', False)
            else:
                raise ValueError(
                    "found '{} expected 'boolean and boolean'"
                    .format(val_a[0]))
            val_b = walk(b)
            if val_b[0] == 'boolean':
                return ('boolean', val_a[1] and val_b[1])
            else:
                raise ValueError(
                    "found '{} and {}' expected 'boolean and boolean'"
                    .format(val_a[0],val_b[0]))
        elif opname == '__eq__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real', 'list', 'tuple', 'boolean', 'string', 'none']:
                if val_b[0] in ['integer', 'real', 'list', 'tuple', 'boolean', 'string', 'none']:
                    if val_a[0]==val_b[0]:
                        if val_a[0] == 'real' and val_a[1] != val_b[1] and isclose(val_a[1],val_b[1]):
                            warning("possible rounding error issue")
                        return ('boolean', val_a[1] == val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} == {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} == {}' expected '{} == {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '=='".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '=='".format(val_a[0]))
        elif opname  == '__ne__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real', 'list', 'tuple', 'boolean', 'string', 'none']:
                if val_b[0] in ['integer', 'real', 'list', 'tuple', 'boolean', 'string', 'none']:
                    if val_a[0]==val_b[0]:
                        if val_a[0] == 'real' and val_a[1] != val_b[1] and isclose(val_a[1],val_b[1]):
                            warning("possible rounding error issue")
                        return ('boolean', val_a[1] != val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} =/= {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} =/= {}' expected '{} =/= {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '=/='".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '=/='".format(val_a[0]))
        elif opname == '__le__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real', 'string']:
                if val_b[0] in ['integer', 'real', 'string']:
                    if val_a[0]==val_b[0]:
                        if val_a[0] == 'real' and val_a[1] != val_b[1] and isclose(val_a[1],val_b[1]):
                            warning("possible rounding error issue")
                        return ('boolean', val_a[1] <= val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} <= {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} <= {}' expected '{} <= {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '<='".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '<='".format(val_a[0]))
        elif opname == '__lt__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real', 'string']:
                if val_b[0] in ['integer', 'real', 'string']:
                    if val_a[0]==val_b[0]:
                        if val_a[0] == 'real' and val_a[1] != val_b[1] and isclose(val_a[1],val_b[1]):
                            warning("possible rounding error issue")
                        return ('boolean', val_a[1] < val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} < {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} < {}' expected '{} < {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '<'".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '<'".format(val_a[0]))
        elif opname == '__ge__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real', 'string']:
                if val_b[0] in ['integer', 'real', 'string']:
                    if val_a[0]==val_b[0]:
                        if val_a[0] == 'real' and val_a[1] != val_b[1] and isclose(val_a[1],val_b[1]):
                            warning("possible rounding error issue")
                        return ('boolean', val_a[1] >= val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} >= {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} >= {}' expected '{} >= {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '>='".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '>='".format(val_a[0])) 
        elif opname == '__gt__':
            val_a = walk(a)
            val_b = walk(b)
            if val_a[0] in ['integer', 'real', 'string']:
                if val_b[0] in ['integer', 'real', 'string']:
                    if val_a[0]==val_b[0]:
                        if val_a[0] == 'real' and val_a[1] != val_b[1] and isclose(val_a[1],val_b[1]):
                            warning("possible rounding error issue")
                        return ('boolean', val_a[1] > val_b[1])
                    else:
                        type = promote(val_a[0], val_b[0])
                        if type == None:
                            raise ValueError(
                                "operation '{} > {}' not supported"
                                .format(val_a[0],val_b[0]))
                        else:
                            raise ValueError(
                                "found '{} > {}' expected '{} > {}'"
                                .format(val_a[0],val_b[0],type,type))
                else:
                    raise ValueError("unsupported type '{}' in '>'".format(val_b[0]))
            else:
                raise ValueError("unsupported type '{}' in '>'".format(val_a[0])) 
        else:
            raise ValueError("unknown builtin binary operator '{}'".format(opname))

    # deal with unary operators
    elif opname in unary_operators:
        arg_val = walk(args)             
        if opname == '__not__':
            if arg_val[0] == 'boolean':
                if arg_val[1] == False:
                    return ('boolean', True)
                else:
                    return ('boolean', False)
            else:
                raise ValueError("found 'not {}' expected 'not boolean'"
                                 .format(arg_val[0]))
        elif opname == '__uminus__':
            if arg_val[0] in ['integer', 'real']:
                return (arg_val[0], - arg_val[1])
            else:
                raise ValueError(
                    "unsupported type '{}' in unary minus"
                    .format(arg_val[0]))
        elif opname == '__uplus__':
            if arg_val[0] in ['integer', 'real']:
                return (arg_val[0], + arg_val[1])
            else:
                raise ValueError(
                    "unsupported type '{}' in unary plus"
                    .format(arg_val[0]))
        elif opname == 'assert':
            if arg_val[0] != 'boolean':
                raise ValueError('the assert operator expected a Boolean value')
            if not arg_val[1]:
                raise ValueError('assert failed')
            else:
                return ('none', None)
        elif opname == 'escape':
            global __retval__
            __retval__ = ('none', None)
            if arg_val[0] != 'string':
                raise ValueError('expected a string as argument to the escape operator')
            exec(arg_val[1])
            return __retval__
        elif opname == 'eval':
            if arg_val[0] == 'string':
                import frontend
                parser = frontend.Parser(filename="<eval>")
                eval_ast = parser.parse(arg_val[1])
                walk(eval_ast)
                return function_return_value[-1]
            else:
                raise ValueError('expected a string as argument to the eval operator')

        else:
            raise ValueError("unknown builtin unary operator '{}'".format(opname))

    # deal with nullary operators
    elif opname in nullary_operators:
        (type, _) = walk(args)
        if type != 'none':
            raise ValueError("{} is a nullary operator".format(opname))
        if opname == 'toplevel':
            return ('boolean', state.mainmodule == state.lineinfo[0])
        else:
            raise ValueError("unknown builtin nullary operator '{}'".format(opname))
        

#########################################################################
def pop_stackframe(error_trace=False): 
    # pop frame off the stack
    state.symbol_table.pop_scope()
    state.symbol_table.set_config(state.symbol_table.saved_configs.pop())
    if error_trace:
        state.error_trace = copy(state.trace_stack)
    state.trace_stack.pop()

#########################################################################
def handle_call(obj_ref, fval, actual_val_args, fname):

    # function calls transfer control - save our caller's lineinfo
    # we save the debug information here to preserve lineinfo between
    # function calls between files.
    old_lineinfo = state.lineinfo

    (FUNCTION_VAL, body_list, closure) = fval
    assert_match(FUNCTION_VAL, 'function-val')

    state.trace_stack.append((state.lineinfo[0],
                              state.lineinfo[1],
                              fname))

    # static scoping for functions
    # Note: we have to do this here because unifying
    # over the body patterns can introduce variable declarations,
    # think conditional pattern matching.
    # Note: we are keeping a stack of configs so that
    # the debugger can look at contents of 
    # Asteroid stack frames
    state.symbol_table.saved_configs.append(
        state.symbol_table.get_config()
    )
    state.symbol_table.set_config(closure)
    state.symbol_table.push_scope({})

    # if we have an obj reference bind it to the
    # variable 'this'
    if obj_ref:
        state.symbol_table.enter_sym('this', obj_ref)

    # iterate over the bodies to find one that unifies with the actual parameters
    (BODY_LIST, (LIST, body_list_val)) = body_list
    unified = False

    for i in range(0, len(body_list_val), 2):
        # Process lineinfo
        lineinfo = body_list_val[ i ]
        process_lineinfo(lineinfo)

        # Deconstruct function body
        (BODY,
          (PATTERN, p),
          stmts) = body_list_val[ i + 1]

        try:
            # Attempt to unify the actual args and the pattern
            unifiers = unify(actual_val_args, p)
            unified = True
        except PatternMatchFailed:
            unifiers = []
            unified = False

        if unified:
            break

    if not unified:
        raise ValueError("actual argument '{}' not recognized by function '{}'"
                         .format(term2string(actual_val_args),fname))
    declare_formal_args(unifiers)

    # OWM: The following segment is a repeat of the bottom of this function.
    # We need to do this because redundant patterns can break scope and
    # some debugger and state features as they exit computation.

    # Check for useless patterns
    try:
        if state.eval_redundancy:
            check_redundancy(body_list, fname)

    # Reset settings
    except RedundantPatternFound as r:
        # restore caller's env
        state.lineinfo = old_lineinfo
        pop_stackframe()
        raise r

    # execute the function
    if state.debugger: state.debugger.enter_function(fname)
    global function_return_value
    try:
        function_return_value.append(None)
        walk(stmts)
        val = function_return_value.pop()
        if val:
            return_value = val
        else:
            return_value = ('none', None)

    except ReturnValue as val:
        # we got here because a return statement threw a return object
        function_return_value.pop()
        return_value = val.value

    except Exception as e:
        # we got some other kind of exception within the function call
        # clean up our runtime stack and rethrow
        # Note: do not reset lineinfo, this way the state points at the source 
        # of the exception
        pop_stackframe(error_trace=True)
        raise e

    # all done with function call -- clean up and exit
    # restore caller's env
    if state.debugger: state.debugger.exit_function(fname)
    state.lineinfo = old_lineinfo
    pop_stackframe()
    return return_value

#########################################################################
def declare_unifiers(unifiers):
    # walk the unifiers and bind name-value pairs into the symtab

    # TODO: check for repeated names in the unfiers
    for unifier in unifiers:

        #lhh
        #print("unifier: {}".format(unifier))
        (lval, value) = unifier

        if lval[0] == 'id':
            if lval[1] == 'this':
                raise ValueError("'this' is a reserved word")
            state.symbol_table.enter_sym(lval[1], value)

        elif lval[0] == 'index': # list/structure lval access
            # Note: structures have to be declared before index access
            # can be successful!!  They have to be declared so that there
            # is memory associated with the structure.

            (INDEX, structure, ix) = lval
            # look at the semantics of 'structure'
            structure_val = walk(structure)
            # indexing/slicing
            # update the memory of the object.
            store_at_ix(structure_val, ix, value)

        else:
            raise ValueError("unknown unifier type '{}'".format(lval[0]))
    
#########################################################################
# node functions
#########################################################################
def stmt_list(node):

    (STMT_LIST, stmts) = node
    assert_match(STMT_LIST, 'stmt-list')
    
    walk(stmts)

#########################################################################
def exp_stmt(node):
    if state.debugger: state.debugger.step()

    (EXP_STMT, exp) = node
    assert_match(EXP_STMT,'exp-stmt')

    walk(exp)
    # statements don't return values
    return

#########################################################################
def global_stmt(node):
    if state.debugger: state.debugger.step()

    (GLOBAL, (LIST, id_list)) = node
    assert_match(GLOBAL, 'global')
    assert_match(LIST, 'list')

    global_str = ""

    for id_tuple in id_list:
        (ID, id_val) = id_tuple
        if state.symbol_table.is_symbol_local(id_val):
            raise ValueError("'{}' is already local, cannot be declared global"
                             .format(id_val))
        state.symbol_table.enter_global(id_val)
        global_str += "{}, ".format(id_val)
 
#########################################################################
def unify_stmt(node):
    if state.debugger: state.debugger.step()

    (UNIFY, pattern, exp) = node
    assert_match(UNIFY, 'unify')

    term = walk(exp)
    unifiers = unify(term, pattern)
    declare_unifiers(unifiers)

#########################################################################
def return_stmt(node):
    if state.debugger: state.debugger.step()

    (RETURN, e) = node
    assert_match(RETURN, 'return')
    
    retval = walk(e)

    raise ReturnValue(retval)

#########################################################################
def break_stmt(node):
    if state.debugger: state.debugger.step()

    (BREAK,) = node
    assert_match(BREAK, 'break')

    raise Break()

#########################################################################
def throw_stmt(node):
    if state.debugger: state.debugger.step()

    (THROW, object) = node
    assert_match(THROW, 'throw')
    
    throw_object = walk(object)

    raise ThrowValue(throw_object)

#########################################################################
def try_stmt(node):
    if state.debugger: state.debugger.step()

    (TRY,
     try_stmts,
     (CATCH_LIST, (LIST, catch_list))) = node

    try:
        walk(try_stmts)

    # NOTE: in Python the 'as inst' variable is only local to the catch block???
    # NOTE: we map user visible Python exceptions into standard Asteroid exceptions
    #       by constructing Exception objects - see prologue.ast
    except ThrowValue as inst:
        except_val = inst.value
        inst_val = inst

    except ReturnValue as inst:
        # return values should never be captured by user level try stmts - rethrow
        raise inst

    except PatternMatchFailed as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('member-names', ('list',["kind","val","__init__"])),
                         ('object-memory',
                          ('list',
                           [('string', 'PatternMatchFailed'),
                            ('string', inst.value)])))
        inst_val = inst

    except RedundantPatternFound as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('member-names', ('list',["kind","val","__init__"])),
                         ('object-memory',
                          ('list',
                           [('string', 'RedundantPatternFound'),
                            ('string', str(inst))])))
        inst_val = inst

    except NonLinearPatternError as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('member-names', ('list',["kind","val","__init__"])),
                         ('object-memory',
                          ('list',
                           [('string', 'NonLinearPatternError'),
                            ('string', str(inst))])))
        inst_val = inst

    except ArithmeticError as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('member-names', ('list',["kind","val","__init__"])),
                         ('object-memory',
                          ('list',
                           [('string', 'ArithmeticError'),
                            ('string', str(inst))])))
        inst_val = inst

    except FileNotFoundError as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('member-names', ('list',["kind","val","__init__"])),
                         ('object-memory',
                          ('list',
                           [('string', 'FileNotFound'),
                            ('string', str(inst))])))
        inst_val = inst

    except Exception as inst:
        # mapping general Python exceptions into Asteroid's SystemError
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('member-names', ('list',["kind","val","__init__"])),
                         ('object-memory',
                          ('list',
                           [('string', 'SystemError'),
                            ('string', str(inst))])))
        inst_val = inst

    else:
        # no exceptions found in the try statements
        return

    # we had an exception - walk the catch list and find an appropriate set of
    # catch statements.
    for catch_val in catch_list:
        (CATCH,
         (CATCH_PATTERN, catch_pattern),
         catch_stmts) = catch_val
        try:
            unifiers = unify(except_val, catch_pattern)
        except PatternMatchFailed:
            pass
        else:
            # handler found - null out error_trace
            state.error_trace = None
            declare_unifiers(unifiers)
            walk(catch_stmts)
            return

    # no exception handler found - rethrow the exception
    raise inst_val

#########################################################################
def loop_stmt(node):
    if state.debugger: state.debugger.step()

    (LOOP, body_stmts) = node
    assert_match(LOOP, 'loop')

    try:
        while True:
            walk(body_stmts)
    except Break:
        pass

#########################################################################
def while_stmt(node):
    if state.debugger: state.debugger.step()

    (WHILE, (COND_EXP, cond), body_stmts) = node
    assert_match(WHILE, 'while')

    try:
        (cond_type, cond_val) = walk(cond)
        if cond_type != 'boolean':
            raise ValueError("found '{}' expected 'boolean' in while loop"
                             .format(cond_type))
        while cond_val:
            walk(body_stmts)
            (cond_type, cond_val) = walk(cond)
            if cond_type != 'boolean':
                raise ValueError("found '{}' expected 'boolean' in while loop"
                                .format(cond_type))
    except Break:
        pass

#########################################################################
def repeat_stmt(node):
    if state.debugger: state.debugger.step()

    (REPEAT, body_stmts, (COND_EXP, cond)) = node
    assert_match(REPEAT, 'repeat')

    try:
        while True:
            walk(body_stmts)
            (cond_type, cond_val) = walk(cond)
            if cond_type != 'boolean':
                raise ValueError("found '{}' expected 'boolean' in repeat loop"
                                .format(cond_type))
            if cond_val:
                break

    except Break:
        pass

#########################################################################
def for_stmt(node):
    if state.debugger: state.debugger.step()

    (FOR, (IN_EXP, in_exp), stmt_list) = node
    assert_match(FOR, 'for')

    (IN, pattern, list_term) = in_exp

    # expand the list_term
    (LIST_TYPE, list_val) = walk(list_term)
    if LIST_TYPE not in ['list','string','tuple']:
        raise ValueError("iteration not supported for type '{}'".format(LIST_TYPE))

    # if it is a string turn the list_val into a list of Asteroid characters.
    if LIST_TYPE == 'string':
        new_list = []
        for c in list_val:
            new_list.append(('string',c))
        list_val = new_list

    # for each term on the list unfiy with pattern, declare the bound variables,
    # and execute the loop body in that context
    # NOTE: just like Python, loop bodies do not create a new scope!
    # NOTE: we can use unification as a filter of elements:
    #
    #      for (2,y) in [(1,11), (1,12), (1,13), (2,21), (2,22), (2,23)]  do
    #             print y.
    #      end for
    try:
        for term in list_val:
            try:
                unifiers = unify(term,pattern)
            except PatternMatchFailed:
                pass
            else:
                declare_unifiers(unifiers)
                walk(stmt_list)
                
    except Break:
        pass

#########################################################################
def match_stmt(node):
    if state.debugger: state.debugger.step()

    (MATCH, val, if_clauses) = node
    assert_match(MATCH, 'match')

    walk(val)
    walk(if_clauses)
    
#########################################################################
def if_stmt(node):
    if state.debugger: state.debugger.step()

    (IF, (LIST, if_list)) = node
    assert_match(IF, 'if')
    assert_match(LIST, 'list')

    for i in range(0,len(if_list),2):

        lineinfo = if_list[ i ]
        process_lineinfo(lineinfo)

        (IF_CLAUSE,
         (COND, cond),
         stmts) = if_list[ i + 1 ]

        (cond_type, cond_val) = walk(cond)
        if cond_type != 'boolean':
            raise ValueError("found '{}' expected 'boolean' in if clause"
                            .format(cond_type))

        if cond_val:
            walk(stmts)
            break

#########################################################################
def struct_def_stmt(node):
    if state.debugger: state.debugger.step()

    (STRUCT_DEF, (ID, struct_id), (MEMBER_LIST, (LIST, member_list))) = node
    assert_match(STRUCT_DEF, 'struct-def')
    assert_match(ID, 'id')
    assert_match(MEMBER_LIST, 'member-list')
    assert_match(LIST, 'list')

    # declare members
    # member names are declared as variables whose value is the slot
    # in a struct object
    struct_memory = [] # this will serve as a template for instanciating objects
    member_names = []

    for member_ix in range(len(member_list)):
        member = member_list[member_ix]
        if member[0] == 'data':
            (DATA, (ID, member_id)) = member
            struct_memory.append(('none', None))
            member_names.append(member_id)

        elif member[0] == 'unify':
            (UNIFY, (ID, member_id), function_exp) = member
            # Note: we have to bind a function VALUE into the structure memory
            function_val = walk(function_exp)
            struct_memory.append(function_val)
            member_names.append(member_id)

        elif member[0] == 'noop':
            pass
        else:
            raise ValueError("unsupported struct member '{}'".format(member[0]))

    struct_type = ('struct',
                  ('member-names', ('list', member_names)),
                  ('struct-memory', ('list', struct_memory)))

    state.symbol_table.enter_sym(struct_id, struct_type)

#########################################################################
def module_def_stmt(node):

    (MODULE_DEF, (ID, modname), stmts) = node
    assert_match(MODULE_DEF, 'module-def')


    state.symbol_table.push_scope({})
    if state.debugger: state.debugger.enter_module(modname)
    walk(stmts)
    closure = state.symbol_table.get_closure()
    if state.debugger: state.debugger.exit_module(modname)
    state.symbol_table.pop_scope()

    module_type = ('module', ('id', modname), ('scope', closure))
    state.symbol_table.enter_sym(modname, module_type)

#########################################################################
def load_stmt(node):
    if state.debugger: state.debugger.step()

    (LOAD_STMT, inlist) = node
    assert_match(LOAD_STMT, 'load-stmt')

    walk(inlist)

    return

#########################################################################
def apply_exp(node):
    (APPLY, f, arg) = node
    assert_match(APPLY, 'apply')

    # handle builtin operators that look like apply lists.
    if f[0] == 'id' and f[1] in builtins:
        return handle_builtins(node)

    # handle function application
    # retrieve the function name from the AST
    if f[0] in ['function-exp','apply']:
        # cannot use the function expression as a name,
        # could be a very complex computation. the apply
        # node means that the lambda function has to still be
        # computed.
        f_name = 'lambda'
    elif f[0] == 'index':
        # member/module function
        (INDEX, ix, (ID, f_name)) = f
        if not isinstance(f_name, str):
            raise ValueError("function names have to be strings")
    else:
        # just a regular function call
        (ID, f_name) = f

    # evaluate the function expression and the arguments
    f_val = walk(f)
    arg_val = walk(arg)

    # object member function
    # NOTE: object member functions are passed an object reference.
    if f_val[0] == 'member-function-val':
        (MEMBER_FUNCTION_VAL, obj_ref, function_val) = f_val
        # Note: lists and strings are objects/mutable data structures, they
        # have member functions defined in the Asteroid prologue.
        result = handle_call(obj_ref,
                             function_val,
                             arg_val,
                             f_name)

    # regular function call
    elif f_val[0] == 'function-val':
        result = handle_call(None, f_val, arg_val, f_name)

    # object constructor call
    elif f_val[0] == 'struct':
        if f[0] == 'index':
            # constructor name qualified with a module name
            (INDEX, (ID, modname), (ID, struct_id)) = f
        else: 
            # inscope constructor call
            modname = None
            (ID, struct_id) = f
        (STRUCT,
         (MEMBER_NAMES, (LIST, member_names)),
         (STRUCT_MEMORY, (LIST, struct_memory))) = f_val

        # create our object memory - memory cells now have initial values
        # we use structure memory as an init template
        object_memory = struct_memory.copy()
        # create our object
        obj_ref = ('object',
                   ('struct-id', ('id', struct_id)),
                   ('member-names', ('list', member_names)),
                   ('object-memory', ('list', object_memory)))
        # if the struct has an __init__ function call it on the object
        # NOTE: constructor functions do not have return values.

        if '__init__' in member_names:
            slot_ix = member_names.index('__init__')
            init_fval = struct_memory[slot_ix]
            # calling a member function
            if modname:
                orig_config = set_module_env(modname)
            handle_call(obj_ref,
                        init_fval,
                        arg_val,
                        f_name)
            if modname:
                set_config(orig_config)
        # the struct does not have an __init__ function but
        # we have a constructor call with args, e.g. Foo(1,2)
        # try to apply a default constructor by copying the
        # values from the arg list to the data slots of the object
        elif arg_val[0] != 'none':
            if arg_val[0] != 'tuple':
                arg_array = [arg_val]
            else:
                arg_array = arg_val[1]
            data_memory = data_only(object_memory)
            if len(data_memory) != len(arg_array):
                raise ValueError(
                    "default constructor expected {} argument{} got {}"
                    .format(len(data_memory),
                            "" if len(data_memory) else "s",
                            len(arg_array)))
            # copy initializers into object memory
            data_ix = data_ix_list(object_memory)
            for (i,k) in zip(data_ix, range(0,len(data_memory))):
                object_memory[i] = arg_array[k]

        # return the new object
        result = obj_ref

    else:
        raise ValueError("term '{}' is not a function, did you forget the end-of-line period?"
                         .format(term2string(f_val)))

    return result

#########################################################################
def index_exp(node):

    (INDEX, structure, ix) = node
    assert_match(INDEX, 'index')

    # look at the semantics of 'structure'
    structure_val = walk(structure)

    # indexing/slicing
    result = read_at_ix(structure_val, ix)
    #lhh
    #from pprint import pprint
    #pprint(structure_val)

    return result

#########################################################################
def list_exp(node):

    (LIST, inlist) = node
    assert_match(LIST, 'list')

    outlist =[]

    for e in inlist:
        outlist.append(walk(e))

    return ('list', outlist)

#########################################################################
def tuple_exp(node):

    (TUPLE, intuple) = node
    assert_match(TUPLE, 'tuple')

    outtuple = []

    for e in intuple:
        outtuple.append(walk(e))

    return ('tuple', outtuple)

#########################################################################
def is_exp(node):

    (IS, term, pattern) = node
    assert_match(IS, 'is')

    term_val = walk(term)

    try:
        unifiers = unify(term_val, pattern)
    except PatternMatchFailed:
        return ('boolean', False)
    else:
        declare_unifiers(unifiers)
        return ('boolean', True)

#########################################################################
def in_exp(node):

    (IN, exp, exp_list) = node
    assert_match(IN, 'in')

    exp_val = walk(exp)
    (EXP_LIST_TYPE, exp_list_val, *_) = walk(exp_list)

    if EXP_LIST_TYPE != 'list':
        raise ValueError("right argument to 'in' operator has to be a list")

    # we simply map our in operator to the Python in operator
    if exp_val in exp_list_val:
        return ('boolean', True)
    else:
        return ('boolean', False)

#########################################################################
def if_exp(node):

    (IF_EXP, cond_exp, then_exp, else_exp) = node
    assert_match(IF_EXP, 'if-exp')

    # if expressions without an else clause are only allowed in
    # conditional patterns.
    if else_exp[0] == 'null':
        raise ValueError("if expressions need an 'else' clause")

    (cond_type, cond_val) = walk(cond_exp)
    if cond_type != 'boolean':
        raise ValueError("found '{}' expected 'boolean' in if expression"
                        .format(cond_type))
    if cond_val:
        return walk(then_exp)
    else:
        return walk(else_exp)

#########################################################################
# NOTE: 'to-list' is not a semantic value and should never appear in
#       any tests.  It is a constructor and should be expanded by the
#       walk function before semantic processing.
def to_list_exp(node):

    (TOLIST,
     (START, start),
     (STOP, stop),
     (STEP, step)) = node

    assert_match(TOLIST, 'to-list')
    assert_match(START, 'start')
    assert_match(STOP, 'stop')
    assert_match(STEP, 'step')

    (START_TYPE, start_val, *_) = walk(start)
    (STOP_TYPE, stop_val, *_) = walk(stop)
    (STEP_TYPE, step_val, *_) = walk(step)

    if START_TYPE != 'integer' or STOP_TYPE != 'integer' or STEP_TYPE != 'integer':
        raise ValueError("only integer values allowed in start, stop, or step")

    out_list_val = []

    if int(step_val) > 0: # generate the list
        ix = int(start_val)
        while ix <= int(stop_val):
            out_list_val.append(('integer', ix))
            ix += int(step_val)

    elif int(step_val) == 0: # error
        raise ValueError("step size of 0 not supported")

    elif int(step_val) < 0: # generate the list
        ix = int(start_val)
        while ix >= int(stop_val):
            out_list_val.append(('integer', ix))
            ix += int(step_val)

    else:
        raise ValueError("{} not a valid step value".format(step_val))

    return ('list', out_list_val)

#########################################################################
# NOTE: this is the value view of the head tail constructor, for the
#       pattern view of this constructor see unify.
def head_tail_exp(node):

    (HEAD_TAIL, head, tail) = node
    assert_match(HEAD_TAIL, 'head-tail')

    head_val = walk(head)
    (TAIL_TYPE, tail_val) = walk(tail)

    if TAIL_TYPE != 'list':
        raise ValueError(
            "unsupported tail type '{}' in head-tail operator".
            format(TAIL_TYPE))

    return ('list', [head_val] + tail_val)

#########################################################################
# turn a function expression into a closure.
def function_exp(node):

    (FUNCTION_EXP, body_list) = node
    assert_match(FUNCTION_EXP,'function-exp')

    return ('function-val',
            body_list,
            state.symbol_table.get_closure())

#########################################################################
def process_lineinfo(node):

    (LINEINFO, lineinfo_val) = node
    assert_match(LINEINFO, 'lineinfo')

    state.lineinfo = lineinfo_val

#########################################################################
def illegal_exp(_):
    # we got here because we tried to use a non-trivial pattern as 
    # constructors.
    raise ValueError("not a valid expression")

#########################################################################
def set_ret_val(node):
    (SET_RET_VAL, exp) = node
    assert_match(SET_RET_VAL,'set-ret-val')

    global function_return_value
    val = walk(exp)
    function_return_value.pop()
    function_return_value.append(val)

    return

#########################################################################
def clear_ret_val(node):
    (CLEAR_RET_VAL,) = node
    assert_match(CLEAR_RET_VAL, 'clear-ret-val')

    global function_return_value    
    # If we have no function return value, then append None    
    if function_return_value != []:
        function_return_value.pop()       
    function_return_value.append(None)
    return

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
        raise ValueError("feature '{}' not yet implemented".format(type))

#########################################################################
# a dictionary to associate tree nodes with node functions
dispatch_dict = {
    # statements - statements do not produce return values
    'load-stmt'     : load_stmt,
    'stmt-list'     : stmt_list,
    'lineinfo'      : process_lineinfo,
    'set-ret-val'   : set_ret_val,
    'clear-ret-val' : clear_ret_val,
    'exp-stmt'      : exp_stmt,
    'noop'          : lambda node : None,
    'unify'         : unify_stmt,
    'while'         : while_stmt,
    'loop'          : loop_stmt,
    'repeat'        : repeat_stmt,
    'for'           : for_stmt,
    'global'        : global_stmt,
    'return'        : return_stmt,
    'break'         : break_stmt,
    'match'         : match_stmt,
    'if'            : if_stmt,
    'throw'         : throw_stmt,
    'try'           : try_stmt,
    'struct-def'    : struct_def_stmt,
    'module-def'    : module_def_stmt,  # this is part of the load statement
    # expressions - expressions do produce return values
    'list'          : list_exp,
    'tuple'         : tuple_exp,
    'to-list'       : to_list_exp,
    'head-tail'     : head_tail_exp,
    'raw-to-list'   : lambda node : walk(('to-list', node[1], node[2], node[3])),
    'raw-head-tail' : lambda node : walk(('head-tail', node[1], node[2])),
    'none'          : lambda node : node,
    'nil'           : lambda node : node,
    'function-exp'  : function_exp,
    'string'        : lambda node : node,
    'integer'       : lambda node : node,
    'real'          : lambda node : node,
    'boolean'       : lambda node : node,
    'object'        : lambda node : node,
    'pattern'       : lambda node : node,
    'scope'         : illegal_exp,
    'typematch'     : illegal_exp,
    # type tag used in conjunction with escaped code in order to store
    # foreign objects in Asteroid data structures
    'foreign'       : lambda node : node,
    'id'            : lambda node : state.symbol_table.lookup_sym(node[1]),
    'apply'         : apply_exp,
    'index'         : index_exp,
    'is'            : is_exp,
    'in'            : in_exp,
    'if-exp'        : if_exp,
    'member-function-val' : lambda node : node,
    'deref'         : illegal_exp,
}

##############################################################################################
# *** The Redundant Pattern Detector ***
#
# Evaluates the presence of redundant, or 'useless', pattern clauses in an Asteroid function:
#
# A redundant, or 'useless', pattern is defined as a pattern which can never be matched
# due to a preceeding pattern consuming all intended pattern matches.
#
# Consider the following Asteroid function:
#
# function testFunction
#   with (x,y) do
#       return 1.
#   orwith (x,1) do
#      return 2.
#   end function.
#
# In the above function, the pattern (x,1) can never be reached as the preceeding pattern (x,y)
# will consume all intended matches. Therefore, it is redundant.
#
# Function check_redundancy takes in a functions body list during parsing.
# This body list contains a functions patterns along with the associated bodies for each
# pattern. This function then evaluates if patterns exist within the passed in function that
# are redundant. If so, a warning is printed to the console identifing the offending
# pattern(s)
#
################################################################################################
def check_redundancy( body_list, f_name ):

    #Node type assertions
    #or "Make sure we are walking down the right part of the tree"
    (BODY_LIST, function_bodies ) = body_list
    assert_match(BODY_LIST,'body-list')
    (LIST, bodies) = function_bodies
    assert_match(LIST,'list')

    #compare every pattern with the patterns that follow it
    for i in range(0, len(bodies), 2):
        # Process lineinfo
        lineinfo = bodies[ i ]
        process_lineinfo(lineinfo)

        #get the pattern with the higher level of precedence
        (BODY_H,(PTRN,ptrn_h),stmts_h) = bodies[i + 1]
        assert_match(BODY_H,'body')
        assert_match(PTRN,'body-pattern')

        for j in range(i + 2, len(bodies), 2):
            lineinfo = bodies[ j ]
            process_lineinfo(lineinfo)

            #get the pattern with the lower level of precedence
            (BODY_L,(PTRN,ptrn_l),stmts_l) = bodies[j + 1]
            assert_match(BODY_L,'body')
            assert_match(PTRN,'body-pattern')

            #Here we get line numbers in case we throw an error
            # we have to do a little 'tree walking' to get to the
            # line #, hence all the unpacking.
            (STMT_LIST,(LIST,LINE_LIST)) = stmts_l
            first_line_l = LINE_LIST[0]
            (LINE_INFO,location_l) = first_line_l

            (STMT_LIST,(LIST,LINE_LIST)) = stmts_h
            first_line_h = LINE_LIST[0]
            (LINE_INFO,location_h) = first_line_h

            # Compare the patterns to determine if the pattern with the
            # higher level of precedence will render the pattern with
            # the lower level of precedence useless/redundant by calling
            # on the unify function to evaluate the subsumption relationship
            # between the two patterns.
            try:                                #CHECK FOR CONFLICTION
                unify( ptrn_l, ptrn_h , False )
            except PatternMatchFailed:          #NO CONFLICTION
                pass
            else:                               #CONFLICTION
                raise RedundantPatternFound( ptrn_h , ptrn_l , f_name, location_h, location_l )

