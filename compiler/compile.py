###########################################################################################
# Asteroid compiler
#
# (c) University of Rhode Island
###########################################################################################

import os
import sys

from asteroid.globals import *
from asteroid.support import *
from state import state
from frontend import Parser
from gen import walk as generate_code, gen_function_list, gen_dispatch, gen_memory

VERSION = "1.0"

# the prologue file is expected to be in the 'modules' folder
prologue_name = 'prologue.ast'

# TODO: adjust the defaults
def compile(input_stream,
           input_name = "<input>",
           tree_dump=False,
           exceptions=False,
           prologue=True):
    try:
        #lhh
        #print("path[0]: {}".format(sys.path[0]))
        #print("path[1]: {}".format(sys.path[1]))

        # build the AST
        parser = Parser(input_name)
        
        # asteroid code prologue is placed here
        import os
        prologue = open(os.getcwd()+"\compiler\prologue.ast", "r")
        input_stream = prologue.read() + input_stream
        
        (LIST, istmts) = parser.parse(input_stream)
        state.AST = ('list', istmts)

        # walk the AST
        if tree_dump:
            dump_AST(state.AST)

        begin_code = "/**************************************/ \n"
        begin_code += "/*** Asteroid Compiler Version {}  ***/ \n".format(VERSION)
        begin_code += "/*** (c) University of Rhode Island ***/ \n"
        begin_code += "/**************************************/ \n"
        begin_code += "#![allow(unused)]\n\n"

        # alternate MALLOC implementation
        #begin_code += "#[cfg(not(target_env = \"msvc\"))]\n" 
        #begin_code += "use tikv_jemallocator::Jemalloc;\n\n"
        #begin_code += "#[cfg(not(target_env = \"msvc\"))]\n"
        #begin_code += "#[global_allocator]\n"
        #begin_code += "static GLOBAL: Jemalloc = Jemalloc;\n\n"
        
        begin_code += "use std::rc::Rc;\n"
        begin_code += "use std::cell::RefCell;\n"
        begin_code += "use std::collections::HashMap;\n"
        begin_code += "use std::ptr;\n\n"
        begin_code += "use shared_arena::*;\n"
        begin_code += "use state::*;\n"
        begin_code += "use symtab::*;\n"
        begin_code += "use ast::*;\n"
        begin_code += "use support::*;\n"
        begin_code += "use avm::*;\n\n"
        begin_code += "static mut POOL: *mut Vec<ArenaRc<Node>> = ptr::null_mut();\n\n"
        begin_code += "fn main() {\n"
        begin_code += "   let mut memory: Arena<Node> = Arena::new();\n"
        begin_code += "   let mut state = State::new().unwrap();\n\n"

        internal_functions = open(os.getcwd()+"\compiler\prologue_rust.ast", "r").read()
        internal_dispatch = open(os.getcwd()+"\compiler\prologue_rust2.ast", "r").read()
        
        compiled_code = generate_code(state.AST)
        flist_code = gen_function_list()
        mem_code = gen_memory()
        dispatch_code = gen_dispatch()
    
        end_code = "   return ();\n"
        end_code += "}\n"

        # assemble the code
        code = ""
        code += begin_code
        code += flist_code
        code += internal_functions
        code += dispatch_code
        code += internal_dispatch
        code += mem_code
        code += compiled_code
        code += end_code

        return code

    except Exception as e:
        if exceptions: # rethrow the exception so that you can see the full backtrace
            raise e
        else:
            module, lineno = state.lineinfo
            print("Error: {}: {}: {}".format(module, lineno, e))
            sys.exit(1)

    except  KeyboardInterrupt as e:
            print("Error: keyboard interrupt")
            sys.exit(1)

    except  BaseException as e:
            print("Error: {}".format(e))
            sys.exit(1)
