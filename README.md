# Asteroid the Programming Language

Asteroid is a general purpose programming language heavily influenced by [Python](https://www.python.org), [Lua](http://www.lua.org), [ML](https://www.smlnj.org), and [Prolog](http://www.swi-prolog.org) currently under development.  Asteroid implements a new programming paradigm called pattern-level programming.  Here are just a few small programs to give you the flavor of the langugage.

Here is the canonical factorial program written in Asteroid:

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
The following program shows off some of Asteroids pattern-level programming capabilities:

```
-- implements Peano addition on terms

-- declare the successor function S as a term constructor so that we 
-- can pattern match on it.
constructor S with arity 1.

-- the 'reduce' function is our reduction engine which recursively pattern matches and
-- rewrites the input term
-- NOTE: during pattern matching free variables are bound to subterms of the original term.
-- For example, the expression S S 0 + 0 is X + 0 will bind X to S S 0 
-- Once a pattern value is bound to a variable it can 
-- be used in the program.  In our case we use the values in the variables to 
-- construct new terms, i.e., S reduce (X + Y)
function reduce
    with X + 0 do                      -- pattern match 'X + 0'
        return reduce X.
    orwith X + S(Y)  do                -- pattern match to 'X + S Y'
        return S(reduce(X + Y)).
    orwith term do                     -- default clause
        return term.
    end function

-- construct a term we want to reduce  
let n = 'S(S(0)) + (S(S(S(0)))).

-- and reduce it!
let rn = reduce n.

-- attach inc behavior/interpretation to the S constructor
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

Here is something a bit more mundane: the quicksort algorithm implemented in Asteroid.  Highlighted here is Asteroid's
pattern match capabality on lists:

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

Asteroid has a very flexible view of expressions and terms which allows the programmer to attach new interpretations to
constructor symbols on the fly:

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

Asteroid also supports prototype-based OO style programming:

```
-- an OO example
load "standard".
load "io".
load "util".

-- Our Dog type constructor
constructor Dog with arity 3.

-- the prototype object
let dog_proto = Dog (
            ("name", ""),
            ("trick", ""),
            ("make_string", 
                lambda with self do return self@{"name"} + " does " + self@{"trick"})
        ).

-- Fido the dog
let fido = copy dog_proto.
let fido@{"name"} = "Fido".
let fido@{"trick"} = "play dead".
print (fido@{"make_string"}()).

-- Buddy the dog
let buddy = copy dog_proto.
let buddy@{"name"} = "Buddy".
let buddy@{"trick"} = "roll over".
print (buddy@{"make_string"}()).
```

For more details look at the 'Asteroid - The Language' notebook.
