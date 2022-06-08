"""
case "s":
case "c":
case "n":
case "break":
case "unbreak"
case "!":
case "l":
case "quit":
case "explicit":
case "unexplicit":
case "clear":
case _:


line ::= command {";" command}
command ::=  "s" | "step"
        | "c" | "continue"
        | "n" | "next"
        | "l"  | "list"
        | "ll" | "longlist"
        | "q"  | "quit"
        | "e"  | "explicit"
        | "u"  | "unexplicit"
        | "cl" | "clear"
        | "!" [asteroid_exp]
        | "b" number | "break" number
        | "u" number | "unbreak" number
        | "macro" name "(" command {";" command} ")"
"""