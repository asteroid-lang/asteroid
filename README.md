![](asteroid-clipart.jpg)

# Asteroid the Programming Language

Asteroid is a general purpose programming language heavily influenced by [Python](https://www.python.org), [Lua](http://www.lua.org), [ML](https://www.smlnj.org), and [Prolog](http://www.swi-prolog.org) currently under development at the University of Rhode Island.  Asteroid implements a new programming paradigm called pattern-matching oriented programming.  In this new programming paradigm patterns and pattern matching is supported by all major programming language constructs making programs succinct and robust.

OK, before we get started here is the obligatory *Hello World!* program,
```
load "io".

print "Hello World!".
```
Since pattern matching is at the core of Asteroid we find that
the simplest pattern matching occurs in Asteroid's `let` statement. For example,
```
let [x,2,y] = [1,2,3].
```
here the list `[1,2,3]` is matched against the pattern `[x,2,y]` successfully with the corresponding assignments `x` &map; 1 and `y` &map; 3.

Pattern matching can also occur in iteration. Consider,
```
load "io".

let list = [1,2,3].

repeat do
    let [head|tail] = list. -- head-tail operator
    print head.
    let list = tail.
until list is [_].
```
Here we use the head-tail operator as a pattern to match a list. The loop iterates until the list from applying the head-tail operator has exactly one element in it.  The output of this program is:
```
1
2
```
Pattern matching can also happen on function arguments using the `with` or `orwith` keywords.
Here is the canonical factorial program written in Asteroid,

```
-- Factorial

load "standard".
load "io".

function fact 
    with 0 do
        return 1
    orwith n do
        return n * fact (n-1).
    end function

print ("The factorial of 3 is: " + fact (3)).
```
The following quicksort implementation has slightly more complicated patterns for the function
arguments. Asteroid inherits this functionality from functional programming languages such as ML.  
```
-- Quicksort

load "standard".
load "io".

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
            else do
                let more = more + [e].
            end if
        end for
                        
        return qsort less + [pivot] + qsort more.
    end function
    
print (qsort [3,2,1,0])
```
The last line of the program prints out the sorted list returned by the quicksort.  The output is,
```
[0,1,2,3]
```
The fact that Asteroid supports matching in all of its major programming constructs and that it has a very flexible view of the interpretations of experssion terms allows the developer to embed symbolic computation right into their programs. The following is a program that uses the [Peano axioms for addition](https://en.wikipedia.org/wiki/Peano_axioms#Addition) to compute addition symbolically.

```
-- implements Peano addition 

-- declare the successor function S as a term constructor  
constructor S with arity 1.

-- the 'reduce' function is our reduction engine which recursively pattern matches and
-- rewrites the input term
function reduce
    with a + 0 do                      -- pattern match 'a + 0'
        return reduce a.
    orwith a + S(b)  do                -- pattern match to 'a + S(b)'
        return S(reduce(a + b)).
    orwith term do                     -- default clause
        return term.
    end function

-- construct a term we want to reduce  
let n = 'S(S(0)) + (S(S(S(0)))).

-- and reduce it!
let rn = reduce n.

-- attach inc interpretation to the S constructor
load "standard".
load "util".
load "io".

function inc 
    with n do
        return n + 1.
    end function
    
attach inc to S.

-- show that with this behavior both the original term and the rewritten term
-- evaluate to the same value
print ((eval rn) == (eval n)).
```
The output of this program is,
```
True
```
As mentioned above, Asteroid has a very flexible view of the interpretation of expression terms which allows the programmer to attach new interpretations to constructor symbols on the fly.  Consider the following program which attaches a new interpretation to the `+` operator symbol, performs a computation, and then removes that interpretation restoring the original interpretation,
```
load "standard".  -- load the standard operator interpretations
load "io".        -- load the io system

function funny_add    -- define a function that given two 
    with a, b do      -- parameters a,b will multiply them
        return a * b.
    end function

attach funny_add to __plus__.   -- attach 'funny_add' to '+'
print (3 + 2).                  -- this will print out the value 6
detach from __plus__.           -- restore default interpretation
print (3 + 2).                  -- this will print out the value 5

-- NOTE: '__plus__' is a special symbol representing the '+' operator
```

Asteroid also supports prototype-based OO style programming inspired by Lua.  Here is the [dog example](docs.python.org/3/tutorial/classes.html) from the Python documentation cast into Asteroid.  This example builds a list of dog objects that all know some tricks.  We then loop over the list and find all the dogs that know "roll over" using pattern matching.

```
load "standard".
load "io".
load "util".

constructor Dog with arity 3.

-- assemble the prototype object
let dog_proto = Dog (
  ("name", ""),
  ("tricks", []),
  ("add_trick",
     lambda with (self,new_trick) do
         let self@{"tricks"} = self@{"tricks"}+[new_trick])).

-- Fido the dog
let fido = copy dog_proto.
let fido@{"name"} = "Fido".

fido@{"add_trick"} "roll over".
fido@{"add_trick"} "play dead".

-- Buddy the dog
let buddy = copy dog_proto.
let buddy@{"name"} = "Buddy".

buddy@{"add_trick"} "roll over".
buddy@{"add_trick"} "sit stay".

-- Fifi the dog
let fifi = copy dog_proto.
let fifi@{"name"} = "Fifi".

fifi@{"add_trick"} "sit stay".

-- print out all the names of dogs
-- whose first trick is 'roll over'.
let dogs = [fido, buddy, fifi]. 

-- use pattern matching to find all the dogs whose first trick is "roll over"
for Dog(("name",name), ("tricks",["roll over"|_]), _) in dogs do
  print (name + " does roll over").
end for

```
Take a look at the [Asteroid User Guide](Asteroid%20User%20Guide.ipynb) notebook for a more detailed discussion of the language.
