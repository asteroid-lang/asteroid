



..
   *** DO NOT EDIT; MACHINE GENERATED ***



.. highlight:: none

Asteroid User Guide
===================

Introduction
------------

Asteroid is a multi-paradigm programming language supporting first-class patterns.
The language is heavily influenced by `Python <https://www.python.org>`_, `Rust <https://www.rust-lang.org>`_, `ML <https://www.smlnj.org>`_, and `Prolog <http://www.swi-prolog.org>`_, and makes pattern matching one of its core computational mechanisms.  We often refer to this as pattern-matching oriented programming. When we talk about pattern matching we mean structural pattern matching
as well as regular expression matching.

In this document we describe the major features of Asteroid and give plenty of examples.  If you have used a programming language like Python or JavaScript before, then Asteroid should appear very familiar.  However, there are some features which differ drastically from other programming languages due to the core pattern-matching programming
paradigm.  Here are just two examples:

**Example:** All statements that look like assignments are actually pattern-match statements.  For example if we state,
::

    let [x,2,y] = [1,2,3].


that means the subject term ``[1,2,3]`` is matched to the pattern ``[x,2,y]`` and ``x`` and ``y`` are bound to the values 1 and 3, respectively.  By the way, there is nothing wrong with the following statement,
::

    let [1,2,3] = [1,2,3].


which is just another pattern match without any variable instantiations.

**Example:** Patterns in Asteroid are first-class citizens of the language.
This is best demonstrated with a program.  Here is a program
that recursively computes the factorial of a positive integer and uses first-class patterns
in order to ensure that the domain of the function is not violated,
::

    -- define first-class patterns
    let POS_INT = pattern (x:%integer) if x > 0.
    let NEG_INT = pattern (x:%integer) if x < 0.

    -- define our factorial function
    function fact
        with 0 do
            return 1
        with n:*POS_INT do            -- use first pattern
            return n * fact (n-1).
        with n:*NEG_INT do            -- use second pattern
            throw Error("undefined for "+n).
        end


As you can see, the program first creates patterns and stores them in the variables
``POS_INT`` and ``NEG_INT`` and it uses those patterns later in the code by
dereferencing those variables with the ``*`` operator.  First-class patterns have
profound implications for software development in that pattern definition and usage
points are now separate and patterns can be reused in different contexts.

These are just two examples where Asteroid differs drastically from other programming languages.
This document is an overview of Asteroid and is intended to get you started quickly
with programming in Asteroid.


The Basics
----------

As with most languages we are familiar with, Asteroid has **variables** (alpha-numeric symbols starting with an alpha character) and **constants**.  Constants are available for all the **primitive data types**,

* ``integer``, e.g. ``1024``
* ``real``, e.g. ``1.75``
* ``string``, e.g. ``"Hello, World!"``
* ``boolean``, e.g. ``true``

Asteroid arranges these data types in a **type hierarchy**,

``boolean`` < ``integer`` < ``real`` < ``string``

Type hierarchies facilitate automatic type promotion.  Here is an example
where automatic type promotion is used to put together a string from different data types,
::

    let x:%string = "value: " + 1.


Here we associate the string ``"value: 1"`` with the variable ``x`` by first promoting the integer value ``1`` to the string ``"1"`` using the fact that ``integer`` < ``string``  according to our type hierarchy  and then interpreting the ``+`` operator as a string concatenation operator.

Asteroid supports two more data types:

* ``list``
* ``tuple``

These are **structured data types** in that they can contain entities that belong to other data types. Both of these data types have constructors which are possibly empty sequences of comma separated values enclosed by square brackets for lists, e.g. ``[1,2,3]``, and enclosed by parentheses for tuples, e.g. ``(x,y)``. For tuples we have the caveat that the 1-tuple is represented by a value followed by a comma to distinguish it from parenthesized expressions, e.g. ``(3,)``.
Here are some examples,
::

    let l = [1,2,3].  -- this is a list
    let t = (1,2,3).  -- this is a tuple


As we said above, in order to distinguish it from a parenthesized value the single element in a 1-tuple has to be followed by a comma, like so,
::

    let one_tuple = (1,).  -- this is a 1-tuple


Lists and tuples themselves are also embedded in type hierarchies, although very simple ones:

* ``list`` < ``string``
* ``tuple`` < ``string``

That is, any list or tuple can be viewed as a string.  This is very convenient for printing lists and tuples,
::

    load system io.
    io @println ("this is my list: " + [1,2,3]).



Finally, Asteroid supports one more type, namely the ``none`` type.  The ``none`` type has
only one member: A constant named ``none``.  However, it turns out that the null-tuple, a tuple with no components
indicated by ``()``, also belongs to this type rather than the tuple type discussed earlier. But the ``none``
data type only has one constant, this implies that ``()`` and ``none`` mean the same thing and can be used
interchangeably.  That is, the following ``let`` statements will succeed,
::

    let none = ().
    let () = none.


showing that ``()`` and ``none`` are equivalent and pattern-match each other.
The ``none`` data type itself does not belong to any type hierarchy.

By now you probably figured out that statements are terminated with a period and that comments start with a ``--`` symbol and continue till the end of the line.  You probably also figured out that the ``let`` statement is Asteroid's version of assignment even though the underlying mechanism is a bit different.

Data Structures
---------------

Lists
^^^^^

In Asteroid the ``list`` is a fundamental, built-in data structure.  A trait it shares with programming languages such as Lisp, Python, ML, and Prolog.  Below is a list reversal example program.  Notice that lists are zero-indexed and
elements of a list are accessed via the ``@`` operator,
::

    load system io.    -- load the io module so we can print

    let a = [1,2,3].             -- construct list a
    let b = [a @2, a @1, a @0].  -- reverse list a
    io @println b.


The output is: ``[3,2,1]``.

We can achieve the same effect by giving a list of index values (a slice) to the ``@`` operator,
::

    load system io.    -- load the io module so we can print

    let a = [1,2,3].     -- construct list a
    let b = a @[2,1,0].  -- reverse list a using slice [2,1,0]
    io @println b.


In Asteroid lists are considered objects with member functions that can manipulate list
objects. We could rewrite the above example as,
::

    load system io.

    let a = [1,2,3].
    let b = a @reverse(). -- reverse list using member function 'reverse'
    io @println b.


The ``@`` operator allows you to access either individual elements, slices, or member functions of a list.
Actually, the ``@`` operator is more general than that, it is Asteroid's substructure access operator.
Notice that in order to access the ``println`` function of the ``io`` module we also use the ``@`` operator.
This is because in Asteroid, **system modules are objects**, so you must use the ``@`` to access the functions
of the module.

For a comprehensive treatment of available member functions for Asteroid lists please see the reference guide.

Besides using the default constructor for lists which consists of the
square brackets enclosing a list of elements we can use **list comprehensions** to construct lists.  In Asteroid a list comprehension consist of a range specifier together with
a step specifier allowing you to generate integer values within that range,
::

    load system io.

    -- build a list of odd values
    let a = [1 to 10 step 2].  -- list comprehension
    io @println ("list: " + a).

    -- reverse the list using a slice computed as comprehension
    let slice = [4 to 0 step -1]. -- list comprehension
    let b = a @slice.
    io @println ("reversed list: " + b).


The output is,
::

    list: [1,3,5,7,9]
    reversed list: [9,7,5,3,1]

Asteroid's simple list comprehensions in conjunction with the ``map`` function for lists allows you to
construct virtually  any kind of list. For example, the following program constructs
a list of alternating 1 and -1,
::

    load system io.
    load system math.

    let a = [1 to 10] @map(lambda with x do return math @mod(x,2))
                      @map(lambda with x do return 1 if x else -1).

    io @println a.


where the output is,
::

    [1,-1,1,-1,1,-1,1,-1,1,-1]

Higher dimensional arrays can easily be simulated with lists of lists,
::

    load system io.

    -- build a 2-D array
    let b = [[1,2,3],
             [4,5,6],
             [7,8,9]].

    -- modify an element in the array
    let b @1 @1 = 0.
    io @println ("["+b@0+"\n "+b@1+"\n "+b@2+"]").


The output is,
::
    [[1,2,3]
     [4,0,6]
     [7,8,9]]


Tuples
^^^^^^

As we saw earlier, the ``tuple`` is another fundamental, built-in data structure that can be found in Asteroid.
Below is an example of a tuple declaration and access.
::

    let a = (1,2,3).    -- construct tuple a
    let b = a @1.       -- access the second element in tuple a
    assert(b == 2).     -- assert that the value of the second element is correct


Lists and tuples may be nested,
::

    -- build a list of tuples
    let b = [("a","b","c"),
             ("d","e","f"),
             ("g","h","i")].
    -- Access an element in the nested structure.
    assert(b @1 @1 == "e").


Unlike lists, tuples are immutable. This means that their contents cannot be changed once they have been declared.  The following code block demonstrates this,
::

    load system io.

    let b = ("a","b","c"). -- build a tuple
    
    try
        let b @1 = "z". -- attempt to modify an element in the tuple
    catch Exception(kind,message) do
        io @println (kind+": "+message).
    end.


Which will print out the following message:
::

    SystemError: term '(a,b,c)' is not a mutable structure

Should we want to change the contents of an already declared tuple, we would need to abandon the original and create a new one with the updated contents.
When to use tuples and when to use lists is really application dependent.
Tuples tend to be preferred over lists when representing some sort of structure,
like abstract syntax trees, where that structure is immutable meaning, for example,
that the arity of a tree node cannot change.

Custom Data Structures
^^^^^^^^^^^^^^^^^^^^^^

You can introduce custom data structures using the ``structure`` keyword.  For example,
the following statement introduces a structure of type ``A`` with data members ``a`` and ``b``,
::
    structure A with
       data a.
       data b.
    end

These custom data structures differ from lists and tuples in the sense that the name of the structure acts like a type tag.  So, when you define a new structure you are in fact introducing a new type into your program.   Asteroid creates
a *default constructor* that instantiates an object from a given structure.  A default constructor copies the arguments given to it into the
data member fields in the order that the data members appear in the
structure definition and as they appear in the parameter list of the constructor.
Also, the data fields of an object are accessed via
their names rather than index values.  Here is a simple example that illustrates
all this,
::

    -- define a structure of type A
    structure A with
        data a.
        data b.
    end

    let obj = A(1,2).       -- call constructor
    assert( obj @a == 1 ).  -- access first data member
    assert( obj @b == 2 ).  -- access second data member


The following is a more involved example,
::

    load system io.

    structure Person with
        data name.
        data age.
        data gender.
    end

    -- make a list of persons
    let people = [
        -- use default constructors to construct Person instances
        Person("George", 32, "man"),
        Person("Sophie", 46, "woman"),
        Person("Oliver", 21, "man")
    ].

    -- retrieve the second person on the list and use pattern
    -- matching to extract member values
    let Person(name,age,gender) = people @1.

    -- print out the member values
    io @println (name + " is " + age + " years old and is a " +  gender + ".").


The output is,
::

    Sophie is 46 years old and is a woman.


The ``structure`` statement introduces a data structure of type ``Person`` with the three data members ``name``, ``age``, and ``gender``.  We use this data structure to build a list of persons.  One of the interesting things is that we can pattern match the generated data structure as in the second ``let`` statement in the program to extract
information from a ``Person`` object.

In addition to the default constructor, structures in Asteroid also support user specified
constructors and member functions.  We'll talk about those later when we talk about OO programming in Asteroid.

The Let Statement
-----------------

The ``let`` statement is a pattern matching statement and can be viewed as Asteroid's version of the assignment statement even though statements like,
::

    let 1 = 1.

where we take the term on the right side and match it to the pattern on the left side of
the ``=`` operator are completely legal and highlight the fact that ``let`` statement is not equivalent to an assignment statement.  Simple patterns are expressions that consist purely of constructors and variables. Constructors themselves consist of constants, list and tuple constructors, and user defined structures.
Here is an example where we do some computations on the right side of a ``let`` statement and then match the result against a pattern on the left,
::

    load system io.

    let [x,2,y] = [1+0,1+1,1+2].
    io @println (x,y).


The output is: ``(1,3)``

Asteroid supports special patterns called **type patterns** that match any value
of a given type.  For instance, the ``%integer`` pattern matches any integer value.  Here is a simple example,
::

    let %integer = 1.


This ``let`` statement succeeds because ``1`` is an integer value can be pattern-matched against
the type pattern ``%integer``.

Asteroid also
supports something called a **named pattern** were a (sub)pattern on the left side
of a ``let`` statement (or any pattern as it appears in Asteroid) can be given a name
and that name will be instantiated with a term during pattern matching.  For example,
::

    load system io.

    let t:(1,2) = (1,2).  -- using a named pattern on lhs
    io @println t.


Here, the construct ``t:(1,2)`` is called a named pattern and the variable ``t`` will be unified with the term ``(1,2)``, or more generally, the variable will be unified with term
that matches the pattern on the right of the colon.  The program will print,
::

    (1,2)

We can combine type patterns and named patterns to give us something that looks
like a variable declaration in other languages. In Asteroid, though, it is still just all
about pattern matching.  Consider,
::

    load system io.
    load system math.
    load system type.

    let x:%real = math @pi.
    io @println (type @tostring (x,type @stringformat (4,2))).


The left side of the ``let`` statement is a named type pattern that matches any real value, and
if that match is successful then the value is bound to the variable ``x``.  Note
that even though this looks like a declaration, it is in fact a pattern matching
operation.  The program will print the value ``3.14`` according to the format of
4 characters with 2 characters after the decimal point.

Flow of Control
---------------

Control structure implementation in Asteroid is along the lines of any of the modern programming languages such as Python, Swift, or Rust.  For example, the ``for`` loop allows you to iterate over lists without having to explicitly define a loop index counter. In addition, the ``if`` statement defines what does or does not happen when certain conditions are met in a very familiar way. For a list of all control statements in Asteroid, please take a look at the reference guide.

As we said, in terms of flow of control statements there are really not a lot of surprises. This is because Asteroid supports loops and conditionals in a very similar way to many of the other modern programming languages.  For example, here is a short program with a ``for`` loop that prints out the first six even positive integers,
::

    load system io.

    for i in 0 to 10 step 2 do
        io @println i.
    end


The output is,
::

    0
    2
    4
    6
    8
    10

Here is another example that iterates over lists,
::

    load system io.
    load system util

    for (ix,bird) in util @zip (["first","second","third"],["turkey","duck","chicken"]) do
        io @println ("the "+ix+" bird is a "+bird).
    end


The output is,
::

    the first bird is a turkey
    the second bird is a duck
    the third bird is a chicken

Here we first create a list of pairs using the ``zip`` function, over which we then
iterate pattern matching on each of the pairs on the list with the pattern ``(ix,bird)``.

The following is a short program that demonstrates an ``if`` statement,
::

    load system io.
    load system type.

    let x = type @tointeger (io @input "Please enter an integer: ").

    if x < 0 do
        let x = 0.
        io @println "Negative, changed to zero".
    elif x == 0 do
        io @println "Zero".
    elif x == 1 do
        io @println "One".
    else do
        io @println "Something else".
    end


Even though Asteroid's flow of control statements look so familiar, they support pattern matching to a degree not found in other programming languages and which we will take a look at below.

Functions
---------

Functions in Asteroid resemble function definitions in functional programming languages such as Haskell and ML.
Here functions have a single formal argument and function calls are expressed via juxtaposition
of the function name and the actual argument.  Here is a simple example,
::

    function double
        with i do -- pattern match the actual arg with i
            return 2*i.
        end

    let d = double 2.  -- function call via juxtaposition
    assert( d == 4 ).


In the ``with`` statement we pattern match the actual argument that is
being passed in against the variable ``i``.  Also note that the function call is expressed via juxtaposition,
no parentheses necessary.

If we wanted to pass more than a single value to a function we have to create
a tuple and then pass that tuple to the function like in this example,
::

    function reduce
        with (a,b) do -- pattern match the actual argument
            return a*b.
        end

    let r = reduce (2,4).  -- function call via juxtaposition
    assert( r == 8 ).


Even though the function call looks like a traditional function call like in
Python it is not.  The underlying mechanism is quite different: on the call site
we construct a tuple that holds all our values which is then passed to the function as the only parameter.
Within the function that tuple is pattern matched and whatever variables are
instantiated during this pattern match can be used within the function body.

In Asteroid functions are multi-dispatch, that is,
a single function can have multiple bodies each attached to a different pattern
matching the formal argument.
The following is the quick sort implemented in
Asteroid where each ``with`` clause introduces a new function body with its
corresponding pattern,
::

    load system io.

    function qsort
        with [] do -- empty list pattern
            return [].
        with [a] do -- single element list pattern
            return [a].
        with [pivot|rest] do -- separating the list into pivot and rest of list
            let less=[].
            let more=[].

            for e in rest do
                if e < pivot do
                    let less = less + [e].
                else
                    let more = more + [e].
                end
            end

            return qsort less + [pivot] + qsort more.
        end

    -- print the sorted list
    io @println (qsort [3,2,1,0])


The output is as expected,
::

    [0,1,2,3]

Notice that we use the multi-dispatch mechanism to deal with the base cases in the first two ``with`` clauses.
In the third ``with`` clause we use the pattern ``[pivot|rest]`` to match the input list.
Here the variable ``pivot`` matches the first element of the list, and the variable ``rest`` matches the remaining list. This remaining list is the original list with its first element removed.
The function body then implements the pretty much standard recursive definition of the
quick sort.  Just keep in mind that function calls are expressed via juxtaposition
of function name and actual argument; no parentheses necessary.

As you have seen in a couple of occasions already in the document, Asteroid also supports anonymous or ``lambda`` functions.  Lambda functions behave just like regular
functions except that you declare them on-the-fly and they are declared without a
name.  Here is an example using a ``lambda`` function,
::

    load system io.

    io @println ((lambda with n do return n+1) 1).


The output is ``2``.  Here, the lambda function is a function that takes a value
and increments it by one.  We then apply the value ``1`` to the function and the
print function prints out the value ``2``.

Pattern Matching
----------------

Pattern matching lies at the heart of Asteroid.  We saw some of Asteroid's pattern matching ability when we discussed the ``let`` statement.  We can also have pattern matching
in expressions using the ``is`` predicate.

Pattern Matching in Expressions: The Is Predicate
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Consider the following example of this predicate among some patterns,
::

    load system io.

    let p = (1,2).

    if p is (x,y,z) do
        io @println ("it's a triple with: "+x+","+y+","+z)
    elif p is (x,y) do
        io @println ("it's a pair with: "+x+","+y).
    else do
        io @println "it's something else".
    end


Here we use patterns to determine if ``p`` is a triple, a pair, or something else. Pattern matching is embedded in the expressions of the ``if`` statement using the ``is`` predicate. The
output of this program is,
::

    it's a pair with: 1,2

Pattern matching with the ``is`` predicate can happen anywhere expressions can
be used.  That means we can use the predicate also in ``let`` statements,
::

    let true = (1,2) is (1,2).

This is kind of strange looking but it succeeds.  Here the
left operand of the ``is`` predicate is a term and
the right operand is a pattern.  Obviously this pattern match will succeed because the
term and the pattern look identical.  The return value of the ``is`` predicate is then
pattern matched against the ``true`` value on the left of the ``=`` symbol.

We can also employ pattern matching in loops.
In the following program we use the ``is`` predicate to test whether a list is empty or not
while looping,
::

    load system io.

    let list = [1,2,3].

    repeat do
        let [head|tail] = list. -- pattern match with head/tail operator
        io @println head.
        let list = tail.
    until list is []. -- pattern match with is predicate


The output is,
::

    1
    2
    3

In addition, the example employs pattern matching using the head-tail operator  in order to iterate over the list elements and print print them.  The
termination condition of the loop is computed with the ``is`` predicate.

Pattern Matching in Function Arguments
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

As we have seen earlier, Asteroid supports pattern matching on function arguments in the style of ML and many other functional programming languages.
Here is an example that uses pattern matching on function arguments using custom data structures.  The program below implements `Peano addition <https://en.wikipedia.org/wiki/Peano_axioms#Addition>`_ on terms using the two Peano axioms,
::

    x + 0 = x
    x + s(y) = s(x+y)

Here ``x`` and ``y`` are variables, ``0`` represents the natural number with value zero, and ``s`` is the successor function.  In Peano arithmetic any natural number can be represented by the appropriate number of applications of the successor function to the natural number ``0``. Here is the program that implements
the Peano arithmetic based on the two axiom where we replaced the ``+`` operator with the
``add`` symbol,
::

    -- implements Peano addition on terms
    load system io.

    structure s with
        data val.
        end

    structure add with
        data left.
        data right.
        end

    function reduce
        with add(x,0) do
            return reduce(x).
        with add(x,s(y))  do
            return s(reduce(add(x,y))).
        with term do
            return term.
        end

    -- add 2 3
    io @println (reduce(add(s(s(0)),s(s(s(0)))))).


Our program defines the structure ``s`` to represent the successor function and the structure ``add`` to represent Peano addition. Next, it defines a function that uses pattern matching to identify the left sides of the two axioms.  If either pattern matches the input to the ``reduce`` function, it will activate the corresponding function body and rewrite the term recursively in an appropriate manner.  We have one additional pattern which matches if neither one of the Peano axiom patterns matches and terminates the recursion.  Finally,  on the last line, we use our ``reduce`` function to compute the Peano term for the addition of 2 + 3. As expected, the output of this program is,
::

    s(s(s(s(s(0)))))

which represents the value 5.

Conditional Pattern Matching
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Asteroid allows the user to attach conditions to patterns that need to hold in order
for the pattern match to succeed.  This is particularly useful for restricting
input values to function bodies.  Consider the following definition of the
``factorial`` function where we use conditional pattern matching to control
the kind of values that are being passed to a particular function body,
::

    load system io.

    function factorial
        with 0 do
            return 1
        with (n:%integer) if n > 0 do
            return n * factorial (n-1).
        with (n:%integer) if n < 0 do
            throw Error("factorial is not defined for "+n).
        end

    io @println ("The factorial of 3 is: " + factorial 3).


Here we see that first, we make sure that we are being passed integers and second,
that the integers are positive using the appropriate conditions on the input values. If
we are being passed a negative integer, then we throw an error.


Pattern Matching in For Loops
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

We have seen pattern matching in ``for`` loops earlier.  Here we show another
example. This combines structural matching with regular expression matching
in ``for`` loops
that selects certain items from a list. Suppose we want to print out the names of persons that contain a lower case 'p',
::

    load system io.

    structure Person with
        data name.
        data age.
        end

    -- define a list of persons
    let people = [
        Person("George", 32),
        Person("Sophie", 46),
        Person("Oliver", 21)
        ].

    -- print names that contain 'p'
    for Person(name:".*p.*",_) in people do
        io @println name.
    end


Here we pattern match the ``Person`` object in the ``for`` loop and then use a regular expression to see if the name of that person matches our requirement that it contains a lower case 'p'.  We can tag the pattern with a variable name, a named pattern, so that we can print out the name if the regular expression matches. The output is ``Sophie``.

Pattern Matching in Try-Catch Statements
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Exception handling in Asteroid is very similar to exception handling in many of the other modern programming languages available today.  The example below shows an Asteroid program  that throws one of two exceptions depending on the randomly generated value ``i``,
::

    load system io.
    load system random.
    load system type.

    structure Head with
        data val.
        end

    structure Tail with
        data val.
        end

    try
        let i = random @random().
        if i >= 0.5 do
            throw Head(i).
        else do
            throw Tail(i).
        end
    catch Head(v) do
        io @println("you win with "+type @tostring(v,type @stringformat(4,2))).
    catch Tail(v) do
        io @println("you loose with "+type @tostring(v,type @stringformat(4,2))).
    end


The ``Head`` and ``Tail`` exceptions are handled by their corresponding ``catch`` statements, respectively.  In both cases the exception object is unpacked using pattern matching and the unpacked value is used in the appropriate message printed to the screen.

It is worth noting that even though Asteroid has builtin exception objects such as ``Error``,
you can construct any kind of object and throw it as part of an exception.


Structures, Object-Based Programming, and Pattern Matching
----------------------------------------------------------

We saw structures such as,
::

    structure Person with
        data name.
        data age.
        data gender.
        end

earlier.  It is Asteroid's way to create custom data structures. These structures
introduce a new type name into a program. For instance, in the case above, the ``structure``
statement introduces the type name ``Person``.   Given a structure definition, we can
create **instances** of that structure.  For example,
::

    let scarlett = Person("Scarlett",28,"F").

The right side of the ``let`` statement invokes the default constructor for the
structure in order to create an instance stored in the variable ``scarlett``. We
can access members of the instance,
::

    load system io.

    structure Person with
        data name.
        data age.
        data gender.
        end

    let scarlett = Person("Scarlett",28,"F").
    -- access the name field of the structure instance
    io @println (scarlett @name).


Asteroid allows you to attach functions to structures.  In member functions
the object identity of the instance is available through the ``this`` keyword.
For example, we can
extend our ``Person`` structure with the ``hello`` function that uses the ``name`` field
of the instance,
::

    load system io.

    structure Person with
        data name.
        data age.
        data gender.
        function hello
            with none do
                io @println ("Hello, my name is "+this @name).
            end
        end

    let scarlett = Person("Scarlett",28,"F").
    -- call the member function
    scarlett @hello().


This program will print out,
::

    Hello, my name is Scarlett

The expression ``this @name`` accesses the ``name`` field of the instance the
function ``hello`` was called on.
Even though our structures are starting to look a bit more like object definitions,
pattern matching continues to work in the same way from when we discussed structures.
The only thing you need to keep in mind is that you **cannot** pattern match on a
function field.  From a pattern matching perspective, a structure consists only of
data fields.  So even if we declare a structure like this,
::

    load system io.

    structure Person with
        data name.
        -- the function is defined in the middle of the data fields
        function hello
            with none do
                io @println ("Hello, my name is "+this @name).
            end
        data age.
        data gender.
        end

    -- pattern matching ignores function definitions
    let Person(name,age,_) = Person("Scarlett",28,"F").
    io @println (name+" is "+age+" years old").


where the function ``hello`` is defined in the middle of the data fields,
pattern matching simply ignores the function definition and pattern matches
only on the data fields.  The output of the program is,
::

    Scarlett is 28 years old

Here is a slightly more involved example based on the
dog example from the `Python documentation <https://docs.python.org/3/tutorial/classes.html>`_.
The idea of the dog example is to have a structure that describes dogs by their
names and the tricks that they can perform.  Tricks can be added to a particular
dog instance by calling the ``add_trick`` function.  Rather than using the default
constructor, we define a constructor for our instances with the ``__init__`` function.
Here is the program listing for the example in Asteroid,
::

    load system io.
    load system type.

    structure Dog with

       data name.
       data tricks.

       function add_trick
          with new_trick:%string do
             this @tricks @append new_trick.
          end

       function __init__
          with name:%string do
             let this @name = name.
             let this @tricks = [].
          end

    end

    let fido = Dog "Fido".
    fido @add_trick "play dead".
    fido @add_trick "fetch".

    let buddy = Dog "Buddy".
    buddy @add_trick "sit stay".
    buddy @add_trick "roll over".

    -- print out all the dogs that know how to fetch
    for (Dog(name,tricks) if type @tostring(tricks) is ".*fetch.*") in [fido,buddy] do
        io @println (name+" knows how to fetch").
    end


After declaring the structure we instantiate two dogs, Fido and Buddy, and add
tricks to their respective trick repertoires.  The last couple of lines
of the program consist of a ``for`` loop over a list of our dogs.
The ``for`` loop is interesting
because here we use structural, conditional, and regular expression pattern
matching in order to only select the dogs that know how to do ``fetch`` from
the list of dogs.  The pattern is,
::

    Dog(name,tricks) if type @tostring(tricks) is ".*fetch.*"

The structural part of the pattern is ``Dog(name,tricks)`` which simply matches
any dog instance on the list.  However, that match is only successful if
the conditional part of the pattern holds,
::

    if type @tostring(tricks) is ".*fetch.*"

This condition only succeeds if the ``tricks`` list viewed as a string matches
the regular expression ``".*fetch.*"``. That is, if the list contains the word ``fetch``.
The output is,
::

    Fido knows how to fetch


Patterns as First-Class Citizens
--------------------------------

A programming language feature that is promoted to first-class status does not
change the power of a programming language in terms of computability but it does
increase its expressiveness.  Think functions as first-class citizens of a programming
language.  First-class functions give us ``lambda`` functions and ``map``, both powerful
programming tools.

The same is true when we promote patterns to first-class citizen status in a language.  It
doesn't change what we can and cannot compute with the language. But it does change how
we can express what we want to compute.  That is, it changes the expressiveness
of a programming language.

In Asteroid first-class patterns are introduced with the keywords ``pattern with``
and patterns themselves are values that we can store in variables and then reference
when we want to use them.  Like so,
::

    let P = pattern (x,y).
    let *P = (1,2).

The left side of the second ``let`` statement dereferences the pattern stored in variable ``P``
and uses the pattern to match against the term ``(1,2)``.

Here we look at three examples of how first-class patterns can add to a developer's
programming toolbox.

Pattern Factoring
^^^^^^^^^^^^^^^^^

Patterns can become very complicated especially when conditional pattern matching
is involved.  First-class patterns allow us to control the complexity of patterns
by breaking patterns up into smaller subpatterns that are more easily managed. Consider
the following function that takes a pair of values.  The twist is that
the first component of the pair is restricted to the primitive data types of
Asteroid,
::

    function foo
        with (x if (x is %boolean) or (x is %integer) or (x is %string),y) do
            io @println (x,y).
        end

That complicated pattern for the first component completely obliterates the
overall structure of the parameter pattern and makes the function definition
difficult to read.

We can express the same function with a first-class pattern,
::

    let TP = pattern
        with q if (q is %boolean) or
                  (q is %integer) or
                  (q is %string).

    function foo
        with (x:*TP,y) do
            io @println (x,y).
        end

It is clear now that the main input structure to the function is a pair and the
conditional type restriction pattern has been relegated to a subpattern stored in the variable
``TP``.

Pattern Reuse
^^^^^^^^^^^^^

In most applications of patterns in programming languages specific patterns appear
in many spots in a program.  If patterns are not first-class citizens the developer
will have to retype the same patterns over and over again in the various different
spots where the patterns occurs. Consider the following program snippet,
::

    function fact
        with 0 do
            return 1
        with (n:%integer) if n > 0 do
            return n * fact (n-1).
        with (n:%integer) if n < 0 do
            throw Error("fact undefined for negative values").
        end

    function sign
        with 0 do
            return 1
        with (n:%integer) if n > 0 do
            return 1.
        with (n:%integer) if n < 0 do
            return -1.
        end


In order to write these two functions we had to repeat the almost identical pattern
four times.  First-class patterns allow us to write the same two functions in a
much more elegant way,
::

    let POS_INT = pattern (x:%integer) if x > 0.
    let NEG_INT = pattern (x:%integer) if x < 0.

    function fact
        with 0 do
            return 1
        with n:*POS_INT do
            return n * fact (n-1).
        with *NEG_INT do
            throw Error("fact undefined for negative values").
        end

    function sign
        with 0 do
                return 1
            with *POS_INT do
                return 1.
            with *NEG_INT do
                return -1.
            end


The relevant patterns are now stored in the variables ``POS_INT`` and ``NEG_INT``
which are then used in the function definitions.

Constraint Patterns
-------------------

Sometimes we want to use patterns as constraints on other patterns.  Consider
the following (somewhat artificial) example,
::

   let x: (v if (v is %integer) and v > 0) = some_value.

Here we want to use the pattern ``v if (v is %integer) and v > 0`` purely as a constraint
on the pattern ``x`` in the sense that we want a match on ``x`` only to succeed
if ``some_value`` is a positive integer.  The problem is that this constraint pattern
introduces a spurious binding of the variable ``v`` into the current environment
which might be undesirable due to variable name clashes.  Our notion of constraint pattern
addresses this.  We can rewrite the above statement as follows,
::

   let x: %[v if (v is %integer) and v > 0]% = some_value.

By placing the pattern ``v if (v is %integer) and v > 0`` within the ``%[...]%``
operators the pattern still functions as before but does not bind the variable ``v``
into the current environment.

The most common use of constraint patterns is the prevention of non-linear patterns
in functions.  Consider the following program,
::

   load system io.

   let POS_INT = pattern %[v if (v is %integer) and v > 0]%.

   function add with (a:*POS_INT,b:*POS_INT) do
      return a+b.
   end

   io @println (add(1,2)).

Without the ``%[...]%`` operators around the pattern ``v if (v is %integer) and v > 0``
the argument list pattern for the function
``(a:*POS_INT,b:*POS_INT)`` would instantiate two instances of the variable ``v``
leading to a non-linear pattern which is not supported by Asteroid.
With the ``%[...]%`` operators in place we prevent
the pattern ``v if (v is %integer) and v > 0`` from instantiating the variable ``v`` thus preventing a non-linearity
to occur in the argument list pattern.

Sometimes we need to use constraint patterns instead of straightforward patterns
in order to avoid non-linearities but
we also want controlled access to the variables these constraint patterns declare.
We achieve this by using the ``bind`` keyword at the pattern-match site.  
Consider the following program,
::

   -- declare a pattern that matches scalar values
   let Scalar = pattern %[p if (p is %integer) or (p is %real)]%.

   -- declare a pattern that matches pairs of scalars
   let Pair = pattern %[(x:*Scalar,y:*Scalar)]%.

   -- compute the dot product of two pairs of scalars
   function dot2d 
      with (*Pair bind [x as a1, y as a2], *Pair bind [x as b1, y as b2]) do
         a1*b1 + a2*b2
   end

   assert(dot2d((1,0),(0,1)) == 0).

In the function definition of ``dot2d`` we see that the ``Pair`` pattern is used 
twice to make sure that the function is called with a pair of pairs as its argument.
However, in order to compute the dot product of those two pairs we need access 
to the values each pair matched.  We use the ``bind`` keyword together with an 
appropriate binding term list to extract the matched values.  For the first
pair we map ``x`` and ``y`` to ``a1`` and ``a2`` and for the second 
pair we map ``x`` and ``y`` to ``b1`` and ``b2``, respectively.

As a quick aside, the ``as`` construction in the binding term list is only necessary 
when trying to resolve non-linearities otherwise the binding term list can just consist
of the variable names appearing in the pattern that you want to bind into the current
scope.

More on Multi-Dispatch
----------------------

With the ``qsort`` function above we saw functional programming style dispatch
where the ``with`` clauses represent a case analysis over a single type, namely
the input type to the function.
However, Asteroid has a much broader view of multi-dispatch where the ``with`` clauses
represent a case analysis over different types.
In order to demonstrate this type of multi-dispatch, we show the example program from the
`multi-dispatch Wikipedia page <https://en.wikipedia.org/wiki/Multiple_dispatch>`_
written in Asteroid,
::

    load system io.
    load system type.

    let pos_num = pattern %[x if type @isscalar(x) and x > 0]%.
   
    structure Asteroid with
       data size.
       function __init_
          with v:*pos_num do
             let this @size = v.
          end
    end

    structure Spaceship with
        data size.
       function __init_
          with v:*pos_num do
             let this @size = v.
          end
    end

    -- we use first-class pattern SpaceObject to
    -- express that both asteroids and space ships are space objects.
    let SpaceObject = pattern %[x if (x is %Asteroid) or (x is %Spaceship)]%.

    -- multi-dispatch function
    function collide_with
      with (a:%Asteroid, b:%Spaceship) do
        return "a/s".
      with (a:%Spaceship, b:%Asteroid) do
        return "s/a".
      with (a:%Spaceship, b:%Spaceship) do
        return "s/s".
      with (a:%Asteroid, b:%Asteroid) do
        return "a/a".
      end

    -- here we use the first-class pattern SpaceObject as a
    -- constraint on the function parameters.
    function collide with (x:*SpaceObject, y:*SpaceObject) do
      return "Big boom!" if (x@size > 100 and y@size > 100) else collide_with(x, y).
    end

    io @println (collide(Asteroid(101), Spaceship(300))).
    io @println (collide(Asteroid(10), Spaceship(10))).
    io @println (collide(Spaceship(101), Spaceship(10))).


Each ``with`` clause in the function ``collide_with`` introduces a new function body with its
corresponding pattern.
The function bodies in this case are simple ``return`` statements
but they could be arbitrary computations.  The output of the program is,
::

    Big boom!
    a/s
    s/s



More on Exceptions
------------------

This section will give further information on how to work with **exceptions**, or unexpected conditions that break the regular flow of execution.  Exceptions generated by Asteroid are ``Exception`` objects with the following structure,
::

    structure Exception with
        data kind.
        data value.
    end

The ``kind`` field will be populated by Asteroid with one of the following strings,

* ``PatternMatchFailed`` - this exception will be thrown if the user attempted an
  explicit pattern match which failed, e.g. a let statement whose left side pattern
  does not match the term on the right side.

* ``NonLinearPatternError`` - this exception occurs when a pattern has more than
  one variable with the same name, e.g. ``let (x,x) = (1,2).``

* ``RedundantPatternFound`` - this exception is thrown if one pattern makes another
  superfluous, e.g. in a multi-dispatch function definition.

* ``ArithmeticError`` - e.g. division by zero

* ``FileNotFound`` - an attempt of opening a file failed.

* ``SystemError`` - a general exception.

In addition to the ``kind`` field, the ``value`` field holds a string with some further details on the exception. Specific exceptions can be caught by pattern matching on the ``kind`` field of the ``Exception`` object.  For
example,
::

    load system io.

    try
        let x = 1/0.
    catch Exception("ArithmeticError", s) do
        io @println s.
    end


The output is,
::

    integer division or modulo by zero


Asteroid also provides a predefined ``Error`` object for user level exceptions,
::

    load system io.

    try
        throw Error("something worth throwing").
    catch Error(s) do
        io @println s.
    end


Of course the user can also use the ``Exception`` object for their own exceptions
by defining a ``kind`` that does not interfere with the predefined ``kind`` strings above,
::

    load system io.

    try
        throw Exception("MyException","something worth throwing").
    catch Exception("MyException",s) do
        io @println s.
    end


The output here is,
::

    something worth throwing

In addition to the Asteroid defined exceptions,
the user is allowed to construct user level exceptions with any kind of object including tuples and lists. Here is an example that constructs a tuple as an exception object,
::

    load system io.

    try
        throw ("funny exception", 42).
    catch ("funny exception", v) do
        io @println v.
    end


The output of this program is ``42``.

Now, if you don't care what kind of exception you catch, you need to use a ``wildcard`` or a variable because exception handlers are activated via pattern matching on the
exception object itself.  Here is an example using a ``wildcard``,
::

    load system io.

    try
        let (x,y) = (1,2,3).
    catch _ do
        io @println "something happened".
    end


Here is an example using a variable,
::

    load system io.
    load system type.

    try
        let (x,y) = (1,2,3).
    catch e do
        io @println ("something happened: "+type @tostring(e)).
    end


In this last example we simply convert the caught exception object into a string
and print it,
::

    something happened: Exception(PatternMatchFailed,pattern match failed: term and pattern
    lists/tuples are not the same length)


Basic Asteroid I/O
------------------

I/O functions are defined as member functions of the ``io`` module. The ``println`` function prints its argument in a readable form to the terminal.  Recall that the ``+`` operator also implements string concatenation.  This allows us to construct nicely formatted output strings,
::

    load system io.

    let a = 1.
    let b = 2.
    io @println ("a + b = " + (a + b)).


The output is
::

    a + b = 3

We can use the ``tostring`` function defined in the ``type`` module to provide some
additional formatting. The idea is that the ``tostring`` function takes a value to be turned into a string together with an optional ``stringformat`` formatting specifier object,
::

    type @tostring(value[,type @stringformat(width spec[,precision spec])])

The width specifier tells the ``tostring`` function how many characters to reserve for the string conversion of the value.  If the value requires more characters than given in the width specifier then the width specifier is ignored.  If the width specifier is larger than than the number of characters required for the value then the value will be right justified.  For real values there is an optional precision specifier.

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

The ``io`` module also defines a function ``print`` which behaves just like ``println``
except that it does not terminate print with a newline.

Another useful function defined in the ``io`` module is the ``input`` function that, given an optional prompt string, will prompt the user at the terminal and return the input value as a string.  Here is a small example,
::

    load system io.

    let name = io @input("What is your name? ").
    io @println ("Hello " + name + "!").


The output is,
::

    What is your name? Leo
    Hello Leo!


We can use the type casting functions such as ``tointeger`` or ``toreal`` defined in the
``type`` module to convert the string returned from ``input`` into a numeric value,
::

    load system io.
    load system type.

    let i if i > 0  = type @tointeger(io @input("Please enter a positive integer value: ")).

    for k in 1 to i do
        io @println k.
    end


The output is,
::

    Please enter a positive integer value: 3
    1
    2
    3


Finally, the function ``read`` reads from ``stdin`` and returns the input as a string.  The function ``write`` writes a string to ``stdout``.

The Module System
-----------------

A module in Asteroid is a file with a set of valid Asteroid statements.
You can load this file into other Asteroid code with the statement::

   load "example_path/example_filename".

or::

   load example_modulename.

The search strategy for a module to be loaded is as follows,

1. raw module name - could be an absolute path
2. search in current directory
3. search in directory where Asteroid is installed
4. search in subdirectory where Asteroid was started

Modules defined by the Asteroid system should be loaded with the keyword ``system``
in order to avoid any clashes with locally defined modules.  If the ``system``
keyword is used then Asteroid only searches in its system folders
rather than in user directories.

Say that you wanted to load the ``math`` module so you could execute a certain trigonometric function. The following Asteroid program loads the ``math`` module as well as the ``io``  module. Only after loading them would you be able to complete the sine function below,
::

    load system io.
    load system math.

    let x = math @sin( math @pi / 2 ).
    io @println("The sine of pi / 2 is " + x + ".").


Both the function ``sin`` and the constant value ``pi`` are defined in the ``math`` module.
In addition, the ``io`` module is where all input/output functions in Asteroid (such as ``println``) come from.
If you want the complete list of modules, make sure to check out the reference guide `here <https://asteroid-lang.readthedocs.io/en/latest/Reference%20Guide.html>`_.

