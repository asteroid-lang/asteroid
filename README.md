<!-- ![](asteroid-clipart.jpg) -->
<img src="asteroid-clipart.jpg" height="42" width="42">

# Asteroid the Programming Language

Asteroid is an open-source, multi-paradigm programming language heavily influenced by [Python](https://www.python.org), [Rust](https://www.rust-lang.org), [ML](https://www.smlnj.org), and [Prolog](http://www.swi-prolog.org) currently under development at the University of Rhode Island.  Asteroid implements a new programming paradigm called *pattern-matching oriented programming*.  In this new programming paradigm patterns and pattern matching is supported by all major programming language constructs making programs succinct and robust.  Furthermore, patterns themselves are first-class citizens and as such can be passed and returned from functions as well as manipulated computationally.

## The Basics

OK, before we get started here is the obligatory *Hello World!* program written in Asteroid,
```
load "io".

println "Hello World!".
```
Since pattern matching is at the core of Asteroid we find that
the simplest pattern matching occurs in Asteroid's `let` statement. For example,
```
let [x,2,y] = [1,2,3].
```
here the list `[1,2,3]` is matched against the pattern `[x,2,y]` successfully with the corresponding assignments `x` &map; 1 and `y` &map; 3.

Pattern matching can also occur in iteration. Consider the following program that prints out the names of persons whose name contains a lower case 'p',
```
load "io".

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
In the for-loop we pattern-match Person objects and then use regular expressions on the name.  The output of this program is,
```
Sophie
```

## Pattern Matching in Function Arguments

Pattern matching can also happen on function arguments using the `with` or `orwith` keywords.  This can be viewed as multiple dynamic dispatch.
Here is the canonical factorial program written in Asteroid,

```
-- Factorial

load "io".

function fact
    with 0 do
        return 1
    orwith n do
        return n * fact (n-1).
    end

println ("The factorial of 3 is: " + fact (3)).
```
As one would expect, the output is,
```
The factorial of 3 is: 6
```

The following quicksort implementation has slightly more complicated patterns for the function
arguments. Asteroid inherits this functionality from functional programming languages such as ML.  
```
-- Quicksort

load "io".

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

print (qsort [3,2,1,0])
```
The last line of the program prints out the sorted list returned by the quicksort.  The output is,
```
[0,1,2,3]
```

## Object-Oriented Programming in Asteroid

Asteroid also supports prototype-based OO style programming.  Here is the [dog example](docs.python.org/3/tutorial/classes.html) from the Python documentation cast into Asteroid.  This example builds a list of dog objects that all know some tricks.  We then loop over the list and find all the dogs that know "roll over" as their first trick using pattern matching.

```
load "io".

structure Dog with
    data name = "".
    data tricks = [].

    function __init__
      with (self, name) do
        let self@name = name.
        let self@tricks = [].
      end

    function add_trick
      with (self, new_trick) do
        let self@tricks = self@tricks + [new_trick].
      end
    end

-- Fido the dog
let fido = Dog("Fido").
fido@add_trick("roll over").
fido@add_trick("play dead").

-- Buddy the dog
let buddy = Dog("Buddy").
buddy@add_trick("roll over").
buddy@add_trick("sit stay").

-- print out the tricks
println ("Fido's tricks: " + fido@tricks).
println ("Buddy's tricks: " + buddy@tricks).
```
The output of this program is,
```
Fido's tricks: [roll over,play dead]
Buddy's tricks: [roll over,sit stay]
```

## Patterns as First-Class Citizens

As we mentioned at the beginning, in Asteroid patterns are first-class
citizens.  The following is a small program that demonstrates that,
```
load "io".

function match with (subject, pattern) do
    return subject is *pattern.
end

println (match('1+1, '_+_)).
```
Here the function `match` takes a subject term and a pattern and returns
`true` if there is a match otherwise it returns `false`. The quote character prevents Asteroid from immediately trying to evaluate that term.

## For more Information...

Take a look at the [Asteroid User Guide](https://nbviewer.jupyter.org/github/lutzhamel/asteroid/blob/master/Asteroid%20User%20Guide.ipynb) notebook for a more detailed discussion of the language.
