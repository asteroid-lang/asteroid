# Quick Start
The Asteroid Debugger (ADB) is a source code debugger for the Asteroid programming language.

It supports the following:
* Breakpoints
* Single stepping at the source level
* Stepping through function calls
* Stack frame inspection
* Evaluation of arbitrary Asteroid code
* Macros
* REPL Instances

ADB features an "explicit" mode which details much of the pattern matching and underlying
mechanics of Asteroid to give developers a better idea of what's going on. Explicit mode
details many of the steps of pattern matching, function calling, statement execution, and
return values.

## Usage
`TDB`

## Commands
Below is a list of the commands available to the debugger. Most of which can be shortened.
These shortenings are shown in parenthesis (`c(ommand)`).

### Step
`s(tep)` through to the next statement or through function call.

### Continue
`c(ontinue) cont(inue)` continue execution until the next breakpoint.

### Next
`n(ext)` continue onto the next executing line at the current scope.

### Break
`b(reak) (number)*` set a breakpoint at one or more lines. Running without any arguments
lists your breakpoints.

Example: `b 1 2 3`, `break`.

### Delete
`d(elete) (number)+` `del(ete) (number)+` delete a list of breakpoints.

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

### ! (Bang)
`!` Open up a repl

### Help
`h(elp) (command)?` gives help for a given command. Running just `help` shows all available commands.
Example:
```
h macro
help break
```

### < (Up)
`<` move up one stack frame

### > (Down)
`>` move down one stack frame

### Where
`w(here)` displays the frame stack and the currently active frame.

### Longlist
`ll (longlist)` lists the entire program

### List
`l(ist)` lists the lines around the currently executing line

### Explicit
`e(xplicit)` enables explicit mode

### Unexplicit
`u(nexplicit)` disables explicit mode

# Explicit Mode

---

# Developer's guide
When trying to integrate the debugger's functionality into new asteroid code, a developer really only
needs to understand the core functions that allow 99% of the debugger to work within `walk`.

## notify_debugger

## message_explicit
`message_explicit` `gen_t2s`

## Tab leveling
increase_debugger_tab_level

decrease_debugger_tab_level