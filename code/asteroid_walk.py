#########################################################################
# A tree walker to interpret Asteroid programs
#
# (c) Lutz Hamel, University of Rhode Island
#########################################################################

from asteroid_globals import *
from asteroid_support import *
from copy import deepcopy
from asteroid_state import state
from re import match as re_match

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
# this list records the current ranges/intervals for integers and real
# numbers that have been covered by patterns in a function with 
# conditional clauses. This information is used to evaluate useless 
# pattern clauses.
conditional_intervals = [False,[]]
warning = [False]

#########################################################################
__retval__ = None  # return value register for escaped code

###########################################################################################
# check if the two type tags match
def match(tag1, tag2):

    if tag1 == tag2:
        return True
    else:
        return False

###########################################################################################
def unify(term, pattern, unifying = True ):
    '''
    unify term and pattern recursively and return the unifier.
    this unification allows for the same variable to appear
    multiple times in the unifier.  the user of this
    function must take appropriate actions if this happens.

    we assume that both the term and the pattern are made up of tuple
    nodes:

             (<type>, children*)

    leaf nodes must be nullary constructors.

    NOTE: if the pattern looks like an lval then it is treated like an lval, e.g.
            let a@[0] = 'a@[0].
          stores the term 'a@[0] into lval a@[0].
    NOTE: Default argument unifying is set to be true. If we are unifying, then we are
          evaluating unification between a pattern and a term. If we are not
          unifying, then we are evaluating subsumption between two patterns for the
          purpose of detecting redundant/useless pattern clauses in functions.
    '''
    #lhh
    # print("unifying:\nterm: {}\npattern: {}\n\n".format(term, pattern))

    # if unifying:
    #     print("unifying:\nterm: {}\npattern: {}\n\n".format(term, pattern))
    # else:
    #     print("evaluating subsumption:\nterm: {}\npattern: {}\n\n".format(term, pattern))

    # 1. We don't care what the pattern is called if evaluating subsumption.
    # 2. A named patterns node(tuple) shape can get us into trouble when unpacking. This intial
    # check allows us to unpack it normally as opposed to checking each time we unpack.
    # most nodes: (1,2)
    # named-patterns: (1,2,3)
    try:
        if ((not unifying) and (term[0] == 'named-pattern')):
            term = term[2]
    except:
        pass

    ### Python value level matching
    # NOTE: in the first rules where we test instances we are comparing
    # Python level values, if they don't match exactly then we have
    # a pattern match fail.
    if isinstance(term, str): # apply regular expression match
        if isinstance(pattern, str) and re_match("^"+pattern+"$", term):
            # Note: a pattern needs to match the whole term.
            return [] # return empty unifier
        else:
            raise PatternMatchFailed(
                "regular expression {} did not match {}"
                .format(pattern, term))

    elif isinstance(term, (int, float, bool)):
        if term == pattern:
            return [] # return an empty unifier
        else:
            raise PatternMatchFailed(
                "{} is not the same as {}"
                .format(term, pattern))

    elif isinstance(term, list) or isinstance(pattern, list):
        if not(isinstance(term, list)) or not(isinstance(pattern, list)):
            raise PatternMatchFailed(
                "term and pattern do not agree on list/tuple constructor")
        elif len(term) != len(pattern):
            raise PatternMatchFailed(
                "term and pattern lists/tuples are not the same length")
        else:
            unifier = []
            for i in range(len(term)):
                if unifying:
                    unifier += unify(term[i], pattern[i])
                else:
                    unifier += unify(term[i], pattern[i], False)
            return unifier

    ### Asteroid value level matching
    elif pattern[0] == 'string' and term[0] != 'string':
        # regular expression applied to a non-string structure
        # this is possible because all data types are subtypes of string
        return unify(term2string(term), pattern[1])

    elif pattern[0] == 'cmatch':
        global tag_number
        tag_number = 0

        # If we are evaluating subsumtion
        if not unifying:
            # Unpack the pattern-side argument
            (CMATCH, pexp, cond_exp) = pattern
            (APPLY,(ID,function_name),apply_list)= cond_exp
            (TUPLE,term_list) = apply_list

            if function_name[0] == '_': #if we are looking at a relational operation

                if not conditional_intervals[0]:#if this is the first time looking
                                                #at this relational operation
                    update_numberline(pexp,function_name,apply_list)

            if term[0] == 'cmatch':

                # Unpack the term-side argument
                (CMATCH, pexp, cond_exp) = term
                (APPLY,(ID,function_name),apply_list)= cond_exp

                # Reset the global id tag index which is used in evaluating 
                # subsumption between condtional pattern clauses with relational operations.
                tag_number = 0

                if function_name[0] == '_': #If we are evaluating two relational condtional
                                            #patterns against each other
                    if check_numberline(pexp,function_name,apply_list):
                        return [] # Throws a subsumption error
                elif (term == pattern): # Identical user function conditional pattern
                    return [] # Throws a subsumption error
                else:
                    # Function warning should only print once per program execution
                    global warning
                    if not warning[0]: 
                        print("Redundant pattern detection is not supported for conditional pattern expressions with function calls.")
                        print("Only relational operations/functions are supported.")
                        print("All conditional pattern clauses which reference functions will be ignored.")
                        warning[0] = True

            elif term[0] == 'typematch':
                if term[1] in ['integer','real']:
                    var_name = (apply_list[1])[0]
                    if is_completely_covered(pexp,var_name):
                        return [] # It will be subsumed completely

            raise PatternMatchFailed(
                "Subsumption relatioship broken, pattern will not be rendered redundant.")

        (CMATCH, pexp, cond_exp) = pattern

        if unifying:
            unifiers = unify(term, pexp)
        else:
            unifiers = unify(term, pexp, False)

        # evaluate the conditional expression in the
        # context of the unifiers.
        #state.symbol_table.push_scope({})
        declare_unifiers(unifiers)
        bool_val = map2boolean(walk(cond_exp))
        #state.symbol_table.pop_scope()

        if bool_val[1]:
            return unifiers
        else:
            raise PatternMatchFailed(
                "conditional pattern match failed")

    elif term[0] == 'cmatch':
        # We will only get here when evaluating subsumption

        # Regardless if the patterns will subsume of not, if the conditional
        # is before the non-conditional pattern, the patterns are correctly
        # ordered in the function.
        (CMATCH, pexp, cond_exp) = term
        return unify(pexp,pattern,False) 

    elif pattern[0] == 'typematch':
        typematch = pattern[1]
        nextIndex = 0 #indicates index of where we will 'look' next

        if typematch in ['string','real','integer','list','tuple','boolean','none']:

            if (not unifying):

                #walk a different path for this node
                if (term[0] == 'typematch'):
                    nextIndex = 1

                #handle lists/head-tails subsuming each other
                elif (term[0] in ['list','head-tail']):
                    if ((typematch == 'list') and (term[0] in ['list','head-tail'])):
                        return []

            if typematch == term[nextIndex]:
                return []
            else:
                raise PatternMatchFailed(
                    "expected typematch {} got a term of type {}"
                    .format(typematch, term[nextIndex]))

        elif typematch == 'function':
            # matching function and member function values
            if term[0] in ['function-val','member-function-val']:
                return []
            else:
                raise PatternMatchFailed(
                    "expected typematch {} got a term of type {}"
                    .format(typematch, term[0]))

        elif term[0] == 'object':
            (OBJECT,
                (STRUCT_ID, (ID, struct_id)),
                (OBJECT_MEMORY, LIST)) = term
            if struct_id == typematch:
                    return []
            else:
                raise PatternMatchFailed(
                    "expected typematch {} got an object of type {}"
                    .format(typematch, struct_id))   

    elif pattern[0] == 'named-pattern':
        # unpack pattern
        (NAMED_PATTERN, name, p) = pattern

        if unifying:
            return unify(term, p ) + [(name, term )]
        else:
            return unify(term, p, False)

    elif pattern[0] == 'none':
        if term[0] != 'none':
            raise PatternMatchFailed("expected 'none' got '{}'"
                    .format(term[0]))
        else:
            return []

    # NOTE: functions/foreign are allowed in terms as long as they are matched
    # by a variable in the pattern - anything else will fail
    elif term[0] in (unify_not_allowed - {'function-val', 'foreign'}):
        raise PatternMatchFailed(
            "term of type '{}' not allowed in pattern matching"
            .format(term[0]))

    elif pattern[0] in unify_not_allowed:
        raise PatternMatchFailed(
            "pattern of type '{}' not allowed in pattern matching"
            .format(pattern[0]))

    elif pattern[0] == 'quote':
        # quotes on the pattern side can always be ignored
        if unifying:
            return unify(term, pattern[1])
        else:
            if term[0] == 'quote':
                return unify(term[1], pattern[1], False)
            else:
                return unify(term, pattern[1], False)
    elif term[0] == 'quote' and pattern[0] not in ['id', 'index']:
        # ignore quote on the term if we are not trying to unify term with
        # a variable or other kind of lval
        if unifying:
            return unify(term[1], pattern)
        else:
            return unify(term, pattern[1], False)

    elif term[0] == 'object' and pattern[0] == 'apply':
        # unpack term
        (OBJECT,
         (STRUCT_ID, (ID, struct_id)),
         (OBJECT_MEMORY, (LIST, obj_memory))) = term
        # unpack pattern
        (APPLY,
         (ID, apply_id),
         arg) = pattern
        if struct_id != apply_id:
            raise PatternMatchFailed("expected type '{}' got type '{}'"
                .format(apply_id, struct_id))
        # we are comparing raw lists here
        if arg[0] == 'tuple':
            pattern_list = arg[1]
        else:
            pattern_list = [arg]
        # only pattern match on object data members
        if unifying:
            return unify(data_only(obj_memory), pattern_list)
        else:
            return unify(data_only(obj_memory), pattern_list, False)

    elif pattern[0] == 'index': # list element lval access
        unifier = (pattern, term)
        return [unifier]

    elif term[0] == 'id' and unifying: # variable in term not allowed
        raise PatternMatchFailed(      # when unifying
            "variable '{}' in term not allowed"
            .format(term[1]))

    elif pattern[0] == 'id': # variable in pattern add to unifier
        sym = pattern[1]
        if sym == '_': # anonymous variable - ignore unifier
            return []
        else:
            unifier = (pattern, term)
            return [unifier]

    elif pattern[0] in ['head-tail', 'raw-head-tail']:

        # if we are unifying or we are not evaluating subsumption
        #  to another head-tail
        if unifying or term[0] not in ['head-tail', 'raw-head-tail']:
            (HEAD_TAIL, pattern_head, pattern_tail) = pattern
            (LIST, list_val) = term

            if LIST != 'list':
                raise PatternMatchFailed(
                    "head-tail operator expected type 'list' got type '{}'"
                    .format(LIST))

            if not len(list_val):
                raise PatternMatchFailed(
                    "head-tail operator expected a non-empty list")

            list_head = list_val[0]
            list_tail = ('list', list_val[1:])

            unifier = []
            if unifying:
                unifier += unify(list_head, pattern_head)
                unifier += unify(list_tail, pattern_tail)
            else:
                unifier += unify(list_head, pattern_head,False)
                unifier += unify(list_tail, pattern_tail,False)
            return unifier

        else: #Else we are evaluating subsumption to another head-tail

            lengthH = head_tail_length(pattern) #H->higher order of predcence pattern
            lengthL = head_tail_length(term)    #L->lower order of predcence pattern

            if lengthH == 2 and lengthL != 2:
                return unify(pattern[1],term[1],False)

            if (lengthH > lengthL): # If the length of the higher presedence pattern is greater
                                    # then length of the lower precedence pattern, it is
                                    # not redundant
                raise PatternMatchFailed(
                    "Subsumption relatioship broken, pattern will not be rendered redundant.")

            else: #Else we continue evaluating the different terms in the head-tail pattern
                (HEAD_TAIL, patternH_head, patternH_tail) = pattern
                (HEAD_TAIL, patternL_head, patternL_tail) = term
                return unify(patternL_head,patternH_head,False) + unify(patternL_tail,patternH_tail,False)

    elif pattern[0] == 'deref':  # ('deref', ('id', sym))

        (ID, sym) = pattern[1]
        p = state.symbol_table.lookup_sym(sym)
        if unifying:
            return unify(term,p)
        else:
            if (term[0] == 'deref'):
                (ID, sym) = term[1]
                t = state.symbol_table.lookup_sym(sym)
                return unify(t,p, False)

            return unify(term,p, False)

    # builtin operators look like apply lists with operator names
    elif pattern[0] == 'apply':
        if term[0] != pattern[0]: # make sure both are applys
            raise PatternMatchFailed(
                "term and pattern disagree on 'apply' node")

        # unpack the apply structures
        (APPLY, (ID, t_id), t_arg) = term
        (APPLY, (ID, p_id), p_arg) = pattern

        # make sure apply id's match
        if t_id != p_id:
            raise PatternMatchFailed(
                "term '{}' does not match pattern '{}'"
                .format(t_id, p_id))

        # unify the args
        if unifying:
            return unify(t_arg, p_arg)
        else:
            return unify(t_arg, p_arg,False)

    elif not match(term[0], pattern[0]):  # nodes are not the same
        raise PatternMatchFailed(
            "nodes '{}' and '{}' are not the same"
            .format(term[0], pattern[0]))

    elif len(term) != len(pattern): # nodes are not of same the arity
        raise PatternMatchFailed(
            "nodes '{}' and '{}'' are not of the same arity"
            .format(term[0], pattern[0]))

    else:
        #lhh
        #print("unifying {}".format(pattern[0]))
        unifier = []
        for i in range(1,len(term)):
            if unifying:
                unifier += unify(term[i], pattern[i])
            else:
                unifier += unify(term[i], pattern[i], False)
        #lhh
        #print("returning unifier: {}".format(unifier))
        return unifier

#########################################################################
def eval_actual_args(args):

    return walk(args)

#########################################################################
def declare_formal_args(unifiers):
    # unfiers is of the format: [ (pattern, term), (pattern, term),...]

    for u in unifiers:
        (pattern, term) = u
        (ID, sym) = pattern
        if ID != 'id':
            raise ValueError("no pattern match possible in function call")
        state.symbol_table.enter_sym(sym, term)

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
         (OBJECT_MEMORY, (LIST, memory))) = structure_val
        # compute the index -- for objects this has to be done
        # in the context of the struct scope
        struct_val = state.symbol_table.lookup_sym(struct_id)
        # unpack the struct value
        (STRUCT,
         (MEMBER_NAMES, (LIST, member_names)),
         (STRUCT_MEMORY, (LIST, struct_memory)),
         (STRUCT_SCOPE, struct_scope)) = struct_val
        state.symbol_table.push_scope(struct_scope)
        ix_val = walk(ix)
        state.symbol_table.pop_scope()

    else:
        raise ValueError("'{}' is not indexable".format(structure_val[0]))

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
         (OBJECT_MEMORY, (LIST, memory))) = structure_val
        # compute the index -- for objects this has to be done
        # in the context of the struct scope
        struct_val = state.symbol_table.lookup_sym(struct_id)
        # unpack the struct value
        (STRUCT,
         (MEMBER_NAMES, (LIST, member_names)),
         (STRUCT_MEMORY, (LIST, struct_memory)),
         (STRUCT_SCOPE, struct_scope)) = struct_val
        state.symbol_table.push_scope(struct_scope)
        ix_val = walk(ix)
        state.symbol_table.pop_scope()

    else:
        raise ValueError("'{}' is not mutable a structure".format(structure_val[0]))


    # index into memory and set the value
    if ix_val[0] == 'integer':
        memory[ix_val[1]] = value
        return

    elif ix_val[0] == 'list':
        raise ValueError("slicing in patterns not supported")

    else:
        raise ValueError("index op '{}' in patterns not supported"
                         .format(ix_val[0]))

#########################################################################
def handle_builtins(node):

    (APPLY, (ID, opname), args) = node
    assert_match(APPLY, 'apply')
    assert_match(ID, 'id')

    if opname in binary_operators:
        (TUPLE, bin_args)= args
        val_a = walk(bin_args[0])
        val_b = walk(bin_args[1])

        if opname == '__plus__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real', 'list', 'boolean']:
                return (type, val_a[1] + val_b[1])
            elif type == 'string':
                return (type, term2string(val_a) + term2string(val_b))
            else:
                raise ValueError('unsupported type {} in +'.format(type))
        elif opname == '__minus__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real']:
                return (type, val_a[1] - val_b[1])
            else:
                raise ValueError('unsupported type {} in -'.format(type))
        elif opname == '__times__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real']:
                return (type, val_a[1] * val_b[1])
            else:
                raise ValueError('unsupported type in *')
        elif opname == '__divide__':
            type = promote(val_a[0], val_b[0])
            if type == 'integer':
                return (type, int(val_a[1]) // int(val_b[1]))
            elif type == 'real':
                return ('real', float(val_a[1]) / float(val_b[1]))
            else:
                raise ValueError('unsupported type in /')
        elif opname == '__or__':
            # NOTE: do we need to typecheck here?
            if map2boolean(val_a)[1] == True or map2boolean(val_b)[1] == True:
               return ('boolean', True)
            else:
               return ('boolean', False)
        elif opname == '__and__':
            # NOTE: do we need to typecheck here?
            if map2boolean(val_a)[1] == True and map2boolean(val_b)[1] == True:
               return ('boolean', True)
            else:
               return ('boolean', False)
        elif opname == '__eq__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real', 'list', 'string']:
                return ('boolean', val_a[1] == val_b[1])
            else:
                raise ValueError('unsupported type in ==')
        elif opname  == '__ne__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real', 'list', 'string']:
                return ('boolean', val_a[1] != val_b[1])
            else:
                raise ValueError('unsupported type in =/=')
        elif opname == '__le__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real']:
                return ('boolean', val_a[1] <= val_b[1])
            else:
                raise ValueError('unsupported type in <=')
        elif opname == '__lt__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real']:
                return ('boolean', val_a[1] < val_b[1])
            else:
                raise ValueError('unsupported type in <')
        elif opname == '__ge__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real']:
                return ('boolean', val_a[1] >= val_b[1])
            else:
                raise ValueError('unsupported type in >=')
        elif opname == '__gt__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real']:
                return ('boolean', val_a[1] > val_b[1])
            else:
                raise ValueError('unsupported type in >')
        else:
            raise ValueError('unknown builtin binary opname {}'.format(opname))

    elif opname in unary_operators:
        arg_val = walk(args)

        if opname == '__not__':
            val = map2boolean(arg_val)
            if val[1] == False:
                return ('boolean', True)
            elif val[1] == True:
                return ('boolean', False)
            else:
                raise ValueError('not a boolean value in not')
        elif opname == '__uminus__':
            if arg_val[0] in ['integer', 'real']:
                return (arg_val[0], - arg_val[1])
            else:
                raise ValueError(
                    'unsupported type {} in unary minus'
                    .format(arg_val[0]))
        else:
            raise ValueError('unknown builtin unary opname {}'.format(opname))

#########################################################################
def handle_call(fval, actual_val_args):

    (FUNCTION_VAL, body_list, closure) = fval
    assert_match(FUNCTION_VAL, 'function-val')

    # static scoping for functions
    # Note: we have to do this here because unifying
    # over the body patterns can introduce variable declarations,
    # think conditional pattern matching.
    save_symtab = state.symbol_table.get_config()
    state.symbol_table.set_config(closure)
    state.symbol_table.push_scope({})

    #lhh
    #print('in handle_call')
    #print("calling: {}\nwith: {}\n\n".format(fval,actual_val_args))

    # iterate over the bodies to find one that unifies with the actual parameters
    (BODY_LIST, (LIST, body_list_val)) = body_list
    unified = False

    for body in body_list_val:

        (BODY,
         (PATTERN, p),
         (STMT_LIST, stmts)) = body

        try:
            unifiers = unify(actual_val_args, p)
            unified = True
        except PatternMatchFailed:
            unifiers = []
            unified = False

        if unified:
            break

    if not unified:
        raise ValueError("none of the function bodies unified with actual parameters")

    declare_formal_args(unifiers)

    # Check for useless patterns
    check_redundancy(body_list, actual_val_args)

    # execute the function
    # function calls transfer control - save our caller's lineinfo
    old_lineinfo = state.lineinfo

    try:
        walk(stmts)
    except ReturnValue as val:
        return_value = val.value
    else:
        return_value = ('none', None) # need that in case function has no return statement

    # coming back from a function call - restore caller's lineinfo
    state.lineinfo = old_lineinfo

    # NOTE: popping the function scope is not necessary because we
    # are restoring the original symtab configuration. this is necessary
    # because a return statement might come out of a nested with statement
    state.symbol_table.set_config(save_symtab)

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
def global_stmt(node):

    (GLOBAL, (LIST, id_list)) = node
    assert_match(GLOBAL, 'global')
    assert_match(LIST, 'list')

    for id_tuple in id_list:
        (ID, id_val) = id_tuple
        if state.symbol_table.is_symbol_local(id_val):
            raise ValueError("{} is already local, cannot be declared global"
                             .format(id_val))
        state.symbol_table.enter_global(id_val)

#########################################################################
def assert_stmt(node):

    (ASSERT, exp) = node
    assert_match(ASSERT, 'assert')

    exp_val = walk(exp)
    # mapping asteroid assert into python assert
    assert exp_val[1], 'assert failed'

#########################################################################
def unify_stmt(node):

    (UNIFY, pattern, exp) = node
    assert_match(UNIFY, 'unify')

    term = walk(exp)
    unifiers = unify(term, pattern)
    declare_unifiers(unifiers)

#########################################################################
def return_stmt(node):

    (RETURN, e) = node
    assert_match(RETURN, 'return')

    raise ReturnValue(walk(e))

#########################################################################
def break_stmt(node):

    (BREAK,) = node
    assert_match(BREAK, 'break')

    raise Break()

#########################################################################
def throw_stmt(node):

    (THROW, object) = node
    assert_match(THROW, 'throw')

    raise ThrowValue(walk(object))

#########################################################################
def try_stmt(node):

    (TRY,
     (STMT_LIST, try_stmts),
     (CATCH_LIST, (LIST, catch_list))) = node

    try:
        walk(try_stmts)

    # NOTE: in Python the 'as inst' variable is only local to the catch block???
    except ThrowValue as inst:
        except_val = inst.value
        inst_val = inst

    except ReturnValue as inst:
        # return values should never be captured by user level try stmts - rethrow
        raise inst

    except PatternMatchFailed as inst:
        # convert a Python string to an Asteroid string
        except_val = ('tuple',
                      [('string', 'PatternMatchFailed'), ('string', inst.value)])
        inst_val = inst

    except Exception as inst:
        # convert exception args to an Asteroid string
        except_val = ('tuple',
                      [('string', 'Exception'), ('string', str(inst))])
        inst_val = inst

    else:
        # no exceptions found in the try statements
        return

    # we had an exception - walk the catch list and find an appropriate set of
    # catch statements.
    for catch_val in catch_list:
        (CATCH,
         (CATCH_PATTERN, catch_pattern),
         (CATCH_STMTS, catch_stmts)) = catch_val
        try:
            unifiers = unify(except_val, catch_pattern)
        except PatternMatchFailed:
            pass
        else:
            declare_unifiers(unifiers)
            walk(catch_stmts)
            return

    # no exception handler found - rethrow the exception
    raise inst_val

#########################################################################
def loop_stmt(node):

    (LOOP, body_stmts) = node
    assert_match(LOOP, 'loop')

    (STMT_LIST, body) = body_stmts

    try:
        while True:
            walk(body)
    except Break:
        pass

#########################################################################
def while_stmt(node):

    (WHILE, cond_exp, body_stmts) = node
    assert_match(WHILE, 'while')

    (COND_EXP, cond) = cond_exp
    (STMT_LIST, body) = body_stmts

    try:
        (COND_TYPE, cond_val) = map2boolean(walk(cond))
        while cond_val:
            walk(body)
            (COND_TYPE, cond_val) = map2boolean(walk(cond))
    except Break:
        pass

#########################################################################
def repeat_stmt(node):

    (REPEAT, body_stmts, cond_exp) = node
    assert_match(REPEAT, 'repeat')

    (COND_EXP, cond) = cond_exp
    (STMT_LIST, body) = body_stmts

    try:
        while True:
            walk(body)
            (COND_TYPE, cond_val) = map2boolean(walk(cond))
            if cond_val:
                break

    except Break:
        pass

#########################################################################
def for_stmt(node):

    (FOR, (IN_EXP, in_exp), (STMT_LIST, stmt_list)) = node
    assert_match(FOR, 'for')

    (IN, pattern, list_term) = in_exp

    # expand the list_term
    (LIST_TYPE, list_val) = walk(list_term)
    if LIST_TYPE not in ['list','string']:
        raise ValueError("only iteration over strings and lists is supported")

    # we allow iteration over two types of structures: (1) lists (2) strings
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
def if_stmt(node):

    (IF, (LIST, if_list)) = node
    assert_match(IF, 'if')
    assert_match(LIST, 'list')

    for if_clause in if_list:

        (IF_CLAUSE,
         (COND, cond),
         (STMT_LIST, stmts)) = if_clause

        (BOOLEAN, cond_val) = map2boolean(walk(cond))

        if cond_val:
            walk(stmts)
            break

#########################################################################
def struct_def_stmt(node):

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
    struct_scope = {}
    state.symbol_table.push_scope(struct_scope)

    for member_ix in range(len(member_list)):
        member = member_list[member_ix]
        if member[0] == 'data':
            (DATA, (ID, member_id)) = member
            state.symbol_table.enter_sym(member_id, ('integer', member_ix))
            struct_memory.append(('none', None))
            member_names.append(member_id)
        elif member[0] == 'unify':
            (UNIFY, (ID, member_id), function_exp) = member
            state.symbol_table.enter_sym(member_id, ('integer', member_ix))
            # Note: we have to bind a function VALUE into the structure memory
            function_val = walk(function_exp)
            struct_memory.append(function_val)
            member_names.append(member_id)
        elif member[0] == 'noop':
            pass
        else:
            raise ValueError("unsupported struct member '{}'".format(member[0]))

    state.symbol_table.pop_scope()

    struct_type = ('struct',
                  ('member-names', ('list', member_names)),
                  ('struct-memory', ('list', struct_memory)),
                  ('struct-scope', struct_scope))

    state.symbol_table.enter_sym(struct_id, struct_type)

#########################################################################
def eval_exp(node):

    (EVAL, exp) = node
    assert_match(EVAL, 'eval')
    # lhh
    #print("in eval with {}".format(node))

    # Note: eval is essentially a macro call - that is a function
    # call without pushing a symbol table record.  That means
    # we have to first evaluate the argument to 'eval' before
    # walking the term.  This is safe because if the arg is already
    # the actual term it will be quoted and nothing happens if it is
    # a variable it will be expanded to the actual term.
    #lhh
    #print("before expand: {}".format(exp))
    exp_val_expand = walk(exp)
    #lhh
    #print("after expand: {}".format(exp_val_expand))
    # now walk the actual term
    state.ignore_quote = True
    exp_val = walk(exp_val_expand)
    #lhh
    #print("after walk: {}".format(exp_val))
    state.ignore_quote = False
    return exp_val

#########################################################################
def apply_exp(node):

    (APPLY, f, arg) = node
    assert_match(APPLY, 'apply')

    # handle builtin operators that look like apply lists.
    if f[0] == 'id' and f[1] in operator_symbols:
        return handle_builtins(node)

    # handle function application
    f_val = walk(f)
    arg_val = walk(arg)

    # object member function
    # NOTE: object member functions are passed an object reference.
    if f_val[0] == 'member-function-val':
        (MEMBER_FUNCTION_VAL, obj_ref, function_val) = f_val
        # Note: lists and tuples are objects/mutable data structures, they
        # have member functions defined in the Asteroid prologue.
        if arg_val[0] == 'none':
            result = handle_call(function_val, obj_ref)
        elif arg_val[0] != 'tuple':
            new_arg_val = ('tuple', [obj_ref, arg_val])
            result = handle_call(function_val, new_arg_val)
        elif arg_val[0] == 'tuple':
            arg_val[1].insert(0, obj_ref)
            result = handle_call(function_val, arg_val)
        else:
            raise ValueError(
                "unknown parameter type '{}' in apply"
                .format(arg_val[0]))

    # regular function call
    elif f_val[0] == 'function-val':
        result = handle_call(f_val, arg_val)

    # object constructor call
    elif f_val[0] == 'struct':
        (ID, struct_id) = f
        (STRUCT,
         (MEMBER_NAMES, (LIST, member_names)),
         (STRUCT_MEMORY, (LIST, struct_memory)),
         (STRUCT_SCOPE, struct_scope)) = f_val

        # create our object memory - memory cells now have initial values
        # TODO: why is this not shared among objects?
        object_memory = struct_memory.copy()
        # create our object
        obj_ref = ('object',
                  ('struct-id', ('id', struct_id)),
                  ('object-memory', ('list', object_memory)))
        # if the struct has an __init__ function call it on the object
        # NOTE: constructor functions do not have return values.
        if '__init__' in member_names:
            slot_ix = member_names.index('__init__')
            init_fval = struct_memory[slot_ix]
            # calling a member function - push struct scope
            state.symbol_table.push_scope(struct_scope)
            if arg_val[0] == 'none':
                handle_call(init_fval, obj_ref)
            elif arg_val[0] != 'tuple':
                arg_val = ('tuple', [obj_ref, arg_val])
                handle_call(init_fval, arg_val)
            elif arg_val[0] == 'tuple':
                arg_val[1].insert(0, obj_ref)
                handle_call(init_fval, arg_val)
            state.symbol_table.pop_scope()
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
                    "default constructor expected {} arguments got {}"
                    .format(len(data_memory), len(arg_array)))
            # copy initializers into object memory
            data_ix = data_ix_list(object_memory)
            for (i,k) in zip(data_ix, range(0,len(data_memory))):
                object_memory[i] = arg_array[k]

        # return the new object
        result = obj_ref

    else:
        raise ValueError("unknown apply term '{}'".format(f_val[0]))

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
def escape_exp(node):

    (ESCAPE, s) = node
    assert_match(ESCAPE, 'escape')

    global __retval__
    __retval__ = ('none', None)

    exec(s)

    return __retval__

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
        raise ValueError("right argument to in operator has to be a list")

    # we simply map our in operator to the Python in operator
    if exp_val in exp_list_val:
        return ('boolean', True)
    else:
        return ('boolean', False)

#########################################################################
def if_exp(node):

    (IF_EXP, cond_exp, then_exp, else_exp) = node
    assert_match(IF_EXP, 'if-exp')

    (BOOLEAN, cond_val) = map2boolean(walk(cond_exp))

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

    # TODO: check out the behavior with step -1 -- is this what we want?
    # the behavior is start and stop included
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
            "unsuported tail type {} in head-tail operator".
            format(TAIL_TYPE))

    return ('list', [head_val] + tail_val)

#########################################################################
# turn a function expression into a closure.
def function_exp(node):

    (FUNCTION_EXP, body_list) = node
    assert_match(FUNCTION_EXP,'function-exp')

    return ('function-val',
            body_list,
            state.symbol_table.get_config())

#########################################################################
# Named patterns - when walking a named pattern we are interpreting a
# a pattern as a constructor - ignore the name
def named_pattern_exp(node):

    (NAMED_PATTERN, name, pattern) = node
    assert_match(NAMED_PATTERN,'named-pattern')

    return walk(pattern)

#########################################################################
def process_lineinfo(node):

    (LINEINFO, lineinfo_val) = node
    assert_match(LINEINFO, 'lineinfo')

    #lhh
    #print("lineinfo: {}".format(lineinfo_val))

    state.lineinfo = lineinfo_val

#########################################################################
def typematch_exp(node):

    (TYPEMATCH, type) = node
    assert_match(TYPEMATCH, 'typematch')

    raise ValueError(
            "typematch {} cannot appear in expressions or constructors"
            .format(type))

#########################################################################
def cmatch_exp(node):

    (CMATCH, exp, cond_exp) = node
    assert_match(CMATCH, 'cmatch')

    # on a walk to interpret the tree as a value we simply
    # ignore the conditional expression
    return walk(exp)

#########################################################################
def deref_exp(node):

    (DEREF, id_exp) = node
    assert_match(DEREF, 'deref')

    # deref operators are only meaningful during pattern matching
    # ignore during a value walk.
    # NOTE: the second walk is necessary to interpret what we retrieved
    # through the indirection
    return walk(walk(id_exp))

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
        raise ValueError("feature {} not yet implemented".format(type))

# a dictionary to associate tree nodes with node functions
dispatch_dict = {
    # statements - statements do not produce return values
    'lineinfo'      : process_lineinfo,
    'noop'          : lambda node : None,
    'assert'        : assert_stmt,
    'unify'         : unify_stmt,
    'while'         : while_stmt,
    'loop'          : loop_stmt,
    'repeat'        : repeat_stmt,
    'for'           : for_stmt,
    'global'        : global_stmt,
    'return'        : return_stmt,
    'break'         : break_stmt,
    'if'            : if_stmt,
    'throw'         : throw_stmt,
    'try'           : try_stmt,
    'struct-def'    : struct_def_stmt,
    # expressions - expressions do produce return values
    'list'          : list_exp,
    'tuple'         : tuple_exp,
    'to-list'       : to_list_exp,
    'head-tail'     : head_tail_exp,
    'raw-to-list'   : lambda node : walk(('to-list', node[1], node[2], node[3])),
    'raw-head-tail' : lambda node : walk(('head-tail', node[1], node[2])),
    'seq'           : lambda node : ('seq', walk(node[1]), walk(node[2])),
    'none'          : lambda node : node,
    'nil'           : lambda node : node,
    'function-exp'  : function_exp,
    'string'        : lambda node : node,
    'integer'       : lambda node : node,
    'real'          : lambda node : node,
    'boolean'       : lambda node : node,
    'eval'          : eval_exp,
    # quoted code should be treated like a constant if not ignore_quote
    'quote'         : lambda node : walk(node[1]) if state.ignore_quote else node,
    # type tag used in conjunction with escaped code in order to store
    # foreign objects in Asteroid data structures
    'foreign'       : lambda node : node,
    'id'            : lambda node : state.symbol_table.lookup_sym(node[1]),
    'apply'         : apply_exp,
    'index'         : index_exp,
    'escape'        : escape_exp,
    'is'            : is_exp,
    'in'            : in_exp,
    'if-exp'        : if_exp,
    'named-pattern' : named_pattern_exp,
    'member-function-val' : lambda node : node,
    'typematch'     : typematch_exp,
    'cmatch'        : cmatch_exp,
    'deref'         : deref_exp,
}

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This exception is used when a pattern has been identified as being 'useless',
# or reundant. This exception is formatted to pack both offending patterns
# information into a single formatted message to the user informing them of
# where and what caused this error.
##############################################################################################
class RedundantPatternFound(Exception):
    """Exception raised for detection of redundant pattern in function declaration.
    Attributes:
        pattern1 -- The pattern with the higher order of precedence (comparer).
        line1    -- The line number location of pattern 1 in its file.
        pattern2 -- The pattern with the lower order of precedence  (comparee).
        line2    -- The line number location of pattern 2 in its file.
        function -- The name of the function where the redundancy was found.
        file     -- The name of the file where the function is from.
    """
    def __init__(self, pattern1, pattern2,function_name,location1,location2):

        self.pattern1 = pattern1
        self.line1 = str(location1[1] - 1) #patterns dont have line #, so we
                                            #compensate here by using the first line
                                            # of the patterns body, then moving back
                                            # 1 line(minus 1)
        self.pattern2 = pattern2
        if (location2 != None):
            self.line2 = str(location2[1] - 1)
        if (location1 != None):
            self.file = location1[0]
        self.function = function_name
        self.message = "Redundant Pattern Detected\n"
        self.message += "\tFunction: " + self.function + " from file " + self.file
        self.message += "\n\tPattern: " + term2string(self.pattern1) + " on line " + self.line1
        self.message += "\n\twill consume all matches for"
        self.message += "\n\tPattern: " + term2string(self.pattern2) + " on line " + self.line2
        super().__init__(self.message)

###########################################################################################
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
def check_redundancy( body_list, actual_val_args ):

    #Node type assertions
    #or "Make sure we are walking down the right part of the tree"
    (BODY_LIST, function_bodies ) = body_list
    assert_match(BODY_LIST,'body-list')
    (LIST, bodies) = function_bodies
    assert_match(LIST,'list')

    # Reset the covered intervals for all data types
    global conditional_intervals
    conditional_intervals = [False,[]]

    # Reset the global id tag index which is used in evaluating 
    # subsumption between condtional pattern clauses with relational operations.
    global tag_number
    tag_number = 0

    #compare every pattern with the patterns that follow it
    for i in range(len(bodies)):

        #get the pattern with the higher level of precedence
        (BODY_H,(PTRN,_ptrn_h),stmts_h) = bodies[i]
        assert_match(BODY_H,'body')
        assert_match(PTRN,'pattern')

        for j in range(i + 1, len(bodies)):


            #get the pattern with the lower level of precedence
            (BODY_L,(PTRN,_ptrn_l),stmts_l) = bodies[j]
            assert_match(BODY_L,'body')
            assert_match(PTRN,'pattern')

            # If we end up coming across a conditional pattern in this evaluation
            # we will need to generalize the pattern, this happens in place.
            # By Making a copy here we will not tamper with the pattern representations
            # in the AST representation.
            ptrn_h = deepcopy(_ptrn_h)
            ptrn_l = deepcopy(_ptrn_l)

            #DEBUGGING
            ###(pattern,code) = body
            # print("COMPARE: ")
            # print(ptrn_l)
            # print("TO: ")
            # print(ptrn_h)

            #Here we get line numbers in case we throw an error
            # we have to do a little 'tree walking' to get to the
            # line #, hence all the unpacking.
            (STMT_LIST,(LIST,LINE_LIST)) = stmts_l
            first_line_l = LINE_LIST[0]
            (LINE_INFO,location_l) = first_line_l

            (STMT_LIST,(LIST,LINE_LIST)) = stmts_h
            first_line_h = LINE_LIST[0]
            (LINE_INFO,location_h) = first_line_h

            tag_number = 0 #Reset global index used to evaluate conditonal patterns 

            # Compare the patterns to determine if the pattern with the
            # higher level of precedence will render the pattern with
            # the lower level of precedence useless/redundant by calling
            # on the unify function to evaluate the subsumption relationship
            # between the two patterns.
            try:
                unify( ptrn_l, ptrn_h , False )
            except PatternMatchFailed:
                pass
            else:

                raise RedundantPatternFound( _ptrn_h , _ptrn_l ,location_h[0], location_h, location_l )

        # Update the tag at the begining of the global conditional intervals list to indicate
        # that the next time we compare a relational pattern against another, it will be 
        # our first time evaluating that relational pattern, so we should record its interval
        conditional_intervals[0] = False

#######################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# Updates the global conditional-expression relational-sumbsumtion numberline with a 
# single relational expression.
#
# The first time a condtional pattern is evaluated by the redundant pattern detector, it
# will have its pattern, along with its condtional expression(s) added to a global data
# structure, the conditional-expression relational-sumbsumtion numberline. This allow us
# to 'look back' at this stucture to see if a previous condtional pattern with a relational
# function has covered, or will subsume, all matches intended for the current pattern. 
# The idea is to catch redundant patterns like this:
# 
# function test_func
#    with (x) %if x > 0 do         
#        return 0.
#    orwith (x) %if x == 10 do         
#        return 1.
#    end.
# 
#  Which is an example of of a redundant conditional pattern with a relational function.
#  
# Should a single possible value exist in which a pattern match could be made to a clause,
# it is the intended behavior that the retundant pattern detector should not throw an error. 
def update_numberline( pattern_expression, function_name, apply_list ):

    #Access global flags
    global conditional_intervals
    conditional_intervals[0] = True
    global tag_number
    tag_number = 0

    # Unpack some of our nodes to see what we are looking at
    operation = function_name[2:-2]
    value = (apply_list[1])[1]
    var_name = (apply_list[1])[0]

    # If we are evaluating a compound relational expression( AND/OR statment)
    if (len((apply_list[1])[1]) == 3 ): 
        #We handle this with a different body of code
        eval_compound_relation_exp(pattern_expression, function_name, apply_list)
        return

    #Else we are just evaluating a single relational expression

    #Generalize the pattern variable names so user-names do not effect subsumption
    #evaluation.
    generalized_pattern = generalize_variable_names(pattern_expression,var_name)

    interval = ''
    index = 0     #Still used???

    # Get access to the correct interval from the global interval list
    for pattern in conditional_intervals[1]:
        try:
            if(pattern[-1] == generalized_pattern):
                interval = pattern
            break
        except:
            pass
        index = index + 1

    # Check to see if the interval variable was populated; If it wasn't, this is the
    # first time we have seen this pattern-varaible id; we need to add it to the global
    # subsumed intervals list.
    if interval == '':
        interval = [(generalized_pattern)]
        conditional_intervals[1].append(interval)

    # Reset the generalize pattern global tag counter
    tag_number = 0

    # Update the interval with the newly covered range
    if (operation == 'ge'):                             #Greater than or equal
        interval.insert( 0, ( value[1], 'inf', '[' ) )
    elif (operation == 'gt'):                           #Greater than
        interval.insert( 0, ( value[1], 'inf', '(' ) )
    elif (operation == 'lt'):                           #Less than
        interval.insert( 0, ( 'inf',value[1], '(' ) )
    elif (operation == 'le'):                           #Less than or equal
        interval.insert( 0, ( 'inf',value[1], '[' ) )
    elif (operation == 'eq'):                           #equal
        if (value[0] in ['integer','real']):
            interval.insert( 0, ( value[1],value[1], '[' ) )
        else:
            interval.insert( 0, ( 'seq' ,value[1] ) )
    elif (operation == 'ne'):                           #not equal
        if (value[0] in ['integer','real']):
            interval.insert( 0,  (value[1],'inf'), '(' )
            interval.insert( 0,  ('inf',value[1]), '(' )
        else:
            interval.insert( 0,  ('neq',value[1]) )

    # Grab the next relational statement from the interva list to check next
    (conditional_intervals[1])[index] = interval

###############################################################################
# *** Part of the Redundant Pattern Detector ***
#
# A driver function for evaluating the expressions found within a compound relational
# condtional pattern clause. Once evaluated, the values that are covered/ or will
# be subsumed for following condtional pattern will be added to the global 
# subsumed/covered intervals list, conditional intervals.
def eval_compound_relation_exp(pattern_expression, function_name, apply_list):

    # Check if we are evaluating an or statement(s); or statements are essentially
    # two different potential pattern matches and as such can cover two different 
    # ranges/intervals. Therefore they are completely treted as a seperate 
    if function_name == '__or__':
        walk_compound_relation_ors(pattern_expression, function_name, apply_list)
    else:
        walk_compound_relation_ands(pattern_expression, function_name, apply_list)

    return

#######################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# Updates the global conditional-expression relational-sumbsumtion numberline.
# Used to evaluate and add compound relational expressions to the numberline. Responsible
# for all 'cmpd' nodes found in the global subsumbed interval numberline. 
def update_numberline_compound(pattern_expression, expression_list, var_list):

    var = var_list[0]     #As we are only doing single variables currently TODO: update for multi-var
    not_equal_vals = []   #Holds all inverse equality vals found in the relationals
    equal_vals = []       #Holds all equality vals found in the relationals
    number_line_vals = [False,False,False,False] #holds the number line values 
    #[greatest 'less than',lowest 'greater than',greatest 'less than or eq' ,lowest  'greater than or eq']
    
    for expr in expression_list:

        #Get the function name
        relational_operation = (((expr[1])[1])[2:-2])

        # If we find we have the wrong function type in hand, we punt
        if relational_operation not in ['eq','ne','lt','le','gt','ge']:
            # Function warning should only print once per program execution
            global warning
            if not warning[0]: 
                print("Redundant pattern detection is not supported for conditional pattern expressions with function calls.")
                print("Only relational operations/functions are supported.")
                print("All conditional pattern clauses which reference functions will be ignored.")
                warning[0] = True
                return

        elif relational_operation == 'eq':              #EQUAL
            value = get_compare_value(expr,var)
            equal_vals.append(value)
        elif relational_operation == 'ne':              #NOT EQUAL
            value = get_compare_value(expr,var)
            not_equal_vals.append(value)
        elif relational_operation == 'lt':              #LESS THAN
            value = get_compare_value(expr,var)
            if number_line_vals[0]:#If this is not the first lt
                if number_line_vals[0] < value:#If if is the lowest seen
                    number_line_vals[0] = value
            else:
                number_line_vals[0] = value
        elif relational_operation == 'le':              #LESS THEN OR EQUAL       
            value = get_compare_value(expr,var)
            if number_line_vals[2]:#If this is not the first lt
                if number_line_vals[2] < value:#If if is the lowest seen
                    number_line_vals[2] = value
            else:
                number_line_vals[2] = value
        elif relational_operation == 'gt':              #GREATER THAN       
            value = get_compare_value(expr,var)
            if number_line_vals[1]:#If this is not the first lt
                if number_line_vals[1] < value:#If if is the lowest seen
                    number_line_vals[1] = value
            else:
                number_line_vals[1] = value
        elif relational_operation == 'ge':              #GREATER THAN OR EQUAL    
            value = get_compare_value(expr,var)
            if number_line_vals[3]:#If this is not the first lt
                if number_line_vals[3] < value:#If if is the lowest seen
                    number_line_vals[3] = value
            else:
                number_line_vals[3] = value
    
    less = False
    less_edge = False
    greater = False
    greater_edge = False

    # Parse the accumulated expression information to determine the actual covered/subsumed
    #intervals 

    # If we have a less then val
    if (number_line_vals[0]):
        less = number_line_vals[0]
        less_edge = '(' # non-inclusive interval border
    
    # If we have a less then val
    if (number_line_vals[1]):
        greater = number_line_vals[1]
        greater_edge = '(' # non-inclusive interval border

    # If we have a less then or equal val
    if (number_line_vals[2]):
        if less:
            if (number_line_vals[2] > less):
                less = number_line_vals[2]
                less_edge = '[' # inclusive interval border
        else:
            less = number_line_vals[2]
            less_edge = '[' # inclusive interval border
    
    # If we have a less then or equal val
    if (number_line_vals[3]):
        if greater:
            if (number_line_vals[3] < greater):
                greater = number_line_vals[3]
                greater_edge = '[' # inclusive interval border
        else:
            greater = number_line_vals[3]
            greater_edge = '[' # inclusive interval border

    # Check to see if we have seen this pattern/variable combination before(or one the subsumes)
    global conditional_intervals
    global tag_number
    tag_number = 0
    var_name = ('id',var)
    general_pattern = generalize_variable_names(pattern_expression,var_name)
    interval_list = ''
    interval_list = get_interval_list( general_pattern )

    # Check to see if the interval variable was populated; If it wasn't, this is the
    # first time we have seen this pattern-varaible id; we need to add it to the global
    # subsumed intervals list.
    if interval_list == '':
        interval_list = [(general_pattern)]
        conditional_intervals[1].append(interval_list)

    interval_list.insert(0, ('cmpd',(greater_edge,greater,less,less_edge),not_equal_vals,equal_vals))
    
###############################################################################
# Part of the Redundant Pattern Detector
#
# Small helper function/getter for the reundant pattern detector. Finds a literal
# value in a argument list and then returns that value. 
#
# ttc : TODO update for multi-variable detection
def get_compare_value(expr,var_name):

    # unpack the node
    if (len(expr) == 3):
        (APPLY,OP,(TUPLE,var_list))= expr
    else:
        (TUPLE,var_list) = expr
    # Iterate through the list looking for a value that is not a 'id'
    # and then return that value
    for var in var_list:
        if (var[1] != var_name):
            if (var[0] != 'id'):
                return var[1]
            else:
                return state.symbol_table.lookup_sym(var[1])
    
    print("ERROR: Redundant Pattern Detector: get_compare_value\nNo Value found for expression:\n",expr)
    #dump_AST(expr)
    return

###############################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This function is a treewalker which helps up break up compound relational 
# condtional patterns found in a conditional pattern clause. Specifically, this
# function takes in a compound relational expression and then breaks that
# expression up at other 'and/or' keywords, passing each piece into the 
# appropriate function.
def walk_compound_relation_ors(pattern_expression, function_name, apply_list): 

    # Unpack the apply list/list of relational expressions
    (TUPLE,expression_list) = apply_list

    # For each expression in the or statement(2 as it is a binary statement)
    for expr in expression_list:

        #3 possible function destinations:
        #its a or
        #its a and
        #its a single expr

        # If we are looking at a nested or, we should recurse to continue to break up
        # the the different or blocks
        if ((expr[1])[1] == '__or__'):
            walk_compound_relation_ors(pattern_expression,(expr[1])[1], (expr[2]) )
        elif ((expr[1])[1] == '__and__'):
            #Else we can move on the evaluate all of the and statements contained
            # within the current block
            walk_compound_relation_ands(pattern_expression,(expr[1])[1], expr)
        else:
            update_numberline( pattern_expression, (expr[1])[1], expr[-1] )

###############################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This function walks down compound relational expressions linked together in a single
# 'and' statement block that are found in compound condtional patterns. As this function
#  walks, it also constucts a list of all of the expressions that are linked together,
# along with all of the variables that are found within the expresssions.
# 
# The list of expressions is then passed into the update_numberline_compound function
# to allow its covered/subsumed range to be recorded in the global data structure.
def walk_compound_relation_ands(pattern_expression, function_name, apply_list):

    # Unpack the apply list/list of relational expressions
    expression_list = []
    expression_list.append(apply_list)

    # Will hold the number of variables seen in all of the relational exps 
    vars_seen = [] #All of the different varaible names/labels seen
    expressions_seen = [] # all the expressions in the compound expression
    function_list = []

    # Process through every statement within this block of expressions
    while (((expression_list[0])[1])[1] == '__and__'):

        expression=expression_list[-1]

        expression_list = ((expression_list[0])[2])[1]
        expressions_seen.append(expression_list[-1])
        function_list.append((expression[1])[1])
    
    function_list.append(((expression_list[-1])[1])[1])
    function_list.append(((expression_list[-2])[1])[1])
    expressions_seen.append(expression_list[-2])

    if (len(vars_seen) > 1):
        print("Error, multi-variable compound relational subsumption detection is under construction.")
        return

    #print("Expressions seen: ")
    for e in expressions_seen:
        # print(e)

        # If we have never seen the varaible used in this relation before, add it to the list
        value_list = ((e[2])[1])

        #Check the values/literals
        for node in value_list:
            if node[0] == 'id': # Is it a derefence/variable?
                if node[1] not in vars_seen: # Have we never seen this before?
                    vars_seen.append(node[1])# Record it if not
    
    #  multi-variable compound relational subsumption detection is under construction
    if (len(vars_seen) > 1):
        print("Error, multi-variable compound-relational redundant pattern detection is not yet implemented.")
        return

    update_numberline_compound( pattern_expression, expressions_seen, vars_seen )
    return

############################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This functions job is to take in a pattern, along with the name of a relational
# function operation and an arguments list and then determine if the passed in
# relational and pattern, which represent a conditional function clause, is
# a redundant/useless pattern because all of its matchs will be consumed by
# preceeding patterns. 
# This is accomplised by checking the intented relational operation and its values
# againt the global condtional pattern covered intervals data structure, conditional_intervals
def check_numberline(pattern_expression, function_name ,apply_list ):

    operation = function_name[2:-2]     #The relational operation
    value = (apply_list[1])[1]          # The literal value
    var_name = (apply_list[1])[0]

    # Generalize the pattern and get a copy of its interval
    # we need a copy so we dont .pop off of the original
    generalized_pattern = generalize_variable_names(pattern_expression,var_name)
 
    interval = get_interval_list_copy( generalized_pattern )

    #If the patterns dont match or its a differnt variable
    if interval == '':
        return False

    #Ensure the entire numberline is not already covered
    if generalized_pattern[0] != 'named-pattern':
        if is_completely_covered( generalized_pattern[0] ,var_name):
            return True
    else:
        if is_completely_covered( generalized_pattern ,var_name):
            return True

    current_range = interval.pop(0)
    
    while (len(interval) > 0):

        if value[0] in ['integer','real']:#If we are evaluating equality between numerical values
            if operation in ['ge','gt']:                            #GREATER THAN
                if current_range[1] == 'inf':#If we are looking at another greater than
                    if (operation == 'ge' and current_range[2] == '('):
                        if current_range[0] < value[1]:
                            return True
                    else:
                        if current_range[0] <= value[1]:
                            return True
                if current_range[0] == 'cmpd':#We less then against a compount relational 
                    if check_cmpd_expr_range_gt(current_range,value,operation):
                        return True
            elif operation in ['le','lt']:                            #LESS THAN
                if current_range[0] == 'inf':#If we are looking at another less than
                    if (operation == 'le' and current_range[2] == '('):
                        if current_range[1] > value[1]:
                            return True 
                    else:
                        if current_range[1] >= value[1]:
                            return True
                if current_range[0] == 'cmpd':#We less then against a compount relational 
                    if check_cmpd_expr_range_lt(current_range,value,operation):
                        return True
            elif (operation == 'eq'):                                  #EQUAL
                if current_range[1] == 'inf':   # handle > for eq
                    if current_range[2] == '(':
                        if current_range[0] < value[1]:
                            return True
                    else: 
                        if current_range[0] <= value[1]:
                            return True
                elif current_range[0] == 'inf': # handle < for eq
                    if current_range[2] == '(':
                        if current_range[1] > (value[1]):
                            return True
                    else:
                        if current_range[1] >= (value[1]):
                            return True
                elif (current_range[0] == 'cmpd'):#Handle compound for eq
                    if check_cmpd_expr_range_eq(current_range,value):
                        return True
                elif (current_range[1] == (value[1]) and (current_range[1] == current_range[0])): #Handle == for equal
                    return True
                elif (check_finite_range(current_range,value)):#handle finite range for eq
                    return True
            elif (operation == 'ne'):               # Not Equal
                pass
        else:       #Else we are evaluating equality between non-numerical values
            if (operation == 'eq'):                 # Equal
                if current_range[0] == 'neq': #Evaluating against a not equal node
                    if (current_range[1] != value[1]):
                        return True
                elif current_range[0] == 'seq':#Evaluating against an equality node
                    if (current_range[1] == value[1]):
                        return True
            elif (operation == 'ne'):               # Not Equal
                if current_range[0] == 'neq': #Evaluating against a not equal node
                    if (current_range[1] == value[1]):
                        return True
                elif current_range[0] == 'seq':#Evaluating against an equality node
                    if (current_range[1] != value[1]):
                        return True
        
        current_range = interval.pop(0)

    return False

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This function checks a compound relational expression range to see if an less than expression
# is rundundant.
#
# This function returns a boolean value indicating if a redundancy has been detected.
def check_cmpd_expr_range_lt(current_range,value,operation):
    (CMPD,(greater_edge,greater,less,less_edge),not_equal_vals,equal_vals) = current_range

    # If there is a lower bound no upper
    if (less and not greater):
        if less > value[1]:
            if (len(equal_vals) == 0):
                for entry in not_equal_vals:
                    if (less_edge == '('):
                        if entry > less:
                            return True
                    else:
                        if entry >= less:
                            return True
    return False

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This function checks a compound relational expression range to see if a greater than expression
# is rundundant.
#
# This function returns a boolean value indicating if a redundancy has been detected.
def check_cmpd_expr_range_gt(current_range,value,operation):
    (CMPD,(greater_edge,greater,less,less_edge),not_equal_vals,equal_vals) = current_range

    # If there is a upper bound but no lower
    if (greater and not less):
        if greater < value[1]:
            if (len(equal_vals) == 0):
                for entry in not_equal_vals:
                    if (greater_edge == '('):
                        if entry < greater:
                            return True
                    else:
                        if entry <= greater:
                            return True
    return False

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This function checks a compound relational expression range to see if an equality expression
# is rundundant.
#
# This function returns a boolean value indicating if a redundancy has been detected.

def check_cmpd_expr_range_eq(current_range,value):

    (CMPD,(greater_edge,greater,less,less_edge),not_equal_vals,equal_vals) = current_range


    if (greater and less):#If the range has a top and bottom end
        if (check_finite_range((greater,less,greater_edge,less_edge),value)):
            return True #Redundancy detected
    elif( less ): # If a less than value exists
        if less_edge == '(':
            if less > value[1]:
                if check_equality_lists(equal_vals,not_equal_vals,value):
                    return True #Redundancy detected
        else:
            if less >= value[1]:
                if check_equality_lists(equal_vals,not_equal_vals,value):
                    return True #Redundancy detected
    elif( greater): # if a greater than value exists
        if greater_edge == '(':
            if greater < value[1]:
                if check_equality_lists(equal_vals,not_equal_vals,value):
                    return True #Redundancy detected
        else:
            if greater <= value[1]:
                if check_equality_lists(equal_vals,not_equal_vals,value):
                    return True #Redundancy detected

    return False #It will NOT be subsumed by this interval
##############################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# Small helper function for the check_cmpd_expr_range_eq function, this function check both the
# equality and the inequality lists for conflictions that would indicate a redundant pattern
# vlause.
#
# This function returns a boolean values indicating if any conflictions were found.
def check_equality_lists(equal_vals,not_equal_vals,value):

    for eq_val in equal_vals:
        if eq_val == value[1]:
            return True #Redundancy detected
    for ne_val in not_equal_vals:
        if ne_val != value[1]:
            return True #Redundancy detected
    
    return False #It will NOT be subsumed by this

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This function is used to check if a given value falls within a finte range on a numberline.
#
# current_range comes in the format: (lower bound, upper bound, lower bound edge, upper bound edge)
# if an edge = '(', it is a non-inclusive bound, else if it is '[' it is an inclusive bound.
#
# This function returns a boolean value.
def check_finite_range(current_range,value):

    if (current_range[2] == '('):
        if (value[1] > current_range[0]):
            if (current_range[3] == '('):
                if (value[1] < current_range[1]):
                    return True #Redundancy detected
            else: 
                if (value[1] <= current_range[1]):
                    return True #Redundancy detected
    else:
        if (value[1] >= current_range[0]):
            if (current_range[3] == '('):
                if (value[1] < current_range[1]):
                    return True #Redundancy detected
            else: 
                if (value[1] <= current_range[1]):
                    return True #Redundancy detected
    
    return False #It will NOT be subsumed by this interval

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This function acts as an accessor for the global data structure which contains all of the 
# information about the subsumed/covered intervals for pattern/varaibles pairs found in
# conditional pattern clauses.
#
# This function returns a copy of the data structure.
def get_interval_list_copy( pattern_expression ):

    interval_list = ''

    for pattern in conditional_intervals[1]:
        if(pattern[-1] == pattern_expression): #If the patterns match
            interval_list = pattern.copy()
            break
        else:                                   #Else see if they subsume
            try:
                unify( pattern_expression[0], (pattern[-1])[0] , False )
            except PatternMatchFailed:
                pass
            else:
                # If they subsume and access the same variable, we want to
                # grab its associated "number line"
                if ( pattern_expression[1] == (pattern[-1])[1] ):  
                    interval_list = pattern.copy()
                    break
    
    return interval_list

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
#
# This function acts as an accessor for the global data structure which contains all of the 
# information about the subsumed/covered intervals for pattern/varaibles pairs found in
# conditional pattern clauses.
#
# This function returns the actual object.
def get_interval_list( pattern_expression ):
    global conditional_intervals
    interval_list = ''

    for pattern in conditional_intervals[1]:
        if(pattern[-1] == pattern_expression): #If the patterns match
            interval_list = pattern
            break
        else:                                   #Else see if they subsume
            try:
                unify( pattern_expression[0], (pattern[-1])[0] , False )
            except PatternMatchFailed:
                pass
            else:
                # If they subsume and access the same variable, we want to
                # grab its associated "number line"
                if ( pattern_expression[1] == (pattern[-1])[1] ):  
                    interval_list = pattern
                    break
    
    return interval_list

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
# 
# The purpose of this function is to determine if a 'number-line' for a pattern/variable pair
# has been fully subsumed or not. A numberline may become completely subsumed through multiple
# condtional pattern clauses with relational expressions.
#
# example:
# -- A testing function with multiple patterns
# function g
#    with (x) %if x < 0 do         
#        return 2.
#    orwith (x) %if x > -10 do         
#        return 3.
#    orwith (x:%integer) do         
#        return 3.
#    end.
#
# The 3rd clause, the type-match, is unreachable as any interger would have to have one of the 
# previous properties found in the preceeding conditional clauses.
#
# This function determines if the above situation exists, and returns a boolean value.
def is_completely_covered( pattern_expression , variable ):

    global tag_number
    tag_number = 0
    

    generalized_pattern = generalize_variable_names(pattern_expression,variable)
    greatest_lt = False #Greatest value in a less than statement( i.e. x < 10 )
    lowest_gt = False #Greatest value in a greater than statement( i.e. x > 10 )
    interval = get_interval_list_copy( generalized_pattern )
    equalities = [] # A list containing all of the unique values seen in 
                    # exact equality conditional expressions. (i.e. x == 10)
    #If the patterns dont match or its a differnt variable
    if interval == '':
        return False

    # Get the first interval, or range, from the front of the list
    current_range = interval.pop(0)

    # Iterate through all of the relational expressions that have already been
    # covered for the pattern-variable identity to determine both the greatest
    # 'less than' value and the greatest 'greater then' calue
    while (len(interval) > 0):

        if (current_range[0] == 'inf'):             # LESS THAN
            if (greatest_lt): #If a greatest less than value exists
                if (current_range[1] < greatest_lt[0]):# If the current value
                                                    # is greater than the lowest 
                    greatest_lt = (current_range[1],current_range[2])

            else: # Else we create an initial greatest_lt record
                greatest_lt = (current_range[1],current_range[2])

        elif (current_range[1] == 'inf'):           # GREATER THAN
            if (lowest_gt): #If a lowest greater than value exists 
                if (current_range[0] > lowest_gt[0]):# If the current value
                                                  # is less than the highest 
                    lowest_gt = (current_range[0],current_range[2])
            else: # Else we create an initial lowest_gt record
                lowest_gt = (current_range[0],current_range[2])

        elif (current_range[0] == current_range[1]): # EQUALITY
            equalities.append(current_range[1])

        current_range = interval.pop(0)

    # If the greatest 'less than' value and the lowest 'greater than' value 
    # overlap, then the entire range for real and integer values has already
    # been covered.
    if ((greatest_lt and lowest_gt)):

        if ((greatest_lt[1] == '[') or (lowest_gt[1] == ['['] )): #If one of the ranges is inclusive

            if (greatest_lt[0] >= lowest_gt[0]):
                return True 

        else:
            if (greatest_lt[0] > lowest_gt[0]):
                return True 

            elif (greatest_lt[0] == lowest_gt[0]): 

                for value in equalities:
                    if (value == greatest_lt[0]):
                        return True
    return False

##############################################################################################
# *** Part of the Redundant Pattern Detector ***
# 
# Function generalize_variable_names takes in a pattern and variable ID from a relational
# conditional pattern clause in a function and then returns the same pattern with the all of
# the variable ID tags inside the pattern generalized into a "var_1", var_2 format. The ID from
# the pattern that was also passed in is generalized into the tag assosiated with that ID's
# postion within the pattern. 
#
# form: ( pattern , selected variable from pattern )
#       ( (x,y) , x )
# Example Input : (('tuple', [('id', 'x'), ('id', 'y')]), ('id', 'x'))
#         Output: (('tuple', [('id', 'var_0'), ('id', 'var_1')]), ('id', 'var_0'))
tag_number = 0

def generalize_variable_names( pattern, variable ):

    pattern_type = pattern[0] #Peek into the head
    return_variable = ('id',"var") #Dummy value will be copied over
    global tag_number 

    if pattern_type in ['tuple','list']:               # Walk through tuples and lists
        tuple_contents = pattern[1].copy()
    elif pattern_type == 'apply':                      # Structures/Objects
        tuple_contents = (pattern[2])[1]
    elif pattern_type == 'id':                         # Single variable  
        tuple_contents = ('id', "var_"  + str(tag_number))
        return_variable = ('id', "var_"  + str(tag_number))
        tag_number = (tag_number + 1)           # Update global tag index
        return (  tuple_contents , return_variable  )
    elif ((pattern_type == 'head-tail') or (pattern_type == 'raw-head-tail')):#Head/Tail structure
        next_node = generalize_variable_names( pattern[1], variable)
        nodes = generalize_variable_names( pattern[2], variable)
        return ( (pattern[0], next_node[0], nodes[0]), variable)
    elif (pattern_type == 'named-pattern'):
        (NAME_PTRN,ID,ARGS) = pattern
        tuple_contents = generalize_variable_names(ID,variable)
        return (NAME_PTRN, tuple_contents[0] ,ARGS) # ttc TODO fix named patterns
    else:
        print("ERROR: redundant pattern detection - generalize_variable_names\nUnknown pattern type: ",pattern)
        return

    # Process through the contents of a tuple/list/structure contents
    for index in range(len(tuple_contents)):

        #If we have found an id, we should generalize it
        if ((tuple_contents[index])[0] == 'id'):
            if ((tuple_contents[index])[1] == variable[1]):
                return_variable = ('id', "var_"  + str(tag_number))
            tuple_contents[index] = ('id', "var_" + str(tag_number))
            tag_number = (tag_number + 1) # Update globabl tag index
        
        #If we have found a tuple/list, we should process its contents
        elif (tuple_contents[index])[0] in ['tuple','list']:
            structure_contents = generalize_variable_names( (tuple_contents[index]) , variable )
            tuple_contents[index] = ((tuple_contents[index])[0], structure_contents[0])
    
    # Reconstruct the node with the generalized pattern
    if pattern_type in ['tuple','list']: 
        return ( (pattern[0], tuple_contents ) , return_variable )
    elif pattern_type == 'apply':
        return ( ( pattern[0],pattern[1],tuple_contents ), return_variable )
        