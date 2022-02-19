#########################################################################
# A tree walker to generate code for the AVM
#
# (c) University of Rhode Island
#########################################################################

from copy import deepcopy
from re import match as re_match

from asteroid.globals import *
from asteroid.support import *

#########################################################################
# The target Python code is very fuzzy about indentation levels
# we use the following functions to manage indentation level in
# generated code

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

def get_indent():
    return _indent_level

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
    code = ""

    for id_tuple in id_list:
        (ID, id_val) = id_tuple
        code += indent()+"if state.symbol_table.is_symbol_local({}):\n".format(id_val)
        inc_indent()
        code += indent()+"raise ValueError('{} is already local, cannot be declared global')\n".format(id_val)
        dec_indent()
        code += indent()+"state.symbol_table.enter_global({})\n".format(id_val)

    code += newline()

    return code

#########################################################################
def assert_stmt(node):

    (ASSERT, exp) = node
    assert_match(ASSERT, 'assert')
    code = ""

    exp_str = "{}".format(exp)

    code += indent()+"exp_val = walk({})\n".format(exp_str)
    code += indent()+"assert exp_val[1], 'assert failed'\n"

    code += newline()

    return code

#########################################################################
def unify_stmt(node):

    (UNIFY, pattern, exp) = node
    assert_match(UNIFY, 'unify')
    code = ""

    exp_str = "{}".format(exp)
    pattern_str = "{}".format(pattern)

    code += indent()+"exp_val = walk({})\n".format(exp_str)
    code += indent()+"unifiers = unify(exp_val,{})\n".format(pattern_str)
    code += indent()+"declare_unifiers(unifiers)\n"

    code += newline()

    return code

#########################################################################
def return_stmt(node):

    (RETURN, e) = node
    assert_match(RETURN, 'return')
    code = ""

    e_str = "{}".format(e)

    code += indent()+"val = walk({})\n".format(e_str)
    code += indent()+"state.symbol_table.pop_scope()\n"
    code += indent()+"return val\n"

    code += newline()

    return code

#########################################################################
def break_stmt(node):

    (BREAK,) = node
    assert_match(BREAK, 'break')
    code = ""

    code += indent()+"break\n"
    code += newline()

    return code

#########################################################################
def throw_stmt(node):

    (THROW, object) = node
    assert_match(THROW, 'throw')
    code = ""

    code += indent()+"raise ThrowValue(walk({}))\n".format(object)
    code += newline()

    return code

#########################################################################
# helper function for try_stmt
def gen_catch(catch_list):

    code = ""
    catch_val = catch_list.pop(0)

    (CATCH,
     (CATCH_PATTERN, catch_pattern),
     (CATCH_STMTS, catch_stmts)) = catch_val
    code += indent()+"try:\n"
    inc_indent()
    code += indent()+"unifiers = unify(except_val, {})\n".format(catch_pattern)
    dec_indent()
    code += indent()+"except PatternMatchFailed:\n"
    inc_indent()
    if len(catch_list) == 0:
        code += indent()+"pass\n"
    else:
        code += gen_catch(catch_list)
    dec_indent()
    code += indent()+"else:\n"
    inc_indent()
    code += indent()+"declare_unifiers(unifiers)\n"
    code += walk(catch_stmts)
    code += indent()+"exception_handled = True\n"
    dec_indent()

    return code

#########################################################################
def try_stmt(node):

    (TRY,
     (STMT_LIST, try_stmts),
     (CATCH_LIST, (LIST, catch_list))) = node
    code = ""
    orig_indent = get_indent()

    code += indent()+"try:\n"
    inc_indent()
    code += walk(try_stmts)
    dec_indent()

    # NOTE: in Python the 'as inst' variable is only local to the catch block???
    # NOTE: we map user visible Python exceptions into standard Asteroid exceptions
    #       by constructing Exception objects - see prologue.ast

    code += indent()+"except ThrowValue as inst:\n"
    inc_indent()
    code += indent()+"except_val = inst.value\n"
    code += indent()+"inst_val = inst\n"
    dec_indent()

    code += indent()+"except PatternMatchFailed as inst:\n"
    inc_indent()
    code += indent()+"except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'PatternMatchFailed'),('string', inst.value)])))\n"
    code += indent()+"inst_val = inst\n"
    dec_indent()

    code += indent()+"except RedundantPatternFound as inst:\n"
    inc_indent()
    code += indent()+"except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'RedundantPatternFound'),('string', str(inst))])))\n"
    code += indent()+"inst_val = inst\n"
    dec_indent()

    code += indent()+"except NonLinearPatternError as inst:\n"
    inc_indent()
    code += indent()+"except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'NonLinearPatternError'),('string', str(inst))])))\n"
    code += indent()+"inst_val = inst\n"
    dec_indent()

    code += indent()+"except ArithmeticError as inst:\n"
    inc_indent()
    code += indent()+"except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'ArithmeticError'),('string', str(inst))])))\n"
    code += indent()+"inst_val = inst\n"
    dec_indent()

    code += indent()+"except FileNotFoundError as inst:\n"
    inc_indent()
    code += indent()+"except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'FileNotFound'),('string', str(inst))])))\n"
    code += indent()+"inst_val = inst\n"
    dec_indent()

    code += indent()+"except Exception as inst:\n"
    # mapping general Python exceptions into Asteroid's SystemError
    inc_indent()
    code += indent()+"except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'SystemError'),('string', str(inst))])))\n"
    code += indent()+"inst_val = inst\n"
    dec_indent()

    code += indent()+"else:\n"
    # no exceptions found in the try statements
    inc_indent()
    code += indent()+"except_val = None\n"
    dec_indent()

    # we had an exception - traverse the catch list and find an appropriate set of
    # catch statements.
    code += indent()+"if except_val:\n"
    inc_indent()

    code += indent()+"exception_handled = False\n"

    code += gen_catch(catch_list)

    # no exception handler found - rethrow the exception
    code += indent()+"if not exception_handled:\n"
    inc_indent()
    code += indent()+"raise inst_val\n"
    dec_indent()

    set_indent(orig_indent)
    code += newline()

    return code

#########################################################################
def loop_stmt(node):

    (LOOP, body_stmts) = node
    assert_match(LOOP, 'loop')
    (STMT_LIST, body) = body_stmts
    code = ""

    code += indent()+"while True:\n"
    inc_indent()
    code += walk(body)
    dec_indent()
    code += newline()

    return code

#########################################################################
def while_stmt(node):

    (WHILE, cond_exp, body_stmts) = node
    assert_match(WHILE, 'while')
    (COND_EXP, cond) = cond_exp
    (STMT_LIST, body) = body_stmts
    code = ""

    code += indent()+"while map2boolean(walk({}))[1]:\n".format(cond)
    inc_indent()
    code += walk(body)
    dec_indent()
    code += newline()

    return code

#########################################################################
def repeat_stmt(node):

    (REPEAT, body_stmts, cond_exp) = node
    assert_match(REPEAT, 'repeat')
    (COND_EXP, cond) = cond_exp
    (STMT_LIST, body) = body_stmts
    code = ""

    code += indent()+"while True:\n"
    inc_indent()
    code += walk(body)
    code += indent()+"if map2boolean(walk({}))[1]:\n".format(cond)
    inc_indent()
    code += indent()+"break\n"
    dec_indent()
    dec_indent()
    code += newline()

    return code

#########################################################################
def for_stmt(node):

    (FOR, (IN_EXP, in_exp), (STMT_LIST, stmt_list)) = node
    assert_match(FOR, 'for')
    code = ""

    (IN, pattern, list_term) = in_exp

    # expand the list_term
    code += indent()+"(LIST_TYPE, list_val) = walk({})\n".format(list_term)
    code += indent()+"if LIST_TYPE not in ['list','string']:\n"
    code += indent()+"    raise ValueError('only iteration over strings and lists is supported')\n"

    # we allow iteration over two types of structures: (1) lists (2) strings
    # if it is a string turn the list_val into a list of Asteroid characters.
    code += indent()+"if LIST_TYPE == 'string':\n"
    code += indent()+"    new_list = []\n"
    code += indent()+"    for c in list_val:\n"
    code += indent()+"        new_list.append(('string',c))\n"
    code += indent()+"    list_val = new_list\n"

    # for each term on the list unfiy with pattern, declare the bound variables,
    # and execute the loop body in that context
    # NOTE: just like Python, loop bodies do not create a new scope!
    # NOTE: we can use unification as a filter of elements:
    #
    #      for (2,y) in [(1,11), (1,12), (1,13), (2,21), (2,22), (2,23)]  do
    #             print y.
    #      end for
    code += indent()+"for term in list_val:\n"
    code += indent()+"   try:\n"
    code += indent()+"      unifiers = unify(term,{})\n".format(pattern)
    code += indent()+"   except PatternMatchFailed:\n"
    code += indent()+"      pass\n"
    code += indent()+"   else:\n"
    code += indent()+"      declare_unifiers(unifiers)\n"
    inc_indent()
    inc_indent()
    code += walk(stmt_list)
    dec_indent()
    dec_indent()
    code += newline()

    return code

#########################################################################
def if_stmt(node):

    (IF, (LIST, if_list)) = node
    assert_match(IF, 'if')
    assert_match(LIST, 'list')
    code = ""

    for i in range(0,len(if_list),2):

        #lineinfo = if_list[ i ]
        #code += process_lineinfo(lineinfo)

        (IF_CLAUSE,
         (COND, cond),
         (STMT_LIST, stmts)) = if_list[ i + 1 ]

        if i == 0:
            code += indent()+"if "
        else:
            code += indent()+"elif "

        code += "map2boolean(walk({}))[1]:\n".format(cond)
        inc_indent()
        code += walk(stmts)
        dec_indent()

    code += newline()

    return code

#########################################################################
def struct_def_stmt(node):

    (STRUCT_DEF, (ID, struct_id), (MEMBER_LIST, (LIST, member_list))) = node
    assert_match(STRUCT_DEF, 'struct-def')
    assert_match(ID, 'id')
    assert_match(MEMBER_LIST, 'member-list')
    assert_match(LIST, 'list')
    code = ""

    code = indent()+"# structure def for {}\n".format(struct_id)

    code += indent()+"member_list = {}\n".format(member_list)

    code += indent()+"struct_memory = []\n"
    code += indent()+"member_names = []\n"

    code += indent()+"for member_ix in range(len(member_list)):\n"
    code += indent()+"    member = member_list[member_ix]\n"
    code += indent()+"    if member[0] == 'data':\n"
    code += indent()+"        (DATA, (ID, member_id)) = member\n"
    code += indent()+"        struct_memory.append(('none', None))\n"
    code += indent()+"        member_names.append(member_id)\n"
    code += indent()+"    elif member[0] == 'unify':\n"
    code += indent()+"        (UNIFY, (ID, member_id), function_exp) = member\n"
    code += indent()+"        function_val = walk(function_exp)\n"
    code += indent()+"        struct_memory.append(function_val)\n"
    code += indent()+"        member_names.append(member_id)\n"
    code += indent()+"    elif member[0] == 'noop':\n"
    code += indent()+"        pass\n"
    code += indent()+"    else:\n"
    code += indent()+"        raise ValueError('unsupported struct member {}'.format(member[0]))\n"

    code += indent()+"struct_type = ('struct',('member-names', ('list', member_names)),('struct-memory', ('list', struct_memory)))\n"
    code += indent()+"state.symbol_table.enter_sym('{}', struct_type)\n".format(struct_id)
    code += newline()

    return code

#########################################################################
def process_lineinfo(node):

    (LINEINFO, (module_name,lineno)) = node
    assert_match(LINEINFO, 'lineinfo')

    code = indent()+"set_lineinfo('{}',{})\n".format(module_name,lineno)

    return code

#########################################################################
def list_stmt(node):

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
# the escaped code has been moved onto the func_impl_list by the frontend
# we have a reference to that code.
def escape_stmt(node):

    (ESCAPE, (IMPLEMENTATION, fpointer)) = node
    assert_match(ESCAPE, 'escape')

    code = indent()+"{}()\n".format(fpointer)

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
        raise ValueError("{} not supported in this context".format(type))

# a dictionary to associate tree nodes with node functions
dispatch_dict = {
    # statements - statements do not produce return values
    'lineinfo'      : process_lineinfo,
    'noop'          : lambda node : "", # does not generate any code
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
    'list'          : list_stmt,
    'apply'         : apply_stmt,
    'escape'        : escape_stmt,
}

#########################################################################
def gen_function(def_pair):

    (name,implementation) = def_pair
    code = ""

    if implementation[0] == 'body-list':
        code += indent()+"def {}(arg):\n".format(name)
        inc_indent()
        orig_indent = get_indent()

        # iterate over the bodies to find one that unifies with the actual parameters
        (BODY_LIST, (LIST, body_list_val)) = implementation

        for i in range(0, len(body_list_val), 2):
            # Process lineinfo
            (LINEINFO, (module_name, lineno)) = body_list_val[ i ]
            code += indent()+"set_lineinfo('{}',{})\n".format(module_name,lineno)

            # Deconstruct function body
            (BODY,
            (PATTERN, p),
            (STMT_LIST, stmts)) = body_list_val[ i + 1]

            code += indent()+"try:\n"
            inc_indent()
            code += indent()+"unifiers = unify(arg,{})\n".format(p)
            code += indent()+"state.symbol_table.push_scope({})\n"
            code += indent()+"declare_formal_args(unifiers)\n"
            code += walk(stmts)
            code += indent()+"state.symbol_table.pop_scope()\n"
            dec_indent()
            code += indent()+"except PatternMatchFailed:\n"
            inc_indent()

        code += indent()+"raise ValueError('none of the function bodies unified with actual parameters')\n"
        set_indent(orig_indent)
        code += newline()
        dec_indent()

        return code

    elif implementation[0] == 'escape':
        import io
        (ESCAPE, program_string) = implementation
        buf = io.StringIO(program_string)

        code = ""
        code += indent()+"def {}():\n".format(name)
        inc_indent()
        s = buf.readline()
        while s:
            code += indent()+"{}\n".format(s)
            s = buf.readline()
        code += indent()+"avm.avm.__retval__ = __retval__\n"
        dec_indent()
        code += newline()
        return code

    else:
        raise ValueError("unknown function implementation mode.")

#########################################################################
# Generate Python implementations of functions.  The AST refers back
# to these implementations via ('implementation', fpointer) nodes.
def gen_function_list():

    code = ""
    for p in func_impl_list:
        code += gen_function(p)

    return code

#########################################################################
# we need to be able to translate from the AVM expression space into
# Python's address space for function calls.  this dispatch table
# accomplishes that.
def gen_dispatch():

    code = ""
    for (name,impl) in func_impl_list:
        code += "{}dispatch_table['{}'] = {}\n".format(indent(),name,name)

    code += newline()

    return code
