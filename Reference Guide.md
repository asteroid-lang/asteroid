# Reference Guide

## User Guide Language Features 
(insert text here)

## Asteroid Built-ins

A **built-in function,** also called an **intrinsic function,** can complete a given task directly within a language. Asteroid includes built-ins such as lists and strings. 

As mentioned in [this section of the User Guide](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/Asteroid%20User%20Guide.md#the-basics), a **list** is a structured data type that consists of square brackets enclosing comma-separated values. Lists can be modified after their creation.

A **string** is a sequence of characters that can be used as a variable or a literal constant.

More information about the functions that lists and strings contribute to can be found in the [Prologue.ast Module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/prologue.ast).

## Asteroid Modules

There are a variety of useful modules that can be installed in Asteroid.

[Hash.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/hash.ast) implements the `HashTable` structure, for mapping keys onto paired values.

[IO.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/io.ast) implements Asteroid's I/O system for delivering outputs from given inputs.

[Math.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/math.ast) implements its mathematical constants and operators.

[Pick.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/pick.ast) implements the `Pick` structure. A `pick` object contains a list of items that can be randomly picked from using the `pick` member function.

[Random.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/random.ast) implements the `random` numbers. Using the functions included in this module will return a random value or floating point number within a given range or interval.

[Set.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/set.ast) implements Asteroid sets as lists. Unlike lists, sets do not have repeated members.

[Sort.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/sort.ast) defines a parameterized sort function over a list. `Sort` makes use of a user-defined order predicate on the list's elements to perform the sort. The `Quicksort` is the underlying sort algorithm.

[Stream.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/stream.ast) implements the `Stream` structure. Asteroid stream implementation is based on lists.

[Util.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/util.ast) defines utility functions and structures.

[Vector.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/vector.ast) defines functions useful for vector arithmetic.


## Asteroid Grammar (written in EBNF/PEG Format)

////////////////////////////////////////////////////////////////////////////////////////

// (c) Lutz Hamel, University of Rhode Island  //

////////////////////////////////////////////////////////////////////////////////////////

### Statements


      prog
        : stmt_list

      stmt_list
        : stmt*


    stmt
      : '.' // NOOP
      | LOAD SYSTEM? STRING '.'?
      | GLOBAL id_list '.'?
      | NONLOCAL id_list '.'?
      | ASSERT exp '.'?
      | STRUCTURE ID WITH struct_stmts END
      | LET pattern '=' exp '.'?
      | LOOP DO? stmt_list END
      | FOR pattern IN exp DO stmt_list END
      | WHILE exp DO stmt_list END
      | REPEAT DO? stmt_list UNTIL exp '.'?
      | IF exp DO stmt_list (ELIF exp DO stmt_list)* (ELSE DO? stmt_list)? END
      | MATCH exp WITH? (CASE pattern DO stmt_list)+ (OTHERWISE DO? stmt_list)? END
      | TRY DO? stmt_list (CATCH pattern DO stmt_list)+ END
      | THROW exp '.'?
      | BREAK '.'?
      | RETURN exp? '.'?
      | function_def
      | call_or_index '.'?  
  
### Grammar Snippet of Control Statements in terms of the Non-Terminal `stmt`


     stmt := FOR pattern IN exp DO stmt_list END
     | WHILE exp DO stmt_list END
     | REPEAT DO? stmt_list UNTIL exp '.'?
     | IF exp DO stmt_list (ELIF exp DO stmt_list)* (ELSE DO? stmt_list)? END
     | TRY stmt_list (CATCH pattern DO stmt_list)+ END
     | THROW exp '.'?
     | BREAK '.'?

### More General Statements


     function_def
       : FUNCTION ID body_defs END

     body_defs
       : WITH pattern DO stmt_list (ORWITH pattern DO stmt_list)*

     data_stmt
       : DATA ID

     struct_stmt
       : data_stmt  '.'?
       | function_def '.'?
       | '.'

     struct_stmts
       : struct_stmt*

     id_list
       : ID (',' ID)*

### Expressions/Patterns

////////////////////////////////////////////////////////////////////////////////////////

// NOTE: There is no syntactic difference between a pattern                           //

// and an expression. We introduce the 'pattern' nonterminal                          //

// to highlight the SEMANTIC difference between patterns and                          //

// expressions. ////////////////////////////////////////////////////////////////////////


     pattern
       : exp

     exp
       : quote_exp

     quote_exp
       : QUOTE head_tail
       | PATTERN WITH? head_tail
       | head_tail

     head_tail
       : conditional ('|' exp)?

     conditional
       : compound
           (
              (CMATCH exp) |   // CMATCH == '%'IF
              (IF exp ELSE exp)
           )?

     compound
        : logic_exp0
            (
               (IS pattern) |
               (IN exp) |               // exp has to be a list
               (TO exp (STEP exp)?) |   // list comprehension
            )?

     logic_exp0
       : logic_exp1 (OR logic_exp1)*

     logic_exp1
        : rel_exp0 (AND rel_exp0)*

     rel_exp0
       : rel_exp1 (('==' | '=/=' ) rel_exp1)*

     rel_exp1
       : arith_exp0 (('<=' | '<'  | '>=' | '>') arith_exp0)*

     arith_exp0
       : arith_exp1 (('+' | '-') arith_exp1)*

     arith_exp1
       : call_or_index (('*' | '/') call_or_index)*

     call_or_index
       : primary (primary | '@' primary)*

     primary
       : INTEGER
       | REAL
       | STRING
       | TRUE
       | FALSE
       | NONE
       | ID (':' pattern)?  // named pattern when ': pattern' exists
       | '*' ID         // "dereference" a variable during pattern matching
       | NOT call_or_index
       | MINUS call_or_index
       | ESCAPE STRING
       | EVAL primary
       | '(' tuple_stuff ')' // tuple/parenthesized expr
       | '[' list_stuff ']'  // list or list access
       | function_const
       | TYPEMATCH           // TYPEMATCH == '%'<typename>

     tuple_stuff
       : exp (',' exp?)*
       | empty

     list_stuff
       : exp (',' exp)*
       | empty

     function_const
       : LAMBDA body_defs
