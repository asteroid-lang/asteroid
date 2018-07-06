#########################################################################
# symbol table for Asteroid
#
# it is a scoped symbol table with a dictionary at each scope level
# each symbols is entered with a list of values in order to enable 'attach'
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
#########################################################################

from pprint import pprint

#########################################################################

CURR_SCOPE = 0

class SymTab:

    #-------
    def __init__(self):
        # global scope dictionary must always be present
        self.scoped_symtab = [{}]
        self.globals = [[]]
        self.global_scope = self.scoped_symtab[0] # reference to global dictionary

    #-------
    def dump(self):
        print("*** Symbol Table Dump:")
        pprint(self.scoped_symtab)
        print("*** Globals Table:")
        pprint(self.globals)
        print("*** Global Scope:")
        pprint(self.global_scope)

    #-------
    def get_config(self):
        # we make a shallow copy of the symbol table, globals...
        return (list(self.scoped_symtab), list(self.globals), self.global_scope)
    
    #-------
    def set_config(self, c):
        self.scoped_symtab, self.globals, self.global_scope = c
    
    #-------
    def push_scope(self):
        # push a new dictionary and globals lookup onto the stacks - stacks grow to the left
        self.scoped_symtab.insert(CURR_SCOPE,{})
        self.globals.insert(CURR_SCOPE,[])

    #-------
    def pop_scope(self):
        # pop the left most dictionary off the stack
        if len(self.scoped_symtab) == 1:
            raise ValueError("cannot pop the global scope")
        else:
            self.scoped_symtab.pop(CURR_SCOPE)
            self.gobals.pop(CURR_SCOPE)

    #-------
    def enter_sym(self, sym, value):
        # enter the symbol in the appropriate scope
        # we enter the value as a list because if sym is a constructor
        # we can attach additional functions
        if sym in self.globals[CURR_SCOPE]:
            scope_dict = self.global_scope
        else:
            scope_dict = self.scoped_symtab[CURR_SCOPE]

        scope_dict[sym] = [value]

    #-------
    def enter_global(self, sym):
        # enter the symbol in the global table at the appropriate scope
        self.globals[CURR_SCOPE].append(sym)

    #-------
    def lookup_sym(self, sym, strict=True):
        # find the occurence of sym and return the associated value
        # NOTE: here we define a new lookup semantics: a symbol is either
        # a) function local, or
        # b) global

        if sym in self.scoped_symtab[CURR_SCOPE]:
            val_list = self.scoped_symtab[CURR_SCOPE].get(sym)
            return val_list[0]
        elif sym in self.global_scope:
            val_list = self.global_scope.get(sym)
            return val_list[0]

        # not found
        if strict:
            raise ValueError("{} is not defined".format(sym))
        else:
            return None

    #-------
    def update_sym(self, sym, value):
        # find the first occurence of sym
        # and update the associated value

        if sym in self.scoped_symtab[CURR_SCOPE]:
            scope_dict = self.scoped_symtab[CURR_SCOPE]
            scope_dict[sym] = [value]
            return
        elif sym in self.global_scope:
            scope_dict = self.global_scope
            scope_dict[sym] = [value]
            return

        # not found
        raise ValueError("{} was not declared".format(sym))

    #-------
    def attach_to_sym(self, sym, fvalue):
        # find the first occurence of sym in the symtab stack
        # and attach new function value

        if fvalue[0] != 'function':
            ValueError("Attach for {} needs a function value.".format(sym))

        if sym in self.scoped_symtab[CURR_SCOPE]:
            scope_dict = self.scoped_symtab[CURR_SCOPE]
            scope_dict[sym].insert(0, fvalue)
            return
        elif sym in self.global_scope:
            scope_dict = self.global_scope
            scope_dict[sym].insert(0, fvalue)
            return

        # not found
        raise ValueError("{} was not declared".format(sym))

    #-------
    def detach_from_sym(self, sym):
        # find the first occurence of sym in the symtab stack
        # and detach toplevel function

        if sym in self.scoped_symtab[CURR_SCOPE]:
            scope_dict = self.scoped_symtab[CURR_SCOPE]
            val_list = scope_dict[sym]
            if len(val_list) == 1:
                raise ValueError("Cannot detach constructor from {}.".format(sym))
            else:
                val_list.pop(0)
            return
        elif sym in self.global_scope:
            scope_dict = self.global_scope
            val_list = scope_dict[sym]
            if len(val_list) == 1:
                raise ValueError("Cannot detach constructor from {}.".format(sym))
            else:
                val_list.pop(0)
            return

        # not found
        raise ValueError("{} was not declared".format(sym))

    #-------
    def is_symbol_local(self, sym):
        if sym in self.scoped_symtab[CURR_SCOPE]:
            return True
        else:
            return False

    #-------
    def is_global(self, sym):
        if sym in self.globals[CURR_SCOPE]:
            return True
        else:
            return False


#########################################################################


