# Asteroid
Asteroid is a modern, multi-paradigm programming language that supports first-class patterns.  More details can
be found at the website [asteroid-lang.org](https://asteroid-lang.org).
Documentation on Asteroid can be found at
[asteroid-lang.readthedocs.io](https://asteroid-lang.readthedocs.io).

## New in Release 1.1.4

* Allows pattern constraint operator to map certain variables that a pattern 
  matched to be bound into the current scope.

* Supports a new OS module.

* Supports iteration over tuple components

* The `eval` function has been extended to strings for the interpretation of 
  Asteroid programs in those strings.

* Structures now support a `__str__` member function that allows the user
  to specify a string representation of the structure.
  
* Updated user and reference guides.

* Lots of bug fixes.


## New in Release 1.1.3

* Fixes a fatal bug on Windows regarding the `readline` functionality in Python 3.10.

* Fixes a bug with escaped double quotes in strings.

## New in Release 1.1.2

* An experimental implementation of an interactive debugger that supports debugging pattern matching and in
  particular, pattern matching with first-class patterns.

* A Python API allowing the developer to call the Asteroid interpreter from within a Python program and also embed
  Python code in an Asteroid program.

* Eliminated the `stride` notation in list comprehensions and replaced it with the `step` notation.

* Lots of bug fixes.


## New in Release 1.0.0

* Interpreter line-editing features similar to Python include interactive editing, history substitution and code completion on systems that support the GNU Readline library.

* New object-based modules for system modules.  For example the functions within the
  `io` module are now accessed with the `@` operator,
  ```
  load system io.
  io @println "Hello, World!".
  ```

* In the absence of explicit return statements the last expression evaluated within
  a function body provides an implicit return value, e.g.,
  ```
  function inc
     with i do
       i+1
     end
  ```

* Lots of bug fixes!
