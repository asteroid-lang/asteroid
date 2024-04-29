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
from asteroid.lex import get_indentifiers, get_member_identifiers
from sys import stdin,exit
import readline
import re

#Variables needed for autocomplete cycling
prefix = ""
last_completion = ""
last_index = 0

#Change this flag to False to disable autocompletion
state.repl_use_autocompletion = True

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
#returned, and will stop being called if a non string object is returned.
def completion_function(text, state):
    #Here we "hijack" the iteration with readline's calling of the function by
    #iterating on our own only on the first call, returning None on
    #all other calls of the function
    if state == 0:
        #identifiers is the default pool of completions
        identifiers = get_indentifiers()
        
        #If nothing has been initialized yet, we need to interpret
        #something to initialize the asteroid state.
        #We interpret nothing so that it has no impact on the lines
        #interpreted later.
        if identifiers == []:
            interp("",
                    initialize_state=False,
                    redundancy=False,
                    prologue=True,
                    functional_mode=False,
                    exceptions=True)
            identifiers = get_indentifiers()
        
        #Here we start checking if we are doing a completion for accessing a member
        user_in = readline.get_line_buffer()
        #The index of the beginning of the substring that the user has highlighted
        i = readline.get_begidx()
        
        get_members = False
        #Check if there's an @ before or at the autocompletion index
        if i < len(user_in) and user_in[i] == '@':
            get_members = True
        else:
            i -= 1
            while i > 0:
                if user_in[i] == ' ':
                    pass
                elif user_in[i] == '@':
                    get_members = True
                    i -= 1
                    break
                else:
                    break
                i -= 1
            
        #If there was a preceding @, try to use members as the completion pool
        if get_members:
            parent_id = ""
            
            #Find where the parent begins
            while i >= 0 and user_in[i] == " ":
                i -= 1
            
            #Get the parent
            while i >= 0:
                if re.match(r'[a-zA-Z_0-9]',user_in[i]):
                    parent_id = user_in[i] + parent_id
                else:
                    break
                i -= 1

            #Only try to set identifiers to member_list if the
            #parent identifier is valid
            if len(parent_id) > 0 and re.match(r'[a-zA-Z_]',parent_id[0]):
                member_list = get_member_identifiers(parent_id)
                #Make sure the parent is actually a parent
                if member_list:
                    identifiers = list(member_list)
        
        global last_completion
        global last_index
        global prefix
        
        start = 0 #Start at the first identifier
        #This flag allows for the autocompleter to start over
        #if no autcompletions are available
        allow_cycle = False
        
        if text == "": #This otherwise causes weird behavior
            start = 0
            last_index = 0
            prefix = ""
            last_completion = ""
        elif text == last_completion: #Check if this is a repeat tab hit
            allow_cycle = True
            start = last_index + 1 #Jump ahead to avoid redundancy
            if start == len(identifiers):
                start = 0
            text = prefix
        else:
            prefix = text #Set a new prefix for future cycles
        
        i = start
        while i < len(identifiers):
            token = identifiers[i]
            if token.startswith(text) and token != last_completion:
                last_index = i
                last_completion = token
                return token
            i += 1
            
            if i == len(identifiers) and allow_cycle:
                allow_cycle = False
                i = 0
    else:
        return None

def run_repl(redundancy, functional_mode):
    #Setup the autocompleter
    if state.repl_use_autocompletion:
        readline.parse_and_bind("Tab: complete")
        readline.set_completer(completion_function)
    
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
    repl(True, redundancy=False, prologue=True, functional_mode=False)
