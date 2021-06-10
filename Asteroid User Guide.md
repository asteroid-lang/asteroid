# Asteroid User Guide

## Introduction

Asteroid is a
multi-paradigm programming language heavily influenced by [Python](https://www.python.org), [Rust](https://www.rust-lang.org), [ML](https://www.smlnj.org), and [Prolog](http://www.swi-prolog.org), that makes pattern matching one of its core computational mechanisms.  This is often called *pattern-matching oriented programming*.

In this document we describe the major features of Asteroid and give plenty of examples.  If you have used a programming language like Python or JavaScript before, then Asteroid should appear very familiar.  However, there are some features which differ drastically from other programming languages due to the core pattern-matching programming
paradigm.  Here are just two examples:

**Example 1:** All statements that look like assignments are actually pattern-match statements.  For example if we state,
```
let [x,2,y] = [1,2,3].
```
that means the subject term `[1,2,3]` is matched to the pattern `[x,2,y]` and `x` and `y` are bound to the values 1 and 3, respectively.  By the way, there is nothing wrong with the following statement,
```
let [1,2,3] = [1,2,3].
```
which is just another pattern match without any variable instantiations.

**Example 2:** Patterns in Asteroid are first-class citizens of the language.
This is best demonstrated with a program.  Here is a program
that recursively computes the factorial of a positive integer and uses first-class patterns
in order to ensure that the domain of the function is not violated,
```
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
```
As you can see, the program first creates patterns and stores them in the variables
`POS_INT` and `NEG_INT` and it uses those patterns later in the code by
dereferencing those variables with the `*` operator.  First-class patterns have
profound implications for software development in that pattern definition and usage
points are now separate and patterns can be reused in different contexts.

These are just two examples where Asteroid differs drastically from other programming languages.  
This document is an overview of Asteroid and is intended to get you started quickly
with programming in Asteroid.



## Installation

Installation on **Unix-like** systems is nothing more than to either download or clone the [Asteroid github repository](https://github.com/lutzhamel/asteroid) or download one of the [prepackaged releases](https://github.com/lutzhamel/asteroid/releases) and then add the `code` folder of the repository/release to your `PATH` environment variable. Be sure that you have Python 3.x installed. Make sure that the file `asteroid` in the `code` folder has execution privileges on your machine.

On **Windows 10**, after downloading the asteroid files, you will need to set the environment variable `ASTEROID_ROOT` to point to the folder where you cloned the repo or unzipped the downloaded file. Then you will need to add the following to the path environment variable: `%ASTEROID_ROOT%\code`. That's it, now you can use the `asteroid.bat` file in the `code` folder to start the asteroid interpreter.

In addition, there is a Linux-based **cloud based virtual machine** that is completely set up with an Asteroid environment and can be accessed at [Repl.it](https://repl.it/@lutzhamel/asteroid#README.md).

## Running the Asteroid Interpreter

You can now run the interpreter from the command line by simply typing `asteroid`. This will work on both Windows and Unix-like systems as long as you followed the instructions above.
To run asteroid on Unix-like systems and on our virtual machine,
```
$ cat hello.ast
-- the obligatory hello world program

load system "io".

println "Hello, World!".

$ asteroid hello.ast
Hello, World!
$
```
On Windows 10 the same thing looks like this,
```
C:\> type hello.ast
-- the obligatory hello world program

load system "io".

println "Hello, World!".

C:\> asteroid hello.ast
Hello, World!
C:\>
```

As you can see, once you have Asteroid installed on your system you can execute an
Asteroid program by typing,
```
asteroid [flags] <program file>
```
at the command prompt.

## The Basics

As with most languages we are familiar with, Asteroid has **variables** (alpha-numeric symbols starting with an alpha character) and **constants**.  Constants are available for all the **primitive data types**:

* `integer`
* `real`
* `string`
* `boolean`

Asteroid arranges these data types in a **type hierarchy** in order to facilitate automatic type promotion:

`boolean` < `integer` < `real` < `string`

Asteroid supports two more data types:

* `list`
* `tuple`

These are **structured data types** in that they can contain entities of other data types. Both of these data types have the probably familiar constructors which are possibly empty squences of comma separated values enclosed by square brackets for lists and enclosed by parentheses for tuples. For tuples we have the caveat that the 1-tuple is represented by a value followed by a comma to distinguish it from parenthesized expressions, e.g.`(<something>,)`. Furthermore, the null-tuple `()` actually belongs to a different data type as we will see below.
Here are some examples,
```
let a = [1,2,3].  -- this is a list
let c = (1,2,3).  -- this is a tuple
```
In order to distinguish it from a parenthesized value the single element in a 1-tuple has to be followed by a comma, like so,
```
let one_tuple = (1,).  -- this is a 1-tuple
```
Lists and tuples themselves are also embedded in type hierarchies, although very simple ones:

* `list` < `string`
* `tuple` < `string`

That is, any list or tuple can be viewed as a string.  This is very convenient for printing lists and tuples.

Finally, Asteroid supports one more type, namely the `none` type.  The `none` type has
only one member: A constant named conveniently `none`.  As mentioned above, the null-tuple is of this type and therefore the constant `()` can often be used as a convenient short hand for the constant `none`.  That is, the following `let` statements will succeed,
```
let none = ().
let () = none.
```
meaning that the constants `()` and `none` are equivalent and pattern-match each other.
The `none` data type itselft does not belong to any type hierarchy.

By now you probably figured out that statements are terminated with a period and that comments start with a `--` symbol and continue till the end of the line.  You probably also figured out that the `let` statement is Asteroid's version of assignment even though the underlying mechanism is a bit different.

## Data Structures

### Lists

In Asteroid the `list` is the fundamental, built-in data structure.  A trait it shares with programming languages such as Lisp, Python, ML, and Prolog.  Below is the list reversal example from above as an executable Asteroid program. So go ahead and experiment!
```
load system "io".          -- load the io module so we can print

let a = [1,2,3].    -- construct list a
let b = a @[2,1,0].  -- reverse list a
println b.
```
The output is: `[3,2,1]`.
As we have seen the `@` operator allows you to access either individual elements or slices of a list.  We can also use **list comprehensions** to construct lists,
```
load system "io".          

-- build a list of odd values
let a = [1 to 10 step 2].
println ("list: " + a).

-- reverse the list
let slice = [4 to 0 step -1].
let b = a @slice.
println ("reversed list: " + b).
```
The output is,
```
    list: [1,3,5,7,9]
    reversed list: [9,7,5,3,1]
```
Higher dimensional arrays can easily be simulated with lists of lists,
```
load system "io".

-- build a 2-D array
let b = [[1,2,3],
         [4,5,6],
         [7,8,9]].

-- modify an element in the array
let b @1 @1 = 0.
println b.
```
The output is: `[[1,2,3],[4,0,6],[7,8,9]]`

**NOTE**: At this point slicing is not supported on the left side of a `let` statement.

### Custom Data Structures using `structure`

You can introduce custom data structures using the `structure` keyword.  These custom data structures differ from lists in the sense that the name of the structure acts like a type tag.  So, when you define a new structure you are introducing a new type into your program.  We should mention that Asteroid creates
a *default constructor* for a structure.  That constructor copies the arguments given to it into the
data member fields of the structure in the order that the data members appear in the
structure definition and as they appear in the parameter list of the constructor. Here is a simple example,


```
load system "io".

structure Person with
    data name.
    data age.
    data gender.
    end

-- make a list of persons
let people = [
    Person("George", 32, "M"),
    Person("Sophie", 46, "F"),
    Person("Oliver", 21, "X")
    ].

-- retrieve the second person on the list and print
let Person(name,age,gender) = people @1.
println (name + " is " + age + " years old and is " +  gender + ".").
```
The output is,
```
    Sophie is 46 years old and is F.
```

The `structure` statement introduces a new typed data structure. In this case it introduces a data structure of type `Person` with three "slots".  We use this data structure to build a list of persons.  One of the interesting things  is that we can pattern match the generated data structure as in the second `let` statement in the program above.

It turns out that data structures defined with the `structure` command also support prototype based OO programming.  More of that below.

## The `let` Statement

The `let` statement is Asteroid's version of the assignment statement.  Here is a snippet of Asteroid's grammar detailing the statement,
```
stmt := LET pattern '=' exp '.'?
```
In the grammar capital words and symbols in quotes represent language keywords and lower case letters represent non-terminals.  

The notation is an EBNF notation that means the question mark is a meta operator and makes the period at the end of the `let` statement optional.  Even though the period is optional we highly recommend using it because leaving it out can, under certain circumstances, lead to ambiguous statements and therefore will lead to syntax errors.

As we said before, the `let` statement is a pattern matching statement which we can see expressed here by the `pattern` on the left side of the `=` sign. Patterns are expressions that consist purely of constructors and variables. Constructors consist of constants, list constructors, and user defined structures.  The quote operator `'` allows you to turn any Asteroid expression into a term structure that can be used as a pattern.

Here is an example where we do some computations on the right side of a `let` statement and then match against a pattern on the left,
```
load system "io".

-- note 1+1 evaluates to 2 and is then matched
-- the variables x and y are bound to 1 and 3, respectively
let [x,2,y] = [1+0,1+1,1+2].
println (x,y).
```
The output is: `(1,3)`

Here is a similar program but all terms have been quoted and therefore are not evaluated and the actual structure of the terms is matched,
```
load system "io".

-- note 1+1 does NOT evaluate to 2 and 1+1 is matched
-- the variables x and y are bound to term expressions
let [x,'1+1,y] = ['1+0,'1+1,'1+2].
println (x,y).
```
The output is `(__plus__(1,0),__plus__(1,2))`

The fact that none of the terms is being evaluated and their actual structure is being preserved becomes clear what we print what has been bound to the variables `x` and `y`.  Here the symbol `__plus__` is the internal notation of the `+` operator.

## Loops and `if` Statements

Control structure implementation in Asteroid is along the lines of any of the modern programming languages such as Python, Swift, or Rust.  For example, the `for` loop allows you to iterate over lists without having to explicitly define a loop index counter. In addition, the `if` statement defines what does or does not happen when certain conditions are met. For a list of all control statements in Asteroid, see the reference guide of endnotes.

Looking at the list of supported flow of control statements there are really not a lot of surprises.  For example, here is a short program with a `for` loop that prints out the first ten even positive integers,
```
load system "io".
for i in 0 to 10 step 2 do
    println i
end
```
The output is,
```
    0
    2
    4
    6
    8
    10
```
Here is another example that iterates over lists,

```
load system "io".

for bird in ["turkey","duck","chicken"] do
    println bird.
end
```
The output is,
```
    turkey
    duck
    chicken
```

Even though Asteroid's flow of control statements look so familiar, they support pattern matching to a degree not found in other programming languages and which we will take a look at below. Here is a short program with an `if` statement that outlines what text to print as an output when certain inputs are (not) given,

```
x = int(input("Please enter an integer: "))

if x < 0:
    x = 0
    print('Negative changed to zero')
elif x == 0:
    print('Zero')
elif x == 1:
    print('Single')
else:
    print('More')
```


## Functions

Here is the grammar snippet that defines functions,
```
stmt      := FUNCTION ID body_defs END
body_defs := WITH pattern DO stmt_list (ORWITH pattern DO stmt_list)*
```
A closer look reveals that a function can have multiple bodies each associated with a different formal argument pattern.  Asteroid inherits this characteristic directly from functional languages like ML or Haskell.

However, considering that a variable represents the simplest pattern we can write functions that look very familiar to the programmer coming from the Python or Java traditions.  Here is a function that reverses a list,
```
load system "util".
load system "io".

function reverse with list do
    let len = length(list).
    let r_list = list @[(len-1) to 0 step -1].
    return r_list.
end

let my_list = [1,2,3].
let my_reversed_list = reverse(my_list).
println my_reversed_list.
```
The output is `[3,2,1]`.

We'll talk about pattern matching in functions and multiple bodies later on in this document.  Asteroid also supports anonymous or `lambda` functions.  Here is a snippet of the grammar that defines anonymous functions,
```
primary := LAMBDA body_defs
```
where the `body_defs` are the same as for the functions defined above.  This implies that `lambda` functions can also have multiple bodies each associated with a different formal argument pattern.  Here is a simple example using a `lambda` function,
```
load system "io".

println ((lambda with n do return n+1) 1).
```
The output is `2`.

## Basic Pattern Matching

Pattern matching lies at the heart of Asteroid.  We saw some of Asteroid's pattern match ability when we discussed the `let` statement.  Below is another program that highlights a few other aspects of pattern matching.
In particular, quoted expressions allow the programmer to treat expressions as structure and pattern match against that structure.  Quoted expressions can be interpreted as normal expressions using the `eval` function as shown in the following.  In the case that a statement is expected to fail, like the `let` statement `let '1 + 1 = 1 + 1.` we put it into a try-catch block.
```
load system "io".

let '1 + 1 = '1 + 1. -- quoted expression
let 2 = eval('1 + 1).
let 2 = 1 + 1.
try
    let '1 + 1 = 1 + 1.  -- throws an exception
catch _ do
    println "1+1 pattern match failed".
end
```
The output is,
```
    1+1 pattern match failed
```
Asteroid supports pattern matching on function arguments in the style of ML and many other functional programming languages.

Below is the quick sort implemented in Asteroid as an example of this classic style pattern matching.  What is perhaps new is the `head-tail` operator being used in the last `orwith` clause.  Here the variable `pivot` matches the first element of the list and the variable `rest` matches the remaining list which is the original list with its first element removed.  We can  also see that the `+` operator symbols are overloaded operators in the standard model to act as a list concatenation operators in addition to arithmetic operators. What you also will notice is that function calls do not necessarily have to involve parentheses.  Function application is also expressed by simple juxtaposition in Asteroid.  For example, if `foobar` is a function then `foobar(a)` is a function call in Asteroid but so is `foobar a`.  The latter form of function call is used in the last line of the function `qsort` below.
```
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
```
The output is as excpected,
```
    [0,1,2,3]
```
We can also introduce our own custom constructors and use them in pattern matching.  The program below implements [Peano addition](https://en.wikipedia.org/wiki/Peano_axioms#Addition) on terms using the two Peano axioms,
```
x + 0 = x
x + S(y) = S(x+y)
```
Here `x` and `y` are variables, `0` represents the natural number with value zero, and `S` is the successor function.  In Peano arithmetic any natural number can be represented by the appropriate number of applications of the successor function to the natural number `0`. Here is the program where we replaced the `+` operator with the
`add` symbol,
```
-- implements Peano addition on terms
load system "io".
load system "util".

structure S with
    data x.
    end

structure add with
    data left.
    data right.
    end

function reduce
    with add(x,0) do      
        return reduce(x).
    orwith add(x,S(y))  do
        return S(reduce(add(x,y))).
    orwith term do     
        return term.
    end

-- add 2 3
println(reduce(add(S(S(0)),S(S(S(0)))))).
```
Our program defines the structure `S` to represent the successor function and the structure `add` to represent Peano addition. Next, it defines a function that uses pattern matching to identify the left sides of the two axioms.  If either pattern matches the input to the `reduce` function it will activate the corresponding function body and rewrite the term recursively in an appropriate manner.  We have one additional pattern which matches if neither one of the Peano axiom patterns matches and terminates the recursion.  Finally,  on the last line, we use our `reduce` function to compute the Peano term for the addition of 2 + 3. As expected, the output of this program is,
```
S(S(S(S(S(0)))))
```
which represents the value 5.

## Pattern Matching in Control Structures

Before we begin the discussion we need to introduce the `is` predicate  which is a built-in operator that takes the pattern on the right side and applies it to the subject term on the left side.  If there is a match the predicate will return `true` if not then it will return `false`.  Here is a snippet that illustrates the predicate,
```
let true = (1,2) %is (x,y).
```
The subject term `1 + 2` is matched to the pattern `x + y` which of course will succeed with the variable bindings `x`  &#x21A6; `1` and `y` &#x21A6; `2`.

### Pattern Matching in `if` Statements

In Asteroid an `if` statement consists of an `if` clause followed by zero or more `elif` clauses followed by an optional `else` clause.  The semantics of the `if` statement is fairly standard.  The `if` and `elif` clauses test the value of their corresponding expressions for the term `true` and execute their corresponding set of statements if it does evaluate to `true`.  If none of the expressions evaluate to `true` then the `else` clause is executed if present.

In order to enable pattern matching in `if` statements we use the `is` predicate.  We can rewrite the `reduce` function from the above Peano arithmetic example using pattern matching in `if` statements as an illustration,
```
function reduce with term do
   if term %is add(x,0) do
        return reduce(x).
    elif term %is add(x,S(y))  do
        return S(reduce(add(x,y))).
    else do
        return term.
    end
end
```
One thing to note is that the variable bindings of a successful pattern match are immediately available in the surrounding scope and therefore are available in the corresponding statements of the `if` or `elif` clause.

### Pattern Matching in `repeat-until` Loops

Pattern matching in `while` loops follows a similar approach to pattern matching in `if` statements.  The `while` statement tests the evaluation of the loop expression and if it evaluates to the term `true` then the loop body is executed.  Again we use the `is` predicate to enable pattern matching in `while` loops.

The example below shows a program that employs pattern matching using the head-tail operator in the `repeat-until` loop expression in order to iterate over a list and print the list elements.  Note the use of the `is` predicate to test whether the list is empty or not.  
```
load system "io".

let list = [1,2,3].

repeat do
    let [head|tail] = list.
    println head.
    let list = tail.
until list %is [].
```
The output is,
```
    1
    2
    3
```

### Pattern Matching in `for` Loops

For completeness sake we have repeated here an example of a simple `for` from above,
```
load system "io".

for bird in ["turkey","duck","chicken"] do
    println bird.
end
```
Turns out that in simple `for` loops such as the one above the loop variable is actually a pattern that gets matched to the elements of the list the loop iterates over.
We can expand this simple pattern into a much more complicated pattern and do pattern matching while we are iterating.  This allows us to access substructures of the items being iterated over in a direct and succinct way.  The example below shows such a program.  The program constructs a list of `Person` structures that consist of a name and an age.  The `for` loop iterates over this list while pattern matching the `Person` constructor at each iteration binding the age variable to the appropriate value in the structure.  In the loop body it carries a running sum of the age values which it then uses to compute the average age of the persons on the list.  
```
load system "io".

structure Person with
    data name.
    data age.
    end

let people = [
    Person("George", 32),
    Person("Sophie", 46),
    Person("Oliver", 21)
    ].

let n = people @length().
let sum = 0.

for Person(_,age) in people do
    let sum = sum + age.
end

println ("Average Age: " + (sum/n)).
```
The output is,
```
    Average Age: 33
```
We can also use pattern matching in a `for` loop to select certain items from a list. Suppose we want to print out the names of persons that contain a lower case 'p',
```
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
```
The output is `Sophie`.

Here we pattern match the `Person` object in the `for` loop and then use a regular expression to see if the name of that person matches our requirement that it contains a lower case 'p'.  We can tag the pattern with a variable name so that we can print out the name if the regular expression matches.

### Pattern Matching in `try-catch` Statements

Exception handling in Asteroid is very similar to exception handling in many of the other modern programming languages available today.  The example below shows an Asteroid program shows that throws one of two exceptions depending on the randomly generated value `i`,
```
load system "io".
load system "util".

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
    println("you win with "+v).
catch Tail(v) do
    println("you loose with "+v).
end
```
The `Head` and `Tail` exceptions are handled by their corresponding `catch` statements on, respectively.  In both cases the exception object is unpacked using pattern matching and the unpacked value is used in the appropriate message printed to the screen.


## Object-Oriented Programming and Pattern Matching

We introduce Asteroid's objects using the dog example from the [Python documentation](https://docs.python.org/3/tutorial/classes.html).  The code below shows that Python example translated into Asteroid.  Asteroid's object system is prototype based.  Objects are defined with the `structure` keyword and the structure name serves as a new type. The structure name itself also serves as a constructor call in order to instantiate new objects.
Here we provide a constructor using the `__init__` member function name.  If no such member function exists
Asteroid would provide a default constructor to initialize the data members of the form `Dog(a,b)` where the value
`a` would be copied to the data member `name` and the value `b` would be copied to the `tricks` data member.
Asteroid generates an implicit object reference as the first argument to the called function.  Notice that at the call site  we only provide a single argument whereas the function definition has two arguments; the first one capturing the object reference.
```
load system "io".

structure Dog with

  data name.
  data tricks.

  function add_trick
    with (self, new_trick) do
      let self @tricks = self @tricks + [new_trick].
    end

  function __init__
    with (self, name) do
      let self @name = name.
      let self @tricks = [].
    end

  end

-- Fido the dog
let fido = Dog("Fido").
fido @add_trick("roll over").
fido @add_trick("play dead").

-- Buddy the dog
let buddy = Dog("Buddy").
buddy @add_trick("roll over").
buddy @add_trick("sit stay").

-- print out the tricks
println ("Fido: " + fido @tricks).
println ("Buddy: " + buddy @tricks).
```
The output is,
```
    Fido: [roll over,play dead]
    Buddy: [roll over,sit stay]
```
In order to demonstrate pattern matching with objects we add a third dog and add a list of dogs to our program. The resulting program below shows this and we also added code that iterates over the list of the dogs and prints out the names of the dogs whose first trick is `roll over`.  The filtering of the objects on the list is done via pattern matching in the `for` loop.
```
load system "io".

structure Dog with

  data name.
  data tricks.

  function add_trick
    with (self, new_trick) do
      let self @tricks = self @tricks + [new_trick].
    end

  function __init__
    with (self, name) do
      let self @name = name.
      let self @tricks = [].
    end

  end -- structure

-- Fido the dog
let fido = Dog("Fido").
fido @add_trick("roll over").
fido @add_trick("play dead").

-- Buddy the dog
let buddy = Dog("Buddy").
buddy @add_trick("roll over").
buddy @add_trick("sit stay").

-- Fifi the dog
let fifi = Dog("Fifi").
fifi @add_trick("sit stay").

-- print out all the names of dogs
-- whose first trick is 'roll over'.
let dogs = [fido, buddy, fifi].

for Dog(name, ["roll over"|_]) in dogs do
    println (name + " does roll over").
end
```
The output is,
```
    Fido does roll over
    Buddy does roll over
```

There is an elegant way of rewriting the last part of the code of the above example using the fact that in Asteroid patterns are first-class citizens.  In the program below we associate our pattern with the variable `dog`. The quote at the beginning of the pattern is necessary otherwise Asteroid will try to dereference the variable `name` as well as the anonymous variables `_`. We use the pattern associated with `dog` in the `for` loop in order to filter the objects on the list. The `*` operator is necessary in order to tell Asteroid to use the pattern associated with the variable `dog` rather than using the variable itself as a pattern.
```
load system "io".

structure Dog with

  data name.
  data tricks.

  function add_trick
    with (self, new_trick) do
      let self @tricks = self @tricks + [new_trick].
    end

  function __init__
    with (self, name) do
      let self @name = name.
      let self @tricks = [].
    end

end -- structure

-- Fido the dog
let fido = Dog("Fido").
fido @add_trick("roll over").
fido @add_trick("play dead").

-- Buddy the dog
let buddy = Dog("Buddy").
buddy @add_trick("roll over").
buddy @add_trick("sit stay").

-- Fifi the dog
let fifi = Dog("Fifi").
fifi @add_trick("sit stay").

-- print out all the names of dogs
-- whose first trick is 'roll over'.
let dogs = [fido, buddy, fifi].

-- define our pattern
let DOG = 'Dog(name, ["roll over"|_]).

-- iterate over dogs applying our pattern
-- only if the pattern match is successful will the loop body be executed
for *DOG in dogs do
  println (name + " does roll over").
end
```
The output again is,
```
    Fido does roll over
    Buddy does roll over
```

## Patterns as First Class Citizens

We have shown in the above program that patterns can be associated with and dereferenced from variables.  The program below illustrates that we can also pass patterns to functions where they can be used for pattern matching.  Here we define a function `match` that expects a subject term and a pattern.  It proceeds to pattern match the subject term to the pattern using the `is` predicate and returns whatever the predicate returns.  Observe the `*` operator in front of the `pattern` variable stating that we want to use the pattern associated with that variable.  In the program we call the function `match` with subject term `1+1` and pattern `_+_`.  
```
load system "io".

function match with (subject,pattern) do
    return subject %is *pattern.
    end

println (match('1+1, '_+_)).
```
The output is `true`.

We can also construct patterns on-the-fly as shown below.  Here we construct two subpatterns `cl` and `cr`.  These two subpatterns are used to construct the full pattern `p` when the pattern is evaluated during a pattern match. Finally, we check whether our pattern is assembled correctly on last line.  The output of the program is `true` meaning our pattern has the same structure as the subject term `1+2+3`.
```
load system "io".

let cl = '_ + _.
let cr = '3.
let p = 'cl + cr.

println (('1+2+3) is *p).
```
The output is `true`.

With Asteroid's ability to manipulate patterns we can rewrite the Peano addition program from above.  In the rewritten version below the pertinent Peano axioms are stored as rules in a rule table which the program will access during execution.   Our two Peano axioms appear as rules in the rule table.  Note that each rule is written as a pair where the first component is the left side of the corresponding rule and the second component is the right side of the corresponding rule.  The left sides of the rules represent the patterns that need to match the subject term and therefore it is not surprising that they are written as quoted expressions.  We also need to write the right sides of the rules as quoted expressions because we want to delay their evaluations until their corresponding patterns have matched an appropriate subject term.

The function `reduce` searches through the rule table for a match to the current subject term `term`.  If a match is found the corresponding right side of the rule is evaluated.  If no match is found then the term is returned unmodified.  The output of the program is of course the Peano term `S(S(S(S(S(0)))))`.
```
load system "io".

structure S with
    data x.
    end

structure add with
    data left.
    data right.
    end .

let rule_table = [
    ('add(x,0), 'reduce(x)),
    ('add(x,S(y)), 'S(reduce(add(x,y))))
    ].

function reduce
    with term do
        for i in 0 to rule_table@length() - 1 do
            let (lhs, rhs) = rule_table@i.
            if term %is *lhs do
                return eval rhs.
            end
        end
        return term.
    end

println (reduce('add(S(S(0)),S(S(S(0)))))).
```
As before, the output is `S(S(S(S(S(0)))))`.

## More on Exceptions

This section will give further information on how to solve **exceptions,** or unexpected conditions that break the regular flow of execution.

## Escaping Asteroid

The Asteroid interpreter is written in Python and the `escape` expression gives the user full access to the Python ecosystem from within Asteroid code.  In particular it gives the user access to the interpreter internals making it easy to write interpreter extensions.  The following example shows one way to incorporate graphics into Asteroid programs,
```
function circle with (x, y, r) do escape
"
from asteroid_state import state

# get the function parameters from the symbol table
vx = float(state.symbol_table.lookup_sym('x')[1])
vy = float(state.symbol_table.lookup_sym('y')[1])
vr = float(state.symbol_table.lookup_sym('r')[1])

# plot the circle at (vx,vy) with radius vr
import matplotlib.pyplot as plt

circle = plt.Circle((vx, vy), vr, color='blue')
fig, ax = plt.subplots()
ax.add_artist(circle)
plt.show()
"
end

-- call the escaped function
circle(.5, .5, .2)
```

## Basic Asteroid I/O

`Println` is a function that prints its argument in a readable form to the terminal.  Remember that under the standard model the `+` operator also implements string concatenation.  This allows us to construct nicely formatted output strings,
```
load system "io".

let a = 1.
let b = 2.
println ("a + b = " + (a + b)).
```
The output is
```
    a + b = 3
```

`Input` is a function that given a prompt string will prompt the user at the terminal and return the input value as a string.  Here is a small example,
```
load system "io".

let name = input("What is your name? ").
println ("Hello " + name + "!").
```
The output is,
```
    What is your name? Leo
    Hello Leo!
```

We can use the type casting functions such as `tointeger` or `toreal` to convert the string returned from `input` into a numeric value,
```
load system "io".
load system "util".

let i = tointeger(input("Please enter a positive integer value: ")).

if i < 0 do
    throw Error("I want a positive integer value.").
end

for k in 1 to i do
    println k.
end
```
The output is,
```
    Please enter a positive integer value: 3
    1
    2
    3
```

`Raw_print` is a function similar to `println` except that it outputs Asteroid's internal term structure for the given argument,
```
load system "io".

let a = 1.
let b = 2.
raw_print ("a + b = " + (a + b)).
```
The output is,
```
    ('string', 'a + b = 3')
```
Note that here we get the output value represented as a `(<type>,<value>)` tuple.

## The Module System

A module in Asteroid is a file with a set of valid Asteroid statements.  You can include this file into other Asteroid code with the `load "<filename>".` statement.  In the current version of Asteroid modules do not have a separate name space; symbols from a module are entered into Asteroid's global name space.

The search strategy for a module to be loaded is as follows,
1. raw module name - could be an absolute path
1. search in current directory (path[1])
1. search in directory where Asteroid is installed (path[0])
1. search in subdirectory where Asteroid was started

Say that you wanted to load the `math` module so you could execute a certain trigonometric function. The following Asteroid program imports the `math` module as well as the `io` (input/output) module. Only after importing them would you be able to complete the sine function below:
```
load system "io".
load system "math".

let x = sin( pi / 2 ).
println("The sine of pi / 2 is " + x + ".").
```
Both the function `sin()` and the constant value `pi` are defined in the `math` module. In addition, the `io` module is where all input/output functions in Asteroid (such as `println`) come from.
