command_description_table = {
    'macro':        """macro (name) = (command list)
    Define a macro with name "name" and a list of commands.

    Typing solely "macro" will list off the macros currently enabled
    Example:
        macro see_x = continue; explicit; n; p x;
    """,
    
    'step':         """[s]tep
    Step to the next executing line
    """,

    'until':        """[u]ntil ?lineno
    Without a given line number, continue execution until a line that is greater than the current
    one is reached.

    With a line number, continue execution until a line that is greater than or equal to that one
    is reached.

    In both cases, stop when the current frame returns.
    """,
    
    'continue':     """[[c]ont]inue
    Continue to the next breakpoint
    """,
    
    'next':         """[n]ext
    Go to the next top level statement
    """,
    
    'break':        """[b]reak (linenum (if eval("command")))
    Set a breakpoint at linenum. If you supply a command, the breakpoint
    becomes conditional
    """,

    'eval' :     """eval("command")
    Execute asteroid code
    Example:
        eval("let x = 10. io@println(\\"10\\")")

    -- Print out the contents of variable x
        eval("x")
    """,
    
    'delete':       """[[d]el]ete (linenum)
    Delete the breakpoint at linenum
    """,
    
    '!':            """!
    Start a REPL session
    """,
    
    'longlist':     """[ll] longlist
    List the contents of the entire program
    """,
    
    'list':         """[l]ist
    List the contents of the program around the current line
    """,

    'retval':       """[[r]et]val
    Print the most recent return value
    """,
    
    '>':            """>
    Move down one frame
    """,

    '<':            """<
    Move up one frame
    """,

    'where':        """[w]here
    Lists the currently available stack frames and shows the user
    the currently active one.
    """,

    'quit':         """[q]uit
    Exit ADB
    """,
    
    'explicit':     """[e]xplicit (on|off)?
    Toggle explicit mode or set explicit mode to a specific state.

    For more information about explicit mode, refer to "ADB in action".

    Example:
        explicit    -- Toggles explicit mode
        explicit on -- Turns explicit mode on
        explcit off -- Turns explicit mode off
    """,
    
    'help':         """[h]elp ?(name)
    Get help for a command
    """
}
