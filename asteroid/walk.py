#########################################################################
# A tree walker to interpret Asteroid programs
#
# (c) University of Rhode Island
#########################################################################

from copy import deepcopy,copy
from re import match as re_match

from asteroid.globals import *
from asteroid.support import *
from asteroid.state import state, warning

#########################################################################
# These two variables are used by the debugger. "Debugging" is a flag
# to tell if we're currently in debugging mode and "Debugger" is a
# debugger object. For more information on the debugger's helper
# functions. See the function definitions in this file.
#########################################################################
debugging = False
debugger = None

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
def __unify(term, pattern, unifying = True ):
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

    ### Python value level matching
    # NOTE: in the first rules where we test instances we are comparing
    # Python level values, if they don't match exactly then we have
    # a pattern match fail.
    if isinstance(term, str): # apply regular expression match
        message_explicit("Regex match: {} and {}", [pattern, term], level="secondary")

        if isinstance(pattern, str) and re_match("^"+pattern+"$", term):
            # Note: a pattern needs to match the whole term.
            message_explicit("Matched!", level="tertiary")
            return [] # return empty unifier
        else:
            message_explicit("Failed!", level="tertiary")
            raise PatternMatchFailed(
                "regular expression '{}' did not match '{}'"
                .format(pattern, term))

    elif isinstance(term, (int, float, bool)):
        message_explicit("Literal match: {} and {}", [pattern, term], level="primary")

        if term == pattern:
            message_explicit("Matched!", level="secondary")
            return [] # return an empty unifier
        else:
            message_explicit("Failed! {} != {}", [pattern, term], level="secondary")
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
            message_explicit("Matching lists: {} and {}",
                [gen_t2s( ('list', pattern) ), gen_t2s( ('list', term) )],
                increase=True, notify=True)

            # Make our unifier(s)
            unifier = []
            for i in range(len(term)):
                unifier += unify(term[i], pattern[i], unifying)

            message_explicit("Matched!", decrease=True)

            # Ensure we have no non-linear patterns
            check_repeated_symbols(unifier)

            return unifier
    
    elif ((not unifying) and (term[0] == 'named-pattern')):

        # Unpack a term-side named-pattern if evaluating redundant clauses
        message_explicit("Evaluating named pattern", notify=True)
        return unify(term[2],pattern,unifying)

    elif ((not unifying) and (term[0] == 'deref')):

        # Unpack a term-side first-class pattern if evaluating redundant clauses
        term_pattern = walk(term[1])
        return unify(term_pattern, pattern, unifying)

    ### Asteroid value level matching
    elif pattern[0] == 'object' and term[0] == 'object':
        # this can happen when we dereference a variable pointing
        # to an object as a pattern, e.g.
        #    let o = A(1,2). -- A is a structure with 2 data members
        #    let *o = o.
        (OBJECT, (STRUCT_ID, (ID, pid)), (OBJECT_MEMORY, (LIST, pl))) = pattern
        (OBJECT, (STRUCT_ID, (ID, tid)), (OBJECT_MEMORY, (LIST, tl))) = term
        if pid != tid:
            raise PatternMatchFailed(
                "pattern type '{}' and term type '{}' do not agree"
                .format(pid,tid))

        message_explicit("Matching objects {}{} and {}{}",
            [pid, gen_t2s( ('tuple', pl) ), tid, gen_t2s( ('tuple', tl) )],
            increase=True
        )

        unifiers = []

        for i in range(len(pl)):
            # OWM: We can't actually unify function-vals. If we know the type is
            # the same, can we just assume the function-val is the same?
            notify_explicit()

            if tl[i][0] != 'function-val':
                unifiers += unify(tl[i], pl[i])
        
        message_explicit("Objects matched", decrease=True)
        return unifiers

    # no implicit type conversions during pattern matching
    # removing this code, see the discussion in issue 95
    # elif pattern[0] == 'string' and term[0] != 'string':
        # regular expression applied to a non-string structure
        # this is possible because all data types are subtypes of string
    #    message_explicit("Matching string {} and non-string {}",
    #        [gen_t2s(pattern), gen_t2s(term)],
    #        notify=True
    #    )
    #
    #    return unify(term2string(term), pattern[1])

    elif pattern[0] == 'if-exp':

        # If we are evaluating subsumtion
        if not unifying:

            # If we are evaluating subsumption between two different conditional patterns
            # we want to 'punt' and print a warning message.
            if term[0] == 'if-exp':

                if not state.cond_warning:
                    warning("Redundant pattern detection is not supported for conditional pattern expressions.")
                    state.cond_warning = True

            # Otherwise if the term is not another cmatch the clauses are correctly ordered.
            raise PatternMatchFailed(
                "Subsumption relatioship broken, pattern will not be rendered redundant.")

        (IF_EXP, cond_exp, pexp, else_exp) = pattern

        if else_exp[0] != 'null':
            raise PatternMatchFailed("conditional patterns do not support 'else' clauses")

        # Explicit messaging
        message_explicit("Conditional match: if ({})",
            [gen_t2s(cond_exp)],
            notify=True, increase=True
        )

        unifiers = unify(term, pexp, unifying)

        # evaluate the conditional expression in the
        # context of the unifiers.
        declare_unifiers(unifiers)
        bool_val = map2boolean(walk(cond_exp))

        if bool_val[1]:
            message_explicit("Condition met, {}",
                [gen_t2s(cond_exp)], decrease=True
            )
            return unifiers
        else:
            message_explicit("Condition ({}) failed",
                [gen_t2s(cond_exp)], decrease=True
            )
            raise PatternMatchFailed(
                "conditional pattern match failed")

    elif term[0] == 'if-exp':
        # We will only get here when evaluating subsumption

        # If we get here, a conditional pattern clause is placed after a non-conditonal
        # pattern clause. Therefore, we need to check if the subsume because if they do
        # the conditonal clause is redundant.
        (IF_EXP, cond_exp, pexp, else_exp) = term

        if else_exp[0] != 'null':
            raise PatternMatchFailed("conditional patterns do not support 'else' clauses")

        #return unify(pexp,pattern,False)
        # Otherwise if the term is not another cmatch the clauses are correctly ordered.
        raise PatternMatchFailed(
            "conditional patterns not supported.")

    elif pattern[0] == 'typematch':
        typematch = pattern[1]
        nextIndex = 0 #indicates index of where we will 'look' next
        
        message_explicit("Typematch {} to type {}", [gen_t2s(term), typematch])

        if typematch in ['string','real','integer','list','tuple','boolean','none']:

            if (not unifying):

                #walk a different path for this node
                if (term[0] == 'typematch'):
                    nextIndex = 1

                #handle lists/head-tails subsuming each other
                if (term[0] in ["list","head-tail"]):
                    if ((typematch == 'list')):
                        return []

            if typematch == term[nextIndex]:
                message_explicit("Success!", level="secondary")
                return []
            else:
                message_explicit("Failure", level="secondary")
                raise PatternMatchFailed(
                    "expected type '{}' got a term of type '{}'"
                    .format(typematch, term[nextIndex]))

        elif typematch == 'function':
            # matching function and member function values
            if term[0] in ['function-val','member-function-val']:
                message_explicit("Success!", level="secondary")
                return []
            else:
                message_explicit("Failure", level="secondary")
                raise PatternMatchFailed(
                    "expected type '{}' got a term of type '{}'"
                    .format(typematch, term[0]))

        elif typematch == 'pattern':
            if unifying:

                # any kind of structure can be a pattern, and variables
                # see globals.py for a definition of 'patterns'
                if term[nextIndex] in patterns:
                    message_explicit("Success!", level="secondary")
                    return []
                else:
                    message_explicit("Failure", level="secondary")
                    raise PatternMatchFailed(
                            "expected type '{}' got a term of type '{}'"
                            .format(typematch, term[0]))

            else: # Evaluating typematch-pattern subsumption
                #walk a different path for this one node
                if (term[0] == 'typematch'):
                    nextIndex = 1

                #handle lists/head-tails subsuming each other
                if (term[0] in ["list","head-tail"]):
                    if ((typematch == 'list')):
                        message_explicit("Success!", level="secondary")
                        return []

                if term[nextIndex] in pattern_subsumes:
                    message_explicit("Success!", level="secondary")
                    return []
                else:
                    message_explicit("Failure", level="secondary")
                    raise PatternMatchFailed(
                        "expected type '{}' got a term of type '{}'"
                        .format(typematch, term[nextIndex]))

        elif term[0] == 'object':
            if state.symbol_table.lookup_sym(typematch)[0] != 'struct':
                raise PatternMatchFailed( "'{}' is not a type".format(typematch) )

            (OBJECT,
                (STRUCT_ID, (ID, struct_id)),
                (OBJECT_MEMORY, LIST)) = term
            if struct_id == typematch:
                message_explicit("Success!", level="secondary")
                return []
            else:
                message_explicit("Failure", level="secondary", increase=True)
                raise PatternMatchFailed(
                    "expected type '{}' got an object of type '{}'"
                    .format(typematch, struct_id))

        else:
            if state.symbol_table.lookup_sym(typematch)[0] != 'struct':
                message_explicit("Failure", level="secondary", increase=True)

                raise PatternMatchFailed( "'{}' is not a type".format(typematch) )
            else:
                message_explicit("Failure", level="secondary", increase=True)

                raise PatternMatchFailed(
                    "expected type '{}' got an object of type '{}'"
                    .format(typematch, term[0]))


    elif pattern[0] == 'named-pattern':
        # unpack pattern
        (NAMED_PATTERN, name_exp, p) = pattern
        message_explicit("Matching term {} and pattern {} to [{}]",
            [gen_t2s(term), gen_t2s(p), gen_t2s(name_exp)],
            notify=True, increase=True
        )
        # name_exp has to be an id or an index expression.
        if name_exp[0] not in ['id','index']:
            raise ValueError(
                "expected a variable, list element, or data member in named pattern\
                 instead of {}".format(term2string(name_exp)))

        unifiers = unify(term, p, unifying) + [(name_exp, term)]
        
        message_explicit("Matched ({} and {})", 
            [gen_t2s(unifiers[0][0]), gen_t2s(unifiers[0][1])],
            decrease=True
        )

        return unifiers

    elif pattern[0] == 'none':
        if term[0] != 'none':
            message_explicit("{} and none do not match",
                [gen_t2s(term)]
            )
            raise PatternMatchFailed("expected 'none' got '{}'"
                    .format(term[0]))
        else:
            message_explicit("None and None match", level="secondary")
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

    elif pattern[0] == 'pattern':
        # pattern operator on the pattern side can always be ignored
        # --TODO double check - ttc
        if term[0] == 'pattern':
            return unify(term[1], pattern[1], unifying)
        else:
            return unify(term, pattern[1], unifying)

    elif term[0] == 'pattern' and pattern[0] not in ['id', 'index']:
        # ignore pattern operator on the term if we are not trying to unify term with
        # a variable or other kind of lval
        return unify(term[1], pattern, unifying)

    elif term[0] == 'object' and pattern[0] == 'apply':
        # unpack term
        (OBJECT,
         (STRUCT_ID, (ID, struct_id)),
         (OBJECT_MEMORY, (LIST, obj_memory))) = term
        # unpack pattern
        (APPLY,
         (ID, apply_id),
         arg) = pattern
        type = state.symbol_table.lookup_sym(apply_id)
        if type[0] != 'struct':
            raise PatternMatchFailed("'{}' is not a type".format(apply_id))

        if struct_id != apply_id:
            raise PatternMatchFailed("expected type '{}' got type '{}'"
                .format(apply_id, struct_id))
        # we are comparing raw lists here
        if arg[0] == 'tuple':
            pattern_list = arg[1]
        else:
            pattern_list = [arg]

        # only pattern match on object data members
        message_explicit("Matching object {}{} and pattern {}{}",
            [apply_id,  gen_t2s( ('tuple', data_only(obj_memory)) ),
            struct_id, gen_t2s( ('tuple', pattern_list) )],
            notify=True
        )
        # Running through the list elements indivuidually allows for
        # better debugging information
        unifiers = []
        data = data_only(obj_memory)

        for i in range(len(data)):
            unifiers += unify(data[i], pattern_list[i], unifying)

        return unifiers

    elif pattern[0] == 'index': # list element lval access
        unifier = (pattern, term)
        return [unifier]

# lhh: looking at patterns as values we are now allowed to match
# against patterns as terms where the terms now include variables.
#    elif term[0] == 'id' and unifying: # variable in term not allowed
#        raise PatternMatchFailed(      # when unifying
#            "variable '{}' in term not allowed"
#            .format(term[1]))

    elif pattern[0] == 'id': # variable in pattern add to unifier
        sym = pattern[1]
        if sym == '_': # anonymous variable - ignore unifier
            return []
        else:
            unifier = (pattern, term)
            return [unifier]

    elif pattern[0] in ['head-tail', 'raw-head-tail']:
        message_explicit("Matching {} to {}",
            [gen_t2s(term), gen_t2s(pattern)],
            notify=True
        )
        # if we are unifying or we are not evaluating subsumption
        #  to another head-tail
        if unifying or term[0] not in ['head-tail', 'raw-head-tail']:
            (HEAD_TAIL, pattern_head, pattern_tail) = pattern
            (LIST, list_val) = term

            if LIST != 'list':
                message_explicit("Failed", level="secondary")
                raise PatternMatchFailed(
                    "head-tail operator expected type 'list' got type '{}'"
                    .format(LIST))

            if not len(list_val):
                message_explicit("Failed", level="secondary")

                raise PatternMatchFailed(
                    "head-tail operator expected a non-empty list")

            list_head = list_val[0]
            list_tail = ('list', list_val[1:])
            unifier = []

            unifier += unify(list_head, pattern_head, unifying)
            unifier += unify(list_tail, pattern_tail, unifying)

            check_repeated_symbols(unifier) #Ensure we have no non-linear patterns
            message_explicit("Success!", level="secondary")

            return unifier

        else: #Else we are evaluating subsumption to another head-tail

            lengthH = head_tail_length(pattern) #H->higher order of predcence pattern
            lengthL = head_tail_length(term)    #L->lower order of predcence pattern

            if lengthH == 2 and lengthL != 2:
                return unify(pattern[1],term[1],unifying)

            if (lengthH > lengthL): # If the length of the higher presedence pattern is greater
                                    # then length of the lower precedence pattern, it is not redundant
                message_explicit("Failed", level="secondary")
                raise PatternMatchFailed(
                    "Subsumption relatioship broken, pattern will not be rendered redundant.")

            else: #Else we continue evaluating the different terms in the head-tail pattern
                (HEAD_TAIL, patternH_head, patternH_tail) = pattern
                (HEAD_TAIL, patternL_head, patternL_tail) = term

                unifier = []
                for i in range(lengthH):
                    unifier += unify(patternL_head,patternH_head,unifying)
                    try:
                        (RAW_HEAD_TAIL, patternH_head, patternH_tail) = patternH_tail
                        (RAW_HEAD_TAIL, patternL_head, patternL_tail) = patternL_tail
                    except:
                        break

                check_repeated_symbols(unifier) #Ensure we have no non-linear patterns
                message_explicit("Success!", level="secondary")
                return unifier

    elif pattern[0] == 'deref':  # ('deref', v, bl)
        # v can be an AST representing any computation
        # that produces a pattern.

        message_explicit("Dereferencing {}", [gen_t2s(pattern[1])])

        p = walk(pattern[1])

        if pattern[2][0] != 'nil': # we have a binding term list
            if p[1][0] != 'constraint':
                raise ValueError(
                    "binding term lists only supported for constraint patterns")
            else: 
                # construct a new constraint pattern with the binding term list in place
                p = ('pattern',
                        ('constraint',
                            p[1][1],
                            pattern[2]))


        message_explicit("{} -> {}", [gen_t2s(pattern), gen_t2s(p)], 
            level="secondary")

        notify_explicit()

        return unify(term,p,unifying)

    # builtin operators look like apply lists with operator names
    elif pattern[0] == 'apply':
        if term[0] != pattern[0]: # make sure both are applys
            raise PatternMatchFailed(
                "term and pattern disagree on structure")

        # unpack the apply structures
        (APPLY, (ID, t_id), t_arg) = term
        (APPLY, (ID, p_id), p_arg) = pattern

        # make sure apply id's match
        if t_id != p_id:
            raise PatternMatchFailed(
                "term '{}' does not match pattern '{}'"
                .format(t_id, p_id))

        # unify the args
        return unify(t_arg, p_arg, unifying)

    elif pattern[0] == 'constraint':
        message_explicit("[Begin] constraint pattern: {}",
            [gen_t2s(pattern[1])],
            notify=True, increase=True
        )

        p = pattern[1]
        bl = pattern[2] # binding term list

        # constraint patterns are evaluated in their own scope
        state.symbol_table.push_scope({})
        unifier = unify(term,p)
        state.symbol_table.pop_scope()

        message_explicit("[End] constraint pattern", decrease=True)

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

    elif term[0] != pattern[0]:  # nodes are not the same
        message_explicit("Fail: {} and {} are not the same",
            [term[0], pattern[0]], level="secondary")
        
        raise PatternMatchFailed(
            "nodes '{}' and '{}' are not the same"
            .format(term[0], pattern[0]))

    elif len(term) != len(pattern): # nodes are not of same the arity
        message_explicit("Fail: {} and {} are not the same arity",
            [term[0], pattern[0]], level="secondary")

        raise PatternMatchFailed(
            "nodes '{}' and '{}'' are not of the same arity"
            .format(term[0], pattern[0]))

    else:
        #lhh
        #print("unifying {}".format(pattern[0]))
        unifier = []
        for i in range(1,len(term)):
            unifier += unify(term[i], pattern[i], unifying)
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
         (OBJECT_MEMORY, (LIST, memory))) = structure_val
        # compute the index -- for objects this has to be done
        # in the context of the struct scope
        struct_val = state.symbol_table.lookup_sym(struct_id)
        # unpack the struct value
        (STRUCT,
         (MEMBER_NAMES, (LIST, member_names)),
         (STRUCT_MEMORY, (LIST, struct_memory))) = struct_val

        if ix[0] == 'id' and ix[1] in member_names:
            ix_val = ('integer', member_names.index(ix[1]))
        else:
            ix_val = walk(ix)

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
         (OBJECT_MEMORY, (LIST, memory))) = structure_val
        # compute the index -- for objects this has to be done
        # in the context of the struct scope
        struct_val = state.symbol_table.lookup_sym(struct_id)
        # unpack the struct value
        (STRUCT,
         (MEMBER_NAMES, (LIST, member_names)),
         (STRUCT_MEMORY, (LIST, struct_memory))) = struct_val

        if ix[0] == 'id' and ix[1] in member_names:
            ix_val = ('integer', member_names.index(ix[1]))
        else:
            ix_val = walk(ix)

    elif structure_val[0] == 'pattern':
        # simple patterns are just structures - skip the pattern operator
        return store_at_ix(structure_val[1], ix, value)

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
                raise ValueError("unsupported type '{}' in +".format(type))
        elif opname == '__minus__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real']:
                return (type, val_a[1] - val_b[1])
            else:
                raise ValueError("unsupported type '{}' in -".format(type))
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
            if type in ['integer', 'real', 'list', 'tuple', 'boolean', 'none']:
                return ('boolean', val_a[1] == val_b[1])
            elif type == 'string':
                return ('boolean', term2string(val_a) == term2string(val_b))
            else:
                raise ValueError('unsupported type in ==')
        elif opname  == '__ne__':
            type = promote(val_a[0], val_b[0])
            if type in ['integer', 'real', 'list', 'tuple', 'boolean', 'none']:
                return ('boolean', val_a[1] != val_b[1])
            elif type == 'string':
                return ('boolean', term2string(val_a) != term2string(val_b))
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
            raise ValueError("unknown builtin binary operation '{}'".format(opname))

    elif opname in unary_operators:
        arg_val = walk(args)

        if opname == '__not__':
            val = map2boolean(arg_val)
            if val[1] == False:
                return ('boolean', True)
            elif val[1] == True:
                return ('boolean', False)
            else:
                raise ValueError("not a boolean value in 'not'")
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
        else:
            raise ValueError("unknown builtin unary operation '{}'".format(opname))

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
    # Needed for later
    global debugging

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
    state.symbol_table.saved_configs.append(
        state.symbol_table.get_config()
    )

    state.symbol_table.set_config(closure)
    state.symbol_table.push_scope({})

    # Explicit message
    message_explicit("Call: {}({})",
        [fname, gen_t2s(actual_val_args)],
        increase=True
    )

    # We want to return back to whatever tab level the function
    # call started at, so, we grab this here to reset to at
    # the return. This allows recursive functions to have
    # the proper formatting
    if debugging: cur_tab_level = debugger.tab_level

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
        (STMT_LIST, stmts)) = body_list_val[ i + 1]

        message_explicit("Attempting to match {} with pattern {}",
            [gen_t2s(actual_val_args), gen_t2s(p)], level="primary",
            notify=True, increase=True
        )
        
        try:
            # Attempt to unify the actual args and the pattern
            unifiers = unify(actual_val_args, p)
            unified = True

            # Do our explicit message
            message_explicit("Success! Matched function body", level="primary", decrease=True)

        except PatternMatchFailed:
            # Reset the tab level
            if explicit_enabled():
                debugger.tab_level = cur_tab_level + 1

            # Print the explicit messaging
            message_explicit("Failed to match function body", level="tertiary", decrease=True)

            unifiers = []
            unified = False

        if unified:
            break

    # Reset the tab level back to the function call's level
    if debugging: debugger.tab_level = cur_tab_level

    if not unified:
        raise ValueError("actual argument '{}' not recognized by function '{}'"
                         .format(term2string(actual_val_args),fname))
    declare_formal_args(unifiers)

    # if we have an obj reference bind it to the
    # variable 'this'
    if obj_ref:
        state.symbol_table.enter_sym('this', obj_ref)

    # OWM: The following segment is a repeat of the bottom of this function.
    # We need to do this because redundant patterns can break scope and
    # some debugger and state features as they exit computation.

    # Check for useless patterns
    try:
        if state.eval_redundancy:
            old_debugging = debugging
            debugging = False
            check_redundancy(body_list, fname)
            debugging = old_debugging

    # Reset settings
    except RedundantPatternFound as r:
        debugging = old_debugging
        # restore caller's env
        state.lineinfo = old_lineinfo
        # Keep debugger up to date
        if debugging: debugger.set_lineinfo(state.lineinfo)
        pop_stackframe()
        raise r

    # execute the function
    global function_return_value

    try:
        function_return_value.append(None)

        walk_stmt_list(stmts)

        # Pop the return value
        val = function_return_value.pop()

        # If we have one, set it as the retval
        if val:
            return_value = val

        # Otherwhise default the return value to none
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
    state.lineinfo = old_lineinfo
    # Keep debugger up to date
    if debugging: debugger.set_lineinfo(state.lineinfo)
    pop_stackframe()
    message_explicit("Return: {} from {}",
            [("None" if (not return_value[1]) else gen_t2s(return_value)), 
            fname],
            decrease=True
    )

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
    
    # Explicit messaging for unifiers
    if explicit_enabled():
        # if we've unified anything
        if unifiers:
            # Create our format string and list of terms
            fstring = ""
            terms = []

            # For each unifier except the last one
            for (lval, value) in unifiers[:-1]:
                # Add to our format string and our terms
                fstring += "{} = {}, "
                terms += [gen_t2s(lval), gen_t2s(value)]

            # Get the last unifier and add it to the terms and fstring
            (lval, value) = unifiers[-1]
            fstring += "{} = {}"
            terms += [gen_t2s(lval), gen_t2s(value)]

            # Print our message
            message_explicit(fstring, terms)

#########################################################################
# node functions
#########################################################################
def global_stmt(node):
    notify_debugger()

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
    
    message_explicit("Global defs: {}", [global_str[:-1]] )

#########################################################################
def assert_stmt(node):
    notify_debugger()

    (ASSERT, exp) = node
    assert_match(ASSERT, 'assert')

    exp_val = walk(exp)

    # mapping asteroid assert into python assert
    assert exp_val[1], 'assert failed'
#########################################################################
def unify_stmt(node):
    notify_debugger()

    (UNIFY, pattern, exp) = node
    assert_match(UNIFY, 'unify')

    term = walk(exp)

    unifiers = unify(term, pattern)

    declare_unifiers(unifiers)

#########################################################################
def return_stmt(node):
    notify_debugger()

    (RETURN, e) = node
    assert_match(RETURN, 'return')
    
    retval = walk(e)

    raise ReturnValue(retval)

#########################################################################
def break_stmt(node):
    notify_debugger()

    (BREAK,) = node
    assert_match(BREAK, 'break')

    raise Break()

#########################################################################
def throw_stmt(node):
    notify_debugger()

    (THROW, object) = node
    assert_match(THROW, 'throw')
    
    throw_object = walk(object)

    raise ThrowValue(throw_object)

#########################################################################
def try_stmt(node):
    notify_debugger()

    (TRY,
     (STMT_LIST, try_stmts),
     (CATCH_LIST, (LIST, catch_list))) = node

    stepping = debugger_has_stepped()
    try:
        walk_stmt_list(try_stmts)

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
                         ('object-memory',
                          ('list',
                           [('string', 'PatternMatchFailed'),
                            ('string', inst.value)])))
        inst_val = inst

    except RedundantPatternFound as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('object-memory',
                          ('list',
                           [('string', 'RedundantPatternFound'),
                            ('string', str(inst))])))
        inst_val = inst

    except NonLinearPatternError as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('object-memory',
                          ('list',
                           [('string', 'NonLinearPatternError'),
                            ('string', str(inst))])))
        inst_val = inst

    except ArithmeticError as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('object-memory',
                          ('list',
                           [('string', 'ArithmeticError'),
                            ('string', str(inst))])))
        inst_val = inst

    except FileNotFoundError as inst:
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
                         ('object-memory',
                          ('list',
                           [('string', 'FileNotFound'),
                            ('string', str(inst))])))
        inst_val = inst

    except Exception as inst:
        # mapping general Python exceptions into Asteroid's SystemError
        except_val = ('object',
                         ('struct-id', ('id', 'Exception')),
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
         (CATCH_STMTS, catch_stmts)) = catch_val
        try:
            unifiers = unify(except_val, catch_pattern)
        except PatternMatchFailed:
            pass
        else:
            # handler found - null out error_trace
            state.error_trace = None
            declare_unifiers(unifiers)
            walk_stmt_list(catch_stmts, step_state=stepping)
            return

    # no exception handler found - rethrow the exception
    raise inst_val

#########################################################################
def loop_stmt(node):
    notify_debugger()

    (LOOP, body_stmts) = node
    assert_match(LOOP, 'loop')

    (STMT_LIST, body) = body_stmts

    try:
        stepping = debugger_has_stepped()
        while True:
            walk_stmt_list(body, step_state=stepping)
    except Break:
        pass

#########################################################################
def while_stmt(node):
    notify_debugger()

    (WHILE, cond_exp, body_stmts) = node
    assert_match(WHILE, 'while')

    (COND_EXP, cond) = cond_exp
    (STMT_LIST, body) = body_stmts

    try:
        stepping = debugger_has_stepped()

        (COND_TYPE, cond_val) = map2boolean(walk(cond))
        while cond_val:
            walk_stmt_list(body, step_state=stepping)
            (COND_TYPE, cond_val) = map2boolean(walk(cond))
    except Break:
        pass

#########################################################################
def repeat_stmt(node):
    notify_debugger()

    (REPEAT, body_stmts, cond_exp) = node
    assert_match(REPEAT, 'repeat')

    (COND_EXP, cond) = cond_exp
    (STMT_LIST, body) = body_stmts

    try:
        stepping = debugger_has_stepped()
        while True:
            walk_stmt_list(body, step_state=stepping)
            (COND_TYPE, cond_val) = map2boolean(walk(cond))
            if cond_val:
                break

    except Break:
        pass

#########################################################################
def for_stmt(node):
    notify_debugger()

    (FOR, (IN_EXP, in_exp), (STMT_LIST, stmt_list)) = node
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
        stepping = debugger_has_stepped()
        for term in list_val:
            try:
                unifiers = unify(term,pattern)
            except PatternMatchFailed:
                pass
            else:
                declare_unifiers(unifiers)
                walk_stmt_list(stmt_list, step_state=stepping)
                
    except Break:
        pass

#########################################################################
def if_stmt(node):
    notify_debugger()

    (IF, (LIST, if_list)) = node
    assert_match(IF, 'if')
    assert_match(LIST, 'list')

    for i in range(0,len(if_list),2):

        lineinfo = if_list[ i ]
        process_lineinfo(lineinfo)

        (IF_CLAUSE,
         (COND, cond),
         (STMT_LIST, stmts)) = if_list[ i + 1 ]

        (BOOLEAN, cond_val) = map2boolean(walk(cond))

        if cond_val:
            walk_stmt_list(stmts)
            break
#########################################################################
def struct_def_stmt(node):
    notify_debugger()

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
def import_stmt(node):
    notify_debugger()

    global debugging
    old_debugging = debugging
    debugging = False

    (LIST, inlist) = node
    assert_match(LIST, 'import_stmt')

    for e in inlist:
        walk(e)

    debugging = old_debugging

    return

#########################################################################
def top_level_exp_stmt(node):

    (TOP_LEVEL_EXP, exp) = node
    assert_match(TOP_LEVEL_EXP, 'top-level-exp')

    notify_debugger()

    return walk(exp)

#########################################################################
def eval_exp(node):

    (EVAL, exp) = node
    assert_match(EVAL, 'eval')
 
    # Note: eval is essentially a macro call - that is a function
    # call without pushing a symbol table record.  That means
    # we have to first evaluate the argument to 'eval' before
    # walking the term.  This is safe because if the arg is already
    # the actual term it will be quoted and nothing happens if it is
    # a variable it will be expanded to the actual term.

    state.trace_stack.append((state.lineinfo[0],
                              state.lineinfo[1],
                              "eval"))


    # evaluate actual parameter
    exp_val_expand = walk(exp)

    # now walk the actual term
    if exp_val_expand[0] == 'string':
        import frontend
        parser = frontend.Parser(filename="<eval>")
        eval_ast = parser.parse(exp_val_expand[1])
        walk(eval_ast)
        exp_val = function_return_value[-1]
    else:
        state.ignore_pattern += 1
        exp_val = walk(exp_val_expand)
        state.ignore_pattern -= 1

    state.trace_stack.pop()
    return exp_val

#########################################################################
def apply_exp(node):
    (APPLY, f, arg) = node
    assert_match(APPLY, 'apply')

    # handle builtin operators that look like apply lists.
    if f[0] == 'id' and f[1] in operator_symbols:
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
        # object member function
        (INDEX, ix, (ID, f_name)) = f
        # 'str' is necessary in case we use an index value
        # instead of a function name -- see regression test test085.ast
        f_name = "member function " + str(f_name)
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
                   ('object-memory', ('list', object_memory)))
        # if the struct has an __init__ function call it on the object
        # NOTE: constructor functions do not have return values.

        if '__init__' in member_names:
            slot_ix = member_names.index('__init__')
            init_fval = struct_memory[slot_ix]
            # calling a member function - push struct scope
            handle_call(obj_ref,
                        init_fval,
                        arg_val,
                        f_name)
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

    if debugging:
        debugger.set_lineinfo(lineinfo_val)

    #lhh
    #print("lineinfo: {}".format(lineinfo_val))

    state.lineinfo = lineinfo_val

#########################################################################
def deref_exp(node):

    # deref operators are only meaningful during pattern matching
    # it is an error to have the deref operator appear in an expression
    raise ValueError("dereferencing patterns is not valid in expressions.")

#########################################################################
def constraint_exp(node):

    # Constraint-only pattern matches should not exist where only an
    # expression is expected. If we get here, we have come across this
    # situation.
    # A constraint-only pattern match AST cannot be walked and therefore
    # we raise an error.
    raise ValueError("a constraint pattern cannot be used as an expression.")

#########################################################################
def set_ret_val(node):
    (SET_RET_VAL, exp) = node
    assert_match(SET_RET_VAL,'set-ret-val')

    global function_return_value
    val = walk(exp)
    function_return_value.pop()
    function_return_value.append(val)

    # Debugger keeps track of the most recent
    # return value
    if debugging and val:
        debugger.retval = term2string(val)

    return

#########################################################################
def walk_stmt_list(stmts, step_state=None):
    # step_stae is a flag to tell us if we actually
    # want to step through to the next line of the stmt
    # list while debugging
    
    # We only want to do this if we're debugging
    # and we've stepped through to this point or
    # have continued to this point. Basically, this
    # flag tells us if we've stepped or continued 
    # into this function call, effectively telling
    # the debugger to treat it as the top level

    # This also allows us to keep loop execution
    # control working with the debugger. Meaning
    # we can do a "next" on the last line in a 
    # loop and go back to the top as opposed to the
    # next line after the loop
    if step_state:
        stepping = step_state
    else:
        stepping = debugger_has_stepped()

    for s in stmts[1]:

        if debugging:
            # Set the debugger's info reflect new "top level"
            if stepping: debugger.set_top_level(True)

            # If we've hit a return and we're in continue-until-return
            # mode, notify the debugger

            # TODO: (OWM) get this working for implicit returns as well.
            # This currently doesn't work because you can have an implicit
            # return before a return statement. In this case, the return
            # statement consumes the return behavior because it is the
            # final statement in the stmt_list

            # OWM -- This is still true but I'm not sure the fix for it
            # nor if it should even be fixed

            # The statement is a return the debugger is executing until return
            # then notify the debugger
            if debugging and \
                debugger.exc['RETURN'] and s[0] == 'return':
                
                notify_debugger(at_return=True)
            
        walk(s)

#########################################################################
# walk
#########################################################################
def walk(node):
    # node format: (TYPE, [child1[, child2[, ...]]])
    type = node[0]

    if type == 'clear-ret-val':
        # implemented here instead of dictionary for efficiency reasons
        global function_return_value    

        # If we have no function return value, then append None    
        if function_return_value != []:
            function_return_value.pop()
        
        function_return_value.append(None)
        return
    elif type in dispatch_dict:
        node_function = dispatch_dict[type]
        return node_function(node)
    else:
        raise ValueError("feature '{}' not yet implemented".format(type))

#########################################################################
# walk_program
#########################################################################
def walk_program(node):
    """
    This function exists mostly as a performance
    boost to unify. Running the program through
    this function overrides the debugger wrapping
    on unify.
    """
    global unify
    unify = __unify

    walk(node)

#########################################################################
# debug_walk
#########################################################################
def debug_walk(node, dbg):
    """
    This function allows us to keep the top-level distinction
    at the program level.
    """
    global debugging, debugger, unify
    debugging, debugger = (True, dbg)

    unify = debug_unify

    (LIST, inlist) = node

    for e in inlist:
        # We want to differentiate between top level and nested
        # statements. So, we run the list of statements outright
        # so we can control when we know we are at the top level
        debugger.set_top_level(True)
        walk(e)

# a dictionary to associate tree nodes with node functions
dispatch_dict = {
    # Note: statement lists are now handled by separate functions outside
    #       the walk function
    # statements - statements do not produce return values
    'lineinfo'      : process_lineinfo,
    'set-ret-val'   : set_ret_val,
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
    'top-level-exp' : top_level_exp_stmt,
    # expressions - expressions do produce return values
    'list'          : list_exp,
    'import_stmt'   : import_stmt,
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
    'eval'          : eval_exp,
    # pattern code should be treated like a constant if not ignore_pattern
    'pattern'         : lambda node : walk(node[1]) if state.ignore_pattern else node,
    # constraint patterns
    'constraint'    : constraint_exp,
    'typematch'     : constraint_exp,
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
    'deref'         : deref_exp,
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
        assert_match(PTRN,'pattern')

        for j in range(i + 2, len(bodies), 2):
            lineinfo = bodies[ j ]
            process_lineinfo(lineinfo)

            #get the pattern with the lower level of precedence
            (BODY_L,(PTRN,ptrn_l),stmts_l) = bodies[j + 1]
            assert_match(BODY_L,'body')
            assert_match(PTRN,'pattern')

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

#######################################################################################
# *** Asteroid Debugger (ADB) helper functions ***
# 
# message_explicit:
#   The main messaging function for explicit mode. Operates in a similar way the
#   string.format function does. Read the function's internal docstring for more
#   information.
# 
# notify_debugger:
#   Notifies the debugger of the current program position. If the debugger is
#   accepting notifications at the time of call, the debugger will pause and
#   run the interactive prompt.
# 
# notify_explicit:
#   Similar to notify_debugger but only runs in explicit mode.
# 
# explicit_enabled:
#   A simple helper function. Equivalent to `debugging and debugger.explicit_enabled`
# 
# decrease_tab_level:
#   A simple helper function to manage debugger tab level decrementing.
#
# debugger_has_stepped::
#   Helper function that returns if the debugger has stepped into a function
#   body or other compound statement
#
# gen_t2s:
#   A generator function used in place of term2string in message_explicit calls.
#   This allows the computation to be deferred or completely ignored. Much like
#   a function returning a lambda function
#
# debug_unify:
#   A wrapper for the normal unify function that automatically decreases
#   the tab level of failed pattern matches.
#
#######################################################################################
def message_explicit(fmt_message, terms=None, level="primary", 
    increase=False, decrease=False, notify=False):

    """
    Message explicit expects 1-3 main arguments

    1. A string, which can be a format string or a normal string.
    2. (Optional) A list of terms to express for a given format string
    3. The message level (Defaults to primary)

    Format terms can be either plain strings or generators. If a gener-
    ator is supplied, this function will express it

    fmt_message is a list of terms to be applied in the same
    way as .format

    --------------------------------------------------------------
    This function also supports three other options that shorten
    common development patterns.

    increase: Increases the tab level after the message is written
    decrease: Decreases the tab level before the message is written
    notify: Makes a notify_explicit call to stop the debugger within
            a pattern
    """
    from types import GeneratorType

    if explicit_enabled():
        if decrease: decrease_tab_level()

        if not terms:
            debugger.message_explicit(fmt_message, level)
        else:
            expressed_terms = []
            for t in terms:
                if isinstance(t, GeneratorType):
                    expressed_terms.append(next(t))
                else:
                    expressed_terms.append(t)
            
            expressed_string = fmt_message.format(*expressed_terms)
            debugger.message_explicit(expressed_string, level)
        
        if increase:    debugger.tab_level += 1
        if notify:      notify_explicit()

#########################################################################
def notify_debugger(at_return=False):
    """
    If the debugger is accepting notifications at the time of call,
    the debugger will pause and run the interactive prompt.
    """
    if debugging:
        # We need to save the old lineinfo in case we go into a REPL
        old_lineinfo = state.lineinfo

        debugger.notify(at_return)

        # Reset our lineinfo
        state.lineinfo = old_lineinfo
        debugger.set_lineinfo(state.lineinfo)

#########################################################################
def notify_explicit():
    """
    If the debugger is accepting notifications at the time of call and
    the debugger is in explicit mode, the debugger will pause and run
    the interactive prompt.
    """
    if debugging and explicit_enabled:
        old_lineinfo = state.lineinfo
        
        debugger.notify_explicit()

        # Reset our lineinfo
        state.lineinfo = old_lineinfo
        debugger.set_lineinfo(state.lineinfo)

#########################################################################
def explicit_enabled():
    """
    Helper function that returns the state
    of explicit mode if debugging is enableds
    """
    return debugging and debugger.explicit_enabled

#########################################################################
def gen_t2s(node):
    """
    Generator function for term2string. This cuts down on
    runtime as it defers term2string computations to when
    they're actually needed.
    """
    yield term2string(node)

#########################################################################
def debugger_has_stepped():
    """
    Returns the step/continue state of the debugger
    """
    return debugging and \
            (debugger.exc['STEP'] or debugger.exc['CONTINUE'])

#########################################################################
def decrease_tab_level():
    """
    Decreases/handles the debugger's tab level
    """
    if debugging:
        debugger.tab_level = debugger.tab_level - 1

        if debugger.tab_level < 0:
            debugger.tab_level = 0

#########################################################################
def debug_unify(term, pattern, unifying = True):
    '''
    Proxy function for unify to make the debugger's formatting
    work properly

    OWM - This proxy function allows us to wrap unification
    in an except block and catch when PatternMatchFailed exceptions
    occur. In this situation, we need to decrease the tab level
    and then re-raise the exception. Being able to "unravel"
    tab levels is essential to making things look good.

    In normal, non-debugging execution, this function is overridden
    and not used.
    '''

    # Try to call the actual unify function
    try:
        return __unify(term, pattern, unifying)

    # If there's a pattern match failed, decrease
    # the tab level and reraise the exception
    except PatternMatchFailed as r:
        decrease_tab_level()
        raise r

    # If there's a normal exception, reraise it
    except Exception as e:
        raise e