#########################################################################
# A tree walker to generate code for AVM
#
# (c) University of Rhode Island
#########################################################################

from copy import deepcopy
from re import match as re_match

from asteroid.globals import *
from asteroid.support import *

#########################################################################
_indent_level = 1

def indent():
    code = ""
    for i in range(_indent_level):
        code += "   "
    return code

def inc_indent():
    global _indent_level
    _indent_level += 1

def dec_indent():
    global _indent_level
    if _indent_level == 0:
        raise ValueError("cannot decrement indent level")
    else:
        _indent_level -= 1

def set_indent(val):
    global _indent_level
    _indent_level = val

#########################################################################
def newline():
    return "\n"

#########################################################################
# function implementations
# this is a list of tuples (fname, fbody) that the frontend produces
func_impl_list = []

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

    exp_str = "{}".format(exp)

    code = ""
    code += "{}exp_val = walk({})\n".format(indent(),exp_str)
    code += "{}assert exp_val[1], 'assert failed'\n".format(indent())
    code += newline()

    return code

#########################################################################
def unify_stmt(node):

    (UNIFY, pattern, exp) = node
    assert_match(UNIFY, 'unify')

    exp_str = "{}".format(exp)
    pattern_str = "{}".format(pattern)

    code = ""
    code += "{}exp_val = walk({})\n".format(indent(),exp_str)
    code += "{}unifiers = unify(exp_val,{})\n".format(indent(),pattern_str)
    code += "{}declare_unifiers(unifiers)\n".format(indent())
    code += newline()

    return code

#########################################################################
def return_stmt(node):

    (RETURN, e) = node
    assert_match(RETURN, 'return')

    e_str = "{}".format(e)

    code = ""
    code += "{}val = walk({})\n".format(indent(),e_str)
    code += "{}state.symbol_table.pop_scope()\n".format(indent())
    code += "{}return val\n".format(indent())

    return code

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

    for i in range(0,len(if_list),2):

        lineinfo = if_list[ i ]
        process_lineinfo(lineinfo)

        (IF_CLAUSE,
         (COND, cond),
         (STMT_LIST, stmts)) = if_list[ i + 1 ]

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
    code = "{}# structure def for {}\n".format(indent(),struct_id)

    code += "{}member_list = {}\n".format(indent(),member_list)

    code += "{}struct_memory = []\n".format(indent())
    code += "{}member_names = []\n".format(indent())

    code += "{}for member_ix in range(len(member_list)):\n".format(indent())
    code += "{}    member = member_list[member_ix]\n".format(indent())
    code += "{}    if member[0] == 'data':\n".format(indent())
    code += "{}        (DATA, (ID, member_id)) = member\n".format(indent())
    code += "{}        struct_memory.append(('none', None))\n".format(indent())
    code += "{}        member_names.append(member_id)\n".format(indent())
    code += "{}    elif member[0] == 'unify':\n".format(indent())
    code += "{}        (UNIFY, (ID, member_id), function_exp) = member\n".format(indent())
    code += "{}        function_val = walk(function_exp)\n".format(indent())
    code += "{}        struct_memory.append(function_val)\n".format(indent())
    code += "{}        member_names.append(member_id)\n".format(indent())
    code += "{}    elif member[0] == 'noop':\n".format(indent())
    code += "{}        pass\n".format(indent())
    code += "{}    else:\n".format(indent())
    code += indent()+"        raise ValueError('unsupported struct member {}'.format(member[0]))\n"

    code += "{}struct_type = ('struct',('member-names', ('list', member_names)),('struct-memory', ('list', struct_memory)))\n".format(indent())
    code += "{}state.symbol_table.enter_sym('{}', struct_type)\n".format(indent(),struct_id)
    code += newline()

    return code

#########################################################################
def process_lineinfo(node):

    (LINEINFO, (module_name,lineno)) = node
    assert_match(LINEINFO, 'lineinfo')

    #lhh
    #print("lineinfo: {}".format(lineinfo_val))

    code = "{}set_lineinfo('{}',{})\n".format(indent(),module_name,lineno)

    return code

#########################################################################
def list_exp(node):

    (LIST, inlist) = node
    assert_match(LIST, 'list')

    code = ""

    for c in inlist:
        code += walk(c)

    return code

#########################################################################
def apply_stmt(node):
    code = indent()+"walk({})\n".format(node)
    return code

#########################################################################
def escape_stmt(node):

    (ESCAPE, (IMPLEMENTATION, fpointer)) = node
    assert_match(ESCAPE, 'escape')

    code = "{}{}()\n".format(indent(),fpointer)

    return code

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
    'apply'         : apply_stmt,
    'escape'        : escape_stmt,
}

#########################################################################
def gen_function(def_pair):

    (name,implementation) = def_pair
    code = ""

    if implementation[0] == 'body-list':
        code += "{}def {}(arg):\n".format(indent(),name)
        inc_indent()
        orig_indent = _indent_level

        # iterate over the bodies to find one that unifies with the actual parameters
        (BODY_LIST, (LIST, body_list_val)) = implementation

        for i in range(0, len(body_list_val), 2):
            # Process lineinfo
            (LINEINFO, (module_name, lineno)) = body_list_val[ i ]
            code += "{}set_lineinfo('{}',{})\n".format(indent(),module_name,lineno)
            # Deconstruct function body
            (BODY,
            (PATTERN, p),
            (STMT_LIST, stmts)) = body_list_val[ i + 1]

            code += "{}try:\n".format(indent())
            inc_indent()
            code += "{}unifiers = unify(arg,{})\n".format(indent(),p)
            code += str(indent())+"state.symbol_table.push_scope({})\n"
            code += "{}declare_formal_args(unifiers)\n".format(indent())
            code += walk(stmts)
            code += "{}state.symbol_table.pop_scope()\n".format(indent())
            dec_indent()
            code += "{}except PatternMatchFailed:\n".format(indent())
            inc_indent()

        code += "{}raise ValueError('none of the function bodies unified with actual parameters')\n".format(indent())
        set_indent(orig_indent)
        code += newline()
        dec_indent()

        return code

    elif implementation[0] == 'escape':
        import io
        (ESCAPE, program_string) = implementation
        buf = io.StringIO(program_string)

        code = ""
        code += "{}def {}():\n".format(indent(),name)
        inc_indent()
        s = buf.readline()
        while s:
            code += "{}{}".format(indent(),s)
            s = buf.readline()
        code += "{}avm.avm.__retval__ = __retval__\n".format(indent())
        code += newline()
        dec_indent()
        return code

    else:
        raise ValueError("unknown function implementation mode.")

#########################################################################
def gen_function_list():

    code = ""
    for p in func_impl_list:
        code += gen_function(p)

    return code

#########################################################################
def gen_dispatch():

    code = ""
    for (name,impl) in func_impl_list:
        code += "{}dispatch_table['{}'] = {}\n".format(indent(),name,name)

    code += newline()

    return code
