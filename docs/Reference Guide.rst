



..
   *** DO NOT EDIT; MACHINE GENERATED ***


.. highlight:: none

Asteroid Reference Guide
========================

Language Syntax
---------------

The following is the complete grammar for the Asteroid language. Capitalized
words are either keywords such as ``FOR`` and ``END`` or tokens such as ``STRING`` and ``ID``.  Non-terminals
are written in all lowercase letters.  The grammar utilizes an extended BNF notation
where ``<syntactic unit>*`` means zero or more occurrences of the syntactic unit and
``<syntactic unit>+`` means one or more occurrences of the syntactic unit. Furthermore,
``<syntactic unit>?`` means that the syntactic unit is optional.  Simple terminals
are written in quotes.
::


  ////////////////////////////////////////////////////////////////////////////////////////
  // statements

  prog
    : stmt_list

  stmt_list
    : stmt*

  stmt
    : '.' // NOOP
    | LOAD SYSTEM? (STRING | ID) '.'?
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
    | TRY DO? stmt_list (CATCH pattern DO stmt_list)+ END
    | THROW exp '.'?
    | BREAK '.'?
    | RETURN exp? '.'?
    | function_def
    | call_or_index '.'?

  function_def
    : FUNCTION ID body_defs END

  body_defs
    : WITH pattern DO stmt_list (WITH pattern DO stmt_list)*

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

  ////////////////////////////////////////////////////////////////////////////////////////
  // expressions/patterns
  
  exp
    : pattern

  pattern
    : PATTERN WITH? exp
    | '%[' exp ']%'      
    | head_tail

  head_tail
    : conditional ('|' exp)?

  
  conditional
    : compound (IF exp (ELSE exp)?)?

  compound
    : logic_exp0
        (
           (IS pattern) |
           (IN exp) |               
           (TO exp (STRIDE exp)?) |   
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
    : primary (primary | '@' primary)* (':' pattern)?  

  ////////////////////////////////////////////////////////////////////////////////////////
  // primary expressions/patterns

  primary
    : INTEGER
    | REAL
    | STRING
    | TRUE
    | FALSE
    | NONE
    | ID
    | '*' call_or_index   
    | NOT call_or_index
    | MINUS call_or_index
    | PLUS call_or_index
    | ESCAPE STRING
    | EVAL primary
    | '(' tuple_stuff ')' 
    | '[' list_stuff ']'  
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



Builtin Functions
-----------------

* Function ``len``, when given an input value, returns the length of that input. The
  function can only be applied to lists, strings, tuples, or structures.

* Function ``hd``, when given a list as input returns the first element of that list.
  It is an error to apply this function to an empty list.

* Function ``tl``, when given a list as input returns the rest of the list without the first element.
  It is an error to apply this function to an empty list.

* Function ``range`` will compute a list of values depending on the input values:

  1. ``(start:%integer,stop:%integer)`` returns list ``[start to stop-1]``.
  2. ``(start:%integer,stop:%integer,inc:%integer)`` returns list ``[start to stop-1 stride inc]``.
  3. ``(stop:%integer)`` returns list ``[0 to stop-1]``.

* Function ``getid`` returns the id (physical memory address) of any Asteroid object as an Asteroid integer.

* Function ``isdefined`` returns true if a variable or type name is defined in the
  current environment otherwise it returns false. The variable or type name must be given as a string.

List and String Objects
-----------------------

In Asteroid, both ``lists`` and ``strings,`` are treated like objects. Due to this, they have member functions that can manipulate the contents of those objects.

Lists
^^^^^

A **list** is a structured data type that consists of square brackets enclosing
comma-separated values.
Member functions on lists can be called on the data structure directly, e.g.::

   [1,2,3] @length()

* Function ``length`` returns the number of elements within that list.
* Function ``append``, given ``(item)``, adds that item to the end of a list.
* Function ``extend``, given ``(item)``, will extend the list by adding all the items from the item where ``item`` is either a list, a string or a tuple.
* Function ``insert``, given ``(ix:%integer,item)``, will insert an item at a given position. The first argument is the index of the element before which to insert, so ``a@insert(0, x)`` inserts at the front of the list, and ``a@insert(a@length(), x)`` is equivalent to ``a@append(x)``.
* Function ``remove``, given ``(item)``, removes the first element from the list whose value is equal to ``(item)``. It raises a ValueError if there is no such item.
* Function ``pop``, given ``(ix:%integer)``, removes the item at the given position in the list and returns it. If no index is specified,``a@pop()`` removes and returns the last item in the list.
* Function ``clear``, given ``(none)``, removes all items from the list.
* Function ``index`` returns a zero-based index in the list of the first element whose value is equal to ``(item)``. It raises a ValueError exception if there is no such item. The optional argument ``loc`` allows you to specify ``(startix)`` and ``(endix)`` and are used to limit the search to a particular subsequence of the list. The returned index is computed relative to the beginning of the full sequence rather than the ``(startix)`` argument.   This function can be called with several input configurations:

  1. ``(item,loc(startix:%integer,endix:%integer))``
  2. ``(item,loc(startix:%integer))``
  3. ``item``

* Function ``count``, given ``(item)``, returns the number of times ``(item)`` appears in the list.
* Function ``sort`` sorts the items of the list in place. It can be called with several different inputs:

  1. ``(reverse:%boolean)`` if the boolean is set to true then the sorted list is reversed.
  2. ``none`` returns the reverse list.

* Function ``reverse``, reverses the elements of the list in place.
* Function ``copy``, makes a shallow copy of the list.
* Function ``shuffle``, returns a random permutation of a given list - in place!
* Function ``map``, given ``(f:%function)``, applies ``f`` to each element of the list in place. The modified list is returned.
* Function ``reduce`` reduces the value of elements in a list. This
  function can be called with several different inputs:

  1. Input ``(f:%function)`` returns ``value``, such that ``value = f(value,this@i)``.
  2. Input ``(f:%function,init)`` returns the same format but uses ``init`` as an initial value.

  The first argument to ``f`` is the accumulator.

* Function ``filter``, given ``(f:%function)``, constructs an output list from those elements of the list for which ``f`` returns true. If ``f`` is none, the identity function is assumed, that is, all elements of the input list that are false are removed.
* Function ``member``, given ``(item)``, returns true only if ``item`` exists on the list.
* Function ``join``, given ``(join:%string)``, turns the list into a string using ``join`` between the elements.  The string is returned
  as the return value from this function.


See the `Prologue module <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/prologue.ast>`_ for more on all the functions above.


Strings
^^^^^^^

A string is a sequence of characters that can be used as a variable or a literal constant.
Similar to lists the member functions of strings can be called directly on the
data structure itself, e.g.::

   "Hello there" @length()

* Function ``length`` returns the number of characters within that string.
* Function ``explode``, turns a string into a list of characters.
* Function ``trim``, given the input ``(what:%string)``, returns a copy of the string with the leading and trailing characters removed. The ``what`` argument is a string specifying the set of characters to be removed. If omitted or none, the ``what`` argument defaults to removing whitespace. The ``what`` argument is not a prefix or suffix; rather, all combinations of its values are stripped.
* Function ``replace`` will return a copy of the string with all occurrences of regular expression pattern ``old`` replaced by the string ``new``. If the optional argument count is given, only the first count occurrences are replaced. It can be called with several different inputs:

  * ``(old:%string,new:%string,count:%integer)``
  * ``(old:%string,new:%string)``

* Function ``split`` will return a list of the words in a given string, using ``sep`` as the delimiter string. If ``maxsplit`` is given: at most maxsplit splits are done (thus, the list will have at most maxsplit+1 elements). If maxsplit is not specified or -1, then there is no limit on the number of splits (all possible splits are made).
  If ``sep`` is given, consecutive delimiters are not grouped together and are deemed to delimit empty strings (for example, ``"1,,2"@split(",")`` returns ``["1", "", "2"]``). The ``sep`` argument may consist of multiple characters (for example, ``"1<>2<>3"@split("<>")`` returns ``["1", "2", "3"]``). Splitting an empty string with a specified separator returns ``[""]``.
  If ``sep`` is not specified or is None, a different splitting algorithm is applied: runs of consecutive whitespace are regarded as a single separator, and the result will contain no empty strings at the start or end if the string has leading or trailing whitespace. Consequently, splitting an empty string or a string consisting of just whitespace with a None separator returns ``[]``.
  Function ``split`` can be called with several different inputs:

  1. Input ``(sep:%string,count:%integer)``
  2. Input ``(sep:%string)``
  3. Input ``(none)``

* Function ``toupper``, converts all the lowercase letters in a string to uppercase.
* Function ``tolower``, converts all the uppercase letters in a string to lowercase.
* Function ``index`` allows the user to search for a given ``item`` in a list. It returns an integer index into the string or ``none`` if ``item`` was not found.  The optional argument ``loc`` allows you to specify ``(startix)`` and ``(endix)`` and are used to limit the search to a particular substring of the string. The returned index is computed relative to the beginning of the full string rather than the ``(startix)`` argument.The function can be called with several different inputs:

  1. Input ``(item:%string,loc(startix:%integer,endix:%integer))``
  2. Input ``(item:%string,loc(startix:%integer))``
  3. Input ``(item:%string)``

* Function ``flip`` reverses a string.

See the `Prologue module <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/prologue.ast>`_  for more on all the functions above.


Asteroid Modules
----------------

There are a number of system modules that can be loaded into an Asteroid program using ``load system <module name>``.
The modules are implemented as objects where all the functions of that module are
member functions of that module object. For example, in the case of the ``io`` module
we have ``println`` as one of the member functions.  To call that function::

   load system io.
   io @println "Hello there!".  -- println is a member function of the io module

Bitwise
^^^^^^^

The `bitwise <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/bitwise.ast>`_ module defines Bitwise operations. It supports the following functions,

* Function ``band`` can be called with the input ``(x:%integer, y:%integer)``, and performs the Bitwise AND operation.
* Function ``bor`` can be called with the input ``(x:%integer, y:%integer)``, and performs the Bitwise OR operation.
* Function ``bnot`` can be called with the input ``(x:%integer)``, and performs the Bitwise NOT operation.
* Function ``bxor`` can be called with the input ``(x:%integer, y:%integer)``, and performs the Bitwise XOR operation.
* Function ``blshift`` can be called with the input ``(x:%integer, y:%integer)``, and performs the Bitwise left shift operation.
* Function ``brshift`` can be called with the input ``(x:%integer, y:%integer)``, and performs the Bitwise right shift operation.
* Function ``blrotate`` can be called with the input ``(x:%integer, i:%integer)``, and performs the Bitwise left rotate operation.
* Function ``brrotate`` can be called with the input ``(x:%integer, i:%integer)``, and performs the Bitwise right rotate operation.
* Function ``bsetbit`` can be called with the input ``(x:%integer, i:%integer)``, and sets the ith bit.
* Function ``bclearbit`` can be called with the input ``(x:%integer, i:%integer)``, and clears the ith bit.
* Function ``bsize``can be called with the input ``(x:%integer)``, and returns the bit size.

Hash
^^^^

The `hash <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/hash.ast>`_ module implements a hash for name-values pairs. It supports the following functions,

* Function ``insert``, given the input ``(name,value)``, will insert a given name-value pair into the table.
* Function ``get``, given ``name``, will return the ``value`` associated with the given ``name`` as long as it can be found otherwise an exception will be thrown.
* Function ``aslist`` returns the hash as a list of name-value pairs.

IO
^^

The `io <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/io.ast>`_ module implements Asteroid's I/O system. The module defines three default streams,

1. ``__STDIN__`` - the standard input stream.
2. ``__STDOUT__`` - the standard output stream.
3. ``__STDERR__`` - the standard error stream.

Furthermore, the module supports the following functions,

* Function ``println`` can be called with ``item``, and prints a given argument to the terminal (``__STDOUT__``) with an implicit newline character.
* Function ``print`` can be called with ``item``, and prints a given argument. No implicit newline is appended to the output.
* Function ``input`` can be called with a string ``prompt``.  If ``prompt`` is given it is printed and then input is read from the terminal (``__STDIN__``) and returned as a string.
* Function ``open`` opens a file. Given ``(name:%string, mode:%string)``, it returns a file descriptor of type ``FILE``. The ``mode`` string can be ``"r"`` when the file will only be read, ``"w"`` for only writing (an existing file with the same name will be erased), and ``"a"`` opens the file for appending; any data written to the file is automatically added to the end. The ``"r+"`` opens the file for both reading and writing.
* Function ``close``, given ``file:%FILE``, closes that file.
* Function ``read``, given ``file:%FILE``, reads a file. If no file is given the ``__STDIN__`` stream is read.
* Function ``readln``, given ``file:%FILE``, reads a given line of input from the file. If no file is given the ``__STDIN__`` stream is read.
* Function ``write``, given ``(file:%FILE, what:%string)``, will write ``what`` to the given ``file``.  If ``file`` is not given then it writes to the ``__STDOUT__`` stream.
* Function ``writeln``, works the same way as ``write`` except that it appends a newline character to the output.

Math
^^^^

The `math <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/math.ast>`_ module implements mathematical constants and functions. It supports the following functions,

*Power and logarithmic functions*

* Function ``exp``, given ``x:%integer``, returns e raised to the power ``x``, where e = 2.718281… is the base of natural logarithms.
* Function ``log`` can be called with two different argument setups,

  1. If only one argument, ``(x)``, is input, this returns the natural logarithm of x (to base e).
  2. If two arguments, ``(x,base)``, are input, this returns the logarithm of x to the given base, calculated as log(x)/log(base).

* Function ``pow``, given ``(b,p:%integer)``, returns "b <sup>p</sup>" as long as b is either ``real`` or ``integer``.
* Function ``sqrt``, given ``a``, returns its square root as long as ``a`` is either ``real`` or ``integer``.

*Number-theoretic and representation functions*

* Function ``abs``, given ``x``, returns its absolute value.
* Function ``ceil``, given ``x:%real``, returns the ceiling of x: the smallest integer greater than or equal to x.
* Function ``floor``, given ``x:%real``, returns the floor of x: the largest integer less than or equal to x.
* Function ``gcd``, given ``(a:%integer,b:%integer)``, returns the greatest common denominator that both integers share.
* Function ``isclose`` can be called with two different argument setups,

  1. With input values ``(a,b)``, it returns returns ``true`` if the two values are close to each other and ``False`` otherwise. Default tolerance 1e-09.
  2. With input values ``(a,b,t)``, it compares ``a`` and ``b`` with tolerance ``t``.

* Function ``mod``, given ``(v,d)``, will return the remainder of the operation ``v/d``, as long as ``v`` and ``d`` are either ``real`` or ``integer`` values.

*Trigonometric functions*

* Function ``acos``, given ``x``, returns the arc cosine of x in radians. The result is between 0 and pi.
* Function ``asin``, given ``x``, returns the arc sine of x in radians. The result is between -pi/2 and pi/2.
* Function ``atan``, ,given ``x``, returns the arc tangent of x in radians. The result is between -pi/2 and pi/2.
* Function ``cos``, given ``x``, returns the cosine of x radians.
* Function ``sin``, given ``x``, returns the sine of x radians.
* Function ``tan``, given ``x``, returns the tangent of x radians.
* Function ``acosh``, given ``x``, returns the inverse hyperbolic cosine of x.
* Function ``asinh``, given ``x``, returns the inverse hyperbolic sine of x.
* Function ``atanh``, given ``x``, returns the inverse hyperbolic tangent of x.
* Function ``cosh``, given ``x``, returns the hyperbolic cosine of x.
* Function ``sinh``, given ``x``, returns the hyperbolic sine of x.
* Function ``tanh``, given ``x``, returns the hyperbolic tangent of x.
* Function ``degrees``, given ``x``, converts angle ``x`` from radians to degrees.
* Function ``radians``,  given ``x``, converts angle ``x`` from degrees to radians.

An example,
::

    load system io.
    load system math.

    let x = math @sin( math @pi / 2 ).
    io @println("The sine of pi / 2 is " + x + ".").


Pick
^^^^

The `pick <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/pick.ast>`_ module implements
pick objects that allow a user to randomly pick items from a list using the ``pick`` function.
The ``pick`` function can be called with ``n:%integer`` and returns a list of ``n`` randomly picked objects from the object list.
Here is a simple use case
::

   load system io.
   load system pick.

   let po = pick @pick([1 to 10]).
   let objects = po @pick(3).
   io @println objects.
   


Random
^^^^^^

The `random <https://github.com/lutzhamel/asteroid/blob/master/asteroid/modules/random.ast>`_ module implements the ``random`` numbers. Using the functions included in this module will return a random value within a given range or interval. It supports the following functions,

* Function ``random``, given the input ``none``, returns a random floating point number in the range ``[0.0, 1.0)``.
* Function ``randint`` returns a random value N in the interval lo <= N <= hi. The exact random value output depends on the types of the values specifying the interval. It can be called with two different number interval inputs:

  1. ``(lo:%integer,hi:%integer)``
  2. ``(lo:%real,hi:%real)``
  3. Note: any other interval specification will instead output an error message for "unsupported interval specification in randint."

* Function ``seed``, given ``(sd:%integer)``, provides a seed value for the random number generator.

Set
^^^

The `set <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/set.ast>`_ module implements Asteroid sets as lists. Unlike lists, sets do not have repeated members. It supports the following functions,

* Function ``toset``, given ``(lst:%list)``, converts the input list into a set.
* Function ``diff``, given ``(a:%list,b:%list)``, computes the difference set between the two set ``a`` and ``b``.
* Function ``intersection``, given ``(a:%list,b:%list)``, finds the intersection between  sets ``a`` and ``b``.
* Function ``union``, given ``(a:%list,b:%list)``, computes the union of sets ``a`` and ``b``.
* Function ``xunion``, given ``(a:%list,b:%list)``, returns all elements in ``a`` or ``b``, but not in both.

Sort
^^^^

The `sort <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/sort.ast>`_ module
defines a parameterized ``sort`` function over a list.
The ``sort`` function makes use of a user-defined order predicate on the list's elements to
perform the sort. The ``Quicksort`` is the underlying sort algorithm.
The following is a simple example,
::

   load system io.
   load system sort.
   let sl = sort @sort((lambda with (x,y) do return true if x<y else false),
                       [10,5,110,50]).
    io @println sl.


prints the sorted list::

  [5,10,50,110]

Stream
^^^^^^

The `stream <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/stream.ast>`_ module implements streams that allow
the developer to turn any list into a stream supporting interface functions like ``peeking`` ahead or ``rewinding`` the stream.
The following stream interface functions are available,

* Function ``eof`` returns ``true`` if the stream does not contain any further elements for processing. Otherwise it returns ``false``.
* Function ``peek`` returns the current element available on the stream otherwise it returns ``none``.
* Function ``get`` returns the current element and moves the stream pointer one ahead.
* Function ``rewind`` resets the stream pointer to the first element of the stream.
* Function ``map`` applies a given function to each element in the stream.
* Function ``append``, given ``item``, adds item to the end of the stream.
* Function ``__string__`` maps a the stream to a string representation.

A simple use case.
::

   load system io.
   load system stream.

   let s = stream @stream([1 to 10]).
   while not s @ eof() do
      io @ print (s @get()+" ").
   end
   io @println ("").
   

which outputs::

   1 2 3 4 5 6 7 8 9 10


Type
^^^^

The `type <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/type.ast>`_ module defines type related functions and structures.

*Type Conversion*

* Function ``tointeger`` converts a given input to an integer. It can be called with two different arguments,

  1. ``(item:%string,base:%integer)`` where ``base`` is a valid base for integer conversion
  2. ``item`` where ``item`` is converted to a base 10 integer.


* Function ``toreal``, given ``item``, returns the input as a real number data type.
* Function ``toboolean``, given ``item``, returns the input as a Boolean value of either true or false.
* Function ``tostring`` converts an Asteroid object to a string. If format values are given, it applies the formatting to the object. It can be called with several different inputs where ``*TP`` indicates a``boolean``, ``integer``, or ``string`` type and ``w`` is the width specification, ``p`` is the precision specification and ``s`` is the scientific notation flag.  When no formatting information is provided a default string conversion occurs,

  1. ``(v:*TP,type @stringformat(w:%integer))``
  2. ``(v:%real,type @stringformat(w:%integer))``
  3. ``(v:%real,type @stringformat(w:%integer,p:%integer))``
  4. ``(v:%real,type @stringformat(w:%integer,p:%integer,s:%boolean))``
  5. ``item`` - default conversion

* Function ``tobase`` represents the given integer ``x`` (*specifically* within the given input ``(x:%integer,base:%integer)``) as a string in the given base.

Here is a program that exercises some of the string formatting options,
::

    load system io.
    load system type.
    load system math.

    -- if the width specifier is larger than the length of the value
    -- then the value will be right justified
    let b = type @tostring(true,type @stringformat(10)).
    io @println b.

    let i = type @tostring(5,type @stringformat(5)).
    io @println i.

    -- we can format a string by applying tostring to the string
    let s = type @tostring("hello there!",type @stringformat(30)).
    io @println s.

    -- for floating point values: first value is width, second value precision.
    -- if precision is missing then value is left justified and zero padded on right.
    let r = type @tostring(math @pi,type @stringformat(6,3)).
    io @println r.


The output of the program is,
::

          true
        5
                      hello there!
     3.142

Notice the right justification of the various values within the given string length.


*Type Query Functions*

* Function ``islist`` returns ``true`` if given ``item`` is a list otherwise it will return ``false``.
* Function ``isscalar`` returns ``true`` if given ``item`` is either an integer or a real value.
* Function ``isnone`` returns ``true`` if given ``item`` is equal to the value ``none``.
* Function ``gettype`` returns the type of a given ``item`` as an Asteroid string.

A simple example program using the ``gettype`` function,
::

   load system type.

   let i = 1.
   assert(type @gettype(i) == "integer").
   

Util
^^^^

The `util <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/util.ast>`_ module defines utility functions and structures that don't really fit into any omodules. It supports the following functions,

* Function ``exit`` exits the program. It can be called with two inputs,

  1. ``none``
  2. ``msg:%string``

* Function ``copy``, given Asteroid object ``obj``, makes a deep copy of it.
* Function ``cls`` clears the terminal screen.
* Function ``sleep``,  programs sleep for ``secs`` seconds where the argument ``secs`` is either an integer or real value.
* Function ``zip``, given ``(list1:%list,list2:%list)``, will return a list where element ``i`` of the list is the tuple ``(list1@i,list2@i)``.
* Function ``unzip``, given a list of pairs will return a pair of lists where the first component of the pair is the list of all the first components of the pairs of the input list and the second component of the return list is a list of all the second components of the input list.
* Function ``ascii``, given a character ``item:%string``, returns the corresponding ASCII code of the first character of the input string.
* Function ``achar``, given a decimal ASCII code ``item:%integer``, returns the corresponding character symbol.

Vector
^^^^^^

The `vector <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/vector.ast>`_ defines functions useful for vector arithmetic. It supports the following functions.  Here ``a`` and ``b`` are vectors implemented as lists,

* Function ``add``, given the input ``(a,b)``, returns a vector that contains the element by element sum of the input vectors.
* Function ``sub``, given the input ``(a,b)``, returns the element by element difference vector.
* Function ``mult``, given the input ``(a,b)``, returns the element by element vector multiplication.
* Function ``dot``, given ``(a,b)``, computes the dot product of the two vectors.
* Function ``op``  allows the developer to vectorize any function. It can be called with three different inputs:

  1. ``(f:%function,a:%list,b:%list)``
  2. ``(f:%function,a:%list,b if type @isscalar(b))``
  3. ``(f:%function,a if type @isscalar(a),b:%list)``

Here is a simple example program for the ``vector`` module,
::

   load system io.
   load system vector.

   let a = [1,0].
   let b = [0,1].

   io @println (vector @dot (a,b)).
   

which prints the value ``0``.

