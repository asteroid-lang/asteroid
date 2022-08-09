# ADB Reference Guide
The Asteroid Debugger (ADB) is a source code debugger for the Asteroid programming language.

It supports the following:
* Breakpoints
* Single stepping at the source level
* Stepping through function calls
* Stack frame inspection
* Evaluation of arbitrary Asteroid code
* Macros
* REPL Instances

ADB features a unique "explicit" mode which details much of the pattern matching and underlying
mechanics of Asteroid allow developers to debug pattern matches. Explicit mode
details many of the steps of pattern matching, function calling, statement execution, and
return values. To learn more about explicit mode and its many features, see: [ADB in action](in_action.md)


## Usage
```
asteroid --adb <FILENAMEs>
```

## Debugging sessions
The debugger's prompt `(ADB)` allows the user to enter commands to effect the source environment
and debugger behavior.

Each debugging session will have a few pieces of information:

__Filename and line number__
```
[/home/user/test.ast (1)]
```

__The current line which will be executed__
```
-->> let p = pattern %[(x:%integer) if x > 0 and x < 100]%.
```

__The command prompt__
```
(ADB)
```

ADB runs like any other debugger, here's a small example session where
we see running commands, listing the program, and setting and continuing to breakpoints.
```
[/home/user/test.ast (1)]
-->> let p = pattern %[(x:%integer) if x > 0 and x < 100]%.
(ADB) ll
----- Program Listing -----
>  1 let p = pattern %[(x:%integer) if x > 0 and x < 100]%.
   2 
   3 let d = pattern %[(x:*p, y:*p, z:*p)]%.
   4 let x:*p = 99.
   5 
   6 let t:*d = (1,2,3).
   7 [EOF]
(ADB) break 6
(ADB) continue
----- Breakpoint -----
[/home/user/test.ast (6)]
-->> let t:*d = (1,2,3).
(ADB) 
```

## Commands
Below is a list of the commands available to the debugger. Most of which can be shortened.
These shortenings are shown in parenthesis (`c(ommand)`).

### List
`l(ist)` lists the lines around the currently executing line

### Longlist
`ll (longlist)` lists the entire program

### Step
`s(tep)` through to the next statement or through function call.

### Continue
`c(ontinue) cont(inue)` continue execution until the next breakpoint.

### Next
`n(ext)` continue onto the next executing line at the current scope.

### Breakpoints
`b(reak) number*` set a breakpoint at one or more lines. Running without any arguments
lists your breakpoints.

Example: `b 1 2 3`, `break`.

Conditional breakpoints can be set in the same way, just attach `if eval("condition")`
after each breakpoint number.

Example:
```
-- Set a conditional breakpoint on 10 and normal breakpoints on 11, 15, and 23.
b 10 if eval("x == 123") 11 15 23
```

### Delete
`d(elete) (number)+` `del(ete) (number)+` delete a list of breakpoints.

Example:
```
del 1 5 8 9
```

### Macro
`macro (name) (command list)`. Define a macro. Running just `macro` lists your macros.

Example macro that continues to a breakpoint and prints the value of x:
```
macro gox = c; eval("io@println(x)");
```

### Eval
`eval("asteroid code")` Evaluate the asteroid code between quotes. Works exactly like a single-line
repl.

Example, print out the value of `x`:
```
eval("x")
```

### !
`!` Open up a repl in the current context

### Help
`h(elp) (command)?` gives help for a given command. Running just `help` shows all available commands.
Example:
```
h macro
help break
```

### <
`<` move up one stack frame

### >
`>` move down one stack frame

### Where
`w(here)` displays the frame stack and the currently active frame.

### Explicit
`e(xplicit) (on|off)?` By default, this command run without an argument toggles
explicit mode. If given a literal `on` or `off`, explicit mode will be switched
to the corresponding state.

Explicit:
```
-- Toggle Explicit mode
explicit
e

-- Turn on/off
explicit on
e on
explicit off
e off
```

### Quit
`q(uit)` Quits the current ADB session