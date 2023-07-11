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
from asteroid.support import term2string
from asteroid.version import MAD_VERSION
import copy 

START_DEBUGGER = False   # interactive debugging session
RETURN_TO_INTERP = True  # return control to interpreter

class MAD:

   ###########################################################################################
   def __init__(self):
      # a reference to the interpreter state object
      self.interp_state = None
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
                  'tra'     : self._handle_stack,
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
      print("list\t\t\t- display source code")
      print("next\t\t\t- step execution across a nested scope")
      print("print <name>|*\t\t- print contents of <name>, * lists all vars in scope")
      print("quit\t\t\t- quit debugger")
      print("set [<func>|<line#> [<file>]]\n\t\t\t- set a breakpoint")
      print("stack\t\t\t- display runtime stack")
      print("step\t\t\t- step to next executable statement")
      print("trace\t\t\t- display runtime stack")
      print("up\t\t\t- move up one stack frame")
      print("where\t\t\t- print current program line")
      print()
      return START_DEBUGGER

   def _handle_list(self, _):
      (file,lineno) = self.interp_state.lineinfo
      self._load_program_text(file)
      pt = self.program_text[file]
      # Length around the current line to display
      length = 4
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
      if len(args) > 1:
         print("error: too many arguments")
         return False
      elif len(args) == 0:
         print("error: no argument given")
         return False
      if args[0] == '*':
         var_list = self.interp_state.symbol_table.get_curr_scope(scope=self.frame_ix, option="items")
         for (name,val) in var_list:
            print("{}: {}".format(name,term2string(val)))
      else:
         val = self.interp_state.symbol_table.lookup_sym(args[0],strict=False)
         if not val:
            print("error: variable {} not found".format(args[0]))
         else:
            print("{}: {}".format(args[0],term2string(val)))
      return START_DEBUGGER

   def _handle_quit(self,_):
      raise SystemExit()
   
   def _handle_set(self,args):
      if len(args) == 0:
         # set a breakpoint at the current line
         (file,lineno) = self.interp_state.lineinfo
         self.line_breakpoints.append((lineno,file))
         return START_DEBUGGER
      elif len(args) == 1:
         (file,_) = self.interp_state.lineinfo
         if args[0].isnumeric(): 
            self.line_breakpoints.append((int(args[0]),file))
         else:
            self.function_breakpoints.append((args[0],file))
         return START_DEBUGGER
      elif len(args) == 2:
         if args[0].isnumeric(): 
            self.line_breakpoints.append((int(args[0]),args[1]))
         else:
            self.function_breakpoints.append((args[0],args[1]))
         return START_DEBUGGER
      else:
         print("error: too many arguments to set")
         return START_DEBUGGER

   def _handle_frame(self,_):
      print("you are looking at frame #{}".format(self.frame_ix))
      return START_DEBUGGER

   def _handle_step(self,_):
      self.frame_ix = 0
      return RETURN_TO_INTERP

   def _handle_stack(self,_):
      trace = copy.copy(self.interp_state.trace_stack)
      trace.reverse()
      print("Runtime stack (most recent call first):")
      for i in range(self.frame_ix,len(trace)):
         (module,lineno,fname) = trace[i]
         print("frame #{}: {} @{}".format(i,module,fname))
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
