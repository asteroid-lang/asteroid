"""
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

# token_specs = [
#     (r'[a-zA-Z_][a-zA-Z_0-9]*', "name", 'ID'),
# ]

# keywords = [
#         "c",    "continue",
#         "n",    "next",
#         "l",    "list",
#         "ll",   "longlist",
#         "q",    "quit",
#         "e",    "explicit",
#         "u",    "unexplicit",
#         "cl",   "clear"
#         "!"
# ]


# def tokenize(code):
#     # output token list
#     tokens = []

#     named_re_list = ['(?P<{}>{})'.format(type,re) for (re,_,type) in token_specs]

#     combined_re = '|'.join(named_re_list)

#     match_object_list = list(re.finditer(combined_re, code))
#     for mo in match_object_list:
#         # get the token type and value from
#         # the match object
#         type = mo.lastgroup
#         value = mo.group()
#         # some special processing of tokens
#         if type == 'NUMBER':
#                 pass
#         elif type == 'ID':
#             type = keywords.get(value,'ID')
#         elif type == 'MISMATCH':
#             if value == '\"':
#                 raise ExpectationError(expected='\"', found='EOF')
#             else:
#                 raise ValueError("unexpected character '{}'".format(value))
#     # always append an EOF token so we never run out of tokens
#     # in the lexer.
#     tokens.append(Token('EOF', '', module, line_num))
#     return tokens