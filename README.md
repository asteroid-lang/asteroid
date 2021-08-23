![asteroid](asteroid-small.png)

# Asteroid: The Programming Language

Asteroid is an open source, dynamically typed, multi-paradigm programming language heavily influenced by [Python](https://www.python.org), [Rust](https://www.rust-lang.org), [ML](https://www.smlnj.org), and [Prolog](http://www.swi-prolog.org) currently under development at the University of Rhode Island.  Asteroid implements a new programming paradigm called *pattern-matching oriented programming*.  In this new programming paradigm patterns and pattern matching are supported by all major programming language constructs making programs succinct and robust.  Furthermore, patterns themselves are first-class citizens and as such can be passed to and returned from functions as well as manipulated computationally.

## The Basics

OK, before we get started here is the obligatory ''Hello World!'' program written in Asteroid,
```
load system "io".
println "Hello World!".
```
Since pattern matching is at the core of Asteroid we find that the simplest pattern matching occurs in Asteroid's `let` statement. For example,
```
let [x,2,y] = [1,2,3].
```
here the list  `[1,2,3]` is matched against the pattern `[x,2,y]` successfully with the corresponding assignments x→1 and y→3. Pattern matching can also occur in iteration. Consider the following program that prints out the names of persons whose name contains a lower case 'p',
```
load system "io".

-- define what persons look like

structure Person with
    data name.
    data age.
    data gender.
    end

-- define a list of persons

let people = [
    Person("George", 32, "M"),
    Person("Sophie", 46, "F"),
    Person("Oliver", 21, "X")
    ].

-- print names of persons that contain 'p'

for Person(name:".*p.*",_,_) in people do
  println name.
end
```
In the for-loop we pattern-match Person objects and then use regular expressions on the name.  The output of this program is,
```
Sophie
```

## Pattern Matching in Functions

Asteroid supports functional programming style pattern matching on the arguments to a function dispatching the function implementation corresponding to the pattern matched.  This is related to multiple dispatch implemented in languages like [Raku](https://www.raku.org/). The following Quicksort implementation demonstrates this functionality.  Here we see three distinct patterns each with their own implementation of the corresponding function body,  
```
-- Quicksort

load system "io".

function qsort
    with [] do
        return [].
    orwith [a] do
        return [a].
    orwith [pivot|rest] do -- head-tail operator
        let less=[].
        let more=[].
        for e in rest do  
            if e < pivot do
                let less = less + [e].
            else do
                let more = more + [e].
            end
        end

        return qsort less + [pivot] + qsort more.
    end

println (qsort [3,2,1,0]).
```
The last line of the program prints out the sorted list returned by the Quicksort.  The output is,
```
[0,1,2,3]
```

## Type and Conditional Patterns

Asteroid supports patterns that match whole type classes.  For instance, `%integer` matches any integer but will fail with any other type.  A conditional pattern is a pattern that only matches if a condition is fulfilled in addition to the structural match.  Here is a recursive implementation of the factorial function where we use type and conditional patterns to appropriately restrict the domain of the function,
```
load system "io".
load system "util".

function fact
    with 0 do
        return 1
    orwith (n:%integer) %if n > 0 do
        return n * fact (n-1).
    orwith (n:%integer) %if n < 0 do
        throw Error("factorial is not defined for "+n).
    end

println ("The factorial of 3 is: " + fact (3)).
```
We use the type pattern `%integer` to restrict the domain of the function to integer values and then we use the conditional patterns to select the appropriate function implementation.  The pattern `n:%integer` is called a named pattern where the variable `n` binds to whatever integer the type pattern `%integer` has matched.  That binding becomes immediately available for usage as can be seen in the conditional patterns.

Type patterns are extremely useful in dynamically typed languages like Asteroid in order to provide some additional type safety that would otherwise not be available. Type patterns are also available for user defined types. Consider,
```
load system "io".
load system "util".

structure A with
    data a.
    data b.
    end

let a = A(1,2).
let v:%A = a.

println (tostring v).
```
In the pattern `v:%A` the variable `v` will be bound to objects that the type pattern `%A` matches and that pattern will only match objects of type `A`.

## The `is` Predicate

Pattern matching can be performed anywhere in Asteroid programs where expressions are allowed using the `is` predicate.  Here is a simple program that demonstrates the `is` predicate,
```
let true = (1,2) is (x,y).
assert ((x,y) is (1,2)).
```
The right operand to the predicate is the pattern and the left the subject term.  The first statement destructures the `(1,2)` tuple and the second statement asserts that the tuple constructed from the constituent parts has the same structure as the original tuple.

The `is` is useful for providing pattern matching capabilities in control structures like if-then-else statements and loops.

## Patterns as First-Class Citizens

One of the distinguishing features of Asteroid is the fact that it supports patterns as first-class citizens.  That means, patterns can be stored in variables, passed to functions, and computationally manipulated.  Here is the factorial program from above rewritten using first-class patterns,
```
load system "io".
load system "util".

let POS_INT = pattern with (x:%integer) %if x > 0.
let NEG_INT = pattern with (x:%integer) %if x < 0.

function fact
    with 0 do
        return 1
    orwith n:*POS_INT do
        return n * fact (n-1).
    orwith n:*NEG_INT do
        throw Error("factorial is not defined for "+n).
    end

println ("The factorial of 3 is: " + fact (3)).
```
The interesting part of first-class patterns is that the definition point and the use-point of patterns are physically separated resulting in code that is much easier to read and in addition to that it makes patterns reusable resulting in highly maintainable code.

## Object-Oriented Programming in Asteroid

Asteroid also supports OO style programming.  Here is the [dog example](https://docs.python.org/3/tutorial/classes.html) from the Python documentation implemented in Asteroid.  This example builds a list of dog objects that all know some tricks.  We then loop over the list and find all the dogs that know "roll over" as their first trick using pattern matching. The `[ _ | _ ]` is known as the head-tail operator related to the cons function in Lisp and allows you to decompose a list into the first element and the rest of the list.
```
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
```
The output is,
```
Fido knows how to fetch
```

## For more Information...

Take a look at the [Asteroid User Guide](https://github.com/lutzhamel/asteroid/blob/master/Asteroid%20User%20Guide.md) for a more detailed discussion of the language.

Check out the [Using Asteroid](https://github.com/lutzhamel/asteroid/blob/master/Using%20Asteroid.md) document which is based on Andrew Shitov's excellent book [Using Raku: 100 Programming Challenges Solved with the Brand-New Raku Programming Language](https://andrewshitov.com/wp-content/uploads/2020/01/Using-Raku.pdf).

Try Asteroid online without anything to install at [Repl.it](https://repl.it/@lutzhamel/asteroid#README.md)
