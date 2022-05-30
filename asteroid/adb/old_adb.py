# """
# TODO: breakpoints need to be module specific
# """
# class AdbSessionExit(Exception):
#     pass

# class Exit(Exception):
#     pass

# # Continuation Signal    
# class Cont(Exception):
#     pass

# class UnknownCommand(Exception):
#     pass

# class Keep_Asking(Exception):
#     pass

# class Adb:
#     def __init__(self):
#         self.lineinfo = None
#         self.program_text = None
#         self.breakpoints = []
#         self.at_top_level = False
#         self.call_levels = []
#         self.next = True

#         # When we have a session this will need to exist
#         self.exit = False

#     def set_top_level(self, tl):
#         self.at_top_level = tl

#     def is_at_top_level(self):
#         return self.at_top_level

#     def set_lineinfo(self, lineinfo):
#         self.lineinfo = lineinfo

#         # TODO: FIX LATER THIS DOESNT WORK FOR MODULES
#         try:
#             f = open(self.lineinfo[0])
#             self.program_text = f.readlines()
#             f.close()
#         except:
#             pass

#     def message(self, message):
#         print("----- " + message + " -----")

#     def format_call_levels(self):
#         if self.call_levels:
#             outstr = ""
#             for cl in self.call_levels:
#                 outstr += cl + "->"

#             return "{" + outstr[:-2] + "}"
#         else:
#             return ""

#     def format_display_line(self):
#         mod_name = self.lineinfo[0].split("/")[-1]

#         outstr = ""
#         if self.has_break_here():
#             outstr += "BREAK:\n\t"
#         outstr += "(" + mod_name + "," + str(self.lineinfo[1]) + ")"
#         outstr += self.format_call_levels()
#         outstr += " >> "
#         outstr += self.program_text[self.lineinfo[1] - 1][:-1].strip()

#         return outstr

#     def display_line(self):
#         print(self.format_display_line())

#     def has_break_here(self):
#         return self.lineinfo[1] in self.breakpoints

#     def print_program(self):
#         for ix, line in enumerate(self.program_text):
#             padding = " "
#             if ix + 1 == self.lineinfo[1]:
#                 padding = ">"
#             elif ix + 1 in self.breakpoints:
#                 padding = "*"

#             print(padding, ix+1, line[:-1])
#         print()

#     def parse_command(self, cmd):
#         cmd = cmd.split(" ")
#         match(cmd[0]):
#             case "n":
#                 self.next = True
#                 raise Cont()
#             case "c":
#                 self.next = False
#                 raise Cont()
#             case "break":
#                 self.breakpoints.append(int(cmd[1]))
#                 raise Keep_Asking()
#             case "l":
#                 self.print_program()
#             case "!":
#                 # TODO: PERSIST LINENUMBER AND STUFF FROM BEFORE REPL
#                 from asteroid.repl import repl
                
#                 old_lineinfo = self.lineinfo
#                 repl(new=False)
#                 self.lineinfo = old_lineinfo
#                 raise Keep_Asking()
#             case _:
#                 raise UnknownCommand()

#     def ask_command(self):
#         self.parse_command(input("(ADB) "))

#     def tick(self):
#         self.next = False
#         self.display_line()
#         asking = True
#         while asking:
#             try:
#                 self.ask_command()
#             except UnknownCommand:
#                 print("Unknown command")
#             except Keep_Asking:
#                 pass
#             except Exit:
#                 self.exit = True
#             except Cont:
#                 asking = False
#             except KeyboardInterrupt as e:
#                 raise e
#             except EOFError as e:
#                 raise e

#     def run(self, filename):
#         from asteroid.interp import interp

#         f = open(filename, 'r')
#         input_stream = f.read()
#         f.close()

#         while not self.exit:
#             dbg = Adb()

#             try:
#                 interp(input_stream,
#                     input_name = filename,
#                     do_walk=True,
#                     prologue=False,
#                     exceptions=True,
#                     debugger=dbg)

#                 self.message("End of file reached, restarting session")
#             except (EOFError, KeyboardInterrupt):
#                 exit(0)
#             except Exception as e:
#                 from asteroid.state import dump_trace
#                 print("ADB Error: ", e)
#                 dump_trace()
#                 self.message('Error reached, restarting session')

# """
# General scheme could be like individual lines or something

# Like

# debugger session starts:
#     ask user for input
#     act on input
#     keep acting on input until "next" or some other line movement thing is called

#     How will i keep track of next vs breakpoints?
# """

# db = Adb()
# db.run("/home/oliver/othertest.ast")



# def notify_debugger(message = None):
#     """
#     The debugger will have some internal logic to either print
#     this message if it's just doing next or just skip to a breakpoint
#     """
#     global run_on_line
#     old_lineinfo = state.lineinfo

#     if debugging:
#         debugger.set_lineinfo(state.lineinfo)

#     if debugging and debugger.next and debugger.is_at_top_level():
#         if message:
#             debugger.message(message)
#         debugger.tick()

#     elif debugging and debugger.has_break_here():
#         if message:
#             debugger.message(message)
#         debugger.tick()

#     if debugging:
#         debugger.set_top_level(False)
#         state.lineinfo = old_lineinfo
#         debugger.set_lineinfo(state.lineinfo)     