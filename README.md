# Asteroid
Asteroid is a modern, multi-paradigm programming language supporting first-class patterns and pattern-matching
oriented programming.  More details can
be found at the website [asteroid-lang.org](https://asteroid-lang.org).
Documentation on Asteroid can be found at
[asteroid-lang.readthedocs.io](https://asteroid-lang.readthedocs.io).

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
