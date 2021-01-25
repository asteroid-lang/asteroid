#!/usr/bin/env python
# coding: utf-8

# # Redundant Pattern Detection Tests

# This is a redundant pattern detection test suite for the Asteroid programming language.
# 
# This test suite will demonstrate many of the different situations in which a redundat pattern will be detected.
# 
# 

# In[1]:


#Import the actual Asteroid interpreter.
from asteroid_interp import interp


# ## Tuples, Variables(IDs), Integers
#   
# Below are several examples of Asteroid programs with redundant patterns in functions that are composed of Tuples, Variables, and Integers. The programs are correct Asteroid programs except for the redundant patterns.
# 
# 
# The programs should all throw a "RedundantPatternFound" exception, which will then be caught and have its message printed to the console.

# In[2]:


program = '''
--------------------------------------
-- patterns_test0.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with x do         --will SUBSUME all other patterns
        return 0.
    orwith ( 1, 2, z) do
        return 1.
    orwith ( 1, y, z) do
        return 2.
    orwith ( x, y, z) do
        return 3.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 1 , 2 , 3 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[3]:


program = '''
--------------------------------------
-- patterns_test1.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with ( 1, value1, value2) do --will SUBSUME ( 1, y, z)
        return 1.
    orwith ( 1, y, z) do
        return 2.
    orwith ( x, y, z) do
        return 3.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 1 , 2 , 3 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[4]:


program = '''
--------------------------------------
-- patterns_test2.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with 1 do --will SUBSUME 1 
        return 1.
    orwith 2 do
        return 2.
    orwith 1 do
        return 3.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 1 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[5]:


program = '''
--------------------------------------
-- patterns_test3.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with ( 1, 2, 3) do 
        return 1.
    orwith ( 1, 2, ( x1, "test_string" ) ) do --will SUBSUME ( 1, 2, ( var,"test_string"))
        return 2.
    orwith ( x, y, z) do
        return 3.
    orwith ( 1, 2, ( var, "test_string" ) ) do
        return 3
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 1 , 2 , 3 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[6]:


program = '''
--------------------------------------
-- patterns_test4.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with ( x, y, z) do  -- *** will SUBSUME ( (x,y), (x,y), (x,y) ) ***
        return 1.
    orwith ( (x1,y1), (x2,y2), (x3,y3) ) do 
        return 2.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 1 , 2 , 3 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# # Strings

# In[7]:


program = '''
--------------------------------------
-- patterns_test5.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with "name1" do  
        return 1.
    orwith "name2" do   -- *** will SUBSUME "name2" ***
        return 2.
    orwith "name2" do   -- duplicate pattern
        return 2.
    orwith "name4" do 
        return 2.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( "name4" ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[8]:


program = '''
--------------------------------------
-- patterns_test6.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with "abc" do       -- All strings are treated as regular expressions when both unifing/subsuming
        return 1.       -- this may be a bug.
    orwith "abcdef" do  -- However, "abcdef" is redundant given this current behavior
        return 2.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( "name4" ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# # Float / Double

# In[9]:


program = '''
--------------------------------------
-- patterns_test7.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with 1.1 do       -- *** will subsume 1.1 
        return 1. 
    orwith 1.2 do 
        return 2.
    orwith 1.3 do 
        return 3.
    orwith 1.1 do     -- *** duplicate pattern
        return 4.     
    end.
    

-- Evaluate the function with a test input.
let x = 1.1.
let y = testFunction( x ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + y ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[10]:


program = '''
--------------------------------------
-- patterns_test8.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with 1.10 do       -- *** will subsume 1.10000...
        return 1. 
    orwith 1.2 do 
        return 2.
    orwith 1.3 do 
        return 3.
    orwith 1.10000 do   -- *** duplicate pattern 1.10
        return 4.     
    end.
    

-- Evaluate the function with a test input.
let x = 1.1.
let y = testFunction( x ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + y ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# # Boolean Values

# In[11]:


program = '''
--------------------------------------
-- patterns_test9.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with true do       
        return 1. 
    orwith false do -- *** will subsume false
        return 2.    
    orwith false do -- *** duplicate pattern
        return 2.  
    end.
    

-- Evaluate the function with a test input.
let x = true.
let y = testFunction( x ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + y ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# # Regular Expressions

# In[12]:


program = '''
--------------------------------------
-- patterns_test10.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".
load "util".

function test_function 
  with ".*p.*" do               -- RE ".*p.*"will match to all ".*http.*" and ".*https.*"
    return "Found a p.".
  orwith ".*http.*" do          --Redundant
    return "Found a url.".
  orwith ".*https.*" do         --Redundant
    return "Found a secure url.".
  orwith ".*z.*" do
    return "Found a z.".
  end.

let var = test_function( "test string z" ).
println ( var ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[13]:


program = '''
--------------------------------------
-- patterns_test11.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".
load "util".

function test_function 
  with ".*q.*" do               
    return "Found a p.".
  orwith ".*http.*" do          -- RE ".*http.*" will match to all ".*https.*" patterns
    return "Found a url.".
  orwith ".*https.*" do         --Redundant
    return "Found a secure url.".
  orwith ".*z.*" do
    return "Found a z.".
  end.

let var = test_function( "test string z" ).
println ( var ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# ## Lists
# Includes head-tail patterns.   
# All of the programs should throw a RedundantPatternFound exception.
#   

# In[14]:


program = '''
--------------------------------------
-- patterns_test12.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with [ x, y ] do  -- *** will SUBSUME [ 1, 2 ] ***
        return 1.
    orwith [ 1, 2 ] do 
        return 2.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction[ 3 , 4 ].

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[15]:


program = '''
--------------------------------------
-- patterns_test13.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
-- By Timothy Colaneri
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with [ x, y ] do  
        return 1.
    orwith [ head | tail ] do  -- *** will SUBSUME [ a, b, c, d ] ***
        return 2.
    orwith [ a, b, c, d ] do 
        return 3.    
    end.
    
-- Evaluate the function with a test input.
let x = testFunction[ 3 , 4 ].

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[16]:


program = '''
--------------------------------------
-- patterns_test14.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with [ h1 | h2 | h3 |  tail ] do  -- *** will SUBSUME [ h1 | h2 | h3 | h4 | tail ] ***
        return 1.
    orwith [ h1 | h2 | h3 | h4 | tail ] do
        return 2. 
    orwith [ h1 | tail ] do -- ** will not catch ** can still make matches others can't! ex. [ 1, 2]
        return 3. 
    end.
    
-- Evaluate the function with a test input.
let x = testFunction[ 3 , 4 ].

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[17]:


program = '''
--------------------------------------
-- patterns_test15.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with [ 1 | 2 | 3 | 4 | tail ] do  -- *** will SUBSUME [1, 2, 3, 4 ] ***
        return 1.
    orwith [ 1, 2, 3, 4 ] do
        return 2. 
    orwith [ h1 | tail ] do 
        return 3. 
    orwith [] do
        return 4.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction[ 3 , 4 ].

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# # Named Patterns

# In[18]:


program = '''
--------------------------------------
-- patterns_test16.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with person1:( fname, lname) do  -- *** will SUBSUME location:( long, lat ) ***
        return 1.
    orwith person2:( fname, lname1, lname2) do
        return 2.
    orwith location:( long, lat ) do 
        return 3. 
    orwith () do
        return 4.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 3 , 4 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[19]:


program = '''
--------------------------------------
-- patterns_test17.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with person1:( fname, lname) do  -- *** will SUBSUME (x, y) ***
        return 1.
    orwith person2:( fname, lname1, lname2) do
        return 2.
    orwith ( x, y ) do 
        return 3. 
    orwith () do
        return 4.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 3 , 4 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[20]:


program = '''
--------------------------------------
-- patterns_test18.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with ( fname, lname) do  -- *** will SUBSUME (x, y) ***
        return 1.
    orwith person2:( fname, lname1, lname2) do
        return 2.
    orwith coord:( x, y ) do 
        return 3. 
    orwith () do
        return 4.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 3 , 4 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[21]:


program = '''
--------------------------------------
-- patterns_test19.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

-- A testing function with multiple patterns
function testFunction
    with person:[( fname, lname)| tail] do  -- *** will SUBSUME [( fname1, lname1)|( fname2, lname2)| tail] ***
        return 1.
    orwith person:( fname, lname1, lname2) do
        return 2.
    orwith person:[( fname1, lname1)|( fname2, lname2)| tail] do 
        return 3. 
    orwith () do
        return 4.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction[ (test,name) ].

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ).
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# # Typeclass Patterns

# In[22]:


program = '''
--------------------------------------
-- patterns_test20.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

function fact
    with 0 do
        return 1
    orwith (n:%integer) do   -- Will subsume pattern (n:%integer)
        return n * fact (n-1).
    orwith (n:%integer) do
        throw Error("factorial is not defined for "+n).
    end.
    
-- Evaluate the function with a test input.
let x = fact( 5 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x ). 
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[23]:


program = '''
--------------------------------------
-- patterns_test21.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

function testFunction
    with 0 do
        return 1.
    orwith (x:%integer, y:%integer ) do
        return 2.
    orwith (n:%integer, m:%real) do   -- Will Subsume (value1:%integer, value2:%real)
        return 3.
    orwith (value1:%integer, value2:%real) do
        return 4.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 5,1 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x  ). 
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[24]:


program = '''
--------------------------------------
-- patterns_test22.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

function testFunction
    with 0 do
        return 1.
    orwith ( var1 , var2 ) do          -- Will leave all following patterns redundant
        return -1.
    orwith (x:%integer, y:%integer ) do
        return 2.
    orwith (n:%real, m:%real) do
        return 3.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( 5,1 ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x  ). 
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[25]:


program = '''
--------------------------------------
-- patterns_test23.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

function testFunction
    with x:%tuple do                   -- Will render all following patterns redundant
        return 1.
    orwith (x:%integer, y:%integer ) do
        return 2.
    orwith (n:%real, m:%real) do
        return 3.
    orwith ( x, y, z) do
        return 4.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( (5,1) ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x  ). 
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[26]:


program = '''
--------------------------------------
-- patterns_test24.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".

function testFunction
    with x:%list do    -- Will match everything intended for [ head1 | head2 | tail ]
        return 1.
    orwith (x:%integer, y:%integer ) do
        return 2.
    orwith (n:%real, m:%real) do
        return 3.
    orwith [ head1 | head2 | tail ] do
        return 4.
    end.
    
-- Evaluate the function with a test input.
let x = testFunction( (5,1) ).

-- If the function test worked, print the output we got.
println( "The value returned is: " + x  ). 
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# # Structures and Objects

# In[27]:


program = '''
--------------------------------------
-- patterns_test25.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".
load "util".

structure Person with
    data name.
    data age.
    data gender.
    end

-- define a list of persons
let people = [
    Person("George", 32, "M"),
    Person("Sophie", 46, "F"),
    Person("Oliver", 21, "X")
    ].

-- print names that contain 'p'
for Person(name:".*p.*",_,_) in people do
  println name.
  end
  
function testFunction
    with Person("George", 32, "M") do    -- *** will Subsume Person("George", 32, "M")
        return 1.
    orwith Person("Sophie", 46, "F") do
        return 2.
    orwith Person("George", 32, "M") do  --duplicate structure/object
        return 3.
    end.
    
let x = testFunction( Person("George", 32, "M") ).
println ( x ).

'''

try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[28]:


program = '''
--------------------------------------
-- patterns_test26.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
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
    with add(x,y) do              -- Will subsume all matches for add(x,S(y))
        return 1.
    orwith add(x,0) do      
        return reduce(x).
    orwith add(x,S(y)) do         -- The redundant pattern
        return S(reduce(add(x,y))).
    orwith term do     
        return term.
    end 
    
println(reduce(add(add(add(S(S(0)),S(S(S(0)))),S(0)),S(0)))).
'''

try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[29]:


program = '''
--------------------------------------
-- patterns_test27.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".
load "util".

structure Dog
  with
    data name.
    data tricks.

    function __init__
      with (self, name, tricks) do -- Will subsume all matches for (self, name, [])
        let self@name = name.
        let self@tricks = tricks.
      orwith (self, name, []) do  -- Redundant Constructor 
        let self@name = name.
        let self@tricks = [].
      orwith (self, name) do
        let self@name = name.
        let self@tricks = [].
      end

    function add_trick
      with (self, new_trick) do
        let self@tricks = self@tricks + [new_trick].
      end
  end

-- Fido the dog
let fido = Dog("Fido").
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")


# In[30]:


program = '''
--------------------------------------
-- patterns_test28.ast
-- a simple program to test for
-- subsumption detection in Asteroid.
--------------------------------------
--------------------------------------
load "io".
load "util".

structure Person with
    data name.
    data age.
    data gender.
    end

-- define a list of persons
let people = [
    Person("George", 32, "M"),
    Person("Sophie", 46, "F"),
    Person("Oliver", 21, "X")
    ].

function test_function 
  with Person(name:".*p.*",_,_) do
    return "Found a p.".
  orwith Person(name:".*g.*",_,_) do   -- *** The regular expression ".*g.*" will subsume ".*gg.*"
    return "Found a g.".
  orwith Person(name:".*v.*",_,_) do
    return "Found a v.".
  orwith Person(name:".*gg.*",_,_) do  -- will be redundant
    return "Found a gg.".
  end.

let loopCounter = 0.

-- print names that contain 'p'
for Person(_,_,_) in people do
  println test_function( people @loopCounter ).
  let loopCounter = loopCounter + 1.
  end.
'''
try:
    interp(program, exceptions=True, tree_dump=False, do_walk=True, prologue=True )
except Exception as e:
    print(e)
    pass
else:
    print("Error! Redundant pattern not detected!")

