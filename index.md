![asteroid](asteroid-small.png)

# Asteroid: The Programming Language

[Documentation](https://asteroid-lang.readthedocs.io) | [PyPI Project Page](https://pypi.org/project/asteroid-lang/) | [Try it!](https://replit.com/@lutzhamel/asteroid#.replit) | [GitHub Repository](https://github.com/asteroid-lang) 

Asteroid is a modern, application-oriented programming language designed from the ground up with the user in mind. Its expressive syntax is easy to learn and seamlessly supports procedural, functional, and object-based programming.  Its novel approach to pattern matching provides new solutions to old programming problems.

Here are some example programs that highlight various aspects of Asteroid.

### Hello, World!

Simple things are simple. Here is the ''Hello, World!'' program written in Asteroid,
```
load system io.
io @println "Hello, World!".
```

## Imperative Programming is Straightforward

Imperative programming in Asteroid should seem familiar to anybody who has some programming experience in languages like Python or JavaScript.
Intuitive  structure definitions and standard list notation makes it easy to create the data structures your
program needs.  Here is a  program that prints out the names of persons whose name contains a lower case 'p',
```
load system io.

-- define what a person look like
structure Person with
   data name.
   data age.
end

-- define a list of persons using default
-- constructors for person objects
let people = [
   Person("George", 32),
   Person("Sophie", 46),
   Person("Oliver", 21)
].

-- print names of persons that contain 'p' using structural,
-- conditional, and regular expression pattern matching
for Person(name if name is ".*p.*", age) in people do
   io @println name.
end
```
In the for-loop we use structural pattern matching on Person objects and then use regular expression matching on the name.  The output of this program is,
```
Sophie
```

## A Functional Programming Approach to Function Definitions

Asteroid supports functional programming style pattern matching on the arguments of a function.   
When a pattern matches the corresponding function body is executed.  Here is a  Quicksort implementation 
that demonstrates this functionality.  We see three distinct patterns (indicated by the `with` keyword) each with their own implementation of the corresponding function body,  
```
load system io.

function qsort
   with [] do -- empty list
      return [].
   with [a] do -- single element list
      return [a]. 
   with [pivot|rest] do -- head-tail operator
      let less=[].
      let more=[].
      for e in rest do  
         if e < pivot do
            less @append e.
         else do
            more @append e.
         end
      end
      return qsort less + [pivot] + qsort more.
 end

io @println (qsort [3,2,1,0]).
```
The last line of the program prints out the sorted list returned by the Quicksort.  The output is,
```
[0,1,2,3]
```

## Higher-Order Programming

Asteroid seamlessly supports functional programming style higher-order programming. Here is a program that creates a list 
of alternating positive and negative ones,
```
load system io.
load system math.

let a = [1 to 10] @map(lambda with x do math @mod(x,2))
                  @map(lambda with x do 1 if x else -1).

io @println a.
```
The list constructor `[1 to 10]` constructs a list of values `[1, 2,...,10]`.  The first `map`turns this list into the list
`[1,0,1,...0]` and the second call to `map` turns that list into the list `[1,-1,1,-1,...,-1]`.

## Pattern Reuse

One of the novel aspects of Asteroid is the ability to reuse patterns.  The following program defines two functions that have to deal 
with values over the same domains.  We can define patterns that describe these input values very precisely and then use these
patterns in both functions,
```
-- patterns that define positive and negative integers
let Pos_Int = pattern %[(x:%integer) if x > 0]%.
let Neg_Int = pattern %[(x:%integer) if x < 0]%.

-- define a function that computes the factorial recursively
-- Note: factorial is not defined over negative values
function fact
   with 0 do
      return 1
   with n:*Pos_Int do
      return n * fact (n-1).
   with *Neg_Int do
      throw Error("factorial undefined for negative values").
end

-- define the sign function that produces a 1 if the input >= 0
-- and -1 otherwise.
function sign
   with 0 do
      return 1
   with *Pos_Int do
      return 1.
   with *Neg_Int do
      return -1.
end
```
## Object-Oriented Programming in Asteroid

Asteroid supports OO programming.  Here is a program loosely based on the [dog example](https://docs.python.org/3/tutorial/classes.html) from the Python documentation.  This example builds a list of dog objects that all know some tricks.  After the dogs introduce themselves we loop over the list and find all the dogs that know to 'fetch'.
```
load system io.
load system type.

structure Dog with
   data name.
   data tricks.
   function __init__ with (name:%string, tricks:%list) do -- constructor
      let this@name = name.
      let this@tricks = tricks.
   end
   function hello_string with () do -- member function
      let hello_str = "Hello, my name is " + this@name + " and my tricks are ".
      let trick_str = this@tricks @reduce (lambda with (x,y) do x + " and " + y).
      return hello_str + trick_str.
   end
end

let fido = Dog("Fido",["play dead","fetch"]).
let buddy = Dog("Buddy",["sit stay","roll over"]).
let bella = Dog("Bella",["roll over","fetch"]).

let dogs = [fido,buddy,bella].

-- let dogs introduce themselves
for d in dogs do
   let hs = d @hello_string (). -- call member function on object
   io @println hs.
end

-- print out all the dogs that know how to fetch
for (Dog(name,tricks) if type @tostring tricks is ".*fetch.*") in dogs do
   io @println (name+" knows how to fetch").
end
```
Notice that we have a user supplied constructor function `__init__` as well as a member function `hello_string`
in the structure.  Object identity in functions in supplied via the `this` keyword.

What is perhaps striking in the for loop is that rather than searching through the list of tricks for a "fetch" trick for each dog
match at a loop iteration, we cast the list of tricks as a string
and then use regular expression matching on it to see if it contains a "fetch" trick. The output is,
```
Hello, my name is Fido and my tricks are play dead and fetch
Hello, my name is Buddy and my tricks are sit stay and roll over
Hello, my name is Bella and my tricks are roll over and fetch
Fido knows how to fetch
Bella knows how to fetch
```
You can try Asteroid without installing anything in our cloud-based Asteroid installation (see 'Try it!' below) or you can install Asteroid on your machine via 'pip' (see the PyPI Project Page below).


[Documentation](https://asteroid-lang.readthedocs.io) | [PyPI Project Page](https://pypi.org/project/asteroid-lang/) | [Try it!](https://replit.com/@lutzhamel/asteroid#.replit) | [GitHub Repository](https://github.com/asteroid-lang) 
