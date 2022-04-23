.. role:: raw-html(raw)
    :format: html
    
Welcome To Asteroid!
====================
Thank you for visiting our page about Asteroid! If you have gone to the `Asteroid Documentation <https://asteroid-lang.readthedocs.io/en/latest/>`_ then you have seen that Asteroid is a pattern-matching object-oriented language.
If you have not heard of that programming paradigm (type of programming language) before, Asteroid is the first of its kind. Before you start coding in Asteroid, it would be beneficial to understand what pattern-matching actually is, since the langauges you have worked on in the past may not have been this same paradigm.

What is Pattern-Matching?
-------------------------
Pattern-matching is the idea of testing an epxression to determine if it has certain characteristics. For instance, look at this example program of a function that finds the factorial of a number in the prgramming langauge SML:  
::
    fun f (0) = "zero"
        | fact (x) = "not zero";
        
As you can see, there are two "patterns" that this function can have; either have the function be called with 0 (in this case the function would output "zero") or have the function called with something that is not zero (in this case, "not zero" would be outputted). Depending on what the function call is, the appropriate line will get executed. This is the basis of pattern-matching. In this particular example, we are testing what the function call of f will be; either with 0 or with a number, and depending on which pattern is satisfied, that line will get executed.

This is an important note to remember because **not all programming languages have pattern-matching implemented into them**. C++, without any extra libraries, does not have pattern-matching built into it. If you wanted the same function in C++, it would look something like:
::
    void f(x) {
        if x == 0 {
            std::cout << "zero\n";
        } else {
            std::cout << "not zero\n";
        }
    }

Notice that there are no patterns in this function; the programmer had to use conditionals (which is not a bad practice, however the function looks less cluttered with patterns and also not every programming language has conditionals, however Asteroid does).  
:raw-html:`<br />`
Here is an example of pattern matching in Asteroid:
::
    let POS_INT = pattern with (x:%integer) if x > 0.
    let NEG_INT = pattern with (x:%integer) if x < 0.

In this above example, there are two patterns for where the variable x can fall into: either a positive number (where the variable name would be POS_INT) if it is greater than 0 or a negative number (where the variable name would be NEG_INT) if it is less than 0. While this seems pointless having just the patterns with no use, here is a function to find the factorial of a number using the above mentioned patterns (just for reference, -- signifies a comment in Asteroid):
:: 
    -- define our factorial function
    function fact
    with 0 do
        return 1
    with n:*POS_INT do        -- use first pattern
        return n * fact (n-1).
    with n:*NEG_INT do        -- use second pattern
        throw Error("undefined for "+n).
    end

As you can see, this function does three different things depending on which pattern is present. If the function gets called with 0 passed into it, then the function returns 0. If the function gets called with a positive number (so the variable POS_INT gets used), then the function will recursively find the factorial of that number. If the function gets called with a negative number (so the variable NEG_INT gets used), then the function will throw an error. Patterns and pattern-matching are very powerful tools because they can change how some code is executed depending on what pattern is met.

What does Object Oriented Mean?
-------------------------------
The term object-oriented in programming means that code is broken up into classes and objects. Think of classes as **user defined data types**. While this may sound intimidating, there are many uses of object-oriented programming that can be used to help write efficent, clean code. For instance, there may be a time where you have to write code for software that pertains to families. While you could use tuples or arrays to represent this data, objects and classes are an even better way to achieve this feat. Take a look at this code in C++ that has the class for a family:
::
    class Family {
        public:
            String parent
            String child1
            String child2

    };

Now if you wanted to access one of those properties (the variables parent, child1 and child2), all you would have to do is:
::
    Family myFamily; // creates object of class
    myFamily.parent = "Jim"; // sets the property parent to "Jim"
    myFamily.child1 = "Bob"; // sets the property child1 to "Bob"
    myFamily.child2 = "Ann"; // sets the property child2 to "Ann"

Classes and objects are an easier way to store data that may not fit with any data structure that the language currently has. Asteroid does not have classes and objects, however it has structs which are very similar and can be used in the same way:
::
    structure Family with  
        data parent.
        data child1.
        data child2.
        end
    
    let myFamily = Family("Jim", "Bob", "Ann").

As seen above, this is now a data type (defined by the programmer) that can be used to store data that may be specific to the program that you are working on.
:raw-html:`<br />`
Now that you understand the two different paradigms that Asteroid is made out of, you can start writing your programs in it and explore the versatility of patterns, pattern-matching and object-oriented programming.

How to Get Started in Asteroid
-------------------------------
Now that you know what principles Asteroid is made of, you can now get started in writing programs in it. Directions to install Asteroid can be found at `here <https://asteroid-lang.readthedocs.io/en/latest/Installing%20and%20Running.html>`_. 
After you installed Asteroid correctly, you can write your first program. The first one you can write is a simple hello world program, which looks something like:
::
    -- the obligatory hello world program

    load system io.

    println "Hello, World!".

After you have written your first program, you can run the program by typing in the following line in your terminal:
::
    asteroid <*name of program*>

where name of program is the name of the file that you want to run. 
:raw-html:`<br />`
**Make sure that you are in the same folder in your terminal of the file that you are trying to run!**


