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
        # List of function calls
        self.call_stack = []

        #############################
        self.tab_level = 0

        # File information
        self.lineinfo = None
        self.program_text = None
        self.filename = None
        
        #############################
        # The parser for incoming commands
        self.dbgp = DebuggerParser()
        
        #############################
        # Dictionary of macros
        self.macros = {
            'px': [('CONTINUE',), ('EXPLICIT',), ('EVAL', "io@println(\"x=\"+x)")]
        }

        #############################
        # The queue of commands being executed
        self.command_queue = []
    
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
        self.call_stack = []

    def make_tab_level(self):
        """
        Make the tab level for nested messaging
        """
        return self.tab_level*"  "

    def message_explicit(self, message, level = None):
        """
        Sends a message in explicit mode
        """

        if self.explicit_enabled and not self.is_continuing and self.lineinfo[0] == self.filename:
            match(level):
                case None:
                    print("{}- {}".format(self.make_tab_level(), message))
                case "secondary":
                    print("{} ** {}".format(self.make_tab_level(), message))
                case "tertiary":
                    print("{}  * {}".format(self.make_tab_level(), message))

    def message(self, message):
        """
        Print a formatted message through the debugger
        """
        print("----- {} -----".format(message))

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

        # Main debug loop
        while True:
            try:
                # Interpret our file
                interp(input_stream,
                    input_name = filename,
                    do_walk=True,
                    prologue=True,
                    exceptions=True,
                    debugger=self)
                
                # Give us one final tick before restarting
                # This gives us one last tick before EOF is reached
                self.lineinfo = (self.filename, len(self.program_text))
                self.tick()
                print()

                # Restart session
                self.message("End of file reached, restarting session")
                self.reset_defaults()
            
            except (EOFError, KeyboardInterrupt):
                # If the user tries to exit with CTRL+C/D, exit
                break;

            except Exception as e:
                # Handle exceptions from the interpretation session
                (module, lineno) = state.lineinfo
                print("\nERROR: {}: {}: {}".format(module, lineno, e))
                dump_trace()

                # If the error occured in our file, show the offending line
                if self.lineinfo and (module == self.lineinfo[0]):
                    print("    ==>> " + self.program_text[lineno - 1].strip())
                    print()
                    self.message("Error occured, restarting session")
                    self.reset_defaults()
                    continue
                else:
                    # Otherwhise, just break
                    break

    def has_breakpoint_here(self):
        """
        Check if the user has set a breakpoint at the current line
        """
        # Condition 1: Is there a breakpoint at this line?
        breakpoint_at_line = (self.lineinfo[1] in self.breakpoints)

        # Condition 2
        in_same_file = (self.lineinfo[0] == self.filename)

        # Preliminary check
        if not (breakpoint_at_line and in_same_file):
            return False
        
        # Check the break condition
        break_cond = self.breakpoints.get(self.lineinfo[1])

        # Assume the break condition is true until proven false
        break_cond_met = True

        # If there's a break condition
        if break_cond:

            # Save our old lineinfo/explicit state
            old_lineinfo = self.lineinfo
            old_explicit = self.explicit_enabled
            self.explicit_enabled = False

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

            # Reenable everything
            self.explicit_enabled = old_explicit
            self.set_lineinfo(old_lineinfo)

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
        self.lineinfo = lineinfo

        if self.program_text is None:
            with open(lineinfo[0], "r") as f:
                self.program_text = f.readlines()
            
            # Always add an EOF specifier
            self.program_text.append("[EOF]\n")

    def print_current_line(self):
        """
        Print the current line nicely
        """
        prog_line = self.program_text[self.lineinfo[1] - 1][:-1].strip()
        outline =  ("[" + self.lineinfo[0] + " (" + str(self.lineinfo[1]) + ")]")

        if len(self.call_stack) > 0:
            outline += " ("
            for c in self.call_stack[:-1]:
                outline += c + "->"
            outline += self.call_stack[-1] + ")"

        # If the line is empty don't bother showing it
        if prog_line != "":
            outline += ("\n-->> " + prog_line)

        print(outline)

    def list_breakpoints(self):
        """
        List the breakpoints and their conditions
        """
        self.message("Breakpoints")
        
        for b in self.breakpoints:
            c = self.breakpoints[b]
            print("* {} {}".format(
                b, ": " + c if c else ''))

    def list_program(self, relative=False):
        """
        List the program contents
        """
        self.message("Program Listing")
        start_of_line = "  "

        pt = self.program_text

        length = 8
        start = 0

        if relative:
            lineno = self.lineinfo[1]

            start = (lineno - length) if lineno >= length else 0
            end = lineno + length if lineno < len(pt) - 2 else len(pt)
            pt = pt[start:end]

        for ix, l in enumerate(pt):
            if ix+1+start in self.breakpoints:
                start_of_line = "* "
            if self.lineinfo[1] == ix+1+start:
                start_of_line = "> "

            print(start_of_line, ix+1+start, l[:-1])
            start_of_line = "  "

    def set_config(self, step=False, cont=False, next=False):
        """
        Set the debugger movement configuration
        """
        self.is_stepping = step
        self.is_continuing = cont
        self.is_next = next

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

            # Macro display
            case ('MACRO',):
                for m in self.macros:
                    print("* {} : {}".format(
                        m, self.macros[m]
                    ))
            # Macros
            case ('MACRO', name, l):
                self.macros[name] = l
                self.message("Macro {}".format(name))

            # Literal commands
            case ('EVAL', value):
                old_lineinfo = self.lineinfo
                old_explicit = self.explicit_enabled
                self.explicit_enabled = False

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
                    print(term2string(function_return_value[-1]))
                
                self.explicit_enabled = old_explicit
                self.set_lineinfo(old_lineinfo)
            
            # Step
            case ('STEP', ):
                self.set_config(step=True)
                exit_loop = True

            # Continue
            case ('CONTINUE', ):
                self.set_config(cont=True)
                exit_loop = True

            # Next
            case ('NEXT', ):
                self.set_config(next=True)
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

            # REPL (!)
            case ('BANG', ):
                old_lineinfo = self.lineinfo
                old_explicit = self.explicit_enabled
                self.explicit_enabled = False
                
                repl(new=False)
                
                self.explicit_enabled = old_explicit
                self.set_lineinfo(old_lineinfo)
            
            # Longlist, List, Quit, Explicit, Unexplicit
            case ('LONGLIST',):     self.list_program()
            case ('LIST',):         self.list_program(relative=True)
            case ('EXPLICIT', ):    self.explicit_enabled = True
            case ('UNEXPLICIT', ):  self.explicit_enabled = False

            # Help menu
            case ('HELP', name):
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
                    for c in command_description_table:
                        print("* {}".format(c))

            case ('NAME', v):
                # If the command name is in macros
                if v in self.macros:                    
                    self.command_queue += self.macros[v]
                else:
                    raise ValueError("Unknown macro: {}".format(str(v)))

            case ('UP',):
                """OWM
                How will stack frame traversal work?

                Some kind of loop where we run commands UNTIL an exit_loop is caused. then just
                run as normal

                Going to need to keep a list of contexts

                Save the original config then reset it afterwards
                Need to keep track of when we run out of scopes to go up.

                self.trace_stack = [(module,1,"<toplevel>")]

                old_config = state.symbol_table.get_config()
                """
                stack = state.trace_stack
                if state.symbol_table.at_topmost_frame():
                    self.message("At topmost frame")
                else:
                    """
                    The issue with frame jumping is that we don't actually
                    store currently inactive frames anywhere except the locally
                    saved scope in `handle_call`.

                    So we need to do two things.
                    1) Have some guard to exit the recursive loops of up/down
                    2) Some way of keeping track of the respective scopes
                    """
                    pass
                    # # Save everything
                    # old_lineinfo = self.lineinfo
                    # (old_scoped_symtab, old_globals, old_global_scope) = state.symbol_table.get_config()
                    # old_stack = stack

                    # # Set new lineinfo
                    # self.lineinfo = (old_stack[-1][0], old_stack[-1][1])

                    # # Set new trace_stack
                    # state.trace_stack = old_stack[:-1]

                    # # Set new config
                    # print(old_scoped_symtab)
                    # state.symbol_table.set_config(
                    #     (old_scoped_symtab[:-1], old_globals, old_global_scope)
                    # )

                    # # Print the current line with lineinfo
                    # self.print_current_line()

                    # # Main command loop
                    # self.main_command_loop()

                    # # Reset old config
                    # state.symbol_table.set_config(
                    #     (old_scoped_symtab, old_globals, old_global_scope)
                    # )

                    # # Reset trace_stack
                    # state.trace_stack = old_stack

                    # # Reset lineinfo
                    # self.lineinfo = old_lineinfo

            case ('DOWN',):
                pass

            case ('QUIT', ):        raise SystemExit()

            # Macro/Unknown
            case _:
                raise ValueError("Unknown command: {}".format(str(cmd)))

        return exit_loop

    def main_command_loop(self):
        exit_loop = False

        # Main command loop
        while not exit_loop:
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
                        break;
            
            # Intercept debugger command errors
            except ValueError as e:
                print("Debugger command error [{}]".format(e))

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
                break

        # Print the current line with lineinfo
        self.print_current_line()

        # Main command loop
        self.main_command_loop()

        # Reset the tab level
        self.tab_level = 0
    
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

        # If we're not on the intended file, just return
        if self.lineinfo[0] != self.filename:
            pass

        # If we have a breakpoint here and we're not trying to go
        # to the next top level statement, then tick
        elif self.has_breakpoint_here() and not self.is_next:
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

if __name__ == "__main__":
    db = ADB()
    import sys
    if len(sys.argv) < 2:
        print("No file given to debug")
    else:
        db.run(sys.argv[-1])