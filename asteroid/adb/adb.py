"""
The Asteroid Debugger

Known issues:
    Debugging programs with imports does not work correctly
"""
from asteroid.repl import repl

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
        """
        # List of breakpoints
        self.breakpoints = []

        # Flag if the debugger is continuing to the next breakpoint (continue) 
        self.is_continuing = False

        # OR next line (step)
        self.is_stepping = False

        # OR next top level statement (next)
        self.is_next = True

        # If our program is executing at the top level
        self.top_level = True

        # File information
        self.lineinfo = None
        self.program_text = None
        self.filename = None

        # Explicit mode is the verbose mode where expressions are 
        # totally detailed
        self.explicit = True

    def message(self, message):
        """
        Print a formatted message through the debugger
        """
        print("----- {} -----".format(message))

    def run(self, filename):
        """
        This function runs the given filename through
        the asteroid debugger
        """
        from asteroid.interp import interp
        from asteroid.state import dump_trace
        
        self.filename = filename

        f = open(filename, 'r')
        input_stream = f.read()
        f.close()

        while True:
            try:
                interp(input_stream,
                    input_name = filename,
                    do_walk=True,
                    prologue=False,
                    exceptions=True,
                    debugger=self)
                self.message("End of file reached, restarting session")

                # Reset defaults
                self.is_continuing = False
                self.is_stepping = False
                self.is_next = True
                self.top_level = True
            
            except (EOFError, KeyboardInterrupt):
                exit(0)
            except Exception as e:
                print("ADB Error: ", e)
                dump_trace()

    def has_breakpoint_here(self):
        """
        Check if the user has set a breakpoint at the current line
        """
        return self.lineinfo[1] in self.breakpoints and self.lineinfo[0] == self.filename

    def set_top_level(self, tl):
        """
        Set our flag that tells the debugger if it's at the top level of a program or not
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

    def print_current_line(self):
        """
        Print the current line nicely
        """
        prog_line = self.program_text[self.lineinfo[1] - 1][:-1]
        outline =  ("{" + self.lineinfo[0] + " " + str(self.lineinfo[1]) + "}")
        outline += ("\n -->> " + prog_line)

        print(outline)

    def list_program(self):
        """
        List the program contents
        """
        self.message("Program Listing")
        start_of_line = "  "

        for ix, l in enumerate(self.program_text):
            if ix+1 in self.breakpoints:
                start_of_line = "* "
            if self.lineinfo[1] == ix+1:
                start_of_line = "> "

            print(start_of_line, ix+1, l[:-1])
            start_of_line = "  "

    def tick(self):
        """
        "Tick" the debugger. This refers to hitting some point where the user
        has decided they would like the debugger to come back to life and entering
        the command selection phase.
        """
        # Print the current line with lineinfo
        self.print_current_line()

        # Main command loop
        exit_loop = False
        while not exit_loop:
            # Get our input and split it TODO: Make this a parser with error messaging
            cmd = input("(ADB) ")
            cmd = cmd.split(" ")

            # Match our command
            match(cmd[0]):
                # Step
                case "s":
                    self.is_stepping = True
                    self.is_continuing = False
                    self.is_next = False
                    exit_loop = True

                # Continue
                case "c":
                    self.is_stepping = False
                    self.is_continuing = True
                    self.is_next = False
                    exit_loop = True

                # Next
                case "n":
                    self.is_stepping = False
                    self.is_continuing = False
                    self.is_next = True
                    exit_loop = True

                # Set a breakpoint
                case "break":
                    break_line = cmd[1:]
                    for b in break_line:
                        self.breakpoints.append(int(b))
                
                # Remove a breakpoint
                case "unbreak":
                    break_line = cmd[1:]
                    for b in break_line:
                        if int(b) in self.breakpoints:
                            self.breakpoints.remove(int(b))

                # REPL
                case "!":
                    old_lineinfo = self.lineinfo
                    repl(new=False)
                    self.set_lineinfo(old_lineinfo)

                # List the program
                case "l":
                    self.list_program()

                case _:
                    print("Unknown command: {}".format(cmd[0]))

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

        TODO: Consider refactoring
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
    db.run("/home/oliver/082.ast")