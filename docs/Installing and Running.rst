.. highlight:: none

Installation
============

Download or clone the `Asteroid github repository <https://github.com/lutzhamel/asteroid>`_, or download one of the `prepackaged releases <https://github.com/lutzhamel/asteroid/releases>`_, and then install with `pip <https://pip.pypa.io/en/stable/>`_.

For example, if your working directory is at the top of the repository,
::
    $ python -m pip install .

The same command should work on Unix-like and Windows systems, though you may have to run it with `python3` or some other variation.

In addition, there is a cloud-based Linux virtual machine that is completely set up with an Asteroid environment and can be accessed at `Repl.it <https://repl.it/@lutzhamel/asteroid#README.md>`_.

Running the Asteroid Interpreter
================================

You can now run the interpreter from the command line by simply typing asteroid. This will work on both Windows and Unix-like systems as long as you followed the instructions above,
::
    $ cat simple.ast
    -- a simple program using lambda functions

    load "io".

    println ((lambda with n do return n+1) 1).
    $
    $ asteroid simple.ast
    2
    $
