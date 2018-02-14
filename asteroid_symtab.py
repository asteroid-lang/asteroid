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

    #-------
    def dump(self):
        print("Symbol Table Dump:")
        pprint(self.scoped_symtab)

    #-------
    def get_config(self):
        # we make a shallow copy of the symbol table
        return list(self.scoped_symtab)
    
    #-------
    def set_config(self, c):
        self.scoped_symtab = c
        
    #-------
    def push_scope(self):
        # push a new dictionary onto the stack - stack grows to the left
        self.scoped_symtab.insert(CURR_SCOPE,{})

    #-------
    def pop_scope(self):
        # pop the left most dictionary off the stack
        if len(self.scoped_symtab) == 1:
            raise ValueError("cannot pop the global scope")
        else:
            self.scoped_symtab.pop(CURR_SCOPE)

    #-------
    def enter_sym(self, sym, value):
        # enter the symbol in the current scope
        # we enter the value as a list because if sym is a constructor
        # we can attach additional functions
        scope_dict = self.scoped_symtab[CURR_SCOPE]
        scope_dict[sym] = [value]

    #-------
    def lookup_sym(self, sym, strict=True):
        # find the first occurence of sym in the symtab stack
        # and return the associated value

        n_scopes = len(self.scoped_symtab)

        for scope in range(n_scopes):
            if sym in self.scoped_symtab[scope]:
                val_list = self.scoped_symtab[scope].get(sym)
                return val_list[0]

        # not found
        if strict:
            raise ValueError("{} is not defined".format(sym))
        else:
            return None

    #-------
    def update_sym(self, sym, value):
        # this is for non-local symbols!
        # find the first occurence of sym in the symtab stack
        # and update the associated value

        n_scopes = len(self.scoped_symtab)

        for scope in range(n_scopes):
            if sym in self.scoped_symtab[scope]:
                scope_dict = self.scoped_symtab[scope]
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

        n_scopes = len(self.scoped_symtab)

        for scope in range(n_scopes):
            if sym in self.scoped_symtab[scope]:
                scope_dict = self.scoped_symtab[scope]
                scope_dict[sym].insert(0, fvalue)
                return

        # not found
        raise ValueError("{} was not declared".format(sym))

    #-------
    def detach_from_sym(self, sym):
        # find the first occurence of sym in the symtab stack
        # and detach toplevel function

        n_scopes = len(self.scoped_symtab)

        for scope in range(n_scopes):
            if sym in self.scoped_symtab[scope]:
                scope_dict = self.scoped_symtab[scope]
                val_list = scope_dict[sym]
                if len(val_list) == 1:
                    raise ValueError("Cannot detach constructor from {}.".format(sym))
                else:
                    val_list.pop(0)
                return

        # not found
        raise ValueError("{} was not declared".format(sym))

#########################################################################


