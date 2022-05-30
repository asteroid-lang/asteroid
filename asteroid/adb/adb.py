"""
The Asteroid Debugger
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
            next:       Go to next top level line (i.e. top level statement)
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
        
        try:
            interp(input_stream,
                input_name = filename,
                do_walk=True,
                prologue=False,
                exceptions=True,
                debugger=self)
            self.message("End of file reached, restarting session")
        
        except (EOFError, KeyboardInterrupt):
            exit(0)
        except Exception as e:
            print("ADB Error: ", e)
            dump_trace()

    def has_breakpoint_here(self):
        return self.lineinfo[1] in self.breakpoints and self.lineinfo[0] == self.filename

    def set_top_level(self, tl):
        self.top_level = tl

    def set_lineinfo(self, lineinfo):
        self.lineinfo = lineinfo

        if self.program_text is None:
            with open(lineinfo[0], "r") as f:
                self.program_text = f.readlines()

    def format_current_line(self):
        prog_line = self.program_text[self.lineinfo[1] - 1][:-1]
        outline =  ("{" + self.lineinfo[0] + " " + str(self.lineinfo[1]) + "}")
        outline += ("\n -->> " + prog_line)

        return outline

    def list_program(self):
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
        # Print the current line with lineinfo
        print(self.format_current_line())

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

                case "c":
                    self.is_stepping = False
                    self.is_continuing = True
                    self.is_next = False
                    exit_loop = True

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
        # If we're not on the intended file, just return
        if self.lineinfo[0] != self.filename:
            pass

        elif self.has_breakpoint_here():
            self.message("Breakpoint")
            self.tick()

        # If we're at the top level and we're not continuing
        # to the next breakpoint, and we're going to the next statement
        # do a tick
        elif self.top_level and self.is_next and not self.is_continuing:
            self.tick()

        # Otherwhise, if we're stepping through the program,
        # always tick
        elif self.is_stepping:
            self.tick()
        
        # Reset the top level so that nested statements don't come in
        self.set_top_level(False)

if __name__ == "__main__":
    db = ADB()
    #db.run("/home/oliver/082.ast")