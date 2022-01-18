.. highlight:: none

Asteroid User Guide
###################

Introduction
------------

Asteroid is a multi-paradigm programming language heavily influenced by `Python <https://www.python.org>`_, `Rust <https://www.rust-lang.org>`_, `ML <https://www.smlnj.org>`_, and `Prolog <http://www.swi-prolog.org>`_, that makes pattern matching one of its core computational mechanisms.  This is often called *pattern-matching oriented programming*.

In this document we describe the major features of Asteroid and give plenty of examples.  If you have used a programming language like Python or JavaScript before, then Asteroid should appear very familiar.  However, there are some features which differ drastically from other programming languages due to the core pattern-matching programming
paradigm.  Here are just two examples:

**Example 1:** All statements that look like assignments are actually pattern-match statements.  For example if we state,
::
    let [x,2,y] = [1,2,3].

that means the subject term `[1,2,3]` is matched to the pattern `[x,2,y]` and `x` and `y` are bound to the values 1 and 3, respectively.  By the way, there is nothing wrong with the following statement,
::
    let [1,2,3] = [1,2,3].

which is just another pattern match without any variable instantiations.

**Example 2:** Patterns in Asteroid are first-class citizens of the language.
This is best demonstrated with a program.  Here is a program
that recursively computes the factorial of a positive integer and uses first-class patterns
in order to ensure that the domain of the function is not violated,
::
    -- define first-class patterns
    let POS_INT = pattern with (x:%integer) %if x > 0.
    let NEG_INT = pattern with (x:%integer) %if x < 0.

    -- define our factorial function
    function fact
        with 0 do
            return 1
        orwith n:*POS_INT do            -- use first pattern
            return n * fact (n-1).
        orwith n:*NEG_INT do            -- use second pattern
            throw Error("undefined for "+n).
        end

As you can see, the program first creates patterns and stores them in the variables
`POS_INT` and `NEG_INT` and it uses those patterns later in the code by
dereferencing those variables with the `*` operator.  First-class patterns have
profound implications for software development in that pattern definition and usage
points are now separate and patterns can be reused in different contexts.

These are just two examples where Asteroid differs drastically from other programming languages.  
This document is an overview of Asteroid and is intended to get you started quickly
with programming in Asteroid.



Installation
------------

Download or clone the `Asteroid github repository <https://github.com/lutzhamel/asteroid>`_, or download one of the `prepackaged releases <https://github.com/lutzhamel/asteroid/releases>`_, and then install with `pip <https://pip.pypa.io/en/stable/>`_.

For example, if your working directory is at the top of the repository,
::
    $ python -m pip install .


The same command should work on Unix-like and Windows systems, though you may have to run it with `python3` or some other variation.

In addition, there is a cloud-based Linux virtual machine that is completely set up with an Asteroid environment and can be accessed at `Repl.it <https://repl.it/@lutzhamel/asteroid#README.md>`_.

Running the Asteroid Interpreter
--------------------------------

You can now run the interpreter from the command line by simply typing `asteroid`. This will work on both Windows and Unix-like systems as long as you followed the instructions above.
To run asteroid on Unix-like systems and on our virtual machine,
::
    $ cat hello.ast
    -- the obligatory hello world program

    load system "io".

    println "Hello, World!".

    $ asteroid hello.ast
    Hello, World!
    $

On Windows 10 the same thing looks like this,
::
    C:\> type hello.ast
    -- the obligatory hello world program

    load system "io".

    println "Hello, World!".

    C:\> asteroid hello.ast
    Hello, World!
    C:\>


As you can see, once you have Asteroid installed on your system you can execute an
Asteroid program by typing,
::
    asteroid [flags] <program file>

at the command prompt.

The Basics
----------

As with most languages we are familiar with, Asteroid has **variables** (alpha-numeric symbols starting with an alpha character) and **constants**.  Constants are available for all the **primitive data types**,

* `integer`, e.g. `1024`
* `real`, e.g. `1.75`
* `string`, e.g. `"Hello, World!"`
* `boolean`, e.g. `true`

Asteroid arranges these data types in a **type hierarchy**,

`boolean` < `integer` < `real` < `string`

Type hierarchies facilitate automatic type promotion.  Here is an example
where automatic type promotion is used to put together a string from different data types,
::
    let x:%string = "value: " + 1.

Here we associate the string `"value: 1"` with the variable `x` by first promoting the integer value `1` to the string `"1"` using the fact that `integer` < `string`  according to our type hierarchy  and then interpreting the `+` operator as a string concatenation operator.

Asteroid supports two more data types:

* `list`
* `tuple`

These are **structured data types** in that they can contain entities of other data types. Both of these data types have the probably familiar constructors which are possibly empty squences of comma separated values enclosed by square brackets for lists, e.g. `[1,2,3]`, and enclosed by parentheses for tuples, e.g. `(x,y)`. For tuples we have the caveat that the 1-tuple is represented by a value followed by a comma to distinguish it from parenthesized expressions, e.g.`(3,)`.
Here are some examples,
::
    let a = [1,2,3].  -- this is a list
    let c = (1,2,3).  -- this is a tuple

As we said above, in order to distinguish it from a parenthesized value the single element in a 1-tuple has to be followed by a comma, like so,
::
    let one_tuple = (1,).  -- this is a 1-tuple

Lists and tuples themselves are also embedded in type hierarchies, although very simple ones:

* `list` < `string`
* `tuple` < `string`

That is, any list or tuple can be viewed as a string.  This is very convenient for printing lists and tuples,
::
    load system "io".
    println ("this is my list: " + [1,2,3]).


Finally, Asteroid supports one more type, namely the `none` type.  The `none` type has
only one member: A constant named conveniently `none`.  The null-tuple belongs to this type (rather than the tuple type discussed earlier) and therefore the constant `()` can often be used as a convenient short hand for the constant `none`.  That is, the following `let` statements will succeed,
::
    let none = ().
    let () = none.

meaning that the constants `()` and `none` are equivalent and pattern-match each other.
The `none` data type itself does not belong to any type hierarchy.

By now you probably figured out that statements are terminated with a period and that comments start with a `--` symbol and continue till the end of the line.  You probably also figured out that the `let` statement is Asteroid's version of assignment even though the underlying mechanism is a bit different.

Data Structures
---------------

Lists
^^^^^

In Asteroid the `list` is a fundamental, built-in data structure.  A trait it shares with programming languages such as Lisp, Python, ML, and Prolog.  Below is the list reversal example from above as an executable Asteroid program. So go ahead and experiment!
::
    load system "io".    -- load the io module so we can print

    let a = [1,2,3].     -- construct list a
    let b = a @[2,1,0].  -- reverse list a
    println b.

The output is: `[3,2,1]`.

In Asteroid lists are considered objects with member functions that can manipulate the list
object, e.g. `[1,2,3] @ reverse()`. We could rewrite the above example as,
::
    load system "io".          

    let a = [1,2,3].    
    let b = a @reverse().
    println b.

For a full list of available member functions for Asteroid lists please see the reference guide.

As we have seen, the `@` operator allows you to access either individual elements, slices, or member functions of a list.  

Besides using the default constructor for lists which consists of the
square brackets enclosing a list of elements we can use **list comprehensions** to construct lists.  In Asteroid a list comprehension consist of a range specifier together with
a step specifier allowying you to generate integer values within that range,
::
    load system "io".          

    -- build a list of odd values
    let a = [1 to 10 step 2].  -- list comprehension
    println ("list: " + a).

    -- reverse the list using a slice computed as comprehension
    let slice = [4 to 0 step -1]. -- list comprehension
    let b = a @slice.
    println ("reversed list: " + b).

The output is,
::
    list: [1,3,5,7,9]
    reversed list: [9,7,5,3,1]

Asteroid's simple list comprehensions in conjunction with the `map` function for lists allows you to
construct virtually  any kind of list. For example, the following program constructs
a list of alternating 1 and -1,
::
    load system "io".
    load system "math".

    let a = [1 to 10] @map(lambda with x do return mod(x,2))
                      @map(lambda with x do return 1 if x else -1).

    println a.

where the output is,
::
    [1,-1,1,-1,1,-1,1,-1,1,-1]

Higher dimensional arrays can easily be simulated with lists of lists,
::
    load system "io".

    -- build a 2-D array
    let b = [[1,2,3],
             [4,5,6],
             [7,8,9]].

    -- modify an element in the array
    let b @1 @1 = 0.
    println b.

The output is: `[[1,2,3],[4,0,6],[7,8,9]]`

**NOTE**: At this point slicing is not supported on the left side of a `let` statement.

Tuples
^^^^^^

As we saw earlier, the `tuple` is another fundamental, built-in data structure that can be found in Asteroid.

Below is an example of a tuple declaration and access.
::
    load system "io".   -- load the io module so we can print
    let a = (1,2,3).    -- construct tuple a
    let b = a @1.       -- access the second element in tuple a
    println b.          -- print the element to the console

Like `lists`, `tuples` may also be nested,
::
    load system "io".
    -- build a 2-D array
    let b = (("a","b","c"),
             ("d","e","f"),
             ("g","h","i")).
    -- Access an element in the nested structure.
    println(b @1 @1).

Unlike lists, tuples are immutable. This means that their contents cannot be changed once they have been declared. Should we want to change the contents of an already declared tuple, we would need to abandon the original and declare a new `tuple`. The following code block demonstrates this,
::
    load system "io".
    -- build a tuple
    let b = ("a","b","c").
    -- attempt to modify an element in the tuple
    try
        let b @1 = "z".
    catch Exception(kind,s) do
        println(kind+": "+s).
    end.

Which will print out the following message:
::
    SystemError: 'tuple' is not a mutable structure

When to use tuples and when to use lists is really application dependent.
Tuples tend to be preferred over lists when representing some sort of structure,
like abstract syntax trees, where that structure is unmutable meaning, for example,
that the arity of a tree node cannot change.

Custom Data Structures using `structure`
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

You can introduce custom data structures using the `structure` keyword.  These custom data structures differ from lists and tuples in the sense that the name of the structure acts like a type tag.  So, when you define a new structure you are in fact introducing a new type into your program.  We should mention that Asteroid creates
a *default constructor* for a structure.  That constructor copies the arguments given to it into the
data member fields of the structure in the order that the data members appear in the
structure definition and as they appear in the parameter list of the constructor. Here is a simple example,
::
    load system "io".

    structure Person with
        data name.
        data age.
        data gender.
        end

    -- make a list of persons
    let people = [
        -- use default constructors to construct Person instances
        Person("George", 32, "M"),  
        Person("Sophie", 46, "F"),
        Person("Oliver", 21, "X")
        ].

    -- retrieve the second person on the list and print
    let Person(name,age,gender) = people @1. -- pattern match against the structure
    println (name + " is " + age + " years old and is " +  gender + ".").

The output is,
::
    Sophie is 46 years old and is F.


The `structure` statement introduces a new typed data structure. In this case it introduces a data structure of type `Person` with three "data slots".  We use this data structure to build a list of persons.  One of the interesting things  is that we can pattern match the generated data structure as in the second `let` statement in the program above.

In addition to the default constructor, structures in Asteroid also support user specified
constructors and member functions.  We'll talk about those later when we talk about OO programming in Asteroid.

The `let` Statement
-------------------

The `let` statement is a pattern matching statement and can be viewed as Asteroid's version of the assignment statement even though statements like,
::
    let 1 = 1.

where we take the term on the right side and match it to the pattern on the left side of
the `=` operator are completely legal and highlight the fact that `let` statement is not equivalent to an assignment statement.  Patterns are expressions that consist purely of constructors and variables. Constructors themselves consist of constants, list and tuple constructors, and user defined structures.  

Here is an example where we do some computations on the right side of a `let` statement and then match the result against a pattern on the left,
::
    load system "io".

    -- note 1+1 evaluates to 2 and is then matched
    -- the variables x and y are bound to 1 and 3, respectively,
    -- via pattern matching
    let [x,2,y] = [1+0,1+1,1+2].
    println (x,y).

The output is: `(1,3)`

Asteroid supports special patterns called **type patterns** that match any value
of a given type.  For instance, the `%integer` pattern matches any integer value.  Here is a simple example,
::
    let %integer = 1.

This `let` statement succeeds because the value `1` can be pattern matched against
the type pattern `%integer`

Asteroid also
supports something called a **named pattern** were a (sub)pattern on the left side
of a `let` statement (or any pattern as it appears in Asteroid) can be given a name
and that name will be instantiated with a term during pattern matching.  For example,
::
    load system "io".

    let t:(1,2) = (1,2).  -- using a named pattern on lhs
    println t.

Here, the construct `t:(1,2)` is called a named pattern and the variable `t` will be unified with the term `(1,2)`, or more generally, the variable will be unified with term
that matches the pattern on the right of the colon.  The program will print,
::
    (1,2)

We can combine type patterns and named patterns to give us something that looks
like a variable declaration in other languages. In Asteroid, though, it is still just all
about pattern matching.  Consider,
::
    load system "io".
    load system "math".
    load system "type".

    let x:%real = pi.
    println (tostring(x,stringformat(4,2))).

The left side of the `let` statement is a named type pattern that matches any real value, and
if that match is successful then the value is bound to the variable `x`.  Note
that even though this looks like a declaration, it is in fact a pattern matching
operation.  The program will print the value `3.14`.

Flow of Control
---------------

Control structure implementation in Asteroid is along the lines of any of the modern programming languages in use such as Python, Swift, or Rust.  For example, the `for` loop allows you to iterate over lists without having to explicitly define a loop index counter. In addition, the `if` statement defines what does or does not happen when certain conditions are met. For a list of all control statements in Asteroid, see the reference guide.

As we said, in terms of flow of control statements there are really not a lot of surprises. This is because Asteroid supports loops and conditionals in a very similar way to many of the other modern programming languages in use today.  For example, here is a short program with a `for` loop that prints out the first six even positive integers,
::
    load system "io".

    for i in 0 to 10 step 2 do
        println i.
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
    load system "io".
    load system "util"

    for (ix,bird) in zip(["first","second","third"],["turkey","duck","chicken"]) do
        println ("the "+ix+" bird is a "+bird).
    end

The output is,
::
    the first bird is a turkey
    the second bird is a duck
    the third bird is a chicken

Here we first create a list of pairs using the `zip` function, over which we then
iterate pattern matching on each of the pairs on the list with the pattern `(ix,bird)`.

The following is a short program that demonstrates an `if` statement,
::
    load system "io".
    load system "util".

    let x = tointeger(input("Please enter an integer: ")).

    if x < 0 do
        let x = 0.
        println("Negative, changed to zero").
    elif x == 0 do
        println("Zero").
    elif x == 1 do
        println("Single")
    else do
        println("More").
    end

Even though Asteroid's flow of control statements look so familiar, they support pattern matching to a degree not found in other programming languages and which we will take a look at below.

Functions
---------

Functions in Asteroid resemble function definitions in functional programming languages such as Haskell and ML.
Formal arguments are bound via pattern matching and functions are multi-dispatch, that is,
a single function can have multiple bodies each attached to a different pattern
instantiating the formal arguments.

Let's start with something simple.  Here is a function definition for `revdouble` that reverses a list of integers
then doubles each value before returning the result,
::
    load system "io".

    function revdouble
        with l:%list do
            return l @reverse() @map(lambda with x:%integer do return 2*x).
        end

    println (revdouble [1,2,3]).

The output is `[6,4,2]`.  Notice how we used type patterns to make sure that this
function is only applied to lists of integers.

In order to demonstrate multi-dispatch, the following is the quick sort implemented in
Asteroid. Each `with`/`orwith` clause introduces a new function body with its
corresponding pattern,
::
    load system "io".

    function qsort
        with [] do
            return [].
        orwith [a] do
            return [a].
        orwith [pivot|rest] do
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
    println (qsort [3,2,1,0])

The output is as expected,
::
    [0,1,2,3]

Notice that we use the multi-dispatch mechanism to deal with the base cases of the
`qsort` recursion using separate function bodies in the first two `with` clauses.
In the third `with` clause we use the head-tail operator `[pivot|rest]`
which itself is a pattern matching any non-empty list.
Here the variable `pivot` matches the first element of a list, and the variable `rest` matches the remaining list. This remaining list is the original list with its first element removed.  What you also will notice is that function calls do not necessarily have to involve parentheses.  Function application is expressed by simple juxtaposition in Asteroid.  For example, if `foobar` is a function then `foobar(a)` is a function call in Asteroid but so is `foobar a`.  The latter form of function call is used in the last line of the function `qsort` below.

As you have seen in a couple of occasions already in the document, Asteroid also supports anonymous or `lambda` functions.  Lambda functions behave just like regular
functions except that you declare them on-the-fly and they are declared without a
name.  Here is an example using a `lambda` function,
::
    load system "io".

    println ((lambda with n do return n+1) 1).

The output is `2`.  Here, the lambda function is a function that takes a value
and increments it by one.  We then apply the value `1` to the function and the
print function prints out the value `2`.

Pattern Matching
----------------

Pattern matching lies at the heart of Asteroid.  We saw some of Asteroid's pattern matching ability when we discussed the `let` statement.  We can also have pattern matching
in expressions using the `is` predicate.

Pattern Matching in Expressions: The `is` Predicate
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Consider the following example of this predicate among some patterns,
::
    load system "io".

    let p = (1,2).

    if p is (x,y,z) do
        println ("it's a triple with: "+x+","+y+","+z)
    elif p is (x,y) do
        println ("it's a pair with: "+x+","+y).
    else do
        println "it's something else".
    end

Here we use patterns to determine if `p` is a triple, a pair, or something else. Pattern matching is embedded in the expressions of the `if` statement. The
output of this program is,
::
    it's a pair with: 1,2

Pattern matching with the `is` predicate can happen anywhere expressions can
be used.  That means we can use the predicate also in the `let` statements,
::
    let true = (1,2) is (1,2).

This is kind of strange looking but it succeeds.  Here the
left side of the `is` predicate is the term and
the right side is the pattern.  Obviously this pattern match will succeed because the
term and the pattern look identical.  The return value of the `is` predicate is then
pattern matched against the `true` value on the left of the `=` operator.

We can also employ pattern matching in loops.
In the following program we use the `is` predicate to test whether the list is empty or not
while looping,
::
    load system "io".

    let list = [1,2,3].

    repeat do
        let [head|tail] = list.
        println head.
        let list = tail.
    until list is [].

The output is,
::
    1
    2
    3

The example employs pattern matching using the head-tail operator in the `repeat-until` loop expression in order to iterate over a list and print the list elements.  The
termination condition of the loop is computed with the `is` predicate.

Pattern Matching in Function Arguments
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

As we have seen earlier, Asteroid supports pattern matching on function arguments in the style of ML and many other functional programming languages.
Here is an example that uses pattern matching on function arguments using custom data structures.  The program below implements [Peano addition](https://en.wikipedia.org/wiki/Peano_axioms#Addition) on terms using the two Peano axioms,
::
    x + 0 = x
    x + s(y) = s(x+y)

Here `x` and `y` are variables, `0` represents the natural number with value zero, and `s` is the successor function.  In Peano arithmetic any natural number can be represented by the appropriate number of applications of the successor function to the natural number `0`. Here is the program where we replaced the `+` operator with the
`add` symbol,
::
    -- implements Peano addition on terms
    load system "io".

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
        orwith add(x,s(y))  do
            return s(reduce(add(x,y))).
        orwith term do     
            return term.
        end

    -- add 2 3
    println(reduce(add(s(s(0)),s(s(s(0)))))).

Our program defines the structure `s` to represent the successor function and the structure `add` to represent Peano addition. Next, it defines a function that uses pattern matching to identify the left sides of the two axioms.  If either pattern matches the input to the `reduce` function, it will activate the corresponding function body and rewrite the term recursively in an appropriate manner.  We have one additional pattern which matches if neither one of the Peano axiom patterns matches and terminates the recursion.  Finally,  on the last line, we use our `reduce` function to compute the Peano term for the addition of 2 + 3. As expected, the output of this program is,
::
    s(s(s(s(s(0)))))

which represents the value 5.

Conditional Pattern Matching
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Asteroid allows the user to attach conditions to patterns that need to hold in order
for the pattern match to succeed.  This is particularly useful for restricting
input values to function bodies.  Consider the following definition of the
`factorial` function where we use conditional pattern matching to control
the kind of values that are being passed to a particular function body,
::
    load system "io".

    function factorial
        with 0 do
            return 1
        orwith (n:%integer) %if n > 0 do
            return n * factorial (n-1).
        orwith (n:%integer) %if n < 0 do
            throw Error("factorial is not defined for "+n).
        end

    println ("The factorial of 3 is: " + factorial (3)).

Here we see that first, we make sure that we are being passed integers and second,
that the integers are positive using the appropriate conditions on the input values. If
we are being passed a negative integer, then we throw an error.


Pattern Matching in `for` Loops
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

We have seen pattern matching in `for` loops earlier.  Here we show another
example. This combines structural matching with regular expression matching
in `for` loops
that selects certain items from a list. Suppose we want to print out the names of persons that contain a lower case 'p',
:: 
    load system "io".

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
        println name.
    end

Here we pattern match the `Person` object in the `for` loop and then use a regular expression to see if the name of that person matches our requirement that it contains a lower case 'p'.  We can tag the pattern with a variable name, a named pattern, so that we can print out the name if the regular expression matches. The output is `Sophie`.  

Pattern Matching in `try-catch` Statements
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Exception handling in Asteroid is very similar to exception handling in many of the other modern programming languages available today.  The example below shows an Asteroid program  that throws one of two exceptions depending on the randomly generated value `i`,
::
    load system "io".
    load system "random".
    load system "type".

    structure Head with
        data val.
        end

    structure Tail with
        data val.
        end

    try
        let i = random().
        if i >= .5 do
            throw Head(i).
        else do
            throw Tail(i).
        end
    catch Head(v) do
        println("you win with "+tostring(v,stringformat(4,2))).
    catch Tail(v) do
        println("you loose with "+tostring(v,stringformat(4,2))).
    end

The `Head` and `Tail` exceptions are handled by their corresponding `catch` statements, respectively.  In both cases the exception object is unpacked using pattern matching and the unpacked value is used in the appropriate message printed to the screen.

It is worth noting that even though Asteroid has builtin exception objects such as `Error`,
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
introduce a new type name into a program. For instance, in the case above, the `structure`
statement introduces the type name `Person`.   Given a structure definition, we can
create **instances** of that structure.  For example,
::
    let scarlett = Person("Scarlett",28,"F").

The right side of the `let` statement invokes the default constructor for the
structure in order to create an instance stored in the variable `scarlett`. We
can access members of the instance,
::
    load system "io".

    structure Person with
        data name.
        data age.
        data gender.
        end

    let scarlett = Person("Scarlett",28,"F").
    -- access the name field of the structure instance
    println (scarlett @name).  

Asteroid allows you to attach functions to structures.  In member functions
the object identity of the instance is available through the `this` keyword.
For example, we can
extend our `Person` structure with the `hello` function that uses the `name` field
of the instance,
::
    load system "io".

    structure Person with
        data name.
        data age.
        data gender.
        function hello
            with none do
                println ("Hello, my name is "+this @name).
            end
        end

    let scarlett = Person("Scarlett",28,"F").
    -- call the member function
    scarlett @hello().

This program will print out,
::
    Hello, my name is Scarlett

The expression `this @name` accesses the `name` field of the instance the
function `hello` was called on.
Even though our structures are starting to look a bit more like object definitions,
pattern matching continues to work in the same way from when we discussed structures.
The only thing you need to keep in mind is that you **cannot** pattern match on a
function field.  From a pattern matching perspective, a structure consists only of
data fields.  So even if we declare a structure like this,
::
    load system "io".

    structure Person with
        data name.
        -- the function is defined in the middle of the data fields
        function hello
            with none do
                println ("Hello, my name is "+this @name).
            end
        data age.
        data gender.
        end

    -- pattern matching ignores function definitions
    let Person(name,age,_) = Person("Scarlett",28,"F").
    println (name+" is "+age+" years old").

where the function `hello` is defined in the middle of the data fields,
pattern matching simply ignores the function definition and pattern matches
only on the data fields.  The output of the program is,
::
    Scarlett is 28 years old

Here is a slightly more involved example based on the
dog example from the `Python documentation <https://docs.python.org/3/tutorial/classes.html>`_.  
The idea of the dog example is to have a structure that describes dogs by their
names and the tricks that they can perform.  Tricks can be added to a particular
dog instance by calling the `add_trick` function.  Rather than using the default
constructor, we define a constructor for our instances with the `__init__` function.
Here is the program listing for the example in Asteroid,
::
    load system "io".
    load system "type".

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
    for (Dog(name,tricks) %if tostring(tricks) is ".*fetch.*") in [fido,buddy] do
        println (name+" knows how to fetch").
    end

After declaring the structure we instantiate two dogs, Fido and Buddy, and add
tricks to their respective trick repertiores.  The last couple of lines
of the program consist of a `for` loop over a list of our dogs.
The `for` loop is interesting
because here we use structural, conditional, and regular expression pattern
matching in order to only select the dogs that know how to do `fetch` from
the list of dogs.  The pattern is,
::
    Dog(name,tricks) %if tostring(tricks) is ".*fetch.*"

The structural part of the pattern is `Dog(name,tricks)` which simply matches
any dog instance on the list.  However, that match is only successful if
the conditional part of the pattern holds,
::
    %if tostring(tricks) is ".*fetch.*"

This condition only succeeds if the `tricks` list viewed as a string matches
the regular expression `".*fetch.*"`. That is, if the list contains the word `fetch`.
The output is,
::
    Fido knows how to fetch


Patterns as First-Class Citizens
--------------------------------

A programming language feature that is promoted to first-class status does not
change the power of a programming language in terms of computability but it does
increase its expressiveness.  Think functions as first-class citizens of a programming
language.  First-class functions give us `lambda` functions and `map`, both powerful
programming tools.

The same is true when we promote patterns to first-class citizen status in a language.  It
doesn't change what we can and cannot compute with the language. But it does change how
we can express what we want to compute.  That is, it changes the expressiveness
of a programming language.

In Asteroid first-class patterns are introduced with the keywords `pattern with`
and patterns themselves are values that we can store in variables and then reference
when we want to use them.  Like so,
::
    let P = pattern with (x,y).
    let *P = (1,2).

The left side of the second `let` statement dereferences the pattern stored in variable `P`
and uses the pattern to match against the term `(1,2)`.

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
        with (x %if (x is %boolean) or (x is %integer) or (x is %string),y) do
            println (x,y).
        end

That complicated pattern for the first component completely obliterates the
overall structure of the parameter pattern and makes the function definition
difficult to read.

We can express the same function with a first-class pattern,
::
    let TP = pattern
        with q %if (q is %boolean) or
                   (q is %integer) or
                   (q is %string).

    function foo
        with (x:*TP,y) do
            println (x,y).
        end

It is clear now that the main input structure to the function is a pair and the
conditional type restriction pattern has been relegated to a subpattern stored in the variable
`TP`.

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
        orwith (n:%integer) %if n > 0 do
            return n * fact (n-1).
        orwith (n:%integer) %if n < 0 do
            throw Error("fact undefined for negative values").
        end

    function stepf
        with 0 do
            return 1
        orwith (n:%integer) %if n > 0 do
            return 1.
        orwith (n:%integer) %if n < 0 do
            return -1.
        end

In order to write these two functions we had to repeat the almost identical pattern
four times.  First-class patterns allow us to write the same two functions in a
much more elegant way,
::
    let POS_INT = pattern with (x:%integer) %if x > 0.
    let NEG_INT = pattern with (x:%integer) %if x < 0.

    function fact
        with 0 do
            return 1
        orwith n:*POS_INT do
            return n * fact (n-1).
        orwith *NEG_INT do
            throw Error("fact undefined for negative values").
        end

    function stepf
        with 0 do
                return 1
            orwith *POS_INT do
                return 1.
            orwith *NEG_INT do
                return -1.
            end

The relevant patterns are now stored in the variables `POS_INT` and `NEG_INT`
which are then used in the function definitions.

Running Patterns in Reverse
^^^^^^^^^^^^^^^^^^^^^^^^^^^

One of the challenges when programming with patterns is to keep an object structure and
the patterns aimed at destructuring that object structure in sync.  First-class
patterns solve this problem in an elegant way by viewing first-class patterns as
essentially "object network constructors".  In that way, a first-class pattern is
used to construct an object structure as well as destructure it without having to
worry that the structure and pattern will get out of sync.

In order to use a pattern as a constructor we apply the `eval` function to it which
turns the pattern into a value from Asteroid's point of view which can then be used
in computations.  For example,
::
    load system "io".
    let P = pattern with ([a],[b]).
    let a = 1.
    let b = 2.
    let v = eval P. -- use pattern to construct a value
    println v.

The output of the program is,
::
    ([1],[2])

which is the value computed by the `eval` function given the values associated with
the variables `a` and `b`, and
the first-class pattern `P`.  Of course, first-class patterns can be used
to destructure the constructed value,
::
    load system "io".
    let P = pattern with ([a],[b]).
    let v = ([1],[2]).
    let *P = v.
    println a.
    println b.

As expected, the output is,
::
    1
    2

which are the values of the variables instantiated by the pattern match of the first-class
pattern.

As a more advanced example, consider the following
program that defines a family object network.  It
uses the first-class pattern `FP` to both construct an object network representing
a family and, since it is a pattern, can also be used to destructure a family object
network.  Here is the program listing,
::
    load system "io".

    -----------------------------
    structure Family
    -----------------------------
        with
            data parent1.
            data parent2.
            data children.

            function __init__
                with (p1:%Parent,p2:%Parent,c:%Children) do
                    let this @parent1 = p1.
                    let this @parent2 = p2.
                    let this @children = c.
                end
        end

    -----------------------------
    structure Parent
    -----------------------------
        with
            data name.
            function __init__
                with name:%string do
                    let this @name = name
                end
        end

    -----------------------------
    structure Children
    -----------------------------
        with
            data list.

            function __init__
                with list:%list do
                    let this @list = list.
                end
        end

    -----------------------------
    let FP = pattern
    -----------------------------
        with Family(Parent(p1),Parent(p2),Children(c)).

    -----------------------------
    function construct_family
    -----------------------------
        with (P,p1,p2,c) do
            return eval(P).  -- run pattern in reverse, construct object network.
        end

    -----------------------------
    function destructure_family
    -----------------------------
        with (P,term) do
            let *P = term.   -- pattern match, destructure object network.
            return [p1,p2]+c.
        end

    -----------------------------
    -- construct families
    -----------------------------
    let f1 = construct_family(FP,"Harry","Bridget",["Sue","Peter"]).
    let f2 = construct_family(FP,"Margot","Selma",["Latisha","Rudolf"]).

    -----------------------------
    -- destructure families
    -----------------------------
    println(destructure_family(FP,f1)).
    println(destructure_family(FP,f2)).

The function `construct_family` constructs a family evaluating the pattern using
the `eval` function.  The formal parameters of the function provide values for
the free variables in the pattern.  Since we are dealing with first-class
patterns we can simply pass the pattern to the function as a value.

The function `destructure_family` does the opposite.  It uses the first-class
pattern to pattern-match the passed in term, that is, it destructures that term
using the pattern.  The return statement captures the variables declared as a result
of that pattern match and returns the values as a list. The output of the program is,
::
    [Harry,Bridget,Sue,Peter]
    [Margot,Selma,Latisha,Rudolf]


Notice that the whole program is essentially parameterized over the structure
of the pattern.  We could easily change some internals of this pattern without
affecting the rest of the program.


More on Exceptions
------------------

This section will give further information on how to work with **exceptions**, or unexpected conditions that break the regular flow of execution.  Exceptions generated by Asteroid are `Exception` objects with the following structure,
::
    structure Exception with
        data kind.
        data value.
    end

The `kind` field will be populated by Asteroid with one of the following strings,

* `PatternMatchFailed` - this exception will be thrown if the user attempted an
explicit pattern match which failed, e.g. a let statement whose left side pattern
does not match the term on the right side.

* `NonLinearPatternError` - this exception occurs when a pattern has more than
one variable with the same name, e.g. `let (x,x) = (1,2).`

* `RedundantPatternFound` - this exception is thrown if one pattern makes another
superfluous, e.g. in a multi-dispatch function definition.

* `ArithmeticError` - e.g. division by zero

* `FileNotFound` - an attempt of opening a file failed.

* `SystemError` - a general exception.

In addition to the `kind` field, the `value` field holds a string with some further details on the exception. Specific exceptions can be caught by pattern matching on the `kind` field of the `Exception` object.  For
example,
::
    load system "io".

    try
        let x = 1/0.
    catch Exception("ArithmeticError", s) do
        println s.
    end

The output is,
::
    integer division or modulo by zero


Asteroid also provides a predefined `Error` object for user level exceptions,
::
    load system "io".

    try
        throw Error("something worth throwing").
    catch Error(s) do
        println s.
    end

Of course the user can also use the `Exception` object for their own exceptions
by defining a `kind` that does not interfere with the predefined `kind` strings above,
::
    load system "io".

    try
        throw Exception("MyException","something worth throwing").
    catch Exception("MyException",s) do
        println s.
    end

The output here is,
::
    something worth therefore

In addition to the Asteroid defined exceptions,
the user is allowed to construct user level exceptions with any kind of object including tuples and lists. Here is an example that constructs a tuple as an exception object,
::
    load system "io".

    try
        throw ("funny exception", 42).
    catch ("funny exception", v) do
        println v.
    end

The output of this program is `42`.  

Now, if you don't care what kind of exception you catch, you need to use a `wildcard` or a variable because exception handlers are activated via pattern matching on the
exception object itself.  Here is an example using a `wildcard`,
::
    load system "io".

    try
        let (x,y) = (1,2,3).
    catch _ do
        println "something happened".
    end

Here is an example using a variable,
::
    load system "io".
    load system "type".

    try
        let (x,y) = (1,2,3).
    catch e do
        println ("something happened: "+tostring(e)).
    end

In this last example we simply convert the caught exception object into a string
and print it,
::
    something happened: Exception(PatternMatchFailed,pattern match failed: term and pattern lists/tuples are not the same length)


Basic Asteroid I/O
------------------

I/O functions are defined in the `io` module. The `println` function prints its argument in a readable form to the terminal.  Recall that the `+` operator also implements string concatenation.  This allows us to construct nicely formatted output strings,
::
    load system "io".

    let a = 1.
    let b = 2.
    println ("a + b = " + (a + b)).

The output is
::
    a + b = 3

We can use the `tostring` function defined in the `type` module to provide some
additional formatting. The idea is that the `tostring` function takes a value to be turned into a string together with an optional `stringformat` formatting specifier object,
::
    tostring(value[,stringformat(width spec[,precision spec])])

The width specifier tells the `tostring` function how many characters to reserve for the string conversion of the value.  If the value requires more characters than given in the width specifier then the width specifier is ignored.  If the width specifier is larger than than the number of characters required for the value then the value will be right justified.  For real values there is an optional precision specifier.

Here is a program that exercises some of the string formatting options,
::
    load system "io".
    load system "type".
    load system "math".

    -- if the width specifier is larger than the length of the value
    -- then the value will be right justified
    let b = tostring(true,stringformat(10)).
    println b.

    let i = tostring(5,stringformat(5)).
    println i.

    -- we can format a string by applying tostring to the string
    let s = tostring("hello there!",stringformat(30)).
    println s.

    -- for floating point values: first value is width, second value precision.
    -- if precision is missing then value is left justified and zero padded on right.
    let r = tostring(pi,stringformat(6,3)).
    println r.

The output of the program is,
::
          true
        5
                      hello there!
     3.142

Notice the right justification of the various values within the given string length.

The `io` module also defines a function `print` which behaves just like `println`
except that it does not terminate print with a newline.

Another useful function defined in the `io` module is the `input` function that, given an optional prompt string, will prompt the user at the terminal and return the input value as a string.  Here is a small example,
::
    load system "io".

    let name = input("What is your name? ").
    println ("Hello " + name + "!").

The output is,
::
    What is your name? Leo
    Hello Leo!


We can use the type casting functions such as `tointeger` or `toreal` defined in the
`type` module to convert the string returned from `input` into a numeric value,
::
    load system "io".
    load system "type".

    let i = tointeger(input("Please enter a positive integer value: ")).

    if i < 0 do
        throw Error("I want a positive integer value.").
    end

    for k in 1 to i do
        println k.
    end

The output is,
::
    Please enter a positive integer value: 3
    1
    2
    3


Finally, the function `read` reads from `stdin` and returns the input as a string.  The function `write` writes a string to `stdout`.

The Module System
-----------------

A module in Asteroid is a file with a set of valid Asteroid statements.  You can load this file into other Asteroid code with the `load "<filename>".` statement.  In the current version of Asteroid modules do not have a separate name space; symbols from a module are entered into Asteroid's global name space.

The search strategy for a module to be loaded is as follows, 

#. raw module name - could be an absolute path 
#. search in current directory (path[1]) 
#. search in directory where Asteroid is installed (path[0]) 
#. search in subdirectory where Asteroid was started 

Modules defined by the Asteroid system should be loaded with the keyword `system`
in order to avoid any clashes with locally defined modules.

Say that you wanted to load the `math` module so you could execute a certain trigonometric function. The following Asteroid program loads the `math` module as well as the `io`  module. Only after loading them would you be able to complete the sine function below,
::
    load system "io".
    load system "math".

    let x = sin( pi / 2 ).
    println("The sine of pi / 2 is " + x + ".").

Both the function `sin` and the constant value `pi` are defined in the `math` module. In addition, the `io` module is where all input/output functions in Asteroid (such as `println`) come from.