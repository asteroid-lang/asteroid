#########################################################################
# symbol table for Asteroid
#
# it is a scoped symbol table with a dictionary at each scope level
# each symbols is entered with a list of values in order to enable 'attach'
#
# (c) Lutz Hamel, University of Rhode Island
#########################################################################

from pprint import pprint

#########################################################################

CURR_SCOPE = 0

class SymTab:

    def __init__(self):
        # global scope dictionary must always be present
        self.scoped_symtab = [{}]
        self.globals = [[]]
        self.global_scope = self.scoped_symtab[0] # reference to global dictionary

    def dump(self):
        print("*** Symbol Table Dump:")
        pprint(self.scoped_symtab)
        print("*** Globals Table:")
        pprint(self.globals)
        print("*** Global Scope:")
        pprint(self.global_scope)

    def get_config(self):
        # we make a shallow copy of the symbol table, globals...
        return (list(self.scoped_symtab), list(self.globals), self.global_scope)

    def set_config(self, c):
        self.scoped_symtab, self.globals, self.global_scope = c

    def push_scope(self, scope):
        # push a new dictionary and globals lookup onto the stacks - stacks grow to the left
        self.scoped_symtab.insert(CURR_SCOPE, scope)
        self.globals.insert(CURR_SCOPE,[])

    def pop_scope(self):
        # pop the left most dictionary off the stack
        if len(self.scoped_symtab) == 1:
            raise ValueError("cannot pop the global scope")
        else:
            scope = self.scoped_symtab.pop(CURR_SCOPE)
            self.globals.pop(CURR_SCOPE)
            return scope

    def enter_sym(self, sym, value):
        # enter the symbol in the appropriate scope
        # we enter the value as a list because if sym is a constructor
        # we can attach additional functions
        if sym in self.globals[CURR_SCOPE]:
            scope_dict = self.global_scope
        else:
            scope_dict = self.scoped_symtab[CURR_SCOPE]

        scope_dict[sym] = [value]

    def enter_global(self, sym):
        # enter the symbol in the global table at the appropriate scope
        self.globals[CURR_SCOPE].append(sym)

    def lookup_sym(self, sym, strict=True):
        dict = self.find_sym(sym)
        if not dict:
            if strict:
                raise ValueError("'{}' is not defined".format(sym))
            else:
                return None
        else:
            val_list = dict[sym]
            return val_list[0]

    def update_sym(self, sym, value):
        # find the first occurence of sym
        # and update the associated value

        dict = self.find_sym(sym)
        if not dict:
            raise ValueError("'{}' is not defined".format(sym))
        else:
            dict[sym] = [value]
            return

    def attach_to_sym(self, sym, fvalue):
        # find the first occurence of sym in the symtab stack
        # and attach new function value

        if fvalue[0] != 'function':
            ValueError("Attach for '{}' needs a function value."
                       .format(sym))

        dict = self.find_sym(sym)
        if not dict:
            raise ValueError("'{}' is not defined".format(sym))
        else:
            dict[sym].insert(0, fvalue)
            return

    def detach_from_sym(self, sym):
        # find the first occurence of sym in the symtab stack
        # and detach toplevel function

        dict = self.find_sym(sym)
        if not dict:
            raise ValueError("'{}' is not defined".format(sym))
        else:
            val_list = dict[sym]
            if len(val_list) == 1:
                raise ValueError("Cannot detach constructor from {}.".format(sym))
            else:
                val_list.pop(0)
            return

    def is_symbol_local(self, sym):
        if sym in self.scoped_symtab[CURR_SCOPE]:
            return True
        else:
            return False

    def is_global(self, sym):
        if sym in self.globals[CURR_SCOPE]:
            return True
        else:
            return False

    def find_sym(self, sym):
        n_scopes = len(self.scoped_symtab)
        for scope in range(n_scopes):
            if sym in self.scoped_symtab[scope]:
                return self.scoped_symtab[scope]
        # not found
        return None

    def dbg_find_sym(self, sym):
        n_scopes = len(self.scoped_symtab)
        for scope in range(n_scopes):
            if sym in self.scoped_symtab[scope]:
                return ("---- found symbol {} in scope {} with value {}"
                      .format(sym, scope, self.scoped_symtab[scope].get(sym)))
        # not found
        return ("{} was not found".format(sym))



#########################################################################
