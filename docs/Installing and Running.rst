.. highlight:: none

Installation
============

Asteroid is available from the PyPI website,
::
  https://pypi.org/project/asteroid/

and is installed using `pip <https://pip.pypa.io/en/stable/>`_.,
::
    $ pip install asteroid

Alternatively you can download or clone the
`Asteroid github repository <https://github.com/asteroid-lang/asteroid>`_,
or download one of the `prepackaged releases <https://github.com/asteroid-lang/asteroid/releases>`_,
and then install it with pip by setting your working directory
to the top of the repository and then type,
::
    $ pip install .

The same command should work on Unix-like and Windows systems,
though you may have to use `pip3` or some other variation.

>Don't forget to add the `bin` directory where `pip` installs programs
to your `PATH` variable.

In addition, there is a cloud-based Linux virtual machine that is completely
set up with an Asteroid environment and can be accessed at
`Repl.it <https://repl.it/@lutzhamel/asteroid#README.md>`_.

Running the Asteroid Interpreter
================================

You can now run the interpreter from the command line by simply typing `asteroid`.
This will work on both Windows and Unix-like systems as long as you followed the instructions above.
To run asteroid on Unix-like systems and on our virtual machine,
::
    $ cat hello.ast
    -- the obligatory hello world program

    load system io.

    println "Hello, World!".

    $ asteroid hello.ast
    Hello, World!
    $

On Windows 10 the same thing looks like this,
::
    C:\> type hello.ast
    -- the obligatory hello world program

    load system io.

    println "Hello, World!".

    C:\> asteroid hello.ast
    Hello, World!
    C:\>


As you can see, once you have Asteroid installed on your system you can execute an
Asteroid program by typing,
::
    asteroid [flags] <program file>

at the command prompt.
