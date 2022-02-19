from asteroid.interp import interp
from asteroid.version import VERSION
from asteroid.state import state
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
    while True:
        line = input("> ")

        try:
            interp(line, initialize_state=False, prologue=False)
        except EOFError:
            break
        except:
            pass
