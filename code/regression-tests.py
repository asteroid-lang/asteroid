#!/usr/bin/env python
# coding: utf-8

# # Regression Tests

# This is the regression test suite for Asteroid.  All the programs in this notebook should execute.  
# 
# You can turn this into an executable Python script with:
# ```
# jupyter nbconvert --to script regression-tests.ipynb 
# ```

# In[1]:


import sys
sys.path[0] = '/home/ec2-user/SageMaker/asteroid/code'
from asteroid_interp import interp


# In[2]:


program = '''
load "io".

let digits = "332211" @explode() @sort().
println digits.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[3]:


program = '''
load "io".
let l:"\[.*\]" = [1,2,3].
println l.
'''
interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True)


# In[4]:


program = '''
load "io".
let r = [1,2,3] @reduce (lambda with (x,y) do return x*y).
println r.
assert (r == 6).
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[5]:


program = '''
load "io".
let r = [1,2,3] @map (lambda with x do return x+1).
println r.
assert (r == [2,3,4]).
'''
interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True)


# In[6]:


program = '''
load "io".

structure Dog with

  data name.
  data tricks.

  function add_trick
    with (self, new_trick) do
      let self @tricks = self @tricks + [new_trick].
    end

  function __init__
    with (self, name) do
      let self @name = name.
      let self @tricks = [].
    end

  end -- structure

-- Fido the dog
let fido = Dog("Fido").
fido @add_trick("roll over").
fido @add_trick("play dead").

-- Buddy the dog
let buddy = Dog("Buddy").
buddy @add_trick("roll over").
buddy @add_trick("sit stay").

-- Fifi the dog
let fifi = Dog("Fifi").
fifi @add_trick("sit stay").

-- print out all the names of dogs
-- whose first trick is 'roll over'.
let dogs = [fido, buddy, fifi].

for Dog(name, ["roll over"|_]) in dogs do
    println (name + " does roll over").
end
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[7]:


program = '''
load "io".

structure Family with
  data parent1.
  
  function members
    with self do
        let Parent(n1:%string) = self @parent1.
        return n1.
    end
  end -- structure
  
let FAMILY = pattern with Family(*PARENT1).

structure Parent with
  data name.
  end
  
let PARENT1 = pattern with Parent(p1).


let p1 = "Harry".
let family = eval(FAMILY).
--raw_print(family).
println (family @members()).

'''
interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True)


# In[8]:


program = '''
load "io".
structure A with
    data a.
    end

let A(x:%string) = A("hello").
println x.
'''
interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True)


# In[9]:


program = '''
-- load modules
load "io".

structure Family with
  data parent1.
  
  function members
    with self do
        let Parent(name1) = self @parent1.
        let Parent(name1:%string) = self @parent1.
        return name1.
    end
  end -- structure

structure Parent with
  data name.
  end

let family = Family(Parent("Harry")).
println (family @members()).
'''
interp(program,exceptions=True,symtab_dump=False, tree_dump=False, do_walk=True, prologue=True)


# In[10]:


program = '''
load "io".

let foo = "abc".
let l = foo @explode() @reverse() @join("").
println l.
'''
interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True)


# In[11]:


program = '''
-- Palindrome list filter
load "io".

function filter 
    with (x:%string) %if x @explode() == x @explode() @reverse() do
        return true.
    orwith _ do 
        return false.
    end
    
print (filter "radar").
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[12]:


program = '''
load "io".
let i:%integer = 1.
print i.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[13]:


program = '''
load "io".

try
    let (1,y) = '(1,x).
catch e do
    println e.
end
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[14]:


program = '''
load "io".

-- TODO: this is NOT correct see issue #27
let (x,x) = (1,2).
println x.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[15]:


program = '''
load "io".

let (x,y) %if x is y = ([1],[1]).
println x.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[16]:


program = '''
load "io".
load "util".

let (l:%list) %if l is "\[.*2.*\]" = [1,2,3].
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[17]:


program = '''
load "io".
structure A with
    data a.
    data b.
    end

let a = A(1,2).
let (v:%A) %if v @a + 1 > 1 = a.
let A(x:%string,_) = A("hello",2).
println v.
println x.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[18]:


program = '''
-- Factorial

load "io".
load "util".

function fact 
    with 0 do
        return 1
    orwith (n:%integer) %if n > 0 do
        return n * fact (n-1).
    orwith (n:%integer) %if n < 0 do
        throw Error("factorial is not defined for "+n).
    end 

println ("The factorial of 3 is: " + fact (3)).
assert (fact(3) == 6).

'''
interp(program, exceptions=False, symtab_dump=False)


# In[19]:


program = '''
load "io".

let (x:%integer) %if x > 0 = 2.
println x.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[20]:


program = '''
load "io".

let a:%list = [1,2] .
println a.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[21]:


program = '''
load "io".
structure A with
    data a.
    data b.
    end

let a = A(1,2).
let v:%A = a.
println v.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[22]:


program = '''
-- load "io".
structure A with
    data a.
    data b.
    end

let a = A(1,2).
assert((a @a is 1) and (a @b is 2)).
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[23]:


program = '''
load "io".

function ident with i do return i end

try
    let p = not ident(1).
    println p.
catch e do
    println e.
end
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[24]:


program = '''
load "io".

let q:%integer = 3.
println q.
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[25]:


program = '''
load "io".

function ident with i do return i end

try
    let p = - ident(1).
    println p.
catch e do
    println e.
end
'''
interp(program, exceptions=False, tree_dump=False, do_walk=True, prologue=True)


# In[26]:


program = '''
load "io".

structure A with
    data a.
    function hello with self do return "hello: "+ self @a end.
    end
    
structure B with
    data b.
    end
    
let q = B([1,2,3]).
let p = B(A(1)).

println (q @ b).
println (q @b @length()). 
println (p @b @hello()).
'''
interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True)


# In[27]:


program = '''
load "io".
    
structure B with
    data b.
    function get with self do return self @b end
    end
    
let q = B([1,2,3]).

println (q @get() @length()). 
'''
interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True)


# In[28]:


program = '''
load "io".
load "hash".

let h = HashTable().
h @insert("hello","there").
let result = h @get("hello").
println (result).
assert (result is "there").
'''
interp(program, exceptions=False, tree_dump=False)


# In[29]:


program = '''
load "io".

let name:"a.*" = "abc".
println name.
assert (name is "abc")
'''
interp(program, exceptions=True, tree_dump=False)


# In[30]:


program = '''
load "io".
load "util".

structure Person with
    data name.
    data age.
    data gender.
    end

let people = [
    Person("George", 32, "M"),
    Person("Sophie", 46, "F"),
    Person("Oliver", 21, "X")
    ].

for Person(name:".*p.*",_,_) in people do
  println name.
end
assert(name is "Sophie")
'''
interp(program, exceptions=True, tree_dump=False)


# In[31]:


program = '''
load "io".

function addc with x do return lambda with y do return x + y end
let a = (addc 1) 1.
println a.
assert(a is 2).
'''
interp(program, exceptions=True, tree_dump=False)


# In[32]:


program = '''
load "io".
let a = "abcde".
let i = a @1.
println i.
let b = a @[0,2,4].
println b.
let i = a @length().
println i.
let e = a @explode().
println e.
'''
interp(program, exceptions=False, tree_dump=False)


# In[33]:


program = '''
load "io".
let a = [1,2,3].
println (a @length()).
a @append(3).
a @insert(2,4).
println a.
a @sort(true).
println a.
'''
interp(program, exceptions=True, tree_dump=False)


# In[34]:


program = '''
load "io".
let a = [].

if a is [b|rest] do
    println "yes".
end
'''
interp(program, exceptions=True)


# In[35]:


program = '''
load "io".
let a = [1,2].

if a is [b|c|d|rest] do
    println "yes".
end
'''
interp(program, exceptions=False)


# In[36]:


program = '''
load "io".
let fst = (1,2)@0.
println fst.
'''
interp(program, exceptions=False)


# In[37]:


program = '''
load "io".
let t = (1,2).
println ("tuple = " + t).
'''
interp(program, exceptions=True)


# In[38]:


program = '''
let () = none.
'''
interp(program, exceptions=False)


# In[39]:


program = '''
load "io".
load "util".

structure Dog with

  data name.
  data tricks.

  function add_trick
    with (self, new_trick) do
      let self @tricks = self @tricks + [new_trick].
    end 
    
  function __init__
    with (self, name) do 
      let self @name = name.
      let self @tricks = [].
    end 
    
  end 

-- Fido the dog
let fido = Dog("Fido").

fido @add_trick("roll over").
fido @add_trick("play dead").

-- Buddy the dog
let buddy = Dog("Buddy").

buddy @add_trick("roll over").
buddy @add_trick("sit stay").

-- println out the tricks
println ("Fido: " + fido @tricks).
println ("Buddy: " + buddy @tricks).

assert(fido @tricks is ["roll over", "play dead"]).
assert(buddy @tricks is ["roll over", "sit stay"]).
'''
interp(program, exceptions=False)


# In[40]:



program = '''
load "io".

structure A with
    data x.
    function dump with self do println (self@x) end .
    end 

let obj = A(2).
obj @dump().

let A(x) = obj.
println x.

assert(x is 2).

'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=False, do_walk=True)


# In[41]:


program = '''
load "io".
load "util".

structure A with

    data x.
    data y.

    function __init__
      with (self, a, b) do
        let self @x = a.
        let self @y = b.
      orwith self do
        let self @x = 1.
        let self @y = 2.
      end 
    end 

let obj1 = A("hello","world").
println obj1.
let obj2 = A().
println obj2.

assert((obj1 @x is "hello") and (obj1 @y is "world")).
assert((obj2 @x is 1) and (obj2 @y is 2)).

'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=True, do_walk=True)


# In[42]:


program = '''
load "io".
load "util".

structure Person with
    data name.
    data age.
    data sex.
    end 

let people = [
    Person("George", 32, "M"),
    Person("Sophie", 46, "F"),
    Person("Oliver", 21, "M")
    ].
    
let Person(name,age,sex) = people @1.
let output_str = name + " is " + age + " years old and is " + ("male" if sex is "M" else "female") + ".".
println output_str.
assert(output_str is "Sophie is 46 years old and is female.")
'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=False, do_walk=True)


# In[43]:


program = """
load "io".      
load "util".
let slice = [4 to 0 step -1].
println slice.
assert(slice is [4,3,2,1,0]).
"""
interp(program, tree_dump=False, symtab_dump=False, exceptions=False, do_walk=True)


# In[44]:


program = '''
load "io".
load "util".

let a = [[1,2],[3,4],[5,6]].
let l = (a @0).
println l.
assert(l is [1,2]).
'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=False, do_walk=True)


# In[45]:


program = '''
load "io".
load "util".

let a = [10,20,30].
let x = a @(1).
let y = a @[1].

println x. -- prints out a scalar 
println y. -- prints out a list
assert((x is 20) and (y is [20])).
'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=False, do_walk=True)


# In[46]:


program = '''
load "random".
load "io".

let v = random().

println v.
assert(v <= 1.0 and v >= 0.0).

'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=False, do_walk=True)


# In[47]:


program = '''
load "io".
load "util".

let inc = (lambda with n do return n+1).


println (eval ('inc 1)).

assert(eval ('inc 1) is 2).
'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=True)


# In[48]:



program = '''
load "io".
load "util".
let (f,g) = (1,2).

function foobar 
    with none do
        global f, g.
        let f = 2.
    end 
    
foobar(none).
println (f,g).

assert((f,g) is (2,2)).
'''
interp(program, tree_dump=False, do_walk=True, exceptions=False, symtab_dump=False)


# In[49]:


program = '''
load "io".

try 
    let 2 = 1 + 1. 
catch _ do
    println "pattern match failed".
end 

'''
interp(program, tree_dump=False, do_walk=True, exceptions=False, symtab_dump=False)


# In[50]:


program = '''
load "io".

let 2 = 1 + 1. -- (*@\label{patternmatching-models:let1a}@*)

try 
    let 1 + 1 = 1 + 1. -- throws an exception (*@\label{patternmatching-models:let3}@*)
catch _ do
    println "pattern match failed".
end 

'''
interp(program, tree_dump=False, do_walk=True, exceptions=True, symtab_dump=False)


# In[51]:


program = '''
load "io".

try
    let -1 = -1 .
catch e do
    println e.
end 
'''
interp(program, tree_dump=False, do_walk=True, exceptions=False, symtab_dump=False)


# In[52]:


program = '''
load "io".
load "util".
let p = -1.
let w = p - 1.
let q = [-1].
println (p,w,q).

-- NOTE: workaround, pattern matching does not work on negative constants.
assert(p == -1).
assert(w == -2).
assert(q == [-1]).

'''
interp(program, tree_dump=False, do_walk=True, exceptions=False, symtab_dump=False)


# In[53]:


program = '''
-- implements Peano addition using a lookup table for the rewrite rules

load "util".
load "io".

structure S with
    data x.
    end 
    
structure add with
    data left.
    data right.
    end .

let rule_table = [
    ('add(x,0), 'reduce(x)),
    ('add(x,S(y)), 'S(reduce(add(x,y))))
    ].

function reduce 
    with term do
        for i in 0 to rule_table @length()-1 do
            -- limit visibility of free variables of the rewrite rules
            -- to the with block scope
            let (lhs, rhs) = rule_table@i.
            if term is *lhs do
                return eval rhs.
            end 
        end 
        return term.
    end 

println (reduce('add(S(S(0)),S(S(S(0)))))).
assert (reduce('add(S(S(0)),S(S(S(0))))) is S(S(S(S(S(0)))))).
'''

interp(program,exceptions=False,symtab_dump=False, tree_dump=False, do_walk=True)


# In[54]:


program = '''
load "io".
load "util".

let cl = 1 + 2.
let cr = 3.
let p = cl + cr.

println (1+2+3 is *p).
assert ((1+2+3 is *p) is true).
'''
interp(program, tree_dump=False, exceptions=False)


# In[55]:


program = '''
load "io".
load "util".

function match
    with (subject, p) do
        return subject is *p.
    end 
    
println (match('1+1, '_+_)).
assert (match('1+1, '_+_) is true).
'''
interp(program, tree_dump=False, exceptions=False)


# In[56]:


program = '''
load "io".

structure MyException with 
    data e.
    end 

try
    throw MyException("Hello There!").
catch MyException(v) do
    println v.
end 
'''
interp(program, tree_dump=False, exceptions=False)


# In[57]:


program = '''
load "io".

try
    let i = 10/0.
    println i.
catch e do
    println e.
end 
'''
interp(program, tree_dump=False, exceptions=False)


# In[58]:


program = '''
load "io".

try
    let i = 10/0.
    println i.
catch ("Exception", v) do
    println v.
end 
'''
interp(program, tree_dump=False, exceptions=False)


# In[59]:


program = '''
load "io".
load "util".

structure Person with
    data name.
    data age.
    data sex.
    end 
    
let people = [
    Person("George", 32, "M"),
    Person("Sophie", 46, "F"),
    Person("Oliver", 21, "M")
    ].
    
let n = people @length().
let sum = 0.

for Person(_,age,_) in people do
    let sum = sum + age.
end 

println ("Average Age: " + (sum/n)).
assert ((sum/n) == 33).

for Person(name,_,"M") in people do
    println name.
end 
'''
interp(program, tree_dump=False, exceptions=False)


# In[60]:


program = '''
load "io".

let l = [1,2,3].

repeat do
    let [head|l] = l.
    println head.
until l is [].
'''
interp(program, tree_dump=False, exceptions=False)


# In[61]:


program = '''
load "io".
load "util".

let true = ('1 + 2) is x + y.
println (x,y).
assert ((x,y) is (1,2)).
'''
interp(program, tree_dump=False, exceptions=False)


# In[62]:


program = '''
load "util".
structure S with
    data x.
    end 
    
let v = S(S(S(0))).
assert(v is S(S(S(0)))).
'''
interp(program, tree_dump=False, exceptions=False)


# In[63]:


program = '''
-- implements Peano addition on terms
load "io".
load "util".

structure S with
    data x.
    end 
    
structure add with
    data left.
    data right.
    end

function reduce
    with add(x,0) do      
        return reduce(x).
    orwith add(x,S(y))  do
        return S(reduce(add(x,y))).
    orwith term do     
        return term.
    end 

println(reduce(add(add(add(S(S(0)),S(S(S(0)))),S(0)),S(0)))).
assert(reduce(add(add(add(S(S(0)),S(S(S(0)))),S(0)),S(0))) is S(S(S(S(S(S(S(0)))))))).
'''
interp(program, tree_dump=False, exceptions=False)


# In[64]:


from asteroid_interp import interp
program = '''
-- Quicksort

load "io".
load "util".

function qsort
    with [] do
        return [].
    orwith [a] do
        return [a].
    orwith [pivot|rest] do
        let less=[].
        let more=[].
            
        for e in rest do  
            if e < pivot do
                let less = less + [e].
            else
                let more = more + [e].
            end 
        end 
                     
        return qsort less + [pivot] + qsort more.
    end 
    
println (qsort [3,2,1,0]).
assert (qsort [3,2,1,0] == [0,1,2,3]).
'''
interp(program, symtab_dump=False)


# In[65]:


program = '''
load "io".
load "util".

function postfix
    with (op, cl, cr) do 
        return (postfix cl, postfix cr, op)
    orwith (op, c) do 
        return (postfix c, op)
    orwith (v,) do 
        return (v,)
end 

println (postfix ("+", (1,), (2,))).
assert ((postfix ("+", (1,), (2,))) is ((1,),(2,),"\+")). -- strings in patterns are REs!
'''
interp(program, tree_dump=False, do_walk=True, exceptions=True, symtab_dump=False)


# In[66]:


program ='''
load "util".
load "io".

let cnt = tointeger(input("Please enter an integer value: ")).

for i in 1 to cnt do
    println i.
end 
assert (i is cnt).
'''

interp(program)


# In[67]:


program ='''
load "io".

let name = input("Please enter your name: ").
println("Hello " + name + "!").
'''

interp(program)


# In[68]:


from asteroid_interp import interp
program = '''
load "util".
load "io".

structure S with
    data x.
    end 
    
let x = 'S(S(0)).
let y = 'S(S(x)).
let z = y.

println y.
println z.
println (eval (z)).
assert ((eval (z)) is S(S(S(S(0))))).
'''
interp(program, tree_dump=False, symtab_dump=False, do_walk=True, exceptions=False)


# In[69]:


program = '''
load "io".
load "util".

function ident 
    with n do 
        return n 
    end  

let y = ident ident  0.

println y.
assert (y is 0).
'''
interp(program, tree_dump=False, symtab_dump=False, do_walk=True, exceptions=True)


# In[70]:


program = '''
load "io".
load "util".

function ident 
    with n do 
        return n 
    end  

let y = ident(ident (0)).
let x = ident ident 0.

println (x,y).
assert ((x,y) is (0,0)).
'''
interp(program, tree_dump=False, symtab_dump=False, do_walk=True, exceptions=False)


# In[71]:


program = '''
-- Factorial

load "io".
load "util".

function fact 
    with 0 do
        return 1
    orwith n do
        return n * fact (n-1).
    end 

println ("The factorial of 3 is: " + fact (3)).
assert (fact(3) == 6).
'''
interp(program, exceptions=False, symtab_dump=False)


# In[72]:


program = '''
-- show that the value constructed by head-tail is a list
let [1,2,3] = [1 | [2,3]].

-- show that a list can be decomposed with head-tail
let [1 | [2,3]] = [1,2,3].

-- show that we can nest head-tail operators
let [1,2,3] = [1 | [2 | [3 | []]]].

'''
interp(program, tree_dump=False, symtab_dump=False, do_walk=True, exceptions=False)


# In[73]:


program = '''
load "util".
load "io".

structure MyError with 
    data e.
    end 

try

    throw Error "--- error ---".
    
catch Error(msg) do
    println msg.
    assert (msg == "--- error ---")
end 

'''
interp(program, tree_dump=False, symtab_dump=False, do_walk=True, exceptions=False)


# In[74]:


program = '''
load "io".
load "util".

let [h|t] = [1,2,3].
println ("head: " + h + " tail: " + t).
assert(h == 1 and t == [2,3]).
'''
interp(program, tree_dump=False, symtab_dump=False, do_walk=True, exceptions=False)


# In[75]:


program = '''
load "io".
load "util".

let y = -1.
let x = 4 if y == 3 else 0.
println x.
assert (x == 0).
'''
interp(program, tree_dump=False, symtab_dump=False, do_walk=True, exceptions=False)


# In[76]:


program = '''
load "io".
load "util".

for x in 0 to 10 do
    println x.
    if x == 5 do
        break.
    end 
end 
assert (x == 5).
'''
interp(program, tree_dump=False, do_walk=True, exceptions=False)


# In[77]:


program = '''
load "io".

for (x,y) in [(1,1), (2,2), (3,3)]  do
    println (x,y).
end 

-- use unification as a filter
for (2,y) in [(1,11), (1,12), (1,13), (2,21), (2,22), (2,23)]  do
    println y.
end 
'''
interp(program, tree_dump=False, do_walk=True, symtab_dump=False)


# In[78]:


program = '''
load "io".

for x in 1 to 10 do
    println x.
end 
'''
interp(program, tree_dump=False, do_walk=True, symtab_dump=False)


# In[79]:


program = '''
load "io".

for bird in ["turkey","duck","chicken"] do
    println bird.
end 
'''
interp(program, tree_dump=False, do_walk=True, symtab_dump=False)


# In[80]:


program = '''
load "io".

let x = 42.

if x < 0 do
    let x = 0.
    println("Negative changed to zero").

elif x == 0 do
    println("Zero").

elif x == 1 do
    println("Single").

else do
    println("More").
    
end 

'''
interp(program, tree_dump=False, do_walk=True, exceptions=False)


# In[81]:


program = '''
load "io".
load "util".

let x = 1.
while x <= 10 do
    println x.
    let x = x + 1.
end 
assert (x == 11).
'''
interp(program, exceptions=True)


# In[82]:


program = '''
load "io".
load "util".

let x = 1.
loop
    if x > 10 do
        break.
    end
    println x.
    let x = x + 1.
end 
assert (x == 11).
'''
interp(program, exceptions=True)


# In[83]:


program = '''
let 1 = 1.
'''
interp(program, exceptions=False, tree_dump=False)


# In[84]:


program = '''
load "io".
load "util".

structure A with
    data x.
    data y.
    end 

let a = A(999, (lambda with (self) do return "Hello World!")).
println (a@1()).

assert(a@1() is "Hello World!").

'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=True, do_walk=True)


# In[85]:


program = '''
load "io".

println (1,2,3).
'''
interp(program)


# In[86]:


program = '''
load "io".
load "util".
let nl = [[1 to 5]].
println nl.
assert (nl is [[1,2,3,4,5]]).
'''
interp(program, tree_dump=False, symtab_dump=False)


# In[87]:


program = '''
-- TODO: should this work similar to Rust?
load "io".
load "util".
try 
    let [1 to 3] = [1,2,3].
catch (_, v) do
    println v.
    assert (v is "pattern match failed: pattern of type 'to-list' not allowed in pattern matching").
end 

let [1,2,3] = [1 to 3].
'''
interp(program, symtab_dump=False, tree_dump=False, exceptions=True)


# In[88]:


program = '''
load "io".
load "util".

structure A with
    data x.
    data y.
    data z.
    end 

let a = A(1,2,3).
let b = a@[0 to 2].
println b.
assert (b is [1,2,3]).
'''
interp(program, symtab_dump=False, tree_dump=False, exceptions=True)


# In[89]:


program = '''
load "io".
load "util".

let a = [0,1,2,3].
let b = a@[0 to 3 step 2].
println b.
assert (b is [0,2]).
'''
interp(program, symtab_dump=False, tree_dump=False, exceptions=True)


# In[90]:


program = '''
load "io".
load "util".

let y = [0 to 10 step 2].
let z = 4 in y.
println z.
assert z.
'''
interp(program, symtab_dump=False, tree_dump=False)


# In[91]:


program = '''
let true = 3 in [1,2,3].
'''
interp(program, symtab_dump=False)


# In[92]:


program = '''
let true = (1,2) is (1,x).
let true = (1,3) is (1,x).

'''
interp(program, symtab_dump=False)


# In[93]:


program = '''
load "io".
load "util".

let b = [[1,2,3],
         [4,5,6],
         [7,8,9]].
let b@1@1 = 0.
println b.
assert (b is [[1,2,3],[4,0,6],[7,8,9]]).
'''
interp(program, tree_dump=False, exceptions=False)


# In[94]:


program = '''
load "io".
-- -1 is a value
try
    let -1 = -1.
catch e do
    println e.
end 

-- not true is a computation
try
    let not true = not true.
catch e do
    println e.
end 
'''
interp(program, tree_dump=False, symtab_dump=False)


# In[95]:


program = '''
load "io".
load "util".

structure A with
    data x.
    end 
    
let a = A([1,2,3]).
let a@0@1 = 0.
println a.
assert (a is A([1,0,3])).
'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=False)


# In[96]:


program = '''
load "io".
load "util".

let a = [1,2,3].
let b = [0,0,0].
let [b@2,b@1,b@0] = a.
println b.
assert (b is [3,2,1]).
'''
interp(program, tree_dump=False, symtab_dump=False, do_walk=True, exceptions=False)


# In[97]:


program = '''
load "io".
load "util".

structure A with 
    data a.
    end 
    
structure B with 
    data x.
    data y.
    end 

let x = A(1).
let y = B(1,2).

let A(z) = x.
let B(v,w) = y.

let xx = x@0.
let yy = y@[0,1].

println (x,y,xx,yy).
assert ((x,y,xx,yy) is (A(1),B(1,2),1,[1,2])).
'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=False)


# In[98]:


program = '''
load "io".

let v = 'a@[3].
raw_print v.
'''
interp(program)


# In[99]:


program = '''
load "io".
load "util".

function inc with n do return n+1 end 

let v = inc(inc(0)).
let q = 1 + 1 + 1.
println (v, q).
assert ((v, q) is (2,3)).
'''
interp(program, tree_dump=False, symtab_dump=False)


# In[100]:


program = '''
-- show off our overloaded '+' operator
load "io".
load "util".

println (1 + 1).
assert ((1+1) == 2).

let s1 = "hello".
let s2 = "world".
let s3 = s1 + " " + s2 +"!".
println s3.
assert (s3 == "hello world!").

let l1 = [1,2,3].
let l2 = [4,5,6].
let l3 = l1 + l2.
println l3.
assert (l3 == [1,2,3,4,5,6]).
'''
interp(program, tree_dump=False, symtab_dump=False, exceptions=False)


# In[101]:


program = '''
load "io".
load "util".

-- reverse the list
let a = [1,2,3].
let a = a@[2,1,0].
println a.
assert (a is [3,2,1]).
'''
interp(program, tree_dump=False, do_walk=True, symtab_dump=False, exceptions=False)


# In[102]:


program = '''
load "io".
load "util".

-- reverse the list
let a = [1,2,3].
let a = a@[2,1,0].
println a.
assert (a is [3,2,1]).

-- access multidim array
let b = [[1,2,3],
         [4,5,6],
         [7,8,9]].
let e = b@1@1.
println e.
assert (e is 5).
'''
interp(program, tree_dump=False, do_walk=True, symtab_dump=False, exceptions=False)


# In[103]:


interp('load "io". load "util". let x = 1.3 . println x. assert(x is 1.3).', exceptions=False)


# In[104]:


program ='''
load "io".
load "util".

try
    let 1 = 1.0 .
catch (_,v) do
    println v.
    assert(v is "pattern match failed: nodes 'real' and 'integer' are not the same").
end 
'''

interp(program)


# In[105]:


interp('load "io". load "util". let (1,y) = (1,2). println (1,y). assert((1,y) is (1,2)).')


# In[106]:


interp('load "io". load "util". let (x,2) = (1,2). println (x,2). assert((x,2) is (1,2)).')


# In[107]:


interp('load "io". load "util". let s = (1,2). let (x,y) = s. println (x,y). assert((x,y) is (1,2))')


# In[108]:


program = '''
load "io".
load "util".

function ident with n do return n end 
println (ident(2)).
assert (ident(2) is 2).
'''
interp(program)


# In[109]:


program = '''
load "io".
load "util".
let x = 1. 
function ident with n do return n end.
println (ident( ident (x))) .
assert (ident( ident (x)) is 1) .
'''
interp(program)


# In[110]:


interp('load "util". let (_, x) = ([1], 2). assert (x is 2).', symtab_dump=False)


# In[111]:


program = '''
load "io".
load "util".

println ((lambda with n do return n+1) 1).
assert ((lambda with n do return n+1) 1 is 2).
'''
interp(program, tree_dump=False, symtab_dump=False)


# In[ ]:




