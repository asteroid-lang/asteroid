# ADB in action
### Explicit
`e(xplicit) (on|off)?` By default, this command run without an argument toggles
explicit mode. If given a literal `on` or `off`, explicit mode will be switched
to the corresponding state.

# Explicit Mode
Explicit mode is a feature of ADB that allows the user to understand and inspect
Asteroid's pattern matching.

Let's take a look at a debugging session on a program that uses first-class 
patterns to enforce a type:

We see the simple 4 line program. We have a pattern that is essentially the
range (0,10) and two variable declarations with typematches.
```
[/home/user/example1.ast (1)]
-->> let p = pattern %[(x:%integer) if x > 0 and x < 10]%.
(ADB) ll
----- Program Listing -----
>  1 let p = pattern %[(x:%integer) if x > 0 and x < 10]%.
   2 
   3 let z:*p = 9.
   4 let y:*p = 11.
   5 [EOF]
(ADB) n
```

Here we see explicit mode being enabled using the `e` command 
and a simple typematch
occuring. We can see the constraint-only pattern, the internal
condition, the internal variable `x` being unified, the
condition being met, and finally `z` being set to 9 as 9 succeeded
the typematch.

During unification, explicit mode shows the user the exact
term and pattern which will be matched. Seen as `** pattern:` and
`** term:`.

```
[/home/user/example1.ast (3)]
-->> let z:*p = 9.
(ADB) e
(ADB)[e] n
 ** pattern: z:*p
 ** term: 9
- Matching term z to pattern *p
|   - Dereferencing p
|    ** *p -> %[x:%integer if (condition...)]%
|   - [Begin] constraint pattern: x:%integer if (condition...)
|   |   - Conditional match: if (x > 0 and x < 10)
|   |   |   - Matching term x to pattern %integer
|   |   |   |   - Typematch 9 to type integer
|   |   |   |    ** Success!
|   |   |   - Matched!
|   |   |   - x = 9
|   |   - Condition met, x > 0 and x < 10
|   - [End] constraint pattern
- Matched!
- z = 9
[/home/user/example1.ast (4)]
-->> let y:*p = 11.
(ADB)[e] n
```

Here we see something similar. We can see the constraint-only
pattern, the typematch to integer, but, when we get to the conditional
part of the pattern, we see a failure. With explicit mode, we can see
exactly *where* in the pattern the failure occurs.
```
[/home/user/example1.ast (4)]
-->> let y:*p = 11.
(ADB)[e] n
 ** pattern: y:*p
 ** term: 11
- Matching term y to pattern *p
|   - Dereferencing p
|    ** *p -> %[x:%integer if (condition...)]%
|   - [Begin] constraint pattern: x:%integer if (condition...)
|   |   - Conditional match: if (x > 0 and x < 10)
|   |   |   - Matching term x to pattern %integer
|   |   |   |   - Typematch 11 to type integer
|   |   |   |    ** Success!
|   |   |   - Matched!
|   |   |   - x = 11
|   |   - Condition (x > 0 and x < 10) failed

ERROR: /home/user/example1.ast: 4: pattern match failed: conditional pattern match failed
    ==>> let y:*p = 11.

----- Error occured, session will restart after commands -----
[/home/user/example1.ast (4)]
-->> let y:*p = 11.
(ADB)[e] 
```

## Another example

... By following the tab leveling and pattern tree, we can see the exact point where the
error occurs

```
[/home/user/example2.ast (1)]
-->> let p = pattern %[(x:%integer) if x > 0 and x < 100]%.
(ADB) ll
----- Program Listing -----
>  1 let p = pattern %[(x:%integer) if x > 0 and x < 100]%.
   2 let d = pattern %[(a:*p, b:*p, c:*p)]%.
   3 
   4 assert(not ((1,2,"A") is *d)).
   5 [EOF]
(ADB) s
[/home/user/example2.ast (2)]
-->> let d = pattern %[(a:*p, b:*p, c:*p)]%.
(ADB) s
[/home/user/example2.ast (4)]
-->> assert(not ((1,2,"A") is *d)).
(ADB) e
(ADB)[e] n
- Asserting: not ((1,2,A) is *d)
|   - Dereferencing d
|    ** *d -> %[(a:*p,b:*p,c:*p)]%
|   - [Begin] constraint pattern: (a:*p,b:*p,c:*p)
|   |   - Matching lists: [a:*p,b:*p,c:*p] and [1,2,A]
|   |   |   - Matching term a to pattern *p
|   |   |   |   - Dereferencing p
|   |   |   |    ** *p -> %[x:%integer if (condition...)]%
|   |   |   |   - [Begin] constraint pattern: x:%integer if (condition...)
|   |   |   |   |   - Conditional match: if (x > 0 and x < 100)
|   |   |   |   |   |   - Matching term x to pattern %integer
|   |   |   |   |   |   |   - Typematch 1 to type integer
|   |   |   |   |   |   |    ** Success!
|   |   |   |   |   |   - Matched!
|   |   |   |   |   |   - x = 1
|   |   |   |   |   - Condition met, x > 0 and x < 100
|   |   |   |   - [End] constraint pattern
|   |   |   - Matched!
|   |   |   - Matching term b to pattern *p
|   |   |   |   - Dereferencing p
|   |   |   |    ** *p -> %[x:%integer if (condition...)]%
|   |   |   |   - [Begin] constraint pattern: x:%integer if (condition...)
|   |   |   |   |   - Conditional match: if (x > 0 and x < 100)
|   |   |   |   |   |   - Matching term x to pattern %integer
|   |   |   |   |   |   |   - Typematch 2 to type integer
|   |   |   |   |   |   |    ** Success!
|   |   |   |   |   |   - Matched!
|   |   |   |   |   |   - x = 2
|   |   |   |   |   - Condition met, x > 0 and x < 100
|   |   |   |   - [End] constraint pattern
|   |   |   - Matched!
|   |   |   - Matching term c to pattern *p
|   |   |   |   - Dereferencing p
|   |   |   |    ** *p -> %[x:%integer if (condition...)]%
|   |   |   |   - [Begin] constraint pattern: x:%integer if (condition...)
|   |   |   |   |   - Conditional match: if (x > 0 and x < 100)
|   |   |   |   |   |   - Matching term x to pattern %integer
|   |   |   |   |   |   |   - Typematch A to type integer
|   |   |   |   |   |   |    ** Failure
- Assert Succeeded
[/home/user/example2.ast (5)]
-->> [EOF]
(ADB)[e] 
```