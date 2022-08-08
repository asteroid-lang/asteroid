"""
The Asteroid Debugger
"""

from asteroid.repl import repl
from asteroid.adb.command import DebuggerParser
from asteroid.interp import interp, load_prologue
from asteroid.state import dump_trace, state
from asteroid.support import term2string, map2boolean

from asteroid.walk import function_return_value

class ADB:
    """
    This class implements the behavior and state managment for the
    asteroid debugger
    """
    def __init__(self):
        """
        Major distinction notice:
            stepping:   Go to next executing line
            continuing: Go to next breakpoint
            next:       Go to next top level line (i.e. top level statement) {Can be a breakpoint}

        State management is pretty important so I've segmented it into several pieces.
            * Stepping/continuation/next level (What do we do on next tick?)
            * Top level (Is this statement at the top level?)
                * Explicit (Are we just showing everything for a little bit?)
        """
        # Table of breakpoints and conditions
        self.breakpoints = {}

        #############################
        # Flag if the debugger is continuing to the next breakpoint (continue) 
        self.is_continuing = False

        # OR next line (step)
        self.is_stepping = False

        # OR next top level statement (next)
        self.is_next = True

        #############################
        # If our program is executing at the top level
        self.top_level = True

        # Explicit mode is the verbose mode where more information
        # about the computation is detailed
        self.explicit_enabled = False

        #############################
        # The current level of tabulation
        self.tab_level = 0

        #############################
        # Debugger's internal lineinfo
        # We maintain this separate from the state's lineinfo
        # to help keep track of the currently executing line.
        # Between things like imports or function calls to
        # other modules, the state's lineinfo can fall slightly
        # behind.
        self.lineinfo = None

        #############################
        # Name-content file dictionary
        self.program_text = {}
        
        # The original filename
        self.filename = None
        
        #############################
        # The parser for incoming commands
        self.dbgp = DebuggerParser()
        
        #############################
        # Dictionary of macros
        self.macros = {}

        #############################
        # The queue of commands being executed
        self.command_queue = []

        #############################
        # Stack frame information
        self.config_offset = 0          # The index of the current config we're using
        self.original_config = None     # The original config we started with (before moving between frames)
        self.original_lineinfo = None   # The original lineinfo we started with (before moving between frames)

    def reset_defaults(self):
        """
        Resets the debugger's default config
        """
        # Reset defaults
        self.is_continuing = False
        self.is_stepping = False
        self.is_next = True
        self.top_level = True
        self.explicit_enabled = False

    def reset_config(self):
        """
        Reset the symbol table's original config
        """
        if self.original_config:
            state.symbol_table.set_config(self.original_config)
            self.original_config = None
            self.original_lineinfo = None
            self.config_offset = 0

    def make_tab_level(self):
        """
        Make the tab level for nested messaging
        """
        return self.tab_level*"|   "

    def message_explicit(self, message, level = "primary"):
        """
        Sends a message in explicit mode

        Explicit messaging only gets shown in two scenarios if explicit mode is enabled:
            1. The user is NOT continuing and they're in the same file
            2. The user is stepping through a function call
        """
        if self.explicit_enabled and \
            (not self.is_continuing and self.lineinfo[0] == self.filename) or \
            (self.is_stepping):

            tl = self.make_tab_level()
            match(level):
                case "primary":
                    print("{}- {}".format(tl, message))
                case "secondary":
                    print("{} ** {}".format(tl, message))
                case "tertiary":
                    print("{}  * {}".format(tl, message))

    def message(self, message):
        """
        print a formatted message through the debugger
        """
        print("----- {} -----".format(message))

    def handle_run_exception(self, e):
        # Handle exceptions from the interpretation session
        (module, lineno) = state.lineinfo
        print("\nERROR: {}: {}: {}".format(module, lineno, e))
        dump_trace()
        
        # Set out lineinfo here to be sure that the file is in
        # our program_text dictionary
        self.set_lineinfo( (module, lineno) )
        
        print("    ==>> " + self.program_text[module][lineno - 1].strip())
        print()
        self.message("Error occured, session will restart after commands")

    def run(self, filename):
        """
        This function runs the given filename through the asteroid debugger
        """
        
        # Set our primary filename
        self.filename = filename

        # Open and read the file
        f = open(filename, 'r')
        input_stream = f.read()
        f.close()

        from asteroid.adb.version import VERSION
        print("Welcome to ADB version {}.".format(VERSION))
        print("Type \"help\" to recieve help.")
        print("ADB is experimental and under active development")
        print("Report any suggestions or bugs to https://github.com/asteroid-lang/")
        print()

        # Main debug loop
        while True:
            try:
                # Reset the debugging status
                import asteroid.walk
                asteroid.walk.debugging = False

                # Reinitialize the state and reload the prologue
                state.initialize()
                load_prologue()

                # Interpret our file
                interp(input_stream,
                    input_name = filename,
                    prologue=False,
                    initialize_state=False,
                    exceptions=True,
                    debugger=self)

                asteroid.walk.debugging = True
                
                # Give us one final tick before restarting
                # This gives us one last tick before EOF is reached
                self.set_lineinfo( (self.filename, len(self.program_text[self.filename])) )
                self.tick()
                print()

                # Restart session
                self.message("End of file reached, restarting session")
                self.reset_defaults()
            
            except (EOFError, KeyboardInterrupt):
                break;

            except Exception as e:
                # Handle the runtime exception
                self.handle_run_exception(e)

                # One last tick so the user can play around in
                # the error scope/line
                try:
                    self.tick()
                except (EOFError, KeyboardInterrupt):
                    # If the user tries to exit with CTRL+C/D, exit
                    break;
                
                # Reset the debugger's default state
                self.message("Session restarted")
                self.reset_defaults()
                continue

    def has_breakpoint_here(self):
        """
        Check if the user has set a breakpoint at the current line
        """
        # Condition 1: Is there a breakpoint at this line?
        breakpoint_at_line = (self.lineinfo[1] in self.breakpoints)

        # Condition 2, are we in the original file (TODO: Expand this)
        in_same_file = (self.lineinfo[0] == self.filename)

        # Preliminary check
        if not (breakpoint_at_line and in_same_file):
            return False
        
        # Get the break condition for this breakpoint
        break_cond = self.breakpoints.get(self.lineinfo[1])

        # Assume the break condition is true until proven false
        break_cond_met = True

        # If there's a break condition
        if break_cond:

            # Save our old lineinfo/explicit state
            old_lineinfo = self.lineinfo
            old_explicit = self.explicit_enabled
            self.explicit_enabled = False

            old_state_lineinfo = state.lineinfo

            import asteroid.walk
            asteroid.walk.debugging = False
            # interpret the break conition
            try:
                interp(break_cond,
                    input_name = "<COMMAND>",
                    redundancy=False,
                    prologue=False,
                    initialize_state=False,
                    debugger=None,
                    exceptions=True
            )

            # If an error occurs in the break condition, show the error
            except Exception as e:
                print("Breakpoint condition error: {}".format(e))

            # Else, get the value of the expression calculated.
            else:
                break_cond_met = map2boolean(function_return_value[-1])[1]

            asteroid.walk.debugging = True

            # Reenable everything
            self.explicit_enabled = old_explicit
            self.set_lineinfo(old_lineinfo)

            # Reset the state's internal lineinfo
            state.lineinfo = old_state_lineinfo

        # If there's no break cond, then by default it is true
        else:
            break_cond_met = True

        # Return the state of the break condition
        return break_cond_met

    def set_top_level(self, tl):
        """
        Set our flag that tells the debugger if it's at the top level of a program or not

        We use this to get the `next` command working
        """
        self.top_level = tl

    def set_lineinfo(self, lineinfo):
        """
        Set the debugger's internal lineinfo
        """
        from os.path import exists

        self.lineinfo = lineinfo
        
        # If the program text isn't already loaded and the file actually exists
        # (isn't a stream line <input> or <command>)
        if lineinfo[0][0] == '<' and lineinfo[0][-1] == '>':
            return

        if not self.program_text.get(lineinfo[0]) and exists(lineinfo[0]):
            with open(lineinfo[0], "r") as f:
                self.program_text[lineinfo[0]] = f.readlines()
            
            # Always add an EOF specifier
            self.program_text[lineinfo[0]].append("[EOF]\n")

    def print_given_line(self, lineinfo, header=True):
        """
        print the current line nicely
        """

        # Get the program text
        pt = self.program_text[lineinfo[0]]

        # Isolate the current line
        prog_line = pt[lineinfo[1] - 1][:-1].strip()

        outline = ""

        # The current call stack
        call_stack = [s[-1] for s in state.trace_stack][1:]

        if header:
            # Format it nicely
            outline =  ("[" + lineinfo[0] + " (" + str(lineinfo[1]) + ")]")

            # Display the call stack
            if len(call_stack) > 0 and (self.config_offset < len(call_stack)):
                outline += " ("

                offset = -self.config_offset - 1
                for c in call_stack[:offset]:
                    outline += c + "->"
                outline += call_stack[offset] + ")"

        # If the line is empty don't bother showing it
        if prog_line != "" and header:
            outline += ("\n-->> " + prog_line)
        elif prog_line != "" and not header:
            outline += ("    " + prog_line)
        print(outline)

    def list_breakpoints(self):
        """
        List the breakpoints and their conditions
        """
        self.message("Breakpoints")
        
        # For eaxch breakpoint
        for b in self.breakpoints:

            # Get the condition
            c = self.breakpoints[b]

            # print the breakpoint number and condition if available
            print("* {} {}".format(
                b, ": " + c if c else ''))

    def list_program(self, relative=False):
        """
        List the program contents
        """
        self.message("Program Listing")

        # Get the program text for the current file
        pt = self.program_text[self.lineinfo[0]]

        # Length around the current line to display if relative
        length = 4

        # Relative offset for the file listing
        start = 0

        if relative:
            lineno = self.lineinfo[1]

            # Compute the start and end of relative listing
            start = (lineno - length) if lineno >= length else 0
            end = lineno + length if lineno < len(pt) - 2 else len(pt)

            # Set the program text to the slice between start and end
            pt = pt[start:end]

        # Start of line is blank by default
        start_of_line = "  "

        # GO through each line in the program text
        for ix, l in enumerate(pt):

            # If the offset line number is in breakpoints
            if ix+1+start in self.breakpoints:
                # Set the special start of line
                start_of_line = "* "

            # If the offset linenumber is the current line
            if self.lineinfo[1] == ix+1+start:
                # Set the special start of line
                start_of_line = "> "

            # print the given line
            print(start_of_line, ix+1+start, l[:-1])

            # Reset the start of line
            start_of_line = "  "

    def set_movement(self, step=False, cont=False, next=False):
        """
        Set the debugger movement configuration
        """
        self.is_stepping = step
        self.is_continuing = cont
        self.is_next = next

    def display_macros(self):
        """
        Displays all currently active macros
        """
        for m in self.macros:
            print("* {} : {}".format(
                m, self.macros[m]
            ))

    def set_new_macro(self, name, l):
        """
        Sets a new macro
        """
        self.macros[name] = l
        self.message("Macro {}".format(name))

    def do_eval_command(self, value):
        """
        Evaluates a given value
        """
        # Save the old lineinfo and explicit state
        old_lineinfo = self.lineinfo
        old_explicit = self.explicit_enabled
        old_state_lineinfo = state.lineinfo
        
        # Set the explicit state to false
        self.explicit_enabled = False
        
        # Set the debugging flag to false
        import asteroid.walk
        asteroid.walk.debugging = False
        
        # Run the asteroid code
        try:
            interp(value,
                input_name = "<EVAL>",
                redundancy=False,
                prologue=False,
                initialize_state=False,
                debugger=None,
                exceptions=True)

        except Exception as e:
            print("Command error: {}".format(e))

        else:
            # Check if there's actually a return value in
            # the register
            if function_return_value[-1]:
                # Get the last return value (type, value)
                retval = function_return_value[-1]

                # If it isn't none, print out the value
                if retval[1] != None:
                    print(term2string(function_return_value[-1]))

        # Reset debugging state
        asteroid.walk.debugging = True
        
        # Reset explicit mode and lineinfo
        self.explicit_enabled = old_explicit
        self.set_lineinfo(old_lineinfo)
        
        # Reset the state's internal lineinfo
        state.lineinfo = old_state_lineinfo

    def do_repl_command(self):
        """
        Runs a repl in the current stack frame
        """
        # Keep our lineinfo, and explicit state. Disable explicit state
        old_lineinfo = self.lineinfo
        old_explicit = self.explicit_enabled
        self.explicit_enabled = False

        # Save the *state*'s old lineinfo
        old_state_lineinfo = state.lineinfo

        # Turn off debugging
        import asteroid.walk
        asteroid.walk.debugging = False
        
        # Run the repl
        repl(new=False)

        # Restore debugging flag and give state its old lineinfo
        asteroid.walk.debugging = True
        state.lineinfo = old_state_lineinfo

        # Reenable explicit and reset lineinfo
        self.explicit_enabled = old_explicit
        self.set_lineinfo(old_lineinfo)

    def do_help_command(self, name):
        """
        Lists help options and prints help info
        """
        from asteroid.adb.help import command_description_table
        
        # If a command name is supplied
        if name:
            # Get the command description for that name
            help_msg = command_description_table.get(name)
            # if there's a description, print the info
            if help_msg:
                self.message("Info for {}".format(name))
                print(help_msg)
            # Else, print an error
            else:
                self.message("Unknown command for help: {}".format(
                    name
                ))
        
        # If no command is supplied, then just print out the default command menu
        else:
            print("Type 'help NAME' to get help for a command")
            print("Available commands:")
            for c in command_description_table:
                print("* {}".format(c))

    def move_frame_up(self):
        """
        Moves the context to the next higher stack frame
        """
        if self.config_offset == len(state.trace_stack) - 1:
            self.message("At topmost frame")
        else:
            self.config_offset += 1
            # We're at the bottommost frame and want to go up, but need
            # to save the original config
            if self.original_config == None:
                self.original_config = state.symbol_table.get_config()
                self.original_lineinfo = state.lineinfo
            
            # Get the associated module and line for this frame
            (module, line, _) = state.trace_stack[-self.config_offset]
            
            # Set the config
            state.symbol_table.set_config(
                state.symbol_table.saved_configs[-self.config_offset]
            )
            self.set_lineinfo( (module, line) )
            self.print_given_line(self.lineinfo)

    def move_frame_down(self):
        """
        Moves the context to the next lowest stack frame
        """
        stack = state.trace_stack
        
        if self.config_offset == 0:
            self.message("At bottommost frame")
        
        else:
            self.config_offset -= 1
            bottom_level = (self.config_offset == 0)
        
            if bottom_level:
                if self.original_config:
                    state.symbol_table.set_config(self.original_config)
                    self.set_lineinfo( (self.original_lineinfo) )
            else:
                # We're at the bottommost frame and want to go up, but need
                # to save the original config
                (module, line, _) = state.trace_stack[-self.config_offset]
                
                state.symbol_table.set_config(
                    state.symbol_table.saved_configs[-self.config_offset])

                self.set_lineinfo( (module, line) )
                
            self.print_given_line(self.lineinfo)

    # This function is super spaghetti but the behavior is complicated
    def do_where_command(self):
        """
        Displays a list of available frames and shows the user
        where they currently are
        """
        self.message("Available Frames")
        stack_copy = state.trace_stack[1:].copy()

        # The call stack
        call_stack = [s[-1] for s in state.trace_stack][1:]

        # If we're inside of some scope, append a "bottom" to the stack copy
        if len(call_stack) > 0:
            stack_copy.append((*state.lineinfo, "<bottom>"))
        
        start_of_line = "*"
        
        # If we're just at the top level, just note that
        if len(stack_copy) == 0:
            print("-> <toplevel>")
        
        # For each list in the stack
        for i, s in enumerate(stack_copy):
            # There's only the top level
            if len(stack_copy) == 1:
                start_of_line = ">"
        
            # We're at the bottom of the stack
            elif (self.config_offset == 0 and len(stack_copy) > 0) and i == len(stack_copy) - 1:
                start_of_line = ">"
        
            # We're traversing frames
            elif self.config_offset != 0:
                if i == (len(stack_copy) - self.config_offset) - 1:
                    start_of_line = ">"

            # Bottom of stack
            if s[2] == "<bottom>":
                print("{} {} {}".format(start_of_line, s[0], s[1]))
                self.print_given_line( (s[0], s[1]) , header=False)
        
            else:
                print("{} {} {} (Calling {})".format(start_of_line, s[0], s[1], s[2]))
                self.print_given_line( (s[0], s[1]) , header=False)
            start_of_line = "*"

    def walk_command(self, cmd):
        """
        Walk a given command
        """
        # Loop sentinel value
        # This is returned as True ~iff~ a movement command is
        # executed
        exit_loop = False

        # Match command to behavior
        match(cmd):
            case ('MACRO',):            self.display_macros()
            case ('MACRO', name, l):    self.set_new_macro(name, l)
            case ('EVAL', value):       self.do_eval_command(value)
            case ('BANG', ):            self.do_repl_command()
            case ('HELP', name):        self.do_help_command(name)
            case ('UP',):               self.move_frame_up()
            case ('DOWN',):             self.move_frame_down()
            case ('WHERE',):            self.do_where_command()
            case ('LONGLIST',):         self.list_program()
            case ('LIST',):             self.list_program(relative=True)

            case ('RETURN',):
                self.message("RETURN NOT IMPLEMENTED YET")
                exit_loop = True

            case ('EXPLICIT', set_explicit):
                if set_explicit == False:
                    self.explicit_enabled = False
                elif set_explicit == True:
                    self.explicit_enabled = True
                else:
                    self.explicit_enabled = not self.explicit_enabled

            # Step
            case ('STEP', ):
                self.set_movement(step=True)
                exit_loop = True

            # Continue
            case ('CONTINUE', ):
                self.set_movement(cont=True)
                exit_loop = True

            # Next
            case ('NEXT', ):
                self.set_movement(next=True)
                exit_loop = True

            # Break 
            case ('BREAK', nums, conds):
                if nums:
                    for ix, n in enumerate(nums):
                        self.breakpoints[n] = conds[ix]
                else:
                    self.list_breakpoints()

            # Delete
            case ('DELETE', nums):
                for n in nums:
                    self.breakpoints.pop(n)

            # Macro/Unknown
            case ('NAME', v):
                # If the command name is in macros
                if v in self.macros:                    
                    self.command_queue += self.macros[v]
                else:
                    raise ValueError("Unknown macro: {}".format(str(v)))

            # Quit command
            case ('QUIT', ):
                raise SystemExit()

            case ('NOOP', ):
                pass

            case _:
                raise ValueError("Unknown command: {}".format(str(cmd)))

        return exit_loop

    def command_loop(self, in_pattern=False):
        """
        Main command loop for ADB
        """
        exit_loop = False

        # Main command loop
        while not exit_loop:

            # in_pattern in explicit mode is a flag for currently stepping
            # through a pattern
            if in_pattern:
                query_symbol = "[Pattern] "
            else:
                # Format the input symbol to reflect explicitness
                query_symbol = "(ADB)[e] " if self.explicit_enabled else "(ADB) "

            # Get the command
            cmd = input(query_symbol)
            
            # Try to walk the command
            try:
                # Parse the command
                (LINE, node) = self.dbgp.parse(cmd)

                # Add the new commands to the queue
                self.command_queue += node

                # Add the list of commands to the queue
                while self.command_queue:
                    # Walk the command and get the exit state
                    exit_loop = self.walk_command(self.command_queue.pop(0))

                    # Exit if necessary
                    if exit_loop:
                        self.reset_config()
                        break;
            
            # Intercept debugger command errors
            except ValueError as e:
                print("Debugger command error [{}]".format(e))

            # If we are in a pattern but disabled explicit mode, that's
            # grounds for loop exiting
            if in_pattern and not self.explicit_enabled:
                exit_loop = True

        return exit_loop

    def tick(self):
        """
        "Tick" the debugger. This refers to hitting some point where the user
        has decided they would like the debugger to come back to life and entering
        the command selection phase.

        The debugger uses a queue to store working commands. This allows for more
        complex command execution
        """
        exit_loop = False
        
        # Clear out the command queue
        while self.command_queue:
            exit_loop = self.walk_command(self.command_queue.pop(0))
            if exit_loop:
                if self.original_config:
                    self.reset_config()
                return
        
        # print the current line with lineinfo
        self.print_given_line(self.lineinfo)

        # Main command loop
        self.command_loop()

        # Reset the tab level
        self.tab_level = 0
    
    def notify_explicit(self):
        """
        Run a command loop ~iff~ we're in explicit mode
        """
        if self.is_stepping and self.explicit_enabled:
            self.command_loop(in_pattern=True)

    def notify(self):
        """
        Notify the debugger that a potential tick-point has
        occured and do the necessary checks to see if we can
        tick here.

        This function is a little complicated because the
        behavior is complicated.

        Hierarchy of ticking:
            Step
            Breakpoint (Continue)
            Next

        Explicit mode is a mode in which extra steps in
        computations are revealed to the user
        """
        # If we have a breakpoint here and we're not trying to go
        # to the next top level statement, then tick

        """
        if self.is_return:
            if at_return_stmt:
                self.message("Reached return location")
                self.tick()
                self.set_return(False)
            else:
                pass
            
        elif self.is_return and len(state.trace_stack) == 1:
            self.set_movement(next=True)
        """

        if self.has_breakpoint_here() and not self.is_next:
            self.message("Breakpoint")
            self.tick()

        # If we're at the top level and we're not continuing
        # to the next breakpoint, and we're going to the next statement
        # do a tick
        elif self.top_level and self.is_next and not self.is_continuing:
            if self.has_breakpoint_here():
                self.message("Breakpoint")
            self.tick()

        # Otherwhise, if we're stepping through the program,
        # always tick
        elif self.is_stepping:
            self.tick()
        
        # Reset the top level so that nested statements don't come in
        self.set_top_level(False)