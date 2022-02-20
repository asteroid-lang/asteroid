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
    arrow_prompt, continue_prompt = ("> ", ". ")
    current_prompt = arrow_prompt

    line = ""
    while True:
        line += " " + input(current_prompt)

#        print("#######DEBUG#########")
#        print(line)
#        print("#####################")
        try:
            interp(line, initialize_state=False, prologue=False, exceptions=True)
            line = ""

        except ExpectationError as e:
            # If we expected something but found EOF, it's a continue
            if e.found_EOF:
                current_prompt = continue_prompt
            else:
                print(e)
                line = ""

        except EOFError:
            break

        except Exception as e:
            # FIX THIS
            print(e)
            line = ""
        else:
            if current_prompt == continue_prompt:
                current_prompt = arrow_prompt
