# Reference Guide (Endnotes)


Here is a snippet from the Asteroid EBNF grammar that shows the control statements in terms of the non-terminal `stmt`,
```
stmt := FOR pattern IN exp DO stmt_list END
     | WHILE exp DO stmt_list END
     | REPEAT DO? stmt_list UNTIL exp '.'?
     | IF exp DO stmt_list (ELIF exp DO stmt_list)* (ELSE DO? stmt_list)? END
     | TRY stmt_list (CATCH pattern DO stmt_list)+ END
     | THROW exp '.'?
     | BREAK '.'?
```

