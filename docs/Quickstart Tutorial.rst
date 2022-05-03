.. role:: raw-html(raw)
   :format: html

Welcome To Asteroid!
====================
Thank you for visiting our page about Asteroid! If you have gone to the `Asteroid Documentation <https://asteroid-lang.readthedocs.io/en/latest/>`_ then you have seen that Asteroid is a pattern-matching oriented language.
If you have not heard of that programming paradigm (type of programming language) before, Asteroid is one of the first of its kind. Here we provide a brief introduction to Asteroid geared towards C++ programmers.

Pattern Matching at the Core of Things
--------------------------------------
Pattern-matching is the idea of extracting values from a structure by applying a pattern to that structure.  If the pattern
contains variables then these variables will be instantiated with values from the structure during a successful match.  Consider the structure ``(1,2)``.
If we apply the pattern ``(1,x)`` to that structure then the ``1`` will be matched and the variable ``x`` will be instantiated
with the value ``2``.  The interesting part about pattern matching is that they can also fail to match.  Consider again
our structure ``(1,2)`` but now we want to apply the pattern ``(y,1)``.  This pattern match will fail because the second component of our structure and the pattern do not match.  Because of this mismatch the variable ``y`` will also not be instantiated.  Perhaps the simplest pattern match is between a value
and a variable as a pattern.  In that case the variable is simply instantiated with the value.

A very interesting aspect of pattern matching is that it provides a powerful way to inspect the values passed to a function.
Consider the C++ function ``f`` defined here,
::
  #include <iostream>

  void f(int x) {
    if (x == 0)
      std::cout << "zero\n";
    else
      std::cout << "not zero\n";
  }

If the integer value passed to the function is equal to zero then it prints out the word ``zero``, otherwise it prints out
the words ``not zero``.
Here the programmer has to use conditionals in order to decide which output to produce.  Using conditionals is not a bad practice, however,
functions that use pattern tend to look less cluttered.

Here is the same function written in Asteroid using pattern matching,
::
  load system io.

  function f
     with 0 do
        io @println("zero").
     with x do
        io @println("not zero").
     end

Here the value ``0`` and the variable ``x`` appearing right after the ``with`` keywords are patterns that are applied to the function value.
If the function is called with the value ``0`` then the value will be matched by the first pattern and the associated print statement is executed.
If the value with which the function was called is anything but ``0`` then the second pattern will match and that associated print statement
will be executed. A variable can be considered a pattern
that will match any value (unless other conditions have been placed on that variable).

The interesting part about patterns is that they can include all kinds of additional information like, for example, that the value a pattern is to
match is a positive integer. Consider the following example of the recursive factorial implementation.  Let's look at that function in C++,
::
     int fact(int x) {
       if (x == 0)
          return 1;
       else if (x > 0)
          return x * fact(x-1);
       else
         throw "undefined for negative values";
     }

Factorials are only defined over positive integers.  So notice that in this case we have to declare the argument as an integer argument and then
later in the function we need to make sure that the integer value being passed in is in the correct range of values in order for the computation to be successful.
With pattern matching in Asteroid we can accomplish all this right in the patterns that we apply to the input argument,
::
   -- define patterns matching positive or negative integer values
   let POS_INT = pattern %[(k:%integer) if k > 0]%.
   let NEG_INT = pattern %[(k:%integer) if k < 0]%.

   -- define our factorial function
   function fact
       with 0 do
           return 1
       with x:*POS_INT do            -- use first pattern
           return x * fact(x-1).
       with x:*NEG_INT do            -- use second pattern
           throw Error("undefined for "+x).
       end

The first two lines of the program create the patterns that define what it means to be a positive or negative integer. For example, the first pattern will only match a value that is an integer whose value is larger than zero. Later in the program, these patterns get dereferenced (which means retrieved from where they are stored in memory) using the ``*`` operator. Notice that we have a similar setup here as with the ``f`` function we looked at earlier.  If the ``0`` pattern matches then we will just return the value ``1``. The line after that is saying "with the argument x and the pattern POS_INT (or in other words, if the argument is positive), recursively find the factorial of the number" and the last ``wth`` line is saying "with the argument x and the pattern NEG_INT (if the argument is negative), throw an error".
Notice that patterns allow us to precisely define what we mean by positive or negative integers in one place and then use these patterns in our function.

Pattern matching can be applied in a lot of places in Asteroid.  But one other place is perhaps more prevalent than any other, which is pattern
matching in Asteroid's ``let`` statement.
The ``let`` statement is Asteroid's version of the assignment statement with a twist though:  the left side of the ``=`` sign is not just a variable
but is considered a pattern.  For simple assignments there is no discernible difference between assignments in Asteroid and assignments in other
languages,
::
  let x = val.

Here, the variable ``x`` will match the value stored in ``val``.  However, because the left side of the ``=`` sign is a pattern we
can write something like this,
::
  let x:%[(k:%integer) if mod(k,2)==0]% = val.

where ``x`` will only match the value of ``val`` if that value is an even integer value.  The fact that the left side of the ``=`` is a pattern allows
us to write things like this,
::
   let 1 = 1.

which simply states that the value ``1`` on the right can be matched by the pattern ``1`` on the left.  Having the ability to pattern match
on literals is convenient for statements like these,
::
  let (1,x) = p.

This ``let`` statement is only successful for values of ``p`` which are pairs where the first component of the pair is the value ``1``.
**The thing to remember is that the let statement is not entirely equivalent to the assignment operator in other languages, even though it may look like that.**


Object-Oriented Programming in Asteroid
---------------------------------------
The term object-oriented in programming means that code is broken up into classes and objects. Think of classes as **user defined data types**. While this may sound intimidating, there are many uses of object-oriented programming that can be used to help write efficient, clean code. For instance, there may be a time where you have to write code for software that pertains to families. While you could use tuples or arrays to represent this data, objects and classes are an even better way to achieve this feat. Take a look at this code in C++ that has the class for a family:
::
     #include <string>

     class Family {
     public:
         std::string parent;
         std::string child1;
         std::string child2;

         // constructor
         Family(std::string p, std::string c1, std::string c2) {
             this->parent = p;
             this->child1 = c1;
             this->child2 = c2;
         }
     };

Now if you want to create an instance or object of the Family class, you could write this line to do so:
::
   Family *myfamily = new Family("Jim", "Bob", "Ann");

where the properties parent is "Jim", child1 is "Bob" and child2 is "Ann". Now if you wanted to access one of these properties, you could do,
::
   std::cout << myfamily->child1; // while this looks intimidating, all this is doing is dereferencing child1

Classes and objects are an easier way to store data that may not fit with any data structure that a language currently has.
Asteroid implements object-orientation via structures, an approach it shares with the programming language Rust.
In Asteroid the above example would be written as,
::
   structure Family with
       data parent.
       data child1.
       data child2.

       -- constructor
       function __init__ with (p:%string, c1:%string, c2:%string) do
          let this @parent = p.
          let this @child1 = c1.
          let this @child2 = c2.
       end
   end

And you can create an object from that structure by doing,
::
   let myfamily = Family("Jim", "Bob", "Ann").

Notice how similar the construction of objects are in both languages.
**Think of structures in Asteroid as classes in C++, and in both languages these allow you to instantiate objects** (that means if you have programmed with classes and objects in C++, creating structures in Asteroid should be trivial). Something else to note is that similar to Rust and Go, **Asteroid does not have inheritance for classes**.  That is why programming in Asteroid is sometimes referred to as object-based programming rather than object-oriented programming.

We can access substructures of objects with the access operator ``@``,
::
  io @println (myfamily @child1).

which will print out the name of the first child.

The name of the class above can now be considered a user defined data type and can appear wherever built-in data type names can appear.
For instance it can appear in a pattern restricting the values a particular variable can take on,
::
  let f:%Family = myfamily.

Since we are talking about the ``let`` statement in conjunction with objects, Asteroid allows pattern matching on objects!  This allows for
easy access to substructures of objects,
::
     let Family(parent,first,second) = myfamily.

     assert(parent is "Jim").
     assert(first is "Bob").
     assert(second is "Ann").

Here we are matching the object stored in ``myfamily`` again the pattern ``Family(parent,first,second)`` and the variables will be instantiated
with appropriate values from the ``data`` members of the object.

Now that you understand the two different paradigms that Asteroid is made out of, you can start writing your programs in it and explore the versatility of patterns, pattern-matching and object-oriented programming.


How to Get Started in Asteroid
-------------------------------
Now that you know what principles Asteroid is made of, you can now get started writing programs in it. Directions to install Asteroid can be found `here <https://asteroid-lang.readthedocs.io/en/latest/Installing%20and%20Running.html>`_.
After you installed Asteroid correctly, you can write your first program. The first one you can write is a simple hello world program, which looks something like:
::
   load system io. -- header that allows the programmer to print things out to the screen and to accept input

   io @println "Hello, World!".

After you have written your first program, you can run the program by typing in the following line in your terminal:
::
   asteroid <name of program>

where the name of the program is the name of the file that you want to run.

**Make sure that you are in the same folder in your terminal of the file that you are trying to run!**

Notice how the ``@`` symbol is used in two different places (this is common in programming languages, where one operator can be used multiple ways). In Asteroid, modules (which was the ``load system io.`` line at the top of our files) are actually objects, so to access a method in a module, you use the ``@`` symbol. So in this example, the module is the ``io`` module and we want to use the ``println`` method in that module, which is why you see the ``@`` symbol in there. **A module is a group of code that has already been written (typically by the developers of the language) which can be used again in other people's programs.**

`Here <https://asteroid-lang.readthedocs.io/en/latest/Reference%20Guide.html#asteroid-modules>`_ is the complete list of modules in Asteroid.

Some important things to note in Asteroid:

* Most statements must end with a period (this is equivalent to using a semicolon in C++)
* In order to print things, you must include the ``load system io.`` in your program before you attempt any output.
* lines that start with ``--`` are comment lines
* If you see a line that looks like (``x:%integer``), that is used to match any value of a given type. (The ``%integer`` pattern matches any integer value and can be used with any other type in Asteroid.)

If you would like more information about Asteroid, please see the Asteroid `reference guide <https://asteroid-lang.readthedocs.io/en/latest/Reference%20Guide.html#>`_  and `user guide <https://asteroid-lang.readthedocs.io/en/latest/Asteroid%20User%20Guide.html>`_.
