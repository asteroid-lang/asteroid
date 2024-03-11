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
from lex import get_indentifiers_by_prefix
from sys import stdin,exit

import readline

def repl(new=True, redundancy=False, prologue=False, functional_mode=False):

    if new:
        state.initialize()
        if prologue:
            load_prologue()
        print_repl_menu()
    try:
        run_repl(redundancy, functional_mode)
    except EOFError:
        print()
        pass

def print_repl_menu():
    print("Asteroid", VERSION)
    print("(c) University of Rhode Island")
    print("Type \"help\" for additional information")

#This is the function that will be passed into readline to
#allow for autocompletion. Text is the token currently highlighted
#by the user, while state is the number of times the function was called before
#a valid completion was returned. The function will be called again if a string is
#returned, and will stop being called if None is returned.
def completion_func(text, state):
    #Get the valid autocompletions
    autocompletions = get_indentifiers_by_prefix(text)

    #If nothing has been initialized yet, we need to interpret
    #something to initialize the asteroid state.
    #We interpret nothing so that it has no impact on the lines
    #interpreted later.
    if autocompletions == []:
        interp("",
                   initialize_state=False,
                   redundancy=False,
                   prologue=False, # prologue is managed by repl above
                   functional_mode=False,
                   exceptions=True)
    
    if state < len(autocompletions):
        autocompletion = autocompletions[state]
        if autocompletion.startswith(text):
            return autocompletion
    else:
        return None

def run_repl(redundancy, functional_mode):
    #Setup the autocompleter
    readline.parse_and_bind("tab: complete")
    readline.set_completer(completion_func)
    
    # The two different prompt types either > for a new statement
    # or . for continuing one
    # lhh: changed the prompt since replit.com uses the > as their console prompt
    arrow_prompt, continue_prompt = ("ast> ", ".... ")
    current_prompt = arrow_prompt

    # Our line to be interpreted
    line = ""
    while True:
        ### Line input, breaking, and exiting
        try:
            # Get the new input and append it to the previous line (Possibly empty)
            # with a newline in between

            # If the line is empty, just set the line
            if line == "":
                line = input(current_prompt)

            # Otherwhise append a new line
            else:
                line += "\n" + input(current_prompt)

            # see if we are looking at native repl commands
            if line in ["help","quit"]:
                match line:
                    case "help":
                        print()
                        print("help          -- this message")
                        print("quit          -- leave the Asteroid interpreter")
                        print("load \"<file>\" --  load and run the program in <file>")
                        print()
                        print("or type any valid Asteroid statement at the prompt")
                        print()
                        line = ""
                    case "quit":
                        exit(0)
                continue

        except KeyboardInterrupt:
            line = ""
            current_prompt = arrow_prompt
            print()
            continue

        except EOFError:
            print()
            break

        ### Interpretation, multiline input, and exception handling
        try:
            # Try to interpret the new statement
            interp(line,
                   initialize_state=False,
                   redundancy=redundancy,
                   prologue=False, # prologue is managed by repl above
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

if __name__ == "__main__":
    run_repl(False, False)
