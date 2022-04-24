.. role:: raw-html(raw)
    :format: html
    
Welcome To Asteroid!
====================
Thank you for visiting our page about Asteroid! If you have gone to the `Asteroid Documentation <https://asteroid-lang.readthedocs.io/en/latest/>`_ then you have seen that Asteroid is a pattern-matching object-oriented language.
If you have not heard of that programming paradigm (type of programming language) before, Asteroid is the first of its kind. Before you start coding in Asteroid, it would be beneficial to understand what pattern-matching actually is, since the langauges you have worked on in the past may not have been this same paradigm.

What is Pattern-Matching?
-------------------------
Pattern-matching is the idea of testing an epxression to determine if it has certain characteristics. For instance, look at this example program of a function that determines if a number is zero or not zero in the programming langauge SML:  
::
    fun f (0) = "zero"
        | f (x) = "not zero";
        
As you can see, there are two "patterns" that this function can have; either have the function be called with 0 (in this case the function would output "zero") or have the function called with something that is not zero (in this case, "not zero" would be outputted). Depending on what the function call is, the appropriate line will get executed. This is the basis of pattern-matching. In this particular example, we are testing what the function call of f will be; either with 0 or with a number, and depending on which pattern is satisfied, that line will get executed.

This is an important note to remember because **not all programming languages have pattern-matching implemented into them**. For instance, C++ without any extra libraries does not have pattern-matching built into it. If you wanted the same function in C++, it would look something like:
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
Here is an example of pattern matching in Asteroid (note that -- is the start of a comment in Asteroid):
::
   function is_zero
        with 0 do
            println("This is zero").
        with (n:%integer) if n =/= 0 do      -- the =/= means "not equal to" in Asteroid
            println("This is not zero").
        end

In this above example, there are only two patterns for the arguments of this function: the number n (which would be the number passed into the function) can either be 0 (at which point "This is zero" would be outputted to the screen) or not zero (in other words, anything but 0, at which point "This is not zero" would be outputted to the screen). The with keyword is just splitting up the function bodies with its corresponding pattern. So in this example, the function is split up into two bodies (one body being if the argument is 0 and the other if the argument is not 0). This is also called **function style pattern-matching**, since there is pattern-matching occuring within the body of the function.

This next example of pattern-matching is more in depth using **first-class condtional patterns**, which are patterns that are defined at the beginning of a program that can be used later in the same program. This is an application of first-class patterns in a program that finds the factorial of a number:
::
    -- define first-class patterns
    let POS_INT = pattern with (x:%integer) if x > 0. -- if the number is greater than 0, assign the POS_INT pattern to it
    let NEG_INT = pattern with (x:%integer) if x < 0. -- if the number is less than 0, assign the NEG_INT pattern to it

    -- define our factorial function
    function fact
        with 0 do
            return 1
        with n:*POS_INT do            -- use first pattern
            return n * fact (n-1).
        with n:*NEG_INT do            -- use second pattern
            throw Error("undefined for "+n).
        end

The first two lines of the program create the first class-patterns (the patterns being if the number is positive or negative) and assigning veriables depending on which pattern is met. For instance, if the user inputs the number 7, the pattern POS_INT gets assigned to the input and the first pattern will get executed (which recursively finds the factorial of the number). Later in the program, the patterns get dereferenced (which means retrieved from where they are stored in memory) using the * operator. The line with the first pattern is saying "with the argument n and the pattern POS_INT (or in other words, if the argument is positive), recursively find the factorial of the number" and the line with the second pattern is saying "with the argument n and the pattern NEG_INT (if the argument is negative), throw an error". 

Patterns and pattern-matching are very powerful tools because they can change how code is executed depending on what pattern is met.


What does Object Oriented Mean?
-------------------------------
The term object-oriented in programming means that code is broken up into classes and objects. Think of classes as **user defined data types**. While this may sound intimidating, there are many uses of object-oriented programming that can be used to help write efficent, clean code. For instance, there may be a time where you have to write code for software that pertains to families. While you could use tuples or arrays to represent this data, objects and classes are an even better way to achieve this feat. Take a look at this code in C++ that has the class for a family:
::
    class Family {
    public:
        std::string parent;
        std::string child1;
        std::string child2;
        Family(std::string p, std::string c1, std::string c2) { // default constructor
            parent = p;
            child1 = c1;
            child2 = c2;
        }
    };

Now if you want to create an instance of the Family class, you could write this line to do so:
::
    Family *myfamily = new Family("Jim", "Bob", "Ann");

where the properties parent is "Jim", child1 is "Bob" and child2 is "Ann". Now if you wanted to access one of these properties, you could do
::
    myfamily->child1 = "Mary"; // while this looks intimidating, all this is doing is dereferencing child1 and changing the value to "Mary"

Classes and objects are an easier way to store data that may not fit with any data structure that a language currently has. Asteroid does not implement object-orientation, but has **object based programming**. Unlike other langauges that have classes and objects, Asteroid has structures which are Asteroid's version of classes. Below is the structure Family:
::
    structure Family with  
        data parent.
        data child1.
        data child2.
        end

And you can make an instance of that structure by doing:
::
    let myFamily = Family("Jim", "Bob", "Ann").

Notice how similar the ways to create an object in C++ and Asteroid are:
::
    Family *myfamily = new Family("Jim", "Bob", "Ann"); (C++)
    let myFamily = Family("Jim", "Bob", "Ann"). (Asteroid)

Both are entirely different languages, however Asteroid does not have objects and classes, but instead has structures and the programmer can create instances of that structure. **Think of structures in Asteroid as classes in C++, and an object in C++ is just the instance of a structure in Asteroid** (that means if you have programmed with classes and objects in C++, creating structures in Asteroid should be trivial). Something else to note is that similar to Rust and Go, **Asteroid does not have inheritence for classes** (inheritence.

As seen above, this is now a data type (defined by the programmer) that can be used to store data that may be specific to the program that you are working on.
:raw-html:`<br />`
Now that you understand the two different paradigms that Asteroid is made out of, you can start writing your programs in it and explore the versatility of patterns, pattern-matching and object-oriented programming.

The let Statement
-----------------
The most difficult concept in pattern-matching languages is the idea of the let statement. In Asteroid, the let keyword is Asteroid's assingment operator, however it is not exactly the same as the assignment operator in other languages. Take a look at this line written in Asteroid:
::
    let 1 = 1.

If you tried to assign 1 to 1 in any other programming langauge, you would get a plethora of errors, however this is allowed in Asteroid. **The let statement matches the term on the right side to a pattern on the left side**. Take a look at this other example using the let keyword:
::
    let [x,2,y] = [1+0,1+1,1+2].
    println(x,y).
    -- (1,3) is printed

In this statement, we have two arrays; one on the left side of the = sign (which has the variables x and y in the zeroth and second index respectively) and one on the right hand side (whos values will matched with the pattern corresponding to the index they are in). The zeroth index of the array on the right hand side evaluates to 1 (1 + 0 = 1) and that gets assigned to the variable x (due to pattern matching). The second index of the array on the right side evaluates to 3 (1 + 2 = 3) and that value gets assigned to the variable y (again due to pattern-matching). Therefore, when we print out x and y, those have the values 1 and 3 respectively. **The thing to remember is that the let statement is not entirely equivalent to the assignment operator in other languages, even though it may look like that.**

How to Get Started in Asteroid
-------------------------------
Now that you know what principles Asteroid is made of, you can now get started writing programs in it. Directions to install Asteroid can be found at `here <https://asteroid-lang.readthedocs.io/en/latest/Installing%20and%20Running.html>`_. 
After you installed Asteroid correctly, you can write your first program. The first one you can write is a simple hello world program, which looks something like:
::
    -- the obligatory hello world program

    load system io. -- header that allows the programmer to print things out to the screen and to accept input

    println "Hello, World!".

After you have written your first program, you can run the program by typing in the following line in your terminal:
::
    asteroid <*name of program*>

where name of program is the name of the file that you want to run. 
:raw-html:`<br />`
**Make sure that you are in the same folder in your terminal of the file that you are trying to run!**

Some important things to note in Asteroid:
* Most statements must end with a period (this is equivalent to using a semicolon in C++)
* In order to print things, you must include the load system io. line as the first line of your program.
* -- are comments
* If you see a line that looks like (x:%integer), that is used to match any value of a given type. (The %integer pattern matches any integer value and can be used with any other type in Asteroid.)
