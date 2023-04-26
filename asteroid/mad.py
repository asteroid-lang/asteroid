###########################################################################################
# Minimal Asteroid Debugger (MAD)
#
# (c) University of Rhode Island
###########################################################################################

from os.path import exists, split, basename
from asteroid.support import term2string
from asteroid.version import MAD_VERSION

START_DEBUGGER = False
RETURN_TO_INTERP = True

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
      # command dispatch table, only uses the first two letters of each command
      # in order to interpret the commands
      self.dispatch_table = {
                  'br'     : self._handle_breakpoints,
                  'cl'     : self._handle_clear,
                  'co'     : self._handle_continue,
                  'he'     : self._handle_help,
                  'li'     : self._handle_list,
                  'ne'     : self._handle_next,
                  'pr'     : self._handle_print,
                  'qu'     : self._handle_quit,
                  'se'     : self._handle_set,
                  'st'     : self._handle_step,
                  'wh'     : self._handle_where,
               }
      # the following tables of tuples (func/line#,file) that describes break points at 
      #function entry points or lines
      self.function_breakpoints = []
      self.line_breakpoints = []

   ###########################################################################################
   # debugger API from interpreter
   def start(self, state):
      self.interp_state = state
      print("Minimal Asteroid Debugger -- Version {}".format(MAD_VERSION))
      print("(c) University of Rhode Island")
      print("type 'help' for additional information")
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
         print("reached breakpoint ({}:{})".format(file,fname))
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
      # print out current line within a window of surrounding lines
      self._handle_list([])

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
         self._print_line()
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
      if len(cmd_list) > 0:
         val = self.dispatch_table.get(cmd_list[0][0:2],lambda _:-1) (cmd_list[1:])
         if val == -1:
            print("error: unknown command {}".format(cmd_list[0]))
            return False
         else:
            return val
      else:
         # assume implicit step when user just hits return
         return self._handle_step([])

   def _handle_breakpoints(self,_):
      print("breakpoints:")
      for (bp,file) in self.function_breakpoints:
         print("{}:{}".format(file,bp))
      for (bp,file) in self.line_breakpoints:
         print("{}:{}".format(file,bp))
      return False
      
   def _handle_clear(self,_):
      # TODO: allow for specicific breakpoints to be cleared
      self.function_breakpoints = []
      self.line_breakpoints = []
      return False
      
   def _handle_continue(self,_):
      self.continue_mode = True
      return True

   def _handle_help(self,_):
      print()
      print("Available commands:")
      print("breakpoints\t\t- list all breakpoints")
      print("clear\t\t\t- clear all breakpoints")
      print("continue\t\t- continue execution to next break point")
      print("help\t\t\t- display help")
      print("list\t\t\t- display source code")
      print("next\t\t\t- continue execution across a nested scope")
      print("print <name>|*\t\t- print contents of <name>, * lists all vars in scope")
      print("quit\t\t\t- quit debugger")
      print("set [<func>|<line#> [<file>]]\n\t\t\t- set a breakpoint")
      print("step\t\t\t- step to next executable statement")
      print("where\t\t\t- print current program line")
      print()
      return False

   def _handle_list(self, _):
      (file,lineno) = self.interp_state.lineinfo
      # make sure our file is loaded into the cache
      self._load_program_text(file)
      # Get the program text for the current file
      pt = self.program_text[file]
      # Length around the current line to display if relative
      length = 4

      # Compute the start and end of listing
      start = (lineno - length) if lineno >= length else 0
      end = lineno + length if lineno < len(pt) - 2 else len(pt)
      # Set the program text to the slice between start and end
      pt = pt[start:end]

      # Start of line is blank by default
      start_of_line = "  "
      # GO through each line in the program text
      for ix, l in enumerate(pt):

         # if the offset line number is in breakpoints
         if (ix+1+start,file) in self.line_breakpoints:
               # Set the special start of line
               start_of_line = "* "

         # mark the current line
         if lineno == ix+1+start:
               # Set the special start of line
               start_of_line = "> "

         # print the given line
         print(start_of_line, ix+1+start, l[:-1])

         # Reset the start of line
         start_of_line = "  "
      return START_DEBUGGER

   def _handle_next(self,_):
      # initialize the "scope stack", as long as the scope
      # counter value is > 0 all call backs from the interpreter
      # immediately return control back to the interpreter with
      # one exception: breakpoints.  If a breakpoint is hit
      # the scope counter is reset to -1.
      self.scope_counter = 1
      return RETURN_TO_INTERP

   def _handle_print(self,args):
      if len(args) > 1:
         print("error: too many arguments to print command")
         return False
      elif len(args) == 0:
         print("error: no argument to print command")
         return False
      if args[0] == '*':
         var_list = self.interp_state.symbol_table.get_curr_scope(option="items")
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

   def _handle_step(self,_):
      return RETURN_TO_INTERP

   def _handle_where(self,_):
      self._print_line()
      return START_DEBUGGER
