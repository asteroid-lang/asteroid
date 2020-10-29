# Installation

Installation on Unix-like systems is nothing more than to either download or clone the [Asteroid github repository](https://github.com/lutzhamel/asteroid) or download one of the [prepackaged releases](https://github.com/lutzhamel/asteroid/releases) and then add the `code` folder of the repository/release to your `PATH` environment variable. Be sure that you have Python 3.x installed. Make sure that the file `asteroid` in the `code` folder has execution privileges on your machine.

On Windows 10 you will need to set the environment variable `ASTEROID_ROOT` to point to the folder where you cloned the repo or unzipped the downloaded file. Then you will need to add the following to the path environment variable: `%ASTEROID_ROOT%\code`. That's it, now you can use the `asteroid.bat` file in the `code` folder to start the asteroid interpreter.

# Running the Asteroid Interpreter

You can now run the interpreter from the command line by simply typing asteroid. This will work on both Windows and Unix-like systems as long as you followed the instructions above,

```
$ cat simple.ast
-- a simple program using lambda functions

load "io".

print ((lambda with n do return n+1) 1).
$
$ asteroid simple.ast
2
$
```
