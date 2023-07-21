# Asteroid
Asteroid is a modern, multi-paradigm programming language that supports first-class patterns.  More details can
be found at the website [asteroid-lang.org](https://asteroid-lang.org).
Documentation on Asteroid can be found at
[asteroid-lang.readthedocs.io](https://asteroid-lang.readthedocs.io).

## New in Release 2.0.1

* We now support the ASTEROIDPATH environment variable which is expected to have a colon seperated list of directories to search for user defined modules.

* The logical operators 'and' and 'or' are now evaluated in short-circuit fashion.

* Files loaded with the 'load' statement are now considered modules and work similarly to Python modules.

* Added the 'toplevel' function which returns true if control is in the module originally loaded by the interpreter.

* Added the match statement similar to the match statement in Python.

* No longer supports type hierarchies for the primitive types.  The functions in the 'type' module are now considered builtins. The 'type' module itself has been eliminated.

* The Minimal Asteroid Debugger (MAD) replaces ADB in this release.

* The shorthand conditional pattern can now be applied to arbitrary patterns. E.g. The pattern 
  ```
  (a,b,c):(%integer,%integer,%integer)
  ```
  constrains the triple `(a,b,c)` to be a triple of integers.  The above shorthand conditional pattern is equivalent to the conditional pattern,
  ```
  (a,b,c) if (a,b,c) is (%integer,%integer,%integer)
  ```
  
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
