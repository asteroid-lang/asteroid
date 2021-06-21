# Reference Guide

## User Guide Language Features 

### Asteroid Grammar (written in EBNF/PEG Format)

////////////////////////////////////////////////////////////////////////////////////////

// (c) Lutz Hamel, University of Rhode Island  //

////////////////////////////////////////////////////////////////////////////////////////

#### Statements


      prog
        : stmt_list

      stmt_list
        : stmt*

      stmt
        : '.' // NOOP
        | LOAD SYSTEM? STRING '.'?
        | GLOBAL id_list '.'?
        | NONLOCAL id_list '.'?
        | ASSERT exp '.'?
        | STRUCTURE ID WITH struct_stmts END
        | LET pattern '=' exp '.'?
        | LOOP DO? stmt_list END
        | FOR pattern IN exp DO stmt_list END
        | WHILE exp DO stmt_list END
        | REPEAT DO? stmt_list UNTIL exp '.'?
        | IF exp DO stmt_list (ELIF exp DO stmt_list)* (ELSE DO? stmt_list)? END
        | MATCH exp WITH? (CASE pattern DO stmt_list)+ (OTHERWISE DO? stmt_list)? END
        | TRY DO? stmt_list (CATCH pattern DO stmt_list)+ END
        | THROW exp '.'?
        | BREAK '.'?
        | RETURN exp? '.'?
        | function_def
        | call_or_index '.'?  

#### Grammar Snippet of Control Statements in terms of the Non-Terminal `stmt`


     stmt := FOR pattern IN exp DO stmt_list END
       | WHILE exp DO stmt_list END
       | REPEAT DO? stmt_list UNTIL exp '.'?
       | IF exp DO stmt_list (ELIF exp DO stmt_list)* (ELSE DO? stmt_list)? END
       | TRY stmt_list (CATCH pattern DO stmt_list)+ END
       | THROW exp '.'?
       | BREAK '.'?

#### More General Statements


     function_def
       : FUNCTION ID body_defs END


     body_defs
       : WITH pattern DO stmt_list (ORWITH pattern DO stmt_list)*


     data_stmt
       : DATA ID


     struct_stmt
       : data_stmt  '.'?
       | function_def '.'?
       | '.'

     struct_stmts
       : struct_stmt*
       
      id_list
        : ID (',' ID)*
        
#### Expressions/Patterns

NOTE: There is no syntactic difference between a pattern and an expression. We introduce the `pattern` nonterminal to highlight the SEMANTIC difference between patterns and expressions. 

      pattern
         : exp
         
      exp
        : quote_exp
        
      quote_exp
        : QUOTE head_tail
        | PATTERN WITH? head_tail
        | head_tail
        
      head_tail
        : conditional ('|' exp)?
        
        
     compound
       : logic_exp0
           (
              (IS pattern) |
              (IN exp) |               // exp has to be a list
              (TO exp (STEP exp)?) |   // list comprehension
           )?


      logic_exp0
        : logic_exp1 (OR logic_exp1)*

     logic_exp1
        : rel_exp0 (AND rel_exp0)*

     rel_exp0
       : rel_exp1 (('==' | '=/=' ) rel_exp1)*

     rel_exp1
       : arith_exp0 (('<=' | '<'  | '>=' | '>') arith_exp0)*

     arith_exp0
       : arith_exp1 (('+' | '-') arith_exp1)*

     arith_exp1
       : call_or_index (('*' | '/') call_or_index)*

     call_or_index
       : primary (primary | '@' primary)*

     primary
       : INTEGER
       | REAL
       | STRING
       | TRUE
       | FALSE
       | NONE
       | ID (':' pattern)?  // named pattern when ': pattern' exists
       | '*' ID         // "dereference" a variable during pattern matching
       | NOT call_or_index
       | MINUS call_or_index
       | ESCAPE STRING
       | EVAL primary
       | '(' tuple_stuff ')' // tuple/parenthesized expr
       | '[' list_stuff ']'  // list or list access
       | function_const
       | TYPEMATCH           // TYPEMATCH == '%'<typename>

     tuple_stuff
       : exp (',' exp?)*
       | empty

     list_stuff
       : exp (',' exp)*
       | empty

     function_const
       : LAMBDA body_defs
       
## Asteroid Built-ins

A **built-in function,** also called an **intrinsic function,** can complete a given task directly within a language. Asteroid includes built-ins such as `lists` and `strings.` Both `lists` and `strings,` when instantiated, are treated like objects. Due to this, they have member functions that can manipulate the contents of those objects.

**The following member functions support both lists and strings,**

* Function `len`, when given an input value (or `item_val[0]`), returns the output of whether or not that input can be found in a given list, string, or tuple.
* Function `inherit` lets users contruct an inheritance hierarchy by directly manipulating the structure types. For example, it can change an inputted string into a list.
* Function `__list_extend__` will extend a list by adding all the items from the item where `item` is either a list, a string, or a tuple. The function can be called with the input `(self:%list,item)`, granted that `item_val[0]` is found in a list, string, or a tuple.
* Function `__list_join__` converts an Asteroid list into a Python list. The function can be called with the input `(self:%list,join:%string)`.
* Function `__string_split__`, given the input `(self:%string,sep:%string,count:%integer)`, will return a list of the words in a given string, using `sep` as the delimiter string.

As mentioned in [this section of the User Guide](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/Asteroid%20User%20Guide.md#the-basics), a **list** is a structured data type that consists of square brackets enclosing comma-separated values. Lists can be modified after their creation.

**The following member functions only support lists,**

* Function `islist`, given the input item `do`, returns the item `is %list`.
* The following `list_member_functions`,

      escape
      "
      global list_member_functions

      list_member_functions.update({
          'length'    : '__list_length__',
          'append'    : '__list_append__',
          'extend'    : '__list_extend__',
          'insert'    : '__list_insert__',
          'remove'    : '__list_remove__',
          'pop'       : '__list_pop__',
          'clear'     : '__list_clear__',
          'index'     : '__list_index__',
          'count'     : '__list_count__',
          'sort'      : '__list_sort__',
          'reverse'   : '__list_reverse__',
          'copy'      : '__list_copy__',
          'shuffle'   : '__list_shuffle__',
          'map'       : '__list_map__',
          'reduce'    : '__list_reduce__',
          'filter'    : '__list_filter__',
          'member'    : '__list_member__',
          'join'      : '__list_join__',
          })
      ".

(For implementation details, see Python lists [here](https://docs.python.org/3/tutorial/datastructures.html).)

* Function `__list_length__`, given the input `self:%list`, returns the number of characters within that list.
* Function `__list_append__`, given `(self:%list,item)`, adds an item to the end of a list.
* Function `__list_extend__`, given `(self:%list,item)`, will extend the list by adding all the items from the item where `item` is either a list, a string or a tuple.
* Function `__list_insert__`, given `(self:%list,ix:%integer,item)`, will insert an item at a given position. The first argument is the index of the element before which to insert, so `a@insert(0, x)` inserts at the front of the list, and `a@insert(a@length(), x)` is equivalent to `a@append(x)`.
* Function `__list_remove__`, given `(self:%list,item)`, removes the first element from the list whose value is equal to `item.` It raises a ValueError if there is no such item.
* Function `__list_pop__`, given `(self:%list,ix:%integer)`, removes the item at the given position in the list and returns it. If no index is specified,`a@pop()` removes and returns the last item in the list.
* Function `__list_clear__`, given `self:%list`, removes all items from the list.
* Function `__list_index__`, given `(self:%list,item,startix:%integer,endix:%integer)`, returns a zero-based index in the list of the first element whose value is equal to `item`. Raises a ValueError exception if there is no such item. The optional arguments `startix` and `endix` are interpreted as in the slice notation and are used to limit the search to a particular subsequence of the list. The returned index is computed relative to the beginning of the full sequence rather than the `startix` argument.
* Function `__list_count__`, given `(self:%list,item)`, returns the number of times `item` appears in the list.
* Function `__list_sort__`, given `(self:%list,reverse:%boolean)`, sorts the items of the list in place.
* Function `__list_reverse__`, given `self:%list`, reverses the elements of the list in place.
* Function `__list_copy__`, given `self:%list`, makes a shallow copy of the list.
* Function `__list_shuffle__`, given `self:%list`, returns a random permutation of a given list - in place!
* Function `__list_map__`, given `(self:%list,f:%function)`, applies `f` to each element of the list.
* Function `__list_reduce__` can be called with two different inputs: `(self:%list,f:%function)` or `(self:%list,f:%function,init)`.
* Function `__list_filter__`, given `(self:%list,f:%function)`, constructs an output list from those elements of the list for which `f` returns true. If `f` is none, the identity function is assumed, that is, all elements of input list that are false are removed.
* Function `__list_member__`, given `(self:%list,item)`, returns `true` only if `self @count(item) > 0`.

A **string** is a sequence of characters that can be used as a variable or a literal constant.

**The following member functions only support strings,**
* Function `gettype` will get the type of `x` as an Asteroid string.
* The following `string_member_functions`, 

            escape
            "
            global string_member_functions

            string_member_functions.update({
            'length'    : '__string_length__',
            'explode'   : '__string_explode__',
            'trim'      : '__string_trim__',
            'replace'   : '__string_replace__',
            'split'     : '__string_split__',
            'toupper'   : '__string_toupper__',
            'tolower'   : '__string_tolower__',
            'index'     : '__string_index__',
            'flip'      : '__string_flip__',
            })
            "
(For implementation details, see Python lists [here](https://docs.python.org/3/library/stdtypes.html#text-sequence-type-str).)

* Function `__string_length__`, given the input `self:%string`, returns the number of characters within that string.
* Function `__string_explode__`, given the input `self:%string`, elongates that string.
* Function `__string_trim__`, given the input `(self:%string,what:%string)`, returns a copy of the string with the leading and trailing characters removed. The what argument is a string specifying the set of characters to be removed. If omitted or none, the what argument defaults to removing whitespace. The what argument is not a prefix or suffix; rather, all combinations of its values are stripped.
* Function `__string_replace__`, given the input `(self:%string,old:%string,new:%string,count:%integer)`, will return a copy of the string with all occurrences of regular expression pattern `old` replaced by the string `new`. If the optional argument count is given, only the first count occurrences are replaced.
* Function `__string_toupper__`, given the input `self:%string`, converts all the lowercase letters in a string to uppercase.
* Function `__string_tolower__`, given the input `self:%string`, converts all the uppercase letters in a string to lowercase.
* Function `__string_index__`can be called with three different inputs: `(self:%string,item:%string,startix:%integer,endix:%integer)`, `(self:%string,item:%string,startix:%integer)`, or `(self:%string,item:%string)`. This function allows the user to search for a given `item_val[1]`, and/or `startix_val[1]` and `endix_val[1]` as well.
* Function `__string_flip__` explodes, reverses, and joins the given input `self:%string`.

More information about the functions that `lists` and `strings` contribute to can be found in the [Prologue.ast Module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/prologue.ast).


## Asteroid Modules

There are a variety of useful modules that can be installed in Asteroid.

[Hash.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/hash.ast) implements the `HashTable` structure, for mapping keys onto paired values.

[IO.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/io.ast) implements Asteroid's I/O system for delivering outputs from given inputs.

[Math.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/math.ast) implements its mathematical constants and operators.

[Pick.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/pick.ast) implements the `Pick` structure. A `pick` object contains a list of items that can be randomly picked from using the `pick` member function.

[Random.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/random.ast) implements the `random` numbers. Using the functions included in this module will return a random value or floating point number within a given range or interval.

[Set.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/set.ast) implements Asteroid sets as lists. Unlike lists, sets do not have repeated members.

[Sort.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/sort.ast) defines a parameterized sort function over a list. `Sort` makes use of a user-defined order predicate on the list's elements to perform the sort. The `Quicksort` is the underlying sort algorithm.

[Stream.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/stream.ast) implements the `Stream` structure. Asteroid stream implementation is based on lists.

[Util.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/util.ast) defines utility functions and structures.

[Vector.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/vector.ast) defines functions useful for vector arithmetic.
