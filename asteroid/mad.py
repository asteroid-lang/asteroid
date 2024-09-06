###########################################################################################
# Minimal Asteroid Debugger (MAD)
#
# (c) University of Rhode Island
###########################################################################################
# This is a very basic debugger for Asteroid and is designed around the callback API
# from the interpreter,
#     start           - called after the AST has been created and just before 
#                       execution begins
#     stop            - called after execution terminates just before the 
#                       interpreter terminates
#     error           - this is called when the interpreter encounters an error 
#                       as part of the error handler of the interpreter
#     enter_function  - this call signals that the interpreter is just about to 
#                       execute a function, when the interpreter calls this function
#                       the function stackframe and the function local variables have
#                       been properly initialized
#     exit_function   - this call signals that the interpreter just finished executing 
#                       function, this call is issued just before the function stack frame
#                       is popped off the stack
#     enter_module    - a module is just about to be executed, the module scope has 
#                       has been pushed on the stack
#     exit_module     - the interpreter is just about to exit a module, module stack frame
#                       is still on the stack.
#     step            - this is called by the interpreter at more or less statementlevel 
#                       granularity and represents a single computational step, step always
#                       point to the next statement just about to be executed
#
# the debugger object maintains state accross the callbacks from the interpreter. in particular
# it remembers whether or not it is in "continue" mode, that is, computation should proceed
# uninterrupted  until breakpoint is encountered.  it also remembers whether or not a "next" 
# command was used to step over nested scopes such as functions and modules. At each
# "step" callback the debugger uses the following logic in order to decide whether or not
# to hand back control to the interpreter:
#      if breakpoint encountered:
#         print "reached breakpoint"
#         reset all mode variables
#         return to interactive debugger session
#      elif in continue_mode:
#         return control to the interpreter
#      elif completed execution of nested scope:
#         reset scope mode variable
#         return to interactive debugger session
#      if executing nested scope via 'next' command:
#         return control to the interpreter
#      else:
#         return to interactive debugger session
###########################################################################################

from os.path import exists, split, basename
from asteroid.support import term2string, get_tail_term, term2verbose, find_function
from asteroid.frontend import Parser
from asteroid.version import MAD_VERSION
import copy 

START_DEBUGGER = False   # interactive debugging session
RETURN_TO_INTERP = True  # return control to interpreter

class MAD:

   ###########################################################################################
   def __init__(self, functional_mode=False):
      # a reference to the interpreter state object
      self.interp_state = None
      # the command line argument indicating functional mode
      self.functional_mode = functional_mode
      # a lookup table for the program texts used during debugging
      self.program_text = {}
      # continue mode is used in the implementation of the continue cmd
      self.continue_mode = False
      # scope counter is used as a scope stack in the implementation of the over cmd
      self.scope_counter = -1
      # the frame we are looking at
      # Note: continue, next, and step reset this to 0
      self.frame_ix = 0
      # command dispatch table, only uses the first two letters of each command
      # in order to interpret the commands
      self.dispatch_table = {
                  'bre'     : self._handle_breakpoints,
                  'cle'     : self._handle_clear,
                  'con'     : self._handle_continue,
                  'dow'     : self._handle_down,
                  'fra'     : self._handle_frame,
                  'hel'     : self._handle_help,
                  'lis'     : self._handle_list,
                  'nex'     : self._handle_next,
                  'pri'     : self._handle_print,
                  'qui'     : self._handle_quit,
                  'set'     : self._handle_set,
                  'sta'     : self._handle_stack,
                  'ste'     : self._handle_step,
                  'tra'     : self._handle_trace,
                  'up '     : self._handle_up,
                  'whe'     : self._handle_where,
               }
      # the following tables of tuples (func/line#,file) that describes break points at 
      #function entry points or lines
      self.function_breakpoints = []
      self.line_breakpoints = []

   ###########################################################################################
   # debugger API from interpreter
   def start(self, state):
      self.interp_state = state
      print("Minimal Asteroid Debugger {}".format(MAD_VERSION))
      print("(c) University of Rhode Island")
      print("type \"help\" for additional information")
      self._prompt_cmd(show_code=False)

   def stop(self):
      print("stopping MAD")
      self._prompt_cmd(show_code=False)

   def error(self, e):
      print("error: {}".format(str(e)))
      self._prompt_cmd()

   def enter_function(self, fname):
      # deal with function break points
      (file,_) = self.interp_state.lineinfo
      if self.function_breakpoints.count((fname,basename(file))):
         print("reached breakpoint ({} @{})".format(file,fname))
         self.continue_mode = False
         self.scope_counter = -1
      elif self._check_enter_scope(fname) == RETURN_TO_INTERP:
         return
      print("entering function {}".format(fname))
      self._prompt_cmd()

   def exit_function(self, fname):
      if self._check_exit_scope(fname) == RETURN_TO_INTERP:
         return
      print("exiting function {}".format(fname))
      self._prompt_cmd()

   def enter_module(self, modname):
      if self._check_enter_scope(modname) == RETURN_TO_INTERP:
         return
      print("entering module {}".format(modname))
      self._prompt_cmd()

   def exit_module(self, modname):
      if self._check_exit_scope(modname) == RETURN_TO_INTERP:
         return
      print("exiting module {}".format(modname))
      self._prompt_cmd()

   def step(self):
      if self._check_step() == RETURN_TO_INTERP:
         return
      self._prompt_cmd()

   ###########################################################################################
   # at each call-back from the interpreter we have to make decisions whether or not
   # to return control back to the interpreter or to start the debugger command line
   # interface.  The following functions handle this decision.  With one exception:
   # function break points, these are handled directly by the 'enter_function' API
   # function

   def _check_enter_scope(self,scopename):
      #print("entering scope {} with {}".format(scopename,self.scope_counter))
      if self.continue_mode:
         return RETURN_TO_INTERP
      elif self.scope_counter > 0:
         self.scope_counter += 1
         return RETURN_TO_INTERP
      else:
         return START_DEBUGGER

   def _check_exit_scope(self,scopename):
      #print("exiting scope {} with {}".format(scopename,self.scope_counter))
      if self.continue_mode:
         return RETURN_TO_INTERP
      elif self.scope_counter > 1:
         self.scope_counter -= 1
         return RETURN_TO_INTERP
      else:
         return START_DEBUGGER

   def _check_step(self):
      (file,line) = self.interp_state.lineinfo
      if (int(line),file) in self.line_breakpoints:
         print("reached breakpoint ({}:{})".format(file,line))
         self.continue_mode = False
         self.scope_counter = -1
         return START_DEBUGGER
      elif self.continue_mode:
         return RETURN_TO_INTERP
      elif self.scope_counter == 1:
         self.scope_counter = -1
         return START_DEBUGGER
      if self.scope_counter > 1:
         return RETURN_TO_INTERP
      else:
         return START_DEBUGGER

   ###########################################################################################
   def _print_line(self):
      (file,lineno) = self.interp_state.lineinfo
      self._load_program_text(file)
      pt = self.program_text[file]
      pt_display = pt[lineno-1].strip()
      print("{}:{}:{}".format(file,lineno,pt_display))

   ###########################################################################################
   def _load_program_text(self,fname):
      # load program text into a text cache
      if not self.program_text.get(fname) and exists(fname):
         with open(fname, "r") as f:
               self.program_text[fname] = f.readlines()
         # Always add an EOF specifier
         self.program_text[fname].append("[EOF]\n")

   ###########################################################################################
   # the following is the command interpretation interface using the 
   # debugger dispatch table from above
   # Note: functions of the format _handle_xxxx are functions that
   #       are dispatched by the dispatch table

   def _prompt_cmd(self, show_code=True):
      if show_code:
         #self._print_line()
         self._handle_list([])
      # loop while interpreting commands
      while True:
         cmd = input("mad> ")
         # if the command returns true return control
         # to the Asteroid interpreter
         if self._interpret_cmd(cmd):
            break

   def _interpret_cmd(self, cmd):
      # use the dispatch table in order to interpret commands
      cmd_list = cmd.split()
      # special case our 2-char command
      if cmd_list[0] == 'up': 
         cmd_list[0] = 'up '
      if len(cmd_list) > 0:
         val = self.dispatch_table.get(cmd_list[0][0:3],lambda _:-1) (cmd_list[1:])
         if val == -1:
            print("error: unknown command {}".format(cmd_list[0]))
            return START_DEBUGGER
         else:
            return val
      else:
         # assume implicit step when user just hits return
         return self._handle_step([])

   def _handle_breakpoints(self,_):
      print("breakpoints:")
      for (bp,file) in self.function_breakpoints:
         print("{} @{}".format(file,bp))
      for (bp,file) in self.line_breakpoints:
         print("{}:{}".format(file,bp))
      return START_DEBUGGER
      
   def _handle_clear(self,_):
      # TODO: allow for specicific breakpoints to be cleared
      self.function_breakpoints = []
      self.line_breakpoints = []
      return START_DEBUGGER
      
   def _handle_continue(self,_):
      self.continue_mode = True
      self.frame_ix = 0
      return RETURN_TO_INTERP

   def _handle_down(self, args):
      if self.frame_ix == 0:
         print("error: no such frame")
      else:
         self.frame_ix -= 1
         self._handle_frame(args)
      return START_DEBUGGER

   def _handle_help(self,_):
      print()
      print("Available commands:")
      print("breakpoints\t\t- show all breakpoints")
      print("clear\t\t\t- clear all breakpoints")
      print("continue\t\t- continue execution to next breakpoint")
      print("down\t\t\t- move down one stack frame")
      print("frame\t\t\t- display current stack frame number")
      print("help\t\t\t- display help")
      print("list [<num>|*]\t\t\t- display <num> (default 4) lines of source code, * displays all lines in file")
      print("next\t\t\t- step execution across a nested scope")
      print("print <name>[@<num>|<name>]+|* [-v]\t\t- print contents of <name>, * lists all vars in scope, recursively access (nested) objects with @, '-v' enables verbose printing of nested data")
      print("quit\t\t\t- quit debugger")
      print("set [<func>|<line#> [<file>]]\n\t\t\t- set a breakpoint, breakpoints may only be set on valid statements on already loaded files")
      print("stack [<num>|* [-v]]\t\t\t- display runtime stack, list all items in specific frame with an index or all frames with '*', '-v' toggles verbose printing")
      print("step\t\t\t- step to next executable statement")
      print("trace [<num> [<num>]]\t\t\t- display runtime stack trace, display runtime stack trace, can specify either the first n frames or all of the frames between the start and end")
      print("up\t\t\t- move up one stack frame")
      print("where\t\t\t- print current program line")
      print()
      return START_DEBUGGER

   def _handle_list(self, args):
      (file,lineno) = self.interp_state.lineinfo
      self._load_program_text(file)
      pt = self.program_text[file]
      
      if len(args) > 1:                                  # Too many arguments, reject with an error message
         print("error: too many arguments")
         return START_DEBUGGER
      elif len(args) == 0:                               # No arguments, assume default length and fall through
         length = 4
      elif args[0] == '*':                               # '*' argument, set length to length of the file and lineno to 0
         lineno, length = 0, len(pt)
      elif args[0].isnumeric() and int(args[0]) > 0: # Number greater than 0, cast it and set length to it
         length = int(args[0])
      else:                                              # Any other input should be rejected with an error message
         print("error: expected a number greater than 0 or '*', found '{}'".format(args[0]))
         return START_DEBUGGER
      
      # Compute the start and end of listing
      start = (lineno - length) if lineno >= length else 0
      end = lineno + length if lineno < len(pt) - 2 else len(pt)
      pt_display = pt[start:end]
      start_of_line = "  "
      # GO through each line in the program text
      for ix, l in enumerate(pt_display):
         # if the offset line number is in breakpoints
         if (ix+1+start,file) in self.line_breakpoints:
               start_of_line = "* "
         # mark the current line
         if lineno == ix+1+start:
               start_of_line = "> "
         print(start_of_line, ix+1+start, l[:-1])
         start_of_line = "  "
      return START_DEBUGGER

   def _handle_next(self,_):
      # initialize the "scope stack", as long as the scope
      # counter value is > 0 all call backs from the interpreter
      # immediately return control back to the interpreter with
      # one exception: breakpoints.  If a breakpoint is hit
      # the scope counter is reset to -1.
      self.scope_counter = 1
      self.frame_ix = 0
      return RETURN_TO_INTERP

   def _handle_print(self,args):
      if len(args) > 2:
         print("error: too many arguments")
         return False
      elif len(args) == 0:
         print("error: no argument given")
         return False
      
      if len(args) == 2 and args[1] == '-v':
         verbose = True
      elif len(args) == 1:
         verbose = False
      else:
         print("error: unknown option '{}'".format(args[1]))
         return False
      
      # Split any arguments by the '@' character when necessary
      syms = args[0].split('@')
      # '@' occurs at beginning or end of argument, or multiple `@`s occur next to each other is rejected with an error message
      if '' in syms:
         print("error: any @s must exist between keywords or integers, not adjacent or next to each other")
         return START_DEBUGGER
      
      var_list = self.interp_state.symbol_table.get_curr_scope(scope=self.frame_ix, option="items")
      # If '*' is the only argument, handle output as normal
      if syms[0] == '*' and len(syms) == 1:
         for (name,val) in var_list:
            print("{}: {}".format(name, term2verbose(val) if verbose else term2string(val)))
      else:
         # Loop through scope and check if any symbols in the scope are the first symbol in the list
         term = None
         for (name, val) in var_list:
            if name == syms[0]:
               term = val
               break
         # Iterate over remaining terms to find the final symbol
         val = get_tail_term(syms[0], term, syms[1:])
         # Print the entire argument along with its current symbol if it is found
         if val:
            print("{}: {}".format(args[0], term2verbose(val) if verbose else term2string(val)))
      return START_DEBUGGER

   def _handle_quit(self,_):
      raise SystemExit()
   
   def _handle_set(self,args):
      (file,lineno) = self.interp_state.lineinfo
      if len(args) == 0:
         # set a breakpoint at the current line
         self.line_breakpoints.append((lineno,file))
         return START_DEBUGGER
      elif len(args) == 1:
         self._load_program_text(file)
         if args[0].isnumeric():
            if self._validate_breakpoint_line(file, int(args[0])):
               self.line_breakpoints.append((int(args[0]),file))
         else:
            if self._validate_breakpoint_function(file, args[0]):
               self.function_breakpoints.append((args[0],file))
         return START_DEBUGGER
      elif len(args) == 2:
         self._load_program_text(args[1])
         if args[0].isnumeric():
            if self._validate_breakpoint_line(args[1], int(args[0])):
               self.line_breakpoints.append((int(args[0]),args[1]))
         else:
            if self._validate_breakpoint_function(args[1], args[0]):
               self.function_breakpoints.append((args[0],args[1]))
         return START_DEBUGGER
      else:
         print("error: too many arguments to set")
         return START_DEBUGGER
   
   def _validate_breakpoint_line(self, fname, lineno):
      # Create a temporary Parser and reset the lineinfo
      (module, line) = self.interp_state.lineinfo
      temp_parser = Parser(functional_mode=self.functional_mode)
      self.interp_state.lineinfo = (module, line)
      # Read the file contents and get the current line
      curr_file = self.program_text[fname]
      if lineno <= 0 or lineno >= len(curr_file):
         print("error: cannot place breakpoints outside of file")
         return False
      line_data = curr_file[lineno-1]
      # Reject blank lines and '[EOF]'
      if line_data.strip() == '':
         print("error: cannot place breakpoints on blank lines")
         return False
      elif line_data == '[EOF]':
         print("error: cannot place breakpoints at end of file")
         return False
      # Incrementally add lines to line_data until either a valid statement is generated or every following line has been checked
      for l in range(lineno, len(curr_file)):
         try:
            stmts = temp_parser.parse(line_data)
            self.interp_state.lineinfo = (module, line)
            return True
         except:
            line_data += ("\n" + curr_file[l])
      print("error: line {} in file '{}' cannot accept a breakpoint".format(lineno, fname))
      self.interp_state.lineinfo = (module, line)
      return False
      
   
   def _validate_breakpoint_function(self, fname, func_name):
      # Loop through every scope and check if the function was found
      loaded_syms = self.interp_state.symbol_table.scoped_symtab
      for scope in loaded_syms:
         for (sym, val) in scope.items():
            if find_function(sym, val, fname, func_name): return True
      print("error: unable to find function '{}' in file '{}'".format(func_name, fname))
      return False

   def _handle_frame(self,_):
      print("you are looking at frame #{}".format(self.frame_ix))
      return START_DEBUGGER

   def _handle_step(self,_):
      self.frame_ix = 0
      return RETURN_TO_INTERP

   def _handle_stack(self, args):
      trace = copy.copy(self.interp_state.trace_stack)
      trace.reverse()
      
      # Determine if the verbose flag has been provided
      verbose = len(args) == 2 and args[1] == '-v'
      if len(args) > 2:
         print("error: too many arguments")
      elif len(args) == 2 and args[1] != '-v':
         print("error: unknown argument '{}'".format(args[1]))
      # Stack may only recieve either a positive integer or '*'
      elif len(args) >= 1 and not (args[0].isnumeric() or args[0] == '*'):
         print("error: invalid argument '{}', must be either an integer or '*'".format(args[0]))
      # Print all symbols in a specific stack frame
      elif len(args) >= 1 and args[0].isnumeric():
         prev_frame = self.frame_ix
         self.frame_ix = int(args[0])
         if len(trace) <= self.frame_ix:
            print("error: invalid index {}".format(self.frame_ix))
         else:
            (module, lineno, fname) = trace[self.frame_ix]
            print("frame #{}: {} @{}".format(self.frame_ix, module, fname))
            self._handle_print(['*', '-v'] if verbose else ['*'])
         self.frame_ix = prev_frame
      # Print all symbols in all stack frames
      elif len(args) >= 1 and args[0] == '*':
         prev_frame = self.frame_ix
         print("Runtime stack (most recent call first):")
         for i in range(prev_frame, len(trace)):
            self.frame_ix = i
            (module,lineno,fname) = trace[i]
            print("frame #{}: {} @{}".format(i,module,fname))
            self._handle_print(['*', '-v'] if verbose else ['*'])
         self.frame_ix = prev_frame
      else:
         print("Runtime stack (most recent call first):")
         for i in range(self.frame_ix,len(trace)):
            (module,lineno,fname) = trace[i]
            print("frame #{}: {} @{}".format(i,module,fname))
      
      return START_DEBUGGER

   def _handle_trace(self, args):
      trace = copy.copy(self.interp_state.trace_stack)
      trace.reverse()
      
      if len(args) > 2:
         print("error: too many arguments")
         return START_DEBUGGER
      elif len(args) == 2:
         if not args[0].isnumeric():
            print("error: invalid first argument '{}', must be some positive integer".format(args[0]))
            return START_DEBUGGER
         elif not args[1].isnumeric():
            print("error: invalid second argument '{}', must be some positive integer".format(args[1]))
            return START_DEBUGGER
         start, end = int(args[0]), int(args[1])
      elif len(args) == 1:
         if not args[0].isnumeric():
            print("error: invalid argument '{}', must be some positive integer".format(args[0]))
            return START_DEBUGGER
         start, end = self.frame_ix, self.frame_ix + int(args[0])
      else:
         start, end = self.frame_ix, len(trace)
      
      if end > len(trace):
         print("error: range of stack frames ({}, {}) must not exceed number of existing frames ({})".format(start, end, len(trace)))
         return START_DEBUGGER
      
      print("Runtime stack trace (most recent call first):")
      for i in range(start, end):
         (module, lineno, fname) = trace[i]
         print("frame #{}: {} @{}".format(i, module, fname))
      return START_DEBUGGER

   def _handle_up(self, args):
      if self.frame_ix+1 == len(self.interp_state.trace_stack):
         print("error: no such frame")
      else:
         self.frame_ix += 1
         self._handle_frame(args)
      return START_DEBUGGER

   def _handle_where(self,_):
      self._print_line()
      return START_DEBUGGER
