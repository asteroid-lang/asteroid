from asteroid.interp import interp
from asteroid.version import VERSION
from asteroid.state import state
from asteroid.globals import ExpectationError

from sys import stdin
import readline

def repl():
    state.initialize()

    print_repl_menu()
    try:
        run_repl()
    except EOFError:
        print()
        pass

def print_repl_menu():
    print("Asteroid Version", VERSION)
    print("Press CTRL+D to exit")


def run_repl():
    # The two different prompt types either > for a new statement
    # or . for continuing one
    arrow_prompt, continue_prompt = ("> ", ". ")
    current_prompt = arrow_prompt

    # Our line to be interpreted
    line = ""
    while True:
        try:
            # Get the new input and append it to the previous line (Possibly empty)
            # with a newline in between
            line += "\n" + input(current_prompt)

        except KeyboardInterrupt:
            line = ""
            current_prompt = arrow_prompt
            print()
            continue

        except EOFError:
            print()
            break

        try:
            # Try to interpret the new statement
            interp(line, initialize_state=False, prologue=False, exceptions=True)

            # Try to 
            line = ""

        except ExpectationError as e:
            # If we expected something but found EOF, it's a continue
            if e.found_EOF:
                current_prompt = continue_prompt
            else:
                print(e)
                line = ""

        except Exception as e:
            # FIX THIS
            print(e)
            line = ""
            current_prompt = arrow_prompt

        else:
            current_prompt = arrow_prompt
