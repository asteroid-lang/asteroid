###########################################################################################
# Asteroid interpreter shell
#
# (c) University of Rhode Island
###########################################################################################

from asteroid.interp import interp, load_prologue
from asteroid.version import VERSION
from asteroid.state import state
from asteroid.globals import ExpectationError
from asteroid.walk import function_return_value
from asteroid.support import term2string

from sys import stdin
import platform

if platform.system() == 'Windows':
    import pyreadline3
else:
    import readline

def repl(new=True, redundancy=False, prologue=False, functional_mode=False):

    if new:
        state.initialize()
        load_prologue()
        print_repl_menu()
    try:
        run_repl(redundancy, prologue, functional_mode)
    except EOFError:
        print()
        pass

def print_repl_menu():
    print("Asteroid Version", VERSION)
    print("(c) University of Rhode Island")
    print("Type \"asteroid -h\" for help")
    if platform.system() == 'Windows':
        print("Press CTRL-Z + Return to exit")
    else:
        print("Press CTRL-D to exit")

def run_repl(redundancy, prologue, functional_mode):

    # The two different prompt types either > for a new statement
    # or . for continuing one
    # lhh: changed the prompt since replit.com uses the > as their console prompt
    arrow_prompt, continue_prompt = ("ast> ", ".... ")
    current_prompt = arrow_prompt

    # Our line to be interpreted
    line = ""
    while True:
        """
        Line input, breaking, and exiting
        """
        try:
            # Get the new input and append it to the previous line (Possibly empty)
            # with a newline in between

            # If the line is empty, just set the line
            if line == "":
                line = input(current_prompt)

            # Otherwhise append a new line
            else:
                line += "\n" + input(current_prompt)

        except KeyboardInterrupt:
            line = ""
            current_prompt = arrow_prompt
            print()
            continue

        except EOFError:
            print()
            break

        """
        Interpretation, multiline input, and exception handling
        """
        try:
            # Try to interpret the new statement
            interp(line,
                   initialize_state=False,
                   redundancy=redundancy,
                   prologue=prologue,
                   functional_mode=functional_mode,
                   exceptions=True)

            # Try to
            line = ""

            # Check for return value
            if function_return_value[-1]:
                # Get the last return value (type, value)
                retval = function_return_value[-1]

                # If it isn't none, print out the value
                if retval[1] != None:
                    print(term2string(function_return_value[-1]))

                    # Reset the return value
                    function_return_value[0] = None

        except ExpectationError as e:
            # If we expected something but found EOF, it's a continue
            if e.found_EOF:
                current_prompt = continue_prompt
            else:
                print("error: "+str(e))
                line = ""
                current_prompt = arrow_prompt

        except Exception as e:
            # FIX THIS
            print("error: "+str(e))
            line = ""
            current_prompt = arrow_prompt
        else:
            current_prompt = arrow_prompt
