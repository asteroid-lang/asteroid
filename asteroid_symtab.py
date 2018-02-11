#########################################################################
# symbol table for Asteroid
#
# it is a scoped symbol table with a dictionary at each scope level
#
# (c) 2018 - Lutz Hamel, University of Rhode Island
#########################################################################

# TODO: symbold vs function declaration needs to be cleaned up -- everything is just a value in Asteroid

CURR_SCOPE = 0

class SymTab:

    #-------
    def __init__(self):
        # global scope dictionary must always be present
        self.scoped_symtab = [{}]

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
    def declare_sym(self, sym, init):
        # declare the scalar in the current scope: dict @ position 0
        
        # first we need to check whether the symbol was already declared
        # at this scope
        if sym in self.scoped_symtab[CURR_SCOPE]:
            raise ValueError("symbol {} already declared".format(sym))
        
        # enter the symbol in the current scope
        scope_dict = self.scoped_symtab[CURR_SCOPE]
        scope_dict[sym] = ('scalar', init)

    #-------
    def declare_fun(self, sym, init):
        # declare a function in the current scope: dict @ position 0
        
        # first we need to check whether the symbol was already declared
        # at this scope
        if sym in self.scoped_symtab[CURR_SCOPE]:
            raise ValueError("symbol {} already declared".format(sym))
        
        # enter the function in the current scope
        scope_dict = self.scoped_symtab[CURR_SCOPE]
        scope_dict[sym] = ('function', init)

    #-------
    def lookup_sym(self, sym):
        # find the first occurence of sym in the symtab stack
        # and return the associated value

        n_scopes = len(self.scoped_symtab)

        for scope in range(n_scopes):
            if sym in self.scoped_symtab[scope]:
                val = self.scoped_symtab[scope].get(sym)
                return val

        # not found
        raise ValueError("{} was not declared".format(sym))

    #-------
    def update_sym(self, sym, val):
        # find the first occurence of sym in the symtab stack
        # and update the associated value

        n_scopes = len(self.scoped_symtab)

        for scope in range(n_scopes):
            if sym in self.scoped_symtab[scope]:
                scope_dict = self.scoped_symtab[scope]
                scope_dict[sym] = val
                return

        # not found
        raise ValueError("{} was not declared".format(sym))

#########################################################################


