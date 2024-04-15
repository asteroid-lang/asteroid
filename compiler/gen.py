#########################################################################
# A tree walker to generate code for the AVM
#
# (c) University of Rhode Island
#########################################################################

from copy import deepcopy
from re import match as re_match
import ast

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
# This allows us to know if the code being generated is at the base level
# or not. The state and memory objects must be cast as mutable references
# at the base level, and are already mutable references otherwise

_scope_level = 0

def inc_scope():
    global _scope_level
    _scope_level += 1

def dec_scope():
    global _scope_level
    if _scope_level == 0:
        raise ValueError("Cannot decrement scope level")
    else:
        _scope_level -= 1

def state_and_mem():
    global _scope_level
    if _scope_level:
        return "state, memory"
    else:
        return "&mut state, &mut memory"

def return_or_exit():
    # If we are in the base level and we have an error, we call the
    # exit function, otherwise we bubble the error up
    global _scope_level
    if _scope_level:
        return "   Err( e ) => return Err(e),\n"
    else:
        return "   Err( e ) => exit(e, {}),\n".format(state_and_mem())

#########################################################################
# This allows us to construct the AST of the target program as a single
# global list.
_ASTs = []

#########################################################################
def delimiter():
    return "/******************************************************************************/\n"

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

    global _scope_level
    (ASSERT, exp) = node
    assert_match(ASSERT, 'assert')
    code = ""

    code += indent()+delimiter()
    code += indent()+"// ASSERT statement \n\n"

    exp_str = "{}".format(exp)
    code += indent()+"{\n";
    inc_indent()
    code += indent()+"let exp_val = match walk( ArenaRc::clone(&data[{}]),{}) {\n".format(exp_str,state_and_mem())
    code += indent()+"   Ok( val ) => val,\n"
    code += indent()+return_or_exit()
    code += indent()+"};\n"
    code += indent()+"let _ = match map2boolean(exp_val) {\n"
    code += indent()+"   Ok( Node::AstroBoolean{value:true} ) => _,\n"
    code += indent()+"   Ok( Node::AstroBoolean{value:false} ) => "

    if _scope_level:
        code += "return Err( new_exception(\"AssertFailed\".to_string(), \"Assert Failed.\".to_string(), state, memory ),\n"
    else:
        code += "exit( new_exception(\"AssertFailed\".to_string(), \"Assert Failed.\".to_string(), state, memory ), {}),\n".format(state_and_mem())
        
    code += indent()+return_or_exit()
    code += indent()+"};\n"
    dec_indec()
    code += indent()+"}\n";

    code += newline()

    return code

#########################################################################
def unify_stmt(node):

    (UNIFY, pattern, exp) = node
    assert_match(UNIFY, 'unify')
    code = ""

    exp_str = "{}".format(exp)
    pattern_str = "{}".format(pattern)

    code += indent()+delimiter()
    code += indent()+"// UNIFY statement \n\n"

    code += indent()+"let exp_val = match walk(ArenaRc::clone(&data[{}]),{})  {{ \n".format(python_to_rust(exp),state_and_mem())
    code += indent()+"   Ok( val ) => val,\n"
    code += indent()+return_or_exit()
    code += indent()+"};\n"
    code += indent()+"let unifiers = match unify(exp_val,ArenaRc::clone(&data[{}]),{},true) {{ \n".format(python_to_rust(pattern),state_and_mem())
    code += indent()+"   Ok( val ) => val,\n"
    code += indent()+return_or_exit()
    code += indent()+"};\n"
    code += indent()+"declare_unifiers(&unifiers,{});\n".format(state_and_mem())

    code += newline()

    return code

#########################################################################
def return_stmt(node):

    (RETURN, e) = node
    assert_match(RETURN, 'return')
    code = ""

    e_str = "{}".format(e)

    code += indent()+"let val = match walk(ArenaRc::clone(&data[{}]),{}) {{\n".format(python_to_rust(e_str),state_and_mem())
    code += indent()+"   Ok( val ) => val,\n"
    code += indent()+"   Err( e ) => exit(e, "+state_and_mem()+"),\n"
    code += indent()+"};\n"
    code += indent()+"state.symbol_table.pop_scope();\n"
    code += indent()+"return Ok( val );\n"

    code += newline()

    return code

#########################################################################
def break_stmt(node):

    (BREAK,) = node
    assert_match(BREAK, 'break')
    code = ""

    code += indent()+"break;\n"
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

    code += indent()+delimiter()
    code += indent()+"// LOOP statement \n\n"

    code += indent()+"loop {\n"
    inc_indent()
    code += walk(body)
    dec_indent()
    code += indent()+"}\n"
    code += newline()

    return code

#########################################################################
def while_stmt(node):

    (WHILE, cond_exp, body_stmts) = node
    assert_match(WHILE, 'while')
    (COND_EXP, cond) = cond_exp
    (STMT_LIST, body) = body_stmts
    code = ""

    code += indent()+delimiter()
    code += indent()+"// WHILE statement \n\n"

    code += indent()+"while let Ok( unifiers ) = walk(ArenaRc::clone(&data[{}]),{}) {\n".format(cond)
    inc_indent()
    code += indent()+"declare_formal_args( &unifiers, state, memory);\n"
    code += walk(body)
    dec_indent()
    code += indent()+"}\n"
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

    code += indent()+delimiter()
    code += indent()+"// FOR statement \n"
    
    code += indent()+"let exp_val = match walk( ArenaRc::clone(&data[{}]), {}) {{ \n".format( python_to_rust(list_term), state_and_mem() )
    code += indent()+"   Ok( val ) => val,\n"
    code += indent()+"   Err( e ) => exit(e, {}),\n".format( state_and_mem() )
    code += indent()+"};\n"

    code += indent()+"if let Node::AstroList(AstroList{contents:ref content}) = *exp_val { \n"
    inc_indent()
    code += indent()+"for term in &*content.borrow() { \n"
    inc_indent()
    code += indent()+"let unifiers = match unify( ArenaRc::clone(&term), ArenaRc::clone(&data[{}]), {}, true) {{ \n".format( python_to_rust(pattern), state_and_mem() )
    code += indent()+"   Ok( val ) => val,\n"
    code += indent()+"   Err( e ) => exit(e, {}),\n".format( state_and_mem() )
    code += indent()+"};\n"
    code += indent()+"declare_formal_args( &unifiers, {});\n".format(state_and_mem())
    code += walk(stmt_list)
    dec_indent()
    code += indent()+"};\n"
    dec_indent()
    code += indent()+"};\n"
    
    # expand the list_term
    #code += indent()+"(LIST_TYPE, list_val) = walk({})\n".format(list_term)
    #code += indent()+"if LIST_TYPE not in ['list','string']:\n"
    #code += indent()+"    raise ValueError('only iteration over strings and lists is supported')\n"

    # we allow iteration over two types of structures: (1) lists (2) strings
    # if it is a string turn the list_val into a list of Asteroid characters.
    # code += indent()+"if LIST_TYPE == 'string':\n"
    # code += indent()+"    new_list = []\n"
    # code += indent()+"    for c in list_val:\n"
    # code += indent()+"        new_list.append(('string',c))\n"
    # code += indent()+"    list_val = new_list\n"

    # for each term on the list unfiy with pattern, declare the bound variables,
    # and execute the loop body in that context
    # NOTE: just like Python, loop bodies do not create a new scope!
    # NOTE: we can use unification as a filter of elements:
    #
    #      for (2,y) in [(1,11), (1,12), (1,13), (2,21), (2,22), (2,23)]  do
    #             print y.
    #      end for
    #code += indent()+"let Node::AstroList{content:c} = *list_val else {panic!()};
    #code += indent()+"for term in list_val:\n"
    #code += indent()+"   try:\n"
    #code += indent()+"      unifiers = unify(term,{})\n".format(pattern)
    #code += indent()+"   except PatternMatchFailed:\n"
    #code += indent()+"      pass\n"
    #code += indent()+"   else:\n"
    #code += indent()+"      declare_unifiers(unifiers)\n"
    #inc_indent()
    #inc_indent()
    #code += walk(stmt_list)
    #dec_indent()
    #dec_indent()
    #code += newline()

    return code
#########################################################################
def match_stmt(node):

    (MATCH,var,clauses) = node
    assert_match(MATCH, 'match')
    (UNIFY,temp,var) = var
    assert_match(UNIFY, 'unify')
    (IF,(LIST,clauses)) = clauses
    assert_match(IF, 'if')
    assert_match(LIST, 'list')
    first_pass = True
    code = ""
    
    code += indent()+"let exp_val = match walk( ArenaRc::clone(&data[{}]), {}) {{ \n".format( python_to_rust(var), state_and_mem() )
    code += indent()+"   Ok( val ) => val,\n"
    code += indent()+"   Err( e ) => exit(e, {}),\n".format( state_and_mem() )
    code += indent()+"};\n"
    
    for i in range(0, len(clauses) - 1, 2):
        line_info = clauses[i]
        code += process_lineinfo(line_info)
        if_stmt = clauses[i+1]
        (IF_CLAUSE,(COND,is_stmt),stmt) = if_stmt
        
        (IS,temp,ptrn) = is_stmt
        is_stmt = (IS,var,ptrn)

        if first_pass:
            code += indent()+"if let Ok( unifiers ) = unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&data[{}]), &mut state, &mut memory, true) {{\n".format( python_to_rust(ptrn) )
        else:
            code += indent()+"}} else if let Ok( unifiers ) = unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&data[{}]), &mut state, &mut memory, true) {{\n".format( python_to_rust(ptrn) )
        inc_indent()
 
        (STMT_LIST,(LIST,stmts)) = stmt
        for stmt in stmts:
            code += walk(stmt)
        dec_indent()

        if first_pass:
            first_pass = False
    code += "}\n"
    return code
#########################################################################
def if_stmt(node):

    (IF, (LIST, if_list)) = node
    assert_match(IF, 'if')
    assert_match(LIST, 'list')
    code = ""

    code += indent()+delimiter()
    code += indent()+ "// IF statement \n"

    for i in range(0,len(if_list),2):

        (IF_CLAUSE,
         (COND, cond),
         (STMT_LIST, stmts)) = if_list[ i + 1 ]

        code += indent()+"let result = match walk( ArenaRc::clone(&data[{}]), {}) {{ \n".format( python_to_rust(cond), state_and_mem() )
        code += indent()+"   Ok( val ) => val,\n"
        code += indent()+"   Err( e ) => exit(e, {}),\n".format( state_and_mem() )
        code += indent()+"};\n"
    
        code += indent()+"if let Node::AstroBool(AstroBool{value:true}) = map2boolean( &ArenaRc::clone(&result) ) {\n"
        inc_indent()
        code += walk(stmts)
        dec_indent()
        code += indent()+"}\n"

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

    code = indent()+"// structure def for {}\n".format(struct_id)

    code += indent()+"let member_list = vec![ "
    code += ", ".join("ArenaRc::clone(&data[{}])".format( python_to_rust( member)) for member in member_list)
    code += " ];\n".format(member_list)

    code += indent()+"let mut struct_memory: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);\n"
    code += indent()+"let mut member_names: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);\n"

    code += indent()+"for member in member_list {\n"
    code += indent()+"    let _type = peek( ArenaRc::clone(&member) );\n"
    code += indent()+"    if _type == \"data\" {\n"
    code += indent()+"        let Node::AstroData(AstroData{value:ref id_node}) = *member else {panic!(\"ERROR: object construction: expected object data.\")};\n"
    code += indent()+"        let Node::AstroID(AstroID{name:ref val}) = ** id_node else {panic!(\"ERROR: object construction: expected ID.\")};\n"
    code += indent()+"        struct_memory.borrow_mut().push( memory.alloc_rc(Node::AstroNone(AstroNone::new())) );\n"
    code += indent()+"        member_names.borrow_mut().push( ArenaRc::clone(&id_node));\n"
    code += indent()+"    } else if _type == \"unify\" {\n"
    code += indent()+"        let Node::AstroUnify(AstroUnify{term:ref id_node,pattern:ref function_exp}) = *member else {panic!(\"ERROR: object construction: expection unify node.\")};\n"
    code += indent()+"        let function_val = match walk( ArenaRc::clone(&function_exp), &mut state, &mut memory ) {\n"
    code += indent()+"            Ok( val ) => val,\n"
    code += indent()+"            Err ( e ) => panic!(\"error\"),\n"
    code += indent()+"         };\n"
    code += indent()+"    } else if _type == \"noop\" {\n"
    code += indent()+"        ;//pass\n"
    code += indent()+"    } else {\n"
    code += indent()+"        panic!(\"{}: {}: {}: {}\",\"ValueError\", state.lineinfo.0, state.lineinfo.1, format!(\"unsupported struct member {}\", _type));\n"
    code += indent()+"    }\n"
    code += indent()+"}\n"

    code += indent()+"let struct_type = memory.alloc_rc(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));\n"
    code += indent()+"state.enter_sym( \"{}\", ArenaRc::clone(&struct_type));\n".format(struct_id)
    code += newline()

    return code

#########################################################################
def process_lineinfo(node):

    (LINEINFO, (module_name,lineno)) = node
    assert_match(LINEINFO, 'lineinfo')

    code = indent()+"set_lineinfo( memory.alloc_rc( Node::AstroLineInfo(AstroLineInfo::new(\"{}\".to_string(),{}))), {});\n".format(module_name,lineno,state_and_mem())

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
    code = indent()+"walk(ArenaRc::clone(&data[{}]),".format( python_to_rust(node) )+state_and_mem()+");\n"
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
    'match'         : match_stmt,
}

#########################################################################
def gen_function(def_pair):

    (name,implementation) = def_pair
    code = ""

    if implementation[0] == 'body-list':
        code += indent()+delimiter()
        code += indent()+"// {} \n".format(name)
        code += indent()+delimiter()
        code += indent()+"fn {}<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>> {{\n".format(name)
        inc_indent()
        inc_scope()
        orig_indent = get_indent()

        code += indent()+"let mut data; unsafe{data = &mut *POOL;}\n"

        # iterate over the bodies to find one that unifies with the actual parameters
        (BODY_LIST, (LIST, body_list_val)) = implementation

        for i in range(0, len(body_list_val), 2):
            
            # Process lineinfo
            (LINEINFO, (module_name, lineno)) = body_list_val[ i ]
            code += indent()+"set_lineinfo( memory.alloc_rc( Node::AstroLineInfo(AstroLineInfo::new(\"{}\".to_string(),{}))), state, memory);\n".format(module_name,lineno)

            # Deconstruct function body
            (BODY,
            (PATTERN, p),
            (STMT_LIST, stmts)) = body_list_val[ i + 1]

            code += indent()+delimiter()
            code += indent()+"// clause {} \n".format(i)
            
            if i == 0:
                code += indent()+"if"
            else:
                code += indent()+"} else if"
                
            code += " let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&data[{}]), state, memory, true) {{\n".format( python_to_rust(p) )
            inc_indent()
            code += indent()+"state.symbol_table.push_scope();\n"
            code += indent()+"declare_formal_args( &unifiers, state, memory);\n"
            code += walk(stmts)
            code += indent()+"state.symbol_table.pop_scope();\n"
            code += indent()+"return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));\n"
            dec_indent()
            
        code += indent()+"} else {\n"
        inc_indent()
        code += indent()+"return( Err(new_exception(\"PatternMatchFailed\".to_string(), \"None of the function bodies unified with actual parameters.\".to_string(), state, memory )));\n"
        dec_indent()
        code += indent()+"};\n"
        set_indent(orig_indent)
        code += newline()
        dec_indent()
        code += indent()+"};\n"
        dec_scope()

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
        code += indent()+"{}state.dispatch_table.insert( \"{}\".to_string(), {});\n".format(indent(),name,name)

    code += newline()

    return code

#########################################################################
# generates the rust code for global list containing the ASTs
def gen_memory():
    global _ASTs

    code = ""
    code += indent()+"let mut data;\n"
    code += indent()+"unsafe {\n"
    inc_indent()
    code += indent()+"if POOL.is_null() {\n"
    inc_indent()
    code += indent()+"POOL = Box::into_raw(Box::new(Vec::new()));\n"
    dec_indent()
    code += indent()+"}\n"
    code += indent()+"data = &mut *POOL;\n"
    for node in _ASTs:
        code += indent()+"data.push("+node+");\n"
    dec_indent()
    code += indent()+"}\n\n"
    
    return code

#########################################################################
# This function takes in a Asteroid value in Pyhton format(a tuple) and
# returns a string version of the same value in Rust struct format
def python_to_rust(node):
    global _ASTs

    if isinstance(node,tuple):
        if node[0] == "id":
            _ASTs.append( "memory.alloc_rc( Node::AstroID(AstroID::new({})))".format( "\"" + node[1] + "\".to_string()" ))
            return len(_ASTs) - 1
        elif node[0] == "data":
            _ASTs.append( "memory.alloc_rc( Node::AstroData(AstroData::new(ArenaRc::clone(&data[{}]))))".format( python_to_rust(node[1])))
            return len(_ASTs) - 1
        elif node[0] == "raw-to-list":
            _ASTs.append( "memory.alloc_rc( Node::AstroRawToList(AstroRawToList::new(ArenaRc::clone(&data[{}]),ArenaRc::clone(&data[{}]),ArenaRc::clone(&data[{}]))))".format(python_to_rust(node[1][1]),python_to_rust(node[2][1]),python_to_rust(node[3][1])))
            return len(_ASTs) - 1
        elif node[0] == "integer":
            _ASTs.append( "memory.alloc_rc( Node::AstroInteger(AstroInteger::new({})))".format( node[1] ))
            return len(_ASTs) - 1
        elif node[0] == "real":
            _ASTs.append( "memory.alloc_rc( Node::AstroReal(AstroReal::new({})))".format( node[1] ))
            return len(_ASTs) - 1
        elif node[0] == "apply":
            _ASTs.append( "memory.alloc_rc( Node::AstroApply(AstroApply::new(ArenaRc::clone(&data[{}]),ArenaRc::clone(&data[{}]))))".format( python_to_rust(node[1]), python_to_rust(node[2]) ))
            return len(_ASTs) - 1
        elif node[0] == "tuple":
            if len(node[1]) == 2:
                _ASTs.append( "memory.alloc_rc( Node::AstroPair(AstroPair::new(ArenaRc::clone(&data[{}]),ArenaRc::clone(&data[{}]))))".format( python_to_rust(node[1][0]), python_to_rust(node[1][1]) ))
            else:
                code = "memory.alloc_rc( Node::AstroTuple(AstroTuple::new(Rc::new(RefCell::new(vec![ "
                (TUPLE,items) = node
                for item in items:
                    code += "ArenaRc::clone(&data[{}]),".format( python_to_rust(item) )
                code = code[:-1]# get rid of last comma
                code += "])))))"
                _ASTs.append( code )
            return len(_ASTs) - 1
        elif node[0] == "function-exp":
            _ASTs.append( "memory.alloc_rc( Node::AstroFunction(AstroFunction::new(  memory.alloc_rc(Node::AstroID(AstroID::new( \"{}\".to_string()))))))".format( node[1][1] ))
            return len(_ASTs) - 1
        elif node[0] == "index":
            _ASTs.append( "memory.alloc_rc( Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&data[{}]), ArenaRc::clone(&data[{}]))))".format( python_to_rust(node[1]), python_to_rust(node[2])))
            return len(_ASTs) - 1
        elif node[0] == "unify":
            _ASTs.append( "memory.alloc_rc( Node::AstroUnify(AstroUnify::new( ArenaRc::clone(&data[{}]), ArenaRc::clone(&data[{}]))))".format( python_to_rust(node[1]), python_to_rust(node[2])))
            return len(_ASTs) - 1
        elif node[0] == "named-pattern":
            _ASTs.append( "memory.alloc_rc( Node::AstroNamedPattern(AstroNamedPattern::new( AstroID::new(\"{}\".to_string()), ArenaRc::clone(&data[{}]))))".format( node[1][1], python_to_rust(node[2])))
            return len(_ASTs) - 1
        elif node[0] == "typematch":
            _ASTs.append( "memory.alloc_rc(Node::AstroTypeMatch(AstroTypeMatch::new( memory.alloc_rc(Node::AstroString(AstroString::new(\"{}\".to_string())) ))))".format(node[1]) )
            return len(_ASTs) - 1
        elif node[0] == "string":
            _ASTs.append( "memory.alloc_rc(Node::AstroString(AstroString::new(\"{}\".to_string())))".format( node[1] ))
            return len(_ASTs) - 1
        elif node[0] == "none":
            _ASTs.append( "memory.alloc_rc(Node::AstroNone(AstroNone::new()))" )
            return len(_ASTs) - 1
        elif node[0] == "list":
            code = "memory.alloc_rc(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![ "
            (TUPLE,items) = node
            for item in items:
                code += "ArenaRc::clone(&data[{}]),".format( python_to_rust(item) )
            code = code[:-1]# get rid of last comma
            code += "])))))"
            _ASTs.append( code )
            return len(_ASTs) - 1
        elif node[0] == "to-list":
            _ASTs.append( "memory.alloc_rc( Node::AstroToList(AstroToList::new(ArenaRc::clone(&data[{}]),ArenaRc::clone(&data[{}]),ArenaRc::clone(&data[{}]))))".format(python_to_rust(node[1][1]),python_to_rust(node[2][1]),python_to_rust(node[3][1])))
            return len(_ASTs) - 1
        elif node[0] == "is":
            _ASTs.append( "memory.alloc_rc( Node::AstroIs(AstroIs::new(ArenaRc::clone(&data[{}]),ArenaRc::clone(&data[{}]))))".format(python_to_rust(node[1]),python_to_rust(node[2])))
            return len(_ASTs) - 1
        else:
            print(node)
            exit(1)

    else: # string format
        return python_to_rust(ast.literal_eval(node))
    
        '''
        _type = node[2:5]
        if _type == "int":
            _ASTs.append( "memory.alloc_rc( Node::AstroInteger(AstroInteger::new({})))".format( parse_value(node) ))
            return len(_ASTs) - 1
        elif _type == "rea":
            _ASTs.append( "memory.alloc_rc( Node::AstroReal(AstroReal::new({})))".format( parse_value(node) ))
            return len(_ASTs) - 1
        elif _type == "id'":
            _ASTs.append( "memory.alloc_rc( Node::AstroID(AstroID::new({})))".format( parse_value(node) + ".to_string()" ))
            return len(_ASTs) - 1
        elif _type == "app":
            python_to_rust(ast.literal_eval(node))
        else:
            python_to_rust(ast.literal_eval(node))
        '''


#########################################################################
# parse values out of Asteroid values(Python typles) that are in string
# format. Output is string.
# example input => "('integer',2)"
# example output => "2"
def parse_value(node):
    value = ""
    is_value = False
    for char in node:
        if char == ")":     #end
            is_value = False
        elif is_value:      #middle
            if char == "'" :
                value += "\""
            else:
                value += char    
        elif char == ",":   #start
            is_value = True
    return value


            
