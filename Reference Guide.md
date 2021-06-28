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

* Function `islist`, given the input item `do`, returns the item `is %list`. (See the module [Type.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast) for more on this function.)
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

See the [Prologue module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/prologue.ast)for more on all the functions above.

A **string** is a sequence of characters that can be used as a variable or a literal constant.

**The following member functions only support strings,**
* Function `gettype` will get the type of `x` as an Asteroid string. (See the module [Type.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast) for more on this function.)
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

See the [Prologue module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/prologue.ast)   for more on all the functions above.

/////

* Function `tointeger`, given `(item:%string,base:%integer)`, returns `('integer', int(item_val[1], base=base_val[1]))`. It can also be called with `item`, which gives the output `('integer', int(item_val[1]))`.
* Function `tostring` converts an Asteroid object into a string. If format values are given, it applies the formatting to the object. It can be called with several different inputs: `(v:*TP,w:%integer)`, `(v:%real,w:%integer)`, `(v:%real,w:%integer,p:%integer)`, and `item`.
* Function `tobase`, given `(x:%integer,base:%integer)`, returns the given integer `x` as a numeral in different bases (as a string).

See the [Type.ast Module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast) for the functions above.

More information about the functions that `lists` and `strings` contribute to can be found in the [Prologue.ast Module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/prologue.ast), as well as the [Type.ast Module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast).


## Asteroid Modules

There are a variety of useful modules that can be installed in Asteroid.

[Bitwise.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/bitwise.ast) defines Bitwise operations. It supports the following functions,
* Function `band` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise AND operation.
* Function `bor` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise OR operation.
* Function `bnot` can be called with the input `(x:%integer)`, and performs the Bitwise NOT operation.
* Function `bxor` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise XOR operation.
* Function `blshift` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise left shift operation.
* Function `brshift` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise right shift operation.
* Function `blrotate` can be called with the input `(x:%integer, i:%integer)`, and performs the Bitwise left rotate operation.
* Function `brrotate` can be called with the input `(x:%integer, i:%integer)`, and performs the Bitwise right rotate operation.
* Function `bsetbit` can be called with the input `(x:%integer, i:%integer)`, and sets the ith bit.
* Function `bclearbit` can be called with the input `(x:%integer, i:%integer)`, and clears the ith bit.
* Function `bsize`can be called with the input `(x:%integer)`, and returns the bit size.

[Hash.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/hash.ast) implements the `HashTable` structure, for mapping keys onto paired values. It supports the following functions,
* Function `__init__` can be called with the input `self`. This constructor for HashTable initializes the underlying dictionary, and stores the dictionary as a foreign object in its object memory alongside the table.
* Function `insert`, given the input `(self,name,value)`, will insert a given name-value pair into the table in `self`'s object memory.
* Function `get`, given `(self,name)`, will return the `value_val` associated with the given `name_val` as long as it can be found in `dictionary.keys()`.
* Function `aslist`, given `(self)`, gets the `value_val` associated with `name_val` and then zips the keys and values. It then turns Python tuples into Asteroid tuples and puts them onto an output list, so that the return value output is `('list', out_list)`.

[IO.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/io.ast) implements Asteroid's I/O system for delivering outputs from given inputs. It supports the following functions,
* Function `raw_print` can be called with `item`, and dumps the AST to screen.
* Function `print_ln` can be called with `item`, and prints a given argument.
* Function `print` can be called with `item`, and prints a given argument.
* Function `read` can be called with `none`, and will read a string from `stdin`.
* Function `input` can be called with a string `prompt`, and maps the Asteroid input function onto the Python input function.

[Math.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/math.ast) implements its mathematical constants and operators. It supports the following functions,
* Function `exp`, given `(x:%integer)`, returns e raised to the power `x`, where e = 2.718281… is the base of natural logarithms.
* Function `log` can be called with two different arguments. With one argument, `(x)`, it returns the natural logarithm of x (to base e). With two arguments, `(x,base)` it returns the logarithm of x to the given base, calculated as log(x)/log(base).
* Function `pow`, given `(b,p:%integer)`, returns "b <sup>p</sup>" as long as b can be found in `real` and `integer`.
* Function `sqrt`, given `a`, returns its square root as long as `a` can be found in `real` and `integer`.
* Function `abs`, given `(x)`, returns its absolute value.
* Function `ceil`, given `(x:%real)`, returns the ceiling of x: the smallest integer greater than or equal to x.
* Function `floor`, given `(x:%real)`, returns the floor of x: the largest integer less than or equal to x.
* Function `gcd`, given `(a:%integer,b:%integer)`, returns the greatest common denominator that both integers share.
* Function `isclose`, given `(a,b)` OR `(a,b,t)`, returns `True` if the two or three values are close to each other and `False` otherwise. Default tolerance 1e-09.
* Function `mod`, given `(v,d)`, will return the remainder of the operation `v/d`, as long as `v` and `d` can be found in `real` and `integer`.
* Function `acos`, given `(x)`, returns the arc cosine of x in radians. The result is between 0 and pi.
* Function `asin`, given `(x)`, returns the arc sine of x in radians. The result is between -pi/2 and pi/2.
* Function `atan`, ,given `(x)`, returns the arc tangent of x in radians. The result is between -pi/2 and pi/2.
* Function `cos`, given `(x)`, returns the cosine of x radians.
* Function `sin`, given `(x)`, returns the sine of x radians.
* Function `tan`, given `(x)`, returns the tangent of x radians.
* Function `acosh`, given `(x)`, returns the inverse hyperbolic cosine of x.
* Function `asinh`, given `(x)`, returns the inverse hyperbolic sine of x.
* Function `atanh`, given `(x)`, returns the inverse hyperbolic tangent of x.
* Function `cosh`, given `(x)`, returns the hyperbolic cosine of x.
* Function `sinh`, given `(x)`, returns the hyperbolic sine of x.
* Function `tanh`, given `(x)`, returns the hyperbolic tangent of x.
* Function `degrees`, given `(x)`, converts angle `x` from radians to degrees.
* Function `radians`,  given `(x)`, converts angle `x` from degrees to radians.

[Pick.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/pick.ast) implements the `Pick` structure. A `pick` object contains a list of items that can be randomly picked from using the `pick` member function. It supports the following functions that perform an output of this structure,
* Function `pick` can be called with `n:%integer`.
* Function `__init__` can be called with `l:%list`.

[Random.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/random.ast) implements the `random` numbers. Using the functions included in this module will return a random value or floating point number within a given range or interval. It supports the following functions,
* Function `random`, given the input `none`, returns a random floating point number in the range `[0.0, 1.0)`.
* Function `randint` can be called with two different number interval inputs: `(lo:%integer,hi:%integer)` or `(lo:%real,hi:%real)`. In either case, it returns a random value N in the interval lo <= N <= hi. The exact random value output depends on the types of the values specifying the interval.
* Function `seed`, given `(sd:%integer)`, returns a random value N in the interval lo <= N <= hi. The exact random value output depends on the types of the values specifying the interval.

[Set.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/set.ast) implements Asteroid sets as lists. Unlike lists, sets do not have repeated members. It supports the following functions,
* Function `toset`, given `(lst:%list)`, converts the inputted list into a set.
* Function `sdiff`, given `(a:%list,b:%list)`, does a side-by-side print of the differences between two files.
* Function `sunion`, given `(a:%list,b:%list)`, prints the smallest set which contains all the elements of both `a` and `b`.
* Function `sintersection`, given `(a:%list,b:%list)`, finds the interection between  `a` and `b`.
* Function `sxunion`, given `(a:%list,b:%list)`, returns all elements in `a` or `b`, but not in both.

[Sort.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/sort.ast) defines a parameterized sort function over a list. `Sort` makes use of a user-defined order predicate on the list's elements to perform the sort. The `Quicksort` is the underlying sort algorithm. It supports the function below,
* Function `sort` can be called with three different inputs to perform its sorting output. These are: `(_,[])`, `(_,[a])`, and `(p,[pivot|rest])`.

[Stream.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/stream.ast) implements the `Stream` structure. Asteroid stream implementation is based on lists.
* Function `__init__` can be called with two different inputs: `none` and `stream:%list`. In either case, this function outputs a shallow copy of the input list.
* Function `eof` can be called with the input `none`. If `this @curr_ix == this @stream @length()`, the function returns `true`. If not, it returns `false`.
* Function `peek` can be called with the input `none`. If `this @eof()`, it returns `none`. If not, it returns `this @stream @(this @curr_ix)`.
* Function `next` can be called with the input `none`. If `this @eof()`,  it returns `none`. If not, it decides to `let this @curr_ix = this @curr_ix + 1`.
* Function `get` can be called with the input `none`, and returns `this @peek()`.
* Function `rewind` can be called with the input `none`, and then decides to `let this @curr_ix = 0`.
* Function `map`, given the input `f`, applies a given function to each element of a function.
* Function `append`, given `item`, adds said item to a stream.
* Function `__string__`, given `none`, outputs it as a string.

[Type.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast) defines type related functions and structures.

See the Built-ins section of this Reference Guide for more on the list and string related functions this module supports. Other than those, this module supports the following functions,

* Function `toreal`, given `item`, returns the input as a real number data type `('real', float(item_val[1]))`.
* Function `toboolean`, given `item`, returns the input as a Boolean value of either true or false.
* Function `isscalar`, given `item`, returns `(item is %integer)` or `(item is %real)`.
* Function `isnone`, given `x`, returns `x is %none`.

[Util.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/util.ast) defines utility functions and structures. It supports the following functions,

* Function `exit`, given `none` or `msg:%string`, imports `sys.exit(1)` to exit the program.
* Function `copy`, given Asteroid object `obj`, makes a deep copy of it.
* Function `cls`, given `none`, clears the screen.
* Function `sleep`, given `secs %if isscalar(secs)`, programs sleep for `secs` seconds.
* Function `zip`, given `(list1:%list,list2:%list)`, implements Python's zip function. It turns Python tuples into Asteroid tuples and puts them onto an output list.
* Function `unzip`, given `(list:%list)`, will unzip a list of pairs.
* Function `ascii`, given a character `item:%string`, returns the corresponding ASCII code.
* Function `achar`, given a decimal ASCII code `item:%integer`, returns the corresponding character symbol.

[Vector.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/vector.ast) defines functions useful for vector arithmetic). It supports the following functions,
* Function `vop` can be called with three different inputs: `(f:%function,a:%list,b:%list)`, `(f:%function,a:%list,b %if isscalar(b))`, and `(f:%function,a %if isscalar(a),b:%list)`. In any case, this function implements actual vector arithmetic. It also implements vector/scalar arithmetic.
