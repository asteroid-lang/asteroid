###########################################################################################
# Asteroid support code
#
# (c) Lutz Hamel, University of Rhode Island
###########################################################################################

from asteroid_state import state

#########################################################################
class PatternMatchFailed(Exception):
    def __init__(self, value):
        self.value = "pattern match failed: " + value

    def __str__(self):
        return(repr(self.value))

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
            "Pattern match failed: expected '{}' but got '{}'".
            format(expected, input))

###########################################################################################
# check if the two type tags match
def match(tag1, tag2):

    if tag1 == tag2:
        return True
    else:
        return False

###########################################################################################
# expression nodes not allowed in terms or patterns for unification. these are all nodes
# that express some sort of computation

unify_not_allowed = {
    'function',
    'to-list',
    'where-list',
    'raw-to-list',
    'raw-where-list',
    'if-exp',
    'foreign',
    'escape',
    'is',
    'in',
    'otherwise',
}

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

    NOTE: if the pattern looks like an lval then it is treated like an lval, e.g.
            let a@[0] = 'a@[0].
          stores the term 'a@[0] into lval a@[0].
    '''
    #lhh
    #print("unifying:\nterm {}\npattern {}\n\n".format(term, pattern))

    if isinstance(term, (int, float, bool, str)):
        try:
            if term == pattern:
                return []
            else:
                raise PatternMatchFailed(
                    "{} is not the same as {}"
                    .format(term, pattern))
        except: # just in case the comparison above throws an exception
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
                unifier += unify(term[i], pattern[i])
            return unifier

    elif pattern[0] == 'none':
        if term[0] != 'none':
            raise PatternMatchFailed("expected 'none' got '{}'"
                    .format(term[0]))
        else:
            return []

    # NOTE: functions/foreign are allowed in terms as long as they are matched
    # by a variable in the pattern - anything else will fail
    elif term[0] in (unify_not_allowed - {'function', 'foreign'}):
        raise PatternMatchFailed(
            "term of type '{}' not allowed in pattern matching"
            .format(term[0]))

    elif pattern[0] in unify_not_allowed:
        raise PatternMatchFailed(
            "pattern of type '{}' not allowed in pattern matching"
            .format(pattern[0]))

    elif pattern[0] == 'quote':
        # quotes on the pattern side can always be ignored
        return unify(term, pattern[1])

    elif term[0] == 'quote' and pattern[0] not in ['id', 'structure-ix']:
        # ignore quote on the term if we are not trying to unify term with
        # a variable or other kind of lval
        return unify(term[1], pattern)

    elif term[0] == 'object' and pattern[0] == 'apply-list':
        # unpack term
        (OBJECT,
         (CLASS_ID, (ID, class_id)),
         (OBJECT_MEMORY, (LIST, obj_memory))) = term
        # unpack pattern
        (APPLY_LIST,
         (LIST,
          [(ID, constructor_id),
           (TUPLE, pattern_tuple)])) = pattern
        if TUPLE != 'tuple':
            raise PatternMatchFailed(
                "Constructor '{}' expected tuple argument" \
                .format(constructor_id))
        if class_id != constructor_id:
            raise PatternMatchFailed("expected type '{}' got type '{}'"
                .format(constructor_id, class_id))
        return unify(obj_memory, pattern_tuple)

    elif pattern[0] == 'structure-ix': # list/constructor lval access
        unifier = (pattern, term)
        return [unifier]

    elif pattern[0] == 'id': # variable in pattern add to unifier
        sym = pattern[1]
        if sym == '_': # anonymous variable - ignore unifier
            return []
        else:
            unifier = (pattern, term)
            return [unifier]

    elif term[0] == 'id': # variable in term not allowed
        raise PatternMatchFailed(
            "variable '{}' in term not allowed"
            .format(term[1]))

    elif pattern[0] in ['head-tail', 'raw-head-tail']:
        # unpack the structures
        (HEAD_TAIL, pattern_head, pattern_tail) = pattern
        (LIST, list_val) = term

        if LIST != 'list':
            raise PatternMatchFailed(
                "head-tail operator expected type 'list' got type '{}'"
                .format(LIST))

        list_head = list_val[0]
        list_tail = ('list', list_val[1:])

        unifier = []
        unifier += unify(list_head, pattern_head)
        unifier += unify(list_tail, pattern_tail)
        return unifier

    elif pattern[0] == 'deref':  # ('deref', id)
        sym = pattern[1]
        p = state.symbol_table.lookup_sym(sym)
        return unify(term,p)

    elif pattern[0] == 'apply-list': # constructor
        if term[0] != pattern[0]: # make sure both are apply-lists
            raise PatternMatchFailed(
                "term and pattern disagree on 'apply-list' node")
        elif len(term[1]) > 2 or len(pattern[1]) > 2: # make sure only constructors
            raise PatternMatchFailed(
                "illegal function applications in pattern or term")

        # unpack the apply-list structures
        (APPLY_LIST,
         (LIST, [(ID, t_constr_id), t_arg])) = term

        (APPLY_LIST,
         (LIST, [(ID, p_constr_id), p_arg])) = pattern

        # make sure constructors match
        if t_constr_id != p_constr_id:
            raise PatternMatchFailed(
                "term '{}' does not match pattern '{}'"
                .format(t_constr_id, p_constr_id))

        # unify the args
        return unify(t_arg, p_arg)

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
            unifier += unify(term[i], pattern[i])
        #lhh
        #print("returning unifier: {}".format(unifier))
        return unifier

###########################################################################################
def promote(type1, type2, strict=True):
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
        if strict:
            if type1 == type2:
                raise ValueError("binary operation on type '{}' not supported".format(type1))
            else:
                raise ValueError("type '{}' and type '{}' are incompatible".format(type1, type2))

        else:
            return ('none', None)

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
#  All other values are considered true, in particular any object or constructor.
#
def map2boolean(value):

    if value[0] == 'none':
        return ('boolean', False)

    elif value[0] == 'boolean':
        return value

    elif value[0] in  ['integer', 'real', 'list', 'string']:
        return ('boolean', bool(value[1]))

    else:
        raise ValueError("unsupported type '{}' as truth value".format(value[0]))

###########################################################################################
def term2string(term):

    TYPE = term[0]

    if TYPE in ['id', 'integer', 'real', 'string']:
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
         (CLASS_ID, (ID, class_id)),
         (OBJECT_MEMORY, (LIST, object_memory))) = term
        term_string = class_id + '('
        for ix in range(0, len(object_memory)):
            term_string += term2string(object_memory[ix])
            term_string += ', ' if ix != len(object_memory)-1 else ''
        term_string += ')'
        return term_string

    elif TYPE == 'function':
        # TODO: decide whether it makes sense to print out functions
        return '(function ...)'

    elif TYPE == 'apply-list':
        (LIST, apply_list) = term[1]
        term_string = term2string(apply_list[0])
        for ix in range(1, len(apply_list)):
            if apply_list[ix][0] not in ['tuple', 'apply_list']:
                term_string += '('
            term_string += term2string(apply_list[ix])
            if apply_list[ix][0] not in ['tuple', 'apply_list']:
                term_string += ')'
        return term_string

    elif TYPE == 'quote':
        val = term[1]
        return "'" + term2string(val)

    elif TYPE == 'nil':
        return ''

    else:
        raise ValueError(
            "unknown type '{}' in term2string"
            .format(TYPE))

###########################################################################################
