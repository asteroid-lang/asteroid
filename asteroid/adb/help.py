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
    
    'continue':     """[[c]ont]inue
    Continue to the next breakpoint
    """,
    
    'next':         """[n]ext
    Go to the next top level statement
    """,
    
    'break':        """[b]reak ((linenum) ?(command))*
    Set a breakpoint at linenum. If you supply a command, the breakpoint
    becomes conditional
    """,

    'command' :     """`command`
    Execute asteroid code
    Example:
        `let x = 10. io@println("10").`

    -- Print out the contents of variable x
        `x`
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
    
    'quit':         """[q]uit
    Exit ADB
    """,
    
    'explicit':     """[e]xplicit
    Enable explicit mode
    """,
    
    'unexplicit':   """[u]nexplicit
    Disable explicit mode
    """,
    
    'help':         """[h]elp ?(name)
    Get help for a command
    """
}
