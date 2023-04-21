..
      /******************************************************************
      This is the source file from which the reference guide is
      generated.  We use cpp to insert live code snippets into the
      document. In order to generate the reference guide run the
      following command on a Unix-like system in the directory of
      this doc:

      bash generate_docs

      ******************************************************************/
..
   /* header for generated .rst files */

..
   *** DO NOT EDIT; MACHINE GENERATED ***
.. highlight:: none

Asteroid Reference Guide
========================

Language Syntax
---------------

Note: In the following descriptions ``<something>?`` denotes an optional
something in a piece of syntax.  We also use the notation ``<something>*``
which means that something can appear zero or more times in a program.
Capitalized
words are keywords where ``FOR`` represents the keyword ``for`` and ``END``
represents ``end``.

Statements
^^^^^^^^^^

Assert
%%%%%%

Syntax: ``ASSERT exp '.'?``

If the expression of the assert statement evaluates to a
value equivalent to the Boolean value
``false`` an exception is thrown otherwise the statement is ignored.

For example, the statement,
::
      assert (1+1 == 3).

will generate a runtime error but the statement,
::
      assert (1+1 == 2).

will be ignored once the expression has been evaluated.


Break
%%%%%

Syntax: ``BREAK '.'?``

The break statement immediately breaks out of the closest surrounding looping structure.
Execution will continue at the statement right after the loop. Issuing a break statement
outside of a looping structure will lead to a runtime error.

As an example we break out of the indefinite loop below when ``i`` is equal to 10,
::
      let i = 0.

      loop
         let i = i+1.
         if i==10 do
            break.
         end
      end

      assert (i==10).

Expressions at the Statement Level
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

Expressions at the statement level are supported.  However, they do not have
any effect on the computation unless they contain side effects with one
exception:  In the absence of an explicit return statement, the value of the last expression
evaluated in a function body is considered the return value of the function.

An example,
::
      function inc
         with i do
            i+1.
         end

Notice that because the expression ``i+1`` is the last statement evaluated in the
function body its value becomes the return value of the function.

For-Loop
%%%%%%%%

Syntax: ``FOR pattern IN exp DO stmt_list END``

In a for-loop the expression must evaluate to either a list or a tuple.  The pattern is then matched to
each component of the expression value sequentially starting with the first component.
The loop body is executed for each successful match.

In the following program the body of the loop is executed exactly once when
the pattern matches the tuple ``(1,"chicken")``,
::
      let tuple_list = [
              (0,"duck"),
              (1,"chicken"),
              (2,"turkey")
              ].

      for (1,bird) in tuple_list do
         assert(bird is "chicken").
      end


Function-Definition
%%%%%%%%%%%%%%%%%%%

Syntax: FUNCTION function_name WITH pattern DO stmt_list (WITH pattern DO stmt_list)* END

Function definitions in Asteroid can have one or more function bodies associated
with single function name.  A function body is associated with a particular pattern
that is matched against the actual argument of the function call.  If the match
is successful then the associated function body is executed.  If the match is not
successful then other pattern/body pairs are tried if present.  If none of the
patterns match the actual argument then this constitutes a runtime error.
Patterns are tried in the order they appear in the function definition.

The following is a definition of the ``sign`` function,
::
      function sign
         with x if x >= 0 do
            return 1.
         with x if x < 0 do
            return -1.
      end

Here the first function body returns ``1`` if the actual argument is greater or equal to zero.
The second function body return ``-1`` if the actual argument is less than zero.

Global
%%%%%%

Syntax: ``GLOBAL variable_name (',' variable_name)* '.'?``

The ``global`` statement allows the developer to declare a variable as global
within a function scope and this allows the developer to set the value of a global variable
from within functions.

Consider the following code snippet,
::
      let x = 0.

      function foo
         with none do
            global x.
            let x = 1.
      end

      assert(x==0).
      foo().
      assert(x==1).

The ``global`` statement within the function ``foo`` indicates that the ``let`` statement
on the following line should assign a value to the global variable ``x``.

If-Then-Else
%%%%%%%%%%%%

Syntax: ``IF exp DO stmt_list (ELIF exp DO stmt_list)* (ELSE DO? stmt_list)? END``

If the first expression evaluates to the equivalent of a Boolean ``true`` value
then the associated statements will be executed and the execution
continues after the ``end`` keyword.  If the expression evaluates to the equivalent
of a Boolean ``false`` then the expressions of the optional ``elif`` clauses
are evaluated if present.  If one of them evaluates to the equivalent of a Boolean
value ``true`` then the associated statements are executed and execution continues
after the ``end`` keyword. Otherwise
the statements of the optional ``else`` clause are executed if present and again
flow of control is transferred to the statements following the if-statement.

As an example consider the following ``if`` statement that determines
what kind of integer value the user supplied,
::
      load system io.
      load system type.

      let x = type @tointeger (io @input "Please enter an integer: ").

      if x < 0 do
          io @println "Negative".
      elif x == 0 do
          io @println "Zero".
      elif x == 1 do
          io @println "One".
      else do
          io @println "Positive".
      end


Let
%%%

Syntax: ``LET pattern = exp '.'?``

The ``let`` statement is Asteroid's version of the assignment statement with a twist though:  the left side of the ``=`` sign is not just a variable
but is considered a pattern.  For simple assignments there is no discernible difference between assignments in Asteroid and assignments in other
languages,
::
  let x = val.

Here, the variable ``x`` will match the value stored in ``val``.  However, because the left side of the ``=`` sign is a pattern we
can write something like this,
::
  load system math.
  let x:%[(k:%integer) if math @mod(k,2)==0]% = val.

where ``x`` will only match the value of ``val`` if that value is an even integer value.  The fact that the left side of the ``=`` is a pattern allows
us to write things like this,
::
   let 1 = 1.

which simply states that the value ``1`` on the right can be matched by the pattern ``1`` on the left.  Having the ability to pattern match
on literals is convenient for statements like these,
::
  let (1,x) = p.

This ``let`` statement is only successful for values of ``p`` which are pairs where the first component of the pair is the value ``1``.


Loop
%%%%

Syntax: ``LOOP DO? stmt_list END``

The ``loop`` statement executes the statements in the loop body indefinitely
unless a ``break`` statement is encountered.

Repeat-Until
%%%%%%%%%%%%

Syntax: ``REPEAT DO? stmt_list UNTIL exp '.'?``

Repeatedly execute the statements in the loop body until the
expression evaluates to the equivalent of a Boolean ``true`` value.

Here is an example of a program that prints out the elements
of a list,
::
      load system io.

      let l = ["bmw", "volkswagen", "mercedes"].

      repeat
         let [element|l] = l.
         io @println element.
      until l is [].


Return
%%%%%%

Syntax; ``RETURN exp? '.'?``

Explicitly return from a function with an optional return value.

Structure
%%%%%%%%%

Syntax: ``STRUCTURE type_name WITH data_or_function_stmts END``

The ``structure`` statement introduces a composite data type that defines a physically grouped list of variables under one name.  The variables within a structure can be declared as data members or as function members.
Unless a member function was declared as a constructor (an ``__init__`` function) structures are
instantiated using a default constructor. The default constructor copies the arguments given to it into the data member fields in the order that the data members appear in the structure definition and as they appear in the parameter list of the constructor.  We often refer to instantiated structures as objects.  Member values of objects
are accessed using the access operator ``@``. Here is a simple example,
::
      -- define a structure of type A
      structure A with
          data a.
          data b.
      end

      let obj = A(1,2).       -- call default constructor
      assert( obj @a == 1 ).  -- access first data member
      assert( obj @b == 2 ).  -- access second data member

We can use custom constructors to enforce that only certain types of values
can be copied into an object,
::
      -- define a structure of type Person
      structure Person with
          data name.
          data age.
          function __init__ with (name:%string,age:%integer) do -- constructor
             let this @name = name.
             let this @age = age.
          end
          function __str__ with none do
            return this@name+" is "+this@age+" years old".
          end
      end

      let betty = Person("Betty",21).  -- call constructor
      assert( betty @name == "Betty" ).
      assert( betty @age == 21 ).

      load system type.
      assert(type @tostring betty is "Betty is 21 years old").

Note that object identity is expressed using the ``this`` keyword.
Here we also supplied an instantiation of the ``__str__`` function that allows
us to customize the stringification of the object.  See the last line
where we cast the object ``betty`` to a string.  Without the ``__str__`` function
Asteroid uses a default representation of the object as a string.
The ``__str__`` function does not accept any arguments and has to return a string.

Try-Catch
%%%%%%%%%

Syntax: ``TRY DO? stmt_list (CATCH pattern DO stmt_list)+ END``

This statement allows the programmer to set up exception handlers for
exceptions thrown in the code of the ``try`` part of the statement.
Notice that you can set up one or more handlers within the ``catch`` part of
the statement.  If there are more than one handlers then they are searched in
order starting with the first.  Handlers are selected via pattern matching
on the exception object.  The handler code of the first ``catch`` clause whose
pattern matches the exception object is executed.

Below is an example of a ``try-catch`` statement where the code
in the ``try`` part generates a division-by-zero exception.  The
exception object is pattern-matched in the ``catch`` clause and processed
by the associated handler,
::
      load system io.

      try
          let x = 1/0.
      catch Exception("ArithmeticError", s) do
          io @println s.
      end

For more details on exceptions please see the User Guide.

Throw
%%%%%

Syntax: ``THROW exp '.'?``

Allows the developer to throw an exception.  Any object can serve as an
exception object. However, Asteroid provides some predefined exception objects.
For more details on exceptions please see the User Guide.

While-Loop
%%%%%%%%%%

Syntax: ``WHILE exp DO stmt_list END``

While the expression evaluates to the equivalent of a Boolean ``true`` value
execute the statements in the body of the loop.  The loop expression is reevaluated
after each loop iteration.

Here is an example that prints out a sequence of integer values in reverse order,
::
      load system io.

      let i = 10.

      while i do
         io @println i.
         let i = i-1.
      end

The loop terminates once ``i`` becomes zero which is the equivalent to a Boolean
value ``false``.

Expressions
^^^^^^^^^^^

All the usual arithmetic, relational, and logic operators,
::
      +, -, *, /, ==, =/=, <=, <, >=, >, and, or, not

are supported in
Asteroid.  For extended mathematical operations such as ``mod`` (modulus) or
``sin`` (sine) see the ``math`` module.  Here we discuss expression constructions
that are particular to Asteroid.

Substructure Access
%%%%%%%%%%%%%%%%%%%

Syntax: ``structure_exp @ index_exp``

Asteroid provides the uniform substructure access operator ``@`` for all structures
which includes lists, tuples, and objects. For example, accessing the first
element of a list is accomplished by the expression,
::
      [1,2,3] @0

Similarly, given an object constructed from structure ``A``, member values
are accessed by name via the ``@`` operator,
::
      structure A with
         data a.
         data b.
      end

      let obj = A(1,2).
      assert( obj @a == 1 ).  -- access member a


Head-Tail Operator
%%%%%%%%%%%%%%%%%%

Syntax: ``element_exp | list_exp``

This operator works in one of two ways.  In the first way it allows you to
pre-append an element to a list,
::
      let [1,2,3] = 1 | [2,3].

It can also be nested,
::
      let [1,2,3] = 1 | 2 | 3 | [].

In the second way it works as a pattern to deconstruct a list into its first
element and the remainder of the list, the list with its first element removed,
::
      let h | t = [1,2,3].
      assert(h == 1).
      assert(t == [2,3]).

You can put optional brackets around the operator to highlight the fact that
we are dealing with a list,
::
      let [h | t] = [1,2,3].

The Is Predicate
%%%%%%%%%%%%%%%%%%%%

Syntax: ``exp IS pattern``

This operator matches the structure computed by the expression on the left
side against the pattern on the right side of the operator.  If the match is
successful it returns the Boolean value ``true`` and if not successful then
it returns the Boolean value ``false``.  All regular rules of pattern matching
apply such as instantiating appropriate variable bindings in the current scope.

Example,
::
      if v is (x,y) do
         io @println "success".
         assert(isdefined "x").
         assert(isdefined "y").
      else
         io @println "not matched".
         assert(not isdefined "x").
         assert(not isdefined "y").
      end

The In Predicate
%%%%%%%%%%%%%%%%%%%%

Syntax: ``exp IN list_exp``

This predicate returns ``true`` if the value computed by the expression on the
left in contained in the list computed by the list expression on the right.
It is an error if the expression on the right does not compute a list.

Example,
::
      let true = 1 in [1,2,3].

The Eval Function
%%%%%%%%%%%%%%%%%

The ``eval`` function allows you to evaluate Asteroid expressions.  If the expression
is a string then the contents of the string is treated like Asteroid code and is
interpreted accordingly in the current interpreter environment.  If that code produces a value then the ``eval`` function
will return that value, e.g.,
::
      let a = eval "1+1".
      assert(a == 2).

If the expression to be evaluated is a simple, structural pattern then the pattern is
evaluated as a constructor where variables are instantiated from the current environment.
For example,
::
      let p = pattern (x,y)
      let x = 1.
      let y = 2.
      let o = eval p.
      assert(o is (1,2)).

List Comprehensions
%%%%%%%%%%%%%%%%%%%

Syntax: ``start_exp TO end_exp (STEP exp)?``

This expression constructs a list starting with an element given by the start expression
up to the value of the end expression with a given step.  If the step expression
is not given then a step value of 1 is assumed. The comprehension can be placed between
optional square brackets.

Examples,
::
      let [0,1,2,3,4] = 0 to 4.
      let [0,-2,-4,-6] = [0 to -6 step -2].

Function Calls
%%%%%%%%%%%%%%

Syntax: ``exp exp``

Function calls are defined by function application, more specifically by
juxtaposition of expressions.  Here, the first expression has to evaluated to
a function expression and the second expression has to evaluate to an appropriate
actual function parameter.  Notice that function calls are defined in terms of a
single function parameter.  If you would like to pass more than one value to a
function then you have to create a tuple.  For example, if the function ``foo``
needs two values to be passed to it then you need to create a tuple, e.g. ``foo (1,2)``.
In that respect function calls differ drastically from function calls in languages
like C/C++ or Python.

Examples,
::
      let val = (lambda with i do i+1) 1.
      assert(val == 2).

      function foo with (q,p) do q+p end
      let val = foo (1,2).
      assert(val == 3).

If-Else Expressions
%%%%%%%%%%%%%%%%%%%

Syntax: ``then_exp IF bool_exp ELSE else_exp``

If the boolean expression evaluates to true then this expression returns
the value of the first expression.  Otherwise it will return the value of the
last expression.

Example,
::
      let val = "yup" if b else "nope".

If ``b`` evaluates to true then this expression returns the string ``"yup"``
otherwise it returns the string ``"nope"``.

First-Class Patterns
%%%%%%%%%%%%%%%%%%%%

| Syntax: ``PATTERN exp``
| Syntax: ``'*' exp (BIND '[' ID (AS ID)? (',' ID (AS ID)?)*']')?``

This construction allows the user to construct a pattern as a value using
the ``pattern`` keyword.  The advantage of patterns as values is that they
can be stored in variables or passed to or from functions.  As an example
we construct a pattern which is a pair where the first component is the constant
``1`` and the second component is the variable ``x`` and we store this pattern
in the variable ``p`` for later use,
::
      let p = pattern (1,x).

The pattern derefence operator ``*`` allows us to retrieve patterns from
variables, e.g.
::
      let *p = (1,2).

Here the pair ``(1,2)`` is matched against the pattern stored in the variable ``p``
such that ``x`` is bound to the value ``2``.

The optional ``bind`` term together with an appropriate list of variable names
allows the user to selectively project variable bindings from a constraint pattern
into the current scope.  The ``as`` keyword allows you to rename those bindings.
Consider the following program,
::
      let Pair = pattern %[(x,y)]%.

      -- bindings of the variables x and y are now visible as a and y respetively
      let *Pair bind [x as a, y] = (1,2).
      assert( a == 1).
      assert(y == 2).

At the second  ``let`` statement we bind the ``x`` as ``a`` and ``y`` from the hidden scope
of the constraint pattern into our current scope.

Type Patterns
%%%%%%%%%%%%%

Syntax: ``'%'type_name``

Type patterns match all the values of a particular type.  Type patterns exist
for all the Asteroid builtin types and are also available for user defined
types introduced via a ``structure`` command.

Example,
::
      let true = 1 is %integer.

Named Patterns
%%%%%%%%%%%%%%

Syntax: ``name_exp ':' pattern``

Named patterns allow you to bind the term matched by the pattern to a variable.
Here the name expression has to evaluate to either a variable,
object member variable, or list location.

Example,
::
      let x:%integer = val.

The variable ``x`` will be bound to the value of ``val`` if that value matches the
type pattern ``%integer``.

Named patterns are a syntactic short hand for the equivalent conditional pattern,
::
      name_exp if name_exp is pattern

That means the following two ``let`` statements are equivalent,
::
      let x:(q,p) = (1,2).
      let x if x is (q,p) = (1,2).

Conditional Patterns
%%%%%%%%%%%%%%%%%%%%

Syntax: ``pattern IF cond_exp``

In conditional patterns the pattern only matches if the condition expression
evaluates to true.

Example,
::
      load system math.
      let k if (math @mod(k,2) == 0) = val.

Here ``k`` only matches the value of ``val`` if that value is an even number.

Pure Constraint Patterns
%%%%%%%%%%%%%%%%%%%%%%%%

Syntax: ``%[ pattern ]% (BIND '[' ID (AS ID)? (',' ID (AS ID)?)*']')?``

A pure constraint pattern is a pattern that does not create any bindings
in the current scope.  Any pattern can be turned into a pure constraint pattern
by placing it between the ``%[`` and ``]%`` operators.

Example,
::
      let pos_int = pattern %[(x:%integer) if x > 0]%
      let i:*pos_int = val.

The first line defines a pure constraint pattern for the positive integers.
Notice that the pattern internally uses the variable ``x`` in order to evaluate
the conditional pattern but because it has been declared as a pure constraint
pattern this value binding is not exported to the current scope during pattern matching.
On the second line we constrain the pattern ``i`` to only the positive integer values using
the pure constraint pattern stored in ``p``.  This pattern match will only succeed if ``val``
evaluates to a postive integer.

Asteroid Grammar
^^^^^^^^^^^^^^^^

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
    | exp '.'?

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
    | '%[' exp ']%' binding_list?
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
           (TO exp (STEP exp)?) |
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
    | '*' call_or_index  binding_list?
    | NOT call_or_index
    | MINUS call_or_index
    | PLUS call_or_index
    | ESCAPE STRING
    | EVAL primary
    | '(' tuple_stuff ')'
    | '[' list_stuff ']'
    | function_const
    | TYPEMATCH           // TYPEMATCH == '%'<typename>



  binding_list
    : BIND binding_list_suffix

  binding_list_suffix
     : binding_term
     | '[' binding_term (',' binding_term)* ']'

  binding_term
    : ID (AS ID)?

  tuple_stuff
    : exp (',' exp?)*
    | empty

  list_stuff
    : exp (',' exp)*
    | empty

  function_const
    : LAMBDA body_defs
Notes on Function Argument Notation
-----------------------------------

Functions in Asteroid are multi-dispatch functions and therefore can be called with a variety
of input configurations.  This is reflected in the documentation of built-in functions and
functions belonging to modules: when a function can be called with different input argument
configurations then the documentation reflects this by providing different argument configuration
separated by a '``|``' symbol.  E.g.,

      list @pop () | ix:%integer

indicating that the list member function ``pop`` can be called either with the empty argument ``()`` or with a
single integer value.

Builtin Functions
-----------------

**getid** x
      Returns a unique id of any Asteroid object as an integer.

**hd** x:%list
      Returns the first element of a list. It is an error to apply this
      function to an empty list.

**isdefined** x:%string
      Returns true if a variable or type name is defined in the
      current environment otherwise it returns false. The variable or type name must be given as a string.
**len** x
      Returns the length of x. The
      function can only be applied to lists, strings, tuples, or structures.

**range**  stop:%integer | (start:%integer, stop:%integer) | (start:%integer, stop:%integer, inc:%integer)
      Compute a list of values depending on the input values:

      1. If only the stop value is given then the list [0 to stop-1] is returned.
      2. If the start and stop values are given then the list [start to stop-1] is returned.
      3. If in addition to the start and stop values the inc values is given then the list [start to stop-1 step inc] is returned.

**tl** x:%list
      Returns the rest of the list without the first element.  It is an
      error to apply this function to an empty list.



List and String Objects
-----------------------

In Asteroid, both ``lists`` and ``strings,`` are treated like objects in the OO sense. Due to this, they have member functions that can manipulate the contents of those objects.

Lists
^^^^^

A **list** is a structured data type that consists of square brackets enclosing
comma-separated values.
Member functions on lists can be called on the data structure directly, e.g.::

   [1,2,3] @length ()

Member Functions
%%%%%%%%%%%%%%%%

list **@append** item
      Adds the item to the end of the list.

list **@clear** ()
      Removes all items from the list.

list **@copy** ()
      Returns a shallow copy of the list.

list **@count** item
      Returns the number of times item appears in the list.

list **@extend** item
      Extend the list by adding all the elements from the item to the list where the item is either a list or a tuple.

list **@filter** f:%function
      Returns a list constructed from those elements for which function f returns true.

list **@index** item | (item, loc(startix:%integer) | (item, loc(startix:%integer, endix:%integer))
      Returns a zero-based index of the first element whose value is equal to item.
      It throws an exception if there is no such item. The argument loc allows you to specify
      startix and endix and are used to limit the search to a particular subsequence of the list.
      The returned index is computed relative to the beginning of the list rather than the startix argument.

list **@insert** (ix:%integer, item)
      Insert the item into the list at the position i.
      This means that ``a@insert(0, x)`` inserts x at the front of the list, and ``a@insert(a@length(), x)`` is equivalent to ``a@append(x)``.

list **@join** join_str:%string
      Turns the list into a string using join_str between the elements.  The string is returned
      as the return value from this function.

list **@length** ()
      Returns the number of elements within the list.

list **@map** f:%function
      Applies the function f to each element of the list in place. The modified list is returned.

list **@member** item
      Returns true only if item exists on the list.

list **@pop** () | ix:%integer
      Removes the item at the given position in the list and returns it. If no index is specified
      removes and returns the last item in the list.

list **@reduce** f:%function | (f:%function, init)
      Reduce the list to a value by applying the function f to all the members of the list. The function f has to be
      a function with two arguments where the first argument is the accumulator.  If no initial
      value is given then the first element of the list is assumed to be the first accumulator value.
      In order to illustrate, we have::

            let value = [1,2] @reduce (lambda with (x,y) do x+y, 0).
            assert(value == 3).

      is equivalent to ::

            let l = [1,2].
            let value = 0.
            for i in range(l@length()) do
                  let value = (lambda with (x,y) do x+y) (value,l@i).
            end
            assert(value == 3).

list **@remove** item
      Removes the first element from the list whose value is equal to item.
      It throws an exception if there is no such item.

list **@reverse** ()
      Reverses the elements of the list in place and returns the reversed list.

list **@shuffle** ()
      Creates a random permutation of the list in place and returns the randomized list.

list **@sort** () | reverse:%boolean
      Sorts the items of the list in place and returns the sorted list.
      If the boolean reverse is set to true then the sorted list is reversed.


Strings
^^^^^^^

A string is a sequence of characters surrounded by double quotes.
In Asteroid, single characters are represented as single character strings.
Similar to lists the member functions of strings can be called directly on the
data structure itself, e.g.::

   "Hello there" @length ()

Member Functions
%%%%%%%%%%%%%%%%

string **@explode** ()
      Returns the string as a list of characters.

string **@flip** ()
      Returns a copy of the string with its characters in the reverse order.

string **@index** item:%string | (item:%string, loc(startix:%integer)) | (item:%string, loc(startix:%integer, endix:%integer))
      Returns an integer index of the item in the string or -1 if item was not found.
      The  argument loc allows you to specify startix and endix and are used to limit the search
      to a particular substring of the string. The returned index is computed relative to the beginning
      of the full string rather than the startix.

string **@length** ()
      Returns the number of characters within the string.

string **@replace** (old:%string, new:%string) | (old:%string, new:%string, count:%integer)
      Return a copy of the string with all occurrences of regular expression old replaced by the
      string new. If the argument count is given, only the first count occurrences are replaced.

string **@split** () | sep:%string | (sep:%string, count:%integer)
      Return a list of the words in the string, using sep as the delimiter. If count is given then
      at most count splits are done (thus, the list will have at most count+1 elements). If count is
      not specified or -1, then there is no limit on the number of splits (all possible splits are made).
      Consecutive delimiters are not grouped together and are deemed to delimit empty strings.
      For example::

            let s = "1,,2" @split ",".
            assert (s == ["1", "", "2"]).

      The sep argument may consist of multiple characters.
      For example::

            let s = "1<>2<>3" @split "<>".
            assert (s == ["1", "2", "3"]).

      Splitting an empty string with a specified separator returns ``[""]``.
      If sep is not specified or is None, a different splitting algorithm is applied:
      consecutive whitespace is regarded as a single separator, and the result will contain no empty strings at
      the start or end if the string has leading or trailing whitespace. Consequently, splitting an empty string
      or a string consisting of just whitespace with a none separator returns ``[]``.

string **@tolower** ()
      Returns a copy of the string in all lower case letters.

string **@toupper** ()
      Returns a copy of the string in all upper case letters.

string **@trim** () | what:%string
      Returns a copy of the string with the leading and trailing characters removed.
      The what argument specifies the set of characters to be removed.
      If omitted trim defaults to removing whitespace.
      The what argument is not a prefix or suffix; rather, all combinations of its characters are stripped.



Asteroid Modules
----------------

There are a number of system modules that can be loaded into an Asteroid program using ``load system <module name>``.
The modules are implemented as objects where all the functions of that module are
member functions of that module object. For example, in the case of the ``io`` module
we have ``println`` as one of the member functions.  To call that function::

   load system io.
   io @println "Hello there!".  -- println is a member function of the io module

bitwise
^^^^^^^

This module defines bitwise operations on integers. It supports the following functions,

bitwise **@band** (x:%integer, y:%integer)
      Performs the bitwise AND operation and returns the result as an integer.

bitwise **@bclearbit** (x:%integer, i:%integer)
      Clear the ith bit in x and returns the result as an integer.

bitwise **@blrotate** (x:%integer, i:%integer)
      Performs the bitwise left rotate operation by i bits and returns the result as an integer.

bitwise **@blshift** (x:%integer, y:%integer)
      Performs the bitwise left shift operation where x is shifted by y bits and returns the result as an integer.

bitwise **@bnot** x:%integer
      Performs the bitwise NOT operation and returns the result as an integer.

bitwise **@bor** (x:%integer, y:%integer)
      Performs the bitwise OR operation and returns the result as an integer.

bitwise **@brrotate** (x:%integer, i:%integer)
      Performs the bitwise right rotate operation by i bits and returns the result as an integer.

bitwise **@brshift** (x:%integer, y:%integer)
      Performs the bitwise right shift operation where x is shifted by y bits and returns the result as an integer.

bitwise **@bsetbit** (x:%integer, i:%integer)
      Sets the ith bit in x and returns the result as an integer.

bitwise **@bsize** x:%integer
      Returns the bit size of x.

bitwise **@bxor** (x:%integer, y:%integer)
      Performs the bitwise XOR operation and returns the result as an integer.


hash
^^^^

This module implements a hash for key-value pairs. It supports the following functions,

hash **@hash** ()
      Returns a new hash object of type __HASH__.

__HASH__ **@aslist** ()
      Returns the hash as a list of key-value pairs.

__HASH__ **@get** key
      Return the value associated with the given key as long as it can be found otherwise an exception will be thrown.

__HASH__ **@insert** (key, value) | pairs:%list
      Given a pair of the format (key, value) insert it into the table.  Given a list
      of the format::

            [(key1, val1), (key2, val2), ...]

      insert all the key-value pairs on the list into the hash.

io
^^

This module implements Asteroid's I/O system. The module defines three I/O streams,

1. __STDIN__ - the standard input stream.
2. __STDOUT__ - the standard output stream.
3. __STDERR__ - the standard error stream.

Furthermore, the module supports the following functions,

io **@close** file:%\_\_FILE\_\_
      Closes the file where file is a file descriptor of type \_\_FILE\_\_.

io **@input** () | prompt:%string
      Ask the user for input from __STDIN__.  The input is returned as a string. If prompt is given it is printed and then input is read from terminal.

io **@open** (name:%string, mode:%string)
      Returns a file descriptor of type \_\_FILE\_\_.
      The mode string can be "r" when the file will only be read,
      "w" for only writing (an existing file with the same name will be erased),
      and "a" opens the file for appending; any data written to the file is
      automatically added to the end.
      Finally,  "r+" opens the file for both reading and writing.

io **@print** item
      Prints item to the terminal (__STDOUT__). No implicit newline is appended to the output.

io **@println** item
      Prints item to the terminal (__STDOUT__) with an implicit newline character.

io **@read** () | file:%\_\_FILE\_\_
      Read a file and return the contents as a string. If no file is given the __STDIN__ stream is read.

io **@readln** () | file:%\_\_FILE\_\_
      Reads a line of input from a file and returns it as a string. If no file is given the __STDIN__ stream is read.

io **@write** what:%string | (file:%\_\_FILE\_\_, what:%string)
      Write what to a file.  If file is not given then it writes to the __STDOUT__ stream.

io **@writeln** what:%string | (file:%\_\_FILE\_\_, what:%string)
      Write what to a file and append a newline charater.  If file is not given then it writes to  __STDOUT__.


math
^^^^

The math module implements mathematical constants and functions.
An example:
::
    load system io.
    load system math.

    let x = math @sin( math @pi / 2.0 ).
    io @println("The sine of pi / 2 is " + tostring x + ".").
Constants
%%%%%%%%%

math **@pi**
      The mathematical constant π = 3.141592…, to available precision.

math **@e**
      The mathematical constant e = 2.718281…, to available precision.


Power and logarithmic functions
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

math **@exp** x:%integer
      Returns e raised to the power x, where e = 2.718281… is the base of the natural logarithm.

math **@log** x | (x, base:%integer)
      If only argument x is the input, return the natural logarithm of x (to base e).
      If two arguments, (x, base:%integer), are given as input, return the logarithm
      of x to the given base, calculated as log(x)/log(base).

math **@pow** (b, p:%integer)
      Return b raised to the power p.  The return type depends on the type
      of the base.

math **@sqrt** x
      Return the square root of x as a real.


Number-theoretic and representation functions
%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

math **@abs** x
      Return that absolute value of x.  The return type depends on the type of x.

math **@ceil** x:%real
      Returns the ceiling of x: the smallest integer greater than or equal to x.

math **@floor** x:%real
      Returns the floor of x: the largest integer less than or equal to x.

math **@gcd** (a:%integer, b:%integer)
      Returns the greatest common denominator that both integers share.

math **@isclose** (a:%real, b:%real) | (a:%real, b:%real, t:%real)
      Return true if the values a and b are close to each other and false otherwise.
      Default tolerance is 1e-09.  An alternative tolerance can be specified with
      the t argument.

math **@mod** (v,d)
      Implements the modulus operation. Returns the remainder of the quotient v/d.


Trigonometric functions
%%%%%%%%%%%%%%%%%%%%%%%

math **@acos** x
      Returns the arc cosine of x in radians. The result is between 0 and pi.

math **@asin** x
      Returns the arc sine of x in radians. The result is between -pi/2 and pi/2.

math **@atan** x
      Returns the arc tangent of x in radians. The result is between -pi/2 and pi/2.

math **@cos** x
      Returns the cosine of x radians.

math **@sin** x
      Returns the sine of x radians.

math **@tan** x
      Returns the tangent of x radians.

Hyperbolic functions
%%%%%%%%%%%%%%%%%%%%

math **@acosh** x
      Returns the inverse hyperbolic cosine of x.

math **@asinh** x
      Returns the inverse hyperbolic sine of x.

math **@atanh** x
      Returns the inverse hyperbolic tangent of x.

math **@cosh** x
      Returns the hyperbolic cosine of x.

math **@sinh** x
      Returns the hyperbolic sine of x.

math **@tanh** x
      Returns the hyperbolic tangent of x.

Angular conversion
%%%%%%%%%%%%%%%%%%

math **@degrees** x
      Converts angle x from radians to degrees.

math **@radians** x
      Converts angle x from degrees to radians.


os
^^

This module provides a portable way of using operating system dependent functionality.

Process Parameters
%%%%%%%%%%%%%%%%%%

os **@argv**
      The list of command line arguments passed to an Asteroid script.
      argv[0] is the name of the Asteroid script (it is operating
      system dependent whether this is a full pathname or not).
      In interactive mode argv[0] will be the empty string.

os **@env**
      A hash table where keys and values are strings that represent
      the process environment. For example,
            os @env @get "HOME"
      is the pathname of your home directory (on some platforms),
      and is equivalent to getenv("HOME") in C.

os **@platform**
      This string contains a platform identifier.


Functions
%%%%%%%%%

os **@basename** path:%string
      Return the base name of pathname path. This is the second element of the pair
      returned by passing path to the function split. Note that the result of this
      function is different from the Unix basename program; where basename for '/foo/bar/'
      returns 'bar', the basename function returns an empty string ("").

os **@chdir** path:%string
      Change the current working directory to path.

os **@dirname** path:%string
      Return the directory name of pathname path. This is the first element of the
      pair returned by passing path to the function split.

os **@exists** path:%string
      Return true if path refers to an existing path or an open file descriptor.
      Returns false for broken symbolic links. On some platforms, this function
      may return False if permission is not granted to execute stat on
      the requested file, even if the path physically exists.

os **@exit** () | v:%integer | msg:%string
      Signaling an intention to exit the interpreter.
      When an argument value other than none is provided
      it is considered a status value. If it is
      an integer, zero is considered “successful termination” and any
      nonzero value is considered “abnormal termination” by shells and
      the like. Most systems require it to be in the range 0–127, and
      produce undefined results otherwise. Some systems have a
      convention for assigning specific meanings to specific exit codes,
      but these are generally underdeveloped; Unix programs generally
      use 2 for command line syntax errors and 1 for all other kind
      of errors. If none is given as an argument value then is it
      is considered to be a successful exit equivalent to passing a zero.
      If a string is passed then it is printed printed to
      __STDERR__ and results in an exit code of 1. In particular,
      sys.exit("some error message") is a quick way to exit a program
      when an error occurs.

os **@getdir** ()
      Return a string representing the current working directory.

os **@getpathtime** path:%string | (path:%string,flag:%boolean)
      Returns a triple with (creation, access, modification) times.
      By default the return value is a triple of real numbers
      giving the number of seconds since 1/1/1970.  If the flag is set
      to true then a triple of strings is returned where each string
      represents the respective local time. Throws an exception if the file
      does not exist or is inaccessible.

os **@getsize** path:%string
      Return the size, in bytes, of path. Throws exception if the file
      does not exist or is inaccessible.

os **@isfile** path:%string
      Return true if path is an existing regular file. This follows
      symbolic links.

os **@isdir** path:%string
      Return true if path is an existing directory. This follows
      symbolic links.

os **@join** (path1:%string,path2:%string)
      Join path1 and path2 components intelligently. The return value
      is the concatenation of path and any members of *paths with
      exactly one directory separator following each non-empty part
      except the last, meaning that the result will only end in a
      separator if the last part is empty. If the second component is an
      absolute path, the first component is thrown away.

      On Windows, the drive letter is not reset when an absolute
      path component (e.g., r'\foo') is encountered. If a component
      contains a drive letter, all previous components are thrown away
      and the drive letter is reset. Note that since there is a current
      directory for each drive, os.path.join("c:", "foo") represents a
      path relative to the current directory on drive C: (c:foo), not c:\foo.

os **@split** path:%string
      Split the pathname path into a pair, (head, tail) where tail is
      the last pathname component and head is everything leading up to
      that. The tail part will never contain a slash; if path ends in
      a slash, tail will be empty. If there is no slash in path, head
      will be empty. If path is empty, both head and tail are empty.
      Trailing slashes are stripped from head unless it is the root
      (one or more slashes only). Also see the functions dirname and
      basename.

os **@splitdrive** path:%string
      Split the pathname path into a pair (drive, tail) where drive is
      either a mount point or the empty string. On systems which do not
      use drive specifications, drive will always be the empty string.
      In all cases, drive + tail will be the same as path.

      On Windows, splits a pathname into drive/UNC sharepoint and
      relative path.

      If the path contains a drive letter, drive will contain everything
      up to and including the colon.

os **@splitext** path:%string
      Split the pathname path into a pair (root, ext) such that
      root + ext == path, and the extension, ext, is empty or begins
      with a period and contains at most one period. If the path contains
      no extension, ext will be the empty string.

os **@syscmd** cmd:%string
      Execute a command in a subshell. This is implemented
      by calling the Standard C function system, and has the same
      limitations. If command generates any output, it will be
      sent to the interpreter standard output stream.
      The C standard does not specify the meaning of the return value of
      the C function, so the return value of this function is
      system-dependent.




pick
^^^^

The pick module implements
pick objects that allow a user to randomly pick items from a list of items using the pickitems function.
An example:
::
   load system io.
   load system pick.

   let po = pick @pick([1 to 10]).
   let objects = po @pickitems 3.
   io @println objects.
pick **@pick** l:%list
      Construct a pick object of type __PICK__.

__PICK__ **@pickitems** () | n:%integer
      Return items randomly picked from the list l.  If no input is provided
      then pickitems will return a single, randomly picked item from the list.
      If an integer value n is given then a list of n randomly picked items from
      the list l is returned.  The picked item list is constructed by sampling the
      list l with replacement.


random
^^^^^^

The random module implements random number generation.

random **@randint** (lo:%integer,hi:%integer) | (lo:%real,hi:%real)
      Return a random value N in the interval lo <= N <= hi.
      The type of the random value depends on the types of the
      values specifying the interval.  If the interval is specified
      with integers then a random integer value is returned.
      If the interval is specified with real numbers then a real value is
      is returned, and for everything else an exception is thrown.

random **@random** ()
      Return a random real number in the range [0.0, 1.0).

random **@seed** x:%integer
      Provide the seed value x for the random number generator.

set
^^^

The set module implements Asteroid sets as lists.
Unlike lists, sets do not have repeated elements.
Use the set member function toset to turn any list
into a list that represents a set (remove repeated items).

set **@diff** (a:%list,b:%list)
      Return the difference set between sets a and b.

set **@intersection** (a:%list,b:%list)
      Return the intersection of sets a and b.

set **@toset** l:%list
      Return list l as a set by removing repeated elements.

set **@union** (a:%list,b:%list)
      Return the union of sets a and b.

set **@xunion** (a:%list,b:%list)
      Return the elements in a or b but not both.


sort
^^^^

The sort  module
defines a parameterized sort function over a list.
The sort function makes use of a user-defined order predicate on the list's elements to
perform the sort. The QuickSort is the underlying sort algorithm.
The following is a simple example:
::
   load system io.
   load system sort.
   let sl = sort @sort((lambda with (x,y) do true if x<y else false),
                       [10,5,110,50]).
   io @println sl.
prints the sorted list::

  [5,10,50,110]

sort **@sort** (p:%function,l:%list)
      Returns the sorted list l using the predicate p.


stream
^^^^^^

The stream module implements streams that allow
the developer to turn any list into a stream supporting interface functions like peeking ahead or rewinding
the stream.
A simple use case:
::
   load system io.
   load system stream.

   let s = stream @stream [1 to 10].
   while not s @eof() do
      io @print (tostring (s @get()) + " ").
   end
   io @println "".
which outputs::

   1 2 3 4 5 6 7 8 9 10


stream **@stream** l:%list
      Returns a stream object of type __STREAM__.

__STREAM__ **@append** x
      Adds x to the end of the stream.

__STREAM__ **@eof** ()
      Returns true if the stream does not contain any further elements for processing.
      Otherwise it returns false.

__STREAM__ **@get** ()
      Returns the current element and moves
      the stream pointer one ahead.  Returns none if no elements left in stream.

__STREAM__ **@map** f:%function
      Applies function f to each element in the stream.

__STREAM__ **@peek** ()
      Returns the current element available on the stream otherwise it returns none.

__STREAM__ **@rewind** ()
      Resets the stream pointer to the first element of the stream.


type
^^^^

The type module defines type related functions and structures.
Here is a program that exercises some of the string formatting options:
::
    load system io.
    load system math.

    -- if the width specifier is larger than the length of the value
    -- then the value will be right justified
    let b = tostring(true,stringformat(10)).
    io @println b.

    let i = tostring(5,stringformat(5)).
    io @println i.

    -- we can format a string by applying tostring to the string
    let s = tostring("hello there!",stringformat(30)).
    io @println s.

    -- for floating point values: first value is width, second value precision.
    -- if precision is missing then value is left justified and zero padded on right.
    let r = tostring(math@pi,stringformat(6,3)).
    io @println r.
The output of the program is,
::

          true
        5
                      hello there!
     3.142

Notice the right justification of the various values within the given string length.

Type Conversion
%%%%%%%%%%%%%%%

type **@tobase** (x:%integer,base:%integer)
      Represents the given integer x as a numeral string in different bases.

type **@toboolean** x
      Interpret x as a Boolean value.

type **@tointeger** (x:%string,base:%integer) | x
      Converts a given input to an integer. If a base value is specified then
      the resulting integer is in the corresponding base.

type **@toreal** x
      Returns the input as a real number.

type  **@tostring** x | (x,type @stringformat(width:%integer,precision:%integer,scientific:%boolean))
      Converts an Asteroid object to a string. If format values are given,
      it applies the formatting to the string object.



Type Query Functions
%%%%%%%%%%%%%%%%%%%%

type **@islist** x
      Returns true if x is a list otherwise it will return false.

type **@isnone** x
      Returns true if x is equal to the value none.

type **@isscalar** x
      Returns true if x is either an integer or a real value.

type **@gettype** x
      Returns the type of x as a string.

A simple example program using the ``gettype`` function,
::
   let i = 1.
   assert(gettype(i) == "integer").
util
^^^^

The util module defines utility functions and structures that don't really
fit into any other modules.

util **@achar** x
      Given a decimal ASCII code x, return the corresponding character symbol.

util **@ascii** x:%string
      Given a character x, return the corresponding ASCII code of the first character of the input.

util **@cls** ()
      Clears the terminal screen.

util **@copy** x
      Given the object x, make a deep copy of it.

util **@ctime** x:%real
      Given a real value representing seconds since 1/1/1970 this function
      converts it to a suitable string representation of the date.

type **@sleep** x
      Sleep for x seconds where the x is either an integer or real value.

type **@time** ()
      Returns the local time as a real value in secs since 1/1/1970.

type **@unzip** x:%list
      Given a list of pairs x this function will return a pair of lists
      where the first component of the pair is the list of all the first
      components of the pairs of the input list and the second component
      of the return list is a list of all the second components of the input list.

type **@zip** (list1:%list,list2:%list)
      Returns a list where element i of the list is the tuple (list1@i,list2@i).

vector
^^^^^^

The vector defines functions useful for vector arithmetic. Vectors are implemented as lists.
Here is a simple example program for the ``vector`` module:
::
   load system io.
   load system vector.

   let a = [1,0].
   let b = [0,1].

   io @println (vector @dot (a,b)).
which prints the value ``0``.

vector **@add** (a:%list,b:%list)
      Returns a vector that contains the element by element sum of the input vectors a and b.

vector **@dot** (a:%list,b:%list)
      Computes the dot product of the two vectors a and b.

vector **@mult** (a:%list,b:%list)
      Returns the element by element vector multiplication of vectors a and b.

vector **@op** (f:%function,a:%list,b:%list) | (f:%function,a:%list,b if type @isscalar(b)) | (f:%function,a if type @isscalar(a),b:%list)
      Allows the developer to vectorize any function f. Applying scalar values
      to vectors is also supported by this function.

vector **@sub** (a:%list,b:%list)
      Returns the element by element difference vector.


Interfacing Asteroid with Python
--------------------------------

Asteroid allows integration with Python in one of two ways.  First, we can call the
Asteroid interpreter from within a Python program and second, we can embed
Python code directly within an Asteroid program. We start with looking at
calling the Asteroid interpreter from Python.

Calling Asteroid from Python
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Calling Asteroid from within a Python program is nothing more than calling Asteroid's ``interp``
function with a string representing an Asteroid program as its argument.  In order to make this work you
will have to make sure that the Python interpreter can find the Asteroid modules.
Here we assume that you have installed Asteroid with the ``pip`` installer.
Once you have installed Asteroid you will have to point the ``PYTHONPATH``
environment variable to the directory where ``pip`` installed the Asteroid modules.
You can easily find out where the modules are installed by issuing the ``show`` command,
::
    ubuntu$ pip3 show asteroid-lang
    Name: asteroid-lang
    Version: 1.1.3
    Summary: A pattern-matching oriented programming language.
    Home-page: https://asteroid-lang.org
    Author: University of Rhode Island
    Author-email: lutzhamel@uri.edu
    License: None
    Location: /home/ubuntu/.local/lib/python3.8/site-packages
    Requires: numpy, pandas, matplotlib
    Required-by:
    ubuntu$

The ``Location`` field tells us where the Asteroid modules have been installed.
Under Ubuntu we can now create an environment variable that points to that directory as follows,
::
    ubuntu$ export PYTHONPATH=/home/ubuntu/.local/lib/python3.8/site-packages
    ubuntu$

Now that Python knows how to find the Asteroid modules we can import the
Asteroid interpreter into any Python program using,
::
   from asteroid.interp import interp

where the ``interp`` function takes a string representing of an Asteroid program
as an argument.  Let's test drive this in the Python interactive shell,
::
    ubuntu$ python3
    Python 3.8.10 (default, Nov 26 2021, 20:14:08)
    [GCC 9.3.0] on linux
    Type "help", "copyright", "credits" or "license" for more information.
    >>> from asteroid.interp import interp
    >>> interp('load system io. io @println "Hello, World!".')
    Hello, World!
    >>>

For more detailed information on the ``interp`` function do a ``help(interp)``
at the interactive Python prompt.
Even though we have shown this example under Linux, analogous approaches
should work on both Windows and macOS.

Not only can we execute the Asteroid interpreter
from Python but we can also access its state to look up the results of a
computation for example.  Here is a slight variation of the program above
where the Asteroid program computes the string value containing the greeting but
we are actually printing the value from Python,
::
      # import Asteroid modules
      from asteroid.interp import interp
      from asteroid.state import state

      # run the interpreter to compute the greeting string
      interp('let s = "Hello World!".')

      # retrieve the greeting string from the interpreter state
      # notice the pair of values a symbol table lookup produces:
      # one for the type of the value and one for the actual value
      (type,val) = state.symbol_table.lookup_sym('s')
      print(type)
      print(val)

The program prints out,
::
      string
      Hello World!



Embedding Python into an Asteroid Program
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Using Asteroid's ``escape`` expression allows us to embed arbitray Python
code into an Asteroid program,
::
      -- Printing hello once from each environment

      -- print hello from Asteroid
      load system io.
      io @println "Hello World from Asteroid!".

      -- print hello from Python
      escape
      "
      print('Hello World from Python!')
      ".

Please note that the format of the Python code in the escaped string should follow the
same guidelines as the Python code embedded in strings handed to the Python `exec
function <https://docs.python.org/3/library/functions.html#exec>`_.

Not only does the ``escape`` expression give you access to the Python environment but
it also gives you access to the current Asteroid interpreter state including its
symbol table.  That means we can access any variable defined in the Asteroid
environment from Python,
::
      let s = "Hello World!".

      escape
      "
      (type, val) = state.symbol_table.lookup_sym('s')
      print(type)
      print(val)
      ".

Notice that a symbol table lookup produces a pair of values where the first value
represents the type of the value stored in the symbol table and the second value
is the actual value stored.  In this case our program prints out,
::
      string
      Hello World!

That is the type of the value is a string and the value is the actual string ``Hello World!``.

Since ``escape`` represents an expression we can also return values from the
Python code using a special ``__retval__`` variable.  The only trick is that
we have to remember that values in Asteroid are pairs consisting of type information
and values.  Here is a very simple program that exercises that part of the Python API,
::
      load system io.

      let i = escape
      "
      global __retval__  # access the return value register

      __retval__ = ('integer', 101)
      ".

      io @println i.

This program will print out the value ``101`` from Asteroid even though that value
was created within the Python environment.  Notice that we have to access the
return value register ``__retval__`` with the ``global`` statement in the Python code.

We can pull all of this together and write an Asteroid function that performs its
computations in Python,
::
      function inc with i do return escape
      "
      # access return value register
      global __retval__
      # lookup the value of the formal argument
      (type, val) = state.symbol_table.lookup_sym('i')

      # only perform the increment if the value is an integer
      if type != 'integer':
         raise ValueError('not an integer')
      else:
         __retval__ = (type, val+1)
      ".
      end

      -- call inc and make sure the result is correct
      let k = inc(1)
      assert(k == 2).

Of course the function is just an illustration of how to use the Python API.  This
type of computation is much easier to express in Asteroid directly,
::
      function inc
         with i:%integer do
            i+1
         end

      let k = inc(1)
      assert(k == 2).

The Foreign Type Tag
^^^^^^^^^^^^^^^^^^^^

When working in the hybrid Asteroid-Python environment it is sometimes useful to be able to embed values
in an Asteroid program that have no direct representation in Asteroid.  This is where the ``foreign``
type tage comes into play.  Consider the following program that uses Pandas dataframes within an
Asteroid program,
::

      ------------------------------------------------------------------------
      function pack
      ------------------------------------------------------------------------
      -- this function packs four real values into a Pandas dataframe
      with (a:%real,b:%real,c:%real,d:%real) do return escape
      "
      global __retval__
      # we can ignore type info here because we checked it above
      (_, aval) = state.symbol_table.lookup_sym('a')
      (_, bval) = state.symbol_table.lookup_sym('b')
      (_, cval) = state.symbol_table.lookup_sym('c')
      (_, dval) = state.symbol_table.lookup_sym('d')

      import pandas as pd
      df = pd.DataFrame({'x':[aval,bval], 'y':[cval,dval]})
      __retval__ = ('foreign', df)
      "
      end

      ------------------------------------------------------------------------
      function dump
      ------------------------------------------------------------------------
      -- dump the Pandas dataframe to stdout
      with df do escape
      "
      (dftype, dfval) = state.symbol_table.lookup_sym('df')
      if dftype != 'foreign':
         raise ValueError('expected data frame')
      print(dfval)
      "
      end

      ------------------------------------------------------------------------
      function access
      ------------------------------------------------------------------------
      -- access an element of the Pandas dataframe at row r and column c
      with (df,r:%integer,c:%integer) do return escape
      "
      global __retval__
      (dftype, dfval) = state.symbol_table.lookup_sym('df')
      if dftype != 'foreign':
         raise ValueError('expected data frame')
      # we can ignore type info here because we checked it above
      (_, rval) = state.symbol_table.lookup_sym('r')
      (_, cval) = state.symbol_table.lookup_sym('c')
      # make sure the ret value conforms to the Asteroid value structure
      __retval__ = ('real', dfval.iloc[rval,cval])
      "
      end

      ------------------------------------------------------------------------
      function sum
      ------------------------------------------------------------------------
      -- sum down the columns of the dataframe and return a pair of values,
      -- one component for each column
      with (df) do return escape
      "
      global __retval__
      (dftype, dfval) = state.symbol_table.lookup_sym('df')
      if dftype != 'foreign':
         raise ValueError('expected data frame')
      # sum the value down the columns
      sum = list(dfval.sum(axis=0))
      # construct our tuple, note the type information
      __retval__ = ('tuple', [('real',sum[0]),('real',sum[1])])
      "
      end

      ------------------------------------------------------------------------
      -- exercise our machinery
      let df = pack(1.0,2.0,3.0,4.0).
      dump(df).
      assert(access(df,1,1) == 4).
      assert(sum(df) == (3.0,7.0)).

The ``dump`` function generates the following output,
::
           x    y
      0  1.0  3.0
      1  2.0  4.0

Pandas dataframes are not directly usable in Asteroid but by writing thin Python
wrappers and taking advantage of the ``escape`` expression the ``foreign`` type
tag we can embed Pandas functionality into Asteroid.  As an additional step we could
wrap these individual functions into a ``structure`` with the dataframe as
a data member and the functions as member functions of that structure.  As an
example of this approach see the `dataframe.ast <https://github.com/asteroid-lang/asteroid/blob/master/asteroid/modules/dataframe.ast>`_ system module.
