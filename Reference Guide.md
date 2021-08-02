<!-- Note to Ariel: functions in Asteroid can have different kinds of input
     constellations which is indicated using the `with` and `orwith` keywords.
     When writing documentation you will need to document all the different input
     constellations.  See the `range` function below. -->

# Reference Guide

## Language Features

TBD

## Builtin Functions

* Function `len`, when given an input value, returns the length of that input. The
function can only be applied to lists, strings, tuples, or structures.

* Function `range` will compute a list of values depending on the input values:
1. `(start:%integer,stop:%integer)` returns list `[start to stop-1]`.
1. `(start:%integer,stop:%integer,inc:%integer)` returns list `[start to stop-1 step inc]`.
1. `(stop:%integer)` returns list `[0 to stop-1]`.


## List and String Objects

In Asteroid, both `lists` and `strings,` are treated like objects. Due to this, they have member functions that can manipulate the contents of those objects.

### Lists

As mentioned in [this section of the User Guide](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/Asteroid%20User%20Guide.md#the-basics), a **list** is a structured data type that consists of square brackets enclosing comma-separated values. Lists can be modified after their creation.

<!-- Note to Ariel: the short names in the list below is what users will be
    seeing, the long names are internal names.  So the documentation should
    be written using the short names.

      list_member_functions.update({
          'length'    : '__list_length__',
          'append'    : '__list_append__',
          'extend'    : '__list_extend__',
          'insert'    : '__list_insert__',
          'remove'    : '__list_remove__',
          'pop'       : '__list_pop__',
          'clear'     : '__list_clear__',
          'index'     : '__list_index__',
          'count'     : '__list_count__',
          'sort'      : '__list_sort__',
          'reverse'   : '__list_reverse__',
          'copy'      : '__list_copy__',
          'shuffle'   : '__list_shuffle__',
          'map'       : '__list_map__',
          'reduce'    : '__list_reduce__',
          'filter'    : '__list_filter__',
          'member'    : '__list_member__',
          'join'      : '__list_join__',
          })
      ".

(For implementation details, see Python lists [here](https://docs.python.org/3/tutorial/datastructures.html).)
-->

<!-- Note to Ariel: the member functions to objects have changed.  We no longer
     use the `self` variable.  Make sure that you are using the latest version
     of the `prologue.ast` file. I have edited the first couple of Functions
     so you can see what I mean. -->

* Function `length` returns the number of elements within that list.
* Function `append`, given `(item)`, adds that item to the end of a list.
* Function `extend`, given `(item)`, will extend the list by adding all the items from the item where `item` is either a list, a string or a tuple.
* Function `insert`, given `(ix:%integer,item)`, will insert an item at a given position. The first argument is the index of the element before which to insert, so `a@insert(0, x)` inserts at the front of the list, and `a@insert(a@length(), x)` is equivalent to `a@append(x)`.
* Function `remove`, given `(item)`, removes the first element from the list whose value is equal to `(item)`. It raises a ValueError if there is no such item.
* Function `pop`, given `(ix:%integer)`, removes the item at the given position in the list and returns it. If no index is specified,`a@pop()` removes and returns the last item in the list.
* Function `clear`, given `(none)`, removes all items from the list.
* Function `index` returns a zero-based index in the list of the first element whose value is equal to `(item)`. It raises a ValueError exception if there is no such item. The optional arguments `(startix)` and `(endix)` are interpreted as in the slice notation, and are used to limit the search to a particular subsequence of the list. The returned index is computed relative to the beginning of the full sequence rather than the `(startix)` argument.     

This function can be called with several inputs:
1. Input `(item,startix:%integer,endix:%integer)` returns `('integer', this_val[1].index(item_val,
                                                                          startix_val[1],
                                                                          endix_val[1]))`
1. Input `(item,startix:%integer)` returns `('integer', this_val[1].index(item_val, startix_val[1]))`
1. Input `(item)` returns `('integer', this_val[1].index(item_val))`

* Function `count`, given `(item)`, returns the number of times `(item)` appears in the list.
* Function `sort` sorts the items of the list in place.

It can be called with several different inputs:
1. Input `(reverse:%boolean)` returns `(this_val)`.
1. Input `(none)` returns `(this_val)`.

* Function `reverse`, given `(none)`, reverses the elements of the list in place.
* Function `copy`, given `(none)`, makes a shallow copy of the list.
* Function `shuffle`, given `(none)`, returns a random permutation of a given list - in place!
* Function `map`, given `(f:%function)`, applies `f` to each element of the list.
* Function `reduce` reduces the value of elements in a list. 

This function can be called with several different inputs:
1. Input `(f:%function)` returns `value`, such that `value = f(value,this@i)`.
1. Input `(f:%function,init)` returns the same format.

The first argument to `f` is the accumulator.

* Function `filter`, given `(f:%function)`, constructs an output list from those elements of the list for which `f` returns true. If `f` is none, the identity function is assumed, that is, all elements of the input list that are false are removed.
* Function `member`, given `(item)`, returns `true` only `if this @count(item) > 0`.
* Function `join`, given `(join:%string)`, converts an Asteroid list into a Python list.


See the [Prologue module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/prologue.ast) for more on all the functions above.


### Strings

A string is a sequence of characters that can be used as a variable or a literal constant.

<!-- Note to Ariel: this is not a member function and should be documented as part of the `type` module
* Function `gettype` will get the type of `x` as an Asteroid string. (See the module [Type.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast) for more on this function.) -->

<!-- Note to Ariel: This should not be exposed to the user, this is an internal data structure
* The following `string_member_functions`,

            escape
            "
            global string_member_functions

            string_member_functions.update({
            'length'    : '__string_length__',
            'explode'   : '__string_explode__',
            'trim'      : '__string_trim__',
            'replace'   : '__string_replace__',
            'split'     : '__string_split__',
            'toupper'   : '__string_toupper__',
            'tolower'   : '__string_tolower__',
            'index'     : '__string_index__',
            'flip'      : '__string_flip__',
            })
            "
(For implementation details, see Python lists [here](https://docs.python.org/3/library/stdtypes.html#text-sequence-type-str).)
-->
<!-- Note to Ariel: I edited the first few member functions -->
* Function `length` returns the number of characters within that string.
* Function `explode`, turns a string into a list of characters.
* Function `trim`, given the input `(what:%string)`, returns a copy of the string with the leading and trailing characters removed. The `what` argument is a string specifying the set of characters to be removed. If omitted or none, the `what` argument defaults to removing whitespace. The `what` argument is not a prefix or suffix; rather, all combinations of its values are stripped.

* Function `replace` will return a copy of the string with all occurrences of regular expression pattern `old` replaced by the string `new`. If the optional argument count is given, only the first count occurrences are replaced.

It can be called with several different inputs:
1. Input `(old:%string,new:%string,count:%integer)` returns `('string', sub(old_val[1], new_val[1], this_val[1], count_val[1]))`.
1. Input `(old:%string,new:%string)` returns `('string', sub(old_val[1], new_val[1],this_val[1]))`.

* Function `split` will return a list of the words in a given string, using `sep` as the delimiter string. If `maxsplit` is given: at most maxsplit splits are done (thus, the list will have at most maxsplit+1 elements). If maxsplit is not specified or -1, then there is no limit on the number of splits (all possible splits are made).

If `sep` is given, consecutive delimiters are not grouped together and are deemed to delimit empty strings (for example, '1,,2'.split(',') returns ['1', '', '2']). The sep argument may consist of multiple characters (for example, '1<>2<>3'.split('<>') returns ['1', '2', '3']). Splitting an empty string with a specified separator returns [''].

If `sep` is not specified or is None, a different splitting algorithm is applied: runs of consecutive whitespace are regarded as a single separator, and the result will contain no empty strings at the start or end if the string has leading or trailing whitespace. Consequently, splitting an empty string or a string consisting of just whitespace with a None separator returns [].

Function `split` can be called with several different inputs:
1. Input `(sep:%string,count:%integer)` returns `('list', ast_list)`
1. Input `(sep:%string)` returns the same.
1. Input `(none)` also returns the same.

* Function `toupper`, given `(none)`, converts all the lowercase letters in a string to uppercase.
* Function `tolower`, given `(none)`, converts all the uppercase letters in a string to lowercase.
* Function `index` allows the user to search for a given `item_val[1]`, and/or `startix_val[1]` and `endix_val[1]` as well.

It can be called with several different inputs:
1. Input `(item:%string,startix:%integer,endix:%integer)` returns `('integer',val)` -- **unless** `val` == -1, in which case `__retval__ = ('none', None)`.
1. Input `(item:%string)` returns all of the same.
1. Input `(item:%string)` also returns all of the same.

* Function `flip` explodes, reverses, and joins the given input `(none)`.

See the [Prologue module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/prologue.ast)  for more on all the functions above.

/////

* Function `tointeger`, given `(item:%string,base:%integer)`, returns `('integer', int(item_val[1], base=base_val[1]))`. It can also be called with `item`, which gives the output `('integer', int(item_val[1]))`.
* Function `tostring` converts an Asteroid object into a string. If format values are given, it applies the formatting to the object. It can be called with several different inputs: `(v:*TP,w:%integer)`, `(v:%real,w:%integer)`, `(v:%real,w:%integer,p:%integer)`, and `item`.
* Function `tobase`, given `(x:%integer,base:%integer)`, returns the given integer `x` as a numeral in different bases (as a string).

See the [Type.ast Module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast) for the functions above.

More information about the functions that `lists` and `strings` contribute to can be found in the [Prologue.ast Module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/prologue.ast), as well as the [Type.ast Module](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast).


## Asteroid Modules

There are a variety of useful modules that can be installed in Asteroid.

[Bitwise.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/bitwise.ast) defines Bitwise operations. It supports the following functions,
* Function `band` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise AND operation.
* Function `bor` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise OR operation.
* Function `bnot` can be called with the input `(x:%integer)`, and performs the Bitwise NOT operation.
* Function `bxor` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise XOR operation.
* Function `blshift` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise left shift operation.
* Function `brshift` can be called with the input `(x:%integer, y:%integer)`, and performs the Bitwise right shift operation.
* Function `blrotate` can be called with the input `(x:%integer, i:%integer)`, and performs the Bitwise left rotate operation.
* Function `brrotate` can be called with the input `(x:%integer, i:%integer)`, and performs the Bitwise right rotate operation.
* Function `bsetbit` can be called with the input `(x:%integer, i:%integer)`, and sets the ith bit.
* Function `bclearbit` can be called with the input `(x:%integer, i:%integer)`, and clears the ith bit.
* Function `bsize`can be called with the input `(x:%integer)`, and returns the bit size.

[Hash.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/hash.ast) implements the `HashTable` structure, for mapping keys onto paired values. It supports the following functions,
* Function `__init__` can be called with the input `self`. This constructor for HashTable initializes the underlying dictionary, and stores the dictionary as a foreign object in its object memory alongside the table.
* Function `insert`, given the input `(self,name,value)`, will insert a given name-value pair into the table in `self`'s object memory.
* Function `get`, given `(self,name)`, will return the `value_val` associated with the given `name_val` as long as it can be found in `dictionary.keys()`.
* Function `aslist`, given `(self)`, gets the `value_val` associated with `name_val` and then zips the keys and values. It then turns Python tuples into Asteroid tuples and puts them onto an output list, so that the return value output is `('list', out_list)`.

[IO.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/io.ast) implements Asteroid's I/O system for delivering outputs from given inputs. It supports the following functions,
* Function `raw_print` can be called with `item`, and dumps the AST to screen.
* Function `print_ln` can be called with `item`, and prints a given argument.
* Function `print` can be called with `item`, and prints a given argument.
* Function `read` can be called with `none`, and will read a string from `stdin`.
* Function `input` can be called with a string `prompt`, and maps the Asteroid input function onto the Python input function.

[Math.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/math.ast) implements its mathematical constants and operators. It supports the following functions,
* Function `exp`, given `(x:%integer)`, returns e raised to the power `x`, where e = 2.718281… is the base of natural logarithms.
* Function `log` can be called with two different argument setups. 
1. If only one argument, `(x)`, is inputted, this returns the natural logarithm of x (to base e). 
1. If two arguments, `(x,base)`, are inputted, this returns the logarithm of x to the given base, calculated as log(x)/log(base).

* Function `pow`, given `(b,p:%integer)`, returns "b <sup>p</sup>" as long as b can be found in `real` and `integer`.
* Function `sqrt`, given `a`, returns its square root as long as `a` can be found in `real` and `integer`.
* Function `abs`, given `(x)`, returns its absolute value.
* Function `ceil`, given `(x:%real)`, returns the ceiling of x: the smallest integer greater than or equal to x.
* Function `floor`, given `(x:%real)`, returns the floor of x: the largest integer less than or equal to x.
* Function `gcd`, given `(a:%integer,b:%integer)`, returns the greatest common denominator that both integers share.
* Function `isclose` can be called with two different argument setups.
1. With input values `(a,b)`, it returns returns `True` if the two values are close to each other and `False` otherwise. Default tolerance 1e-09.
1. With input values `(a,b,t)`, it performs the same tasks for comparing these *three* different input values.

* Function `mod`, given `(v,d)`, will return the remainder of the operation `v/d`, as long as `v` and `d` can be found in `real` and `integer`.
* Function `acos`, given `(x)`, returns the arc cosine of x in radians. The result is between 0 and pi.
* Function `asin`, given `(x)`, returns the arc sine of x in radians. The result is between -pi/2 and pi/2.
* Function `atan`, ,given `(x)`, returns the arc tangent of x in radians. The result is between -pi/2 and pi/2.
* Function `cos`, given `(x)`, returns the cosine of x radians.
* Function `sin`, given `(x)`, returns the sine of x radians.
* Function `tan`, given `(x)`, returns the tangent of x radians.
* Function `acosh`, given `(x)`, returns the inverse hyperbolic cosine of x.
* Function `asinh`, given `(x)`, returns the inverse hyperbolic sine of x.
* Function `atanh`, given `(x)`, returns the inverse hyperbolic tangent of x.
* Function `cosh`, given `(x)`, returns the hyperbolic cosine of x.
* Function `sinh`, given `(x)`, returns the hyperbolic sine of x.
* Function `tanh`, given `(x)`, returns the hyperbolic tangent of x.
* Function `degrees`, given `(x)`, converts angle `x` from radians to degrees.
* Function `radians`,  given `(x)`, converts angle `x` from degrees to radians.

[Pick.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/pick.ast) implements the `Pick` structure. A `pick` object contains a list of items that can be randomly picked from using the `pick` member function. It supports the following functions that perform an output of this structure,
* Function `pick` can be called with `n:%integer`.
* Function `__init__` can be called with `l:%list`.

[Random.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/random.ast) implements the `random` numbers. Using the functions included in this module will return a random value or floating point number within a given range or interval. It supports the following functions,
* Function `random`, given the input `none`, returns a random floating point number in the range `[0.0, 1.0)`.
* Function `randint` returns a random value N in the interval lo <= N <= hi. The exact random value output depends on the types of the values specifying the interval. It can be called with two different number interval inputs:
1. `(lo:%integer,hi:%integer)` 
1. `(lo:%real,hi:%real)`
1. Note: if the given input is ` (_,_) `, it will instead output an error message for "unsupported interval specificaton in randint."

* Function `seed`, given `(sd:%integer)`, returns a random value N in the interval lo <= N <= hi. The exact random value output depends on the types of the values specifying the interval.

[Set.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/set.ast) implements Asteroid sets as lists. Unlike lists, sets do not have repeated members. It supports the following functions,
* Function `toset`, given `(lst:%list)`, converts the inputted list into a set.
* Function `sdiff`, given `(a:%list,b:%list)`, does a side-by-side print of the differences between two files.
* Function `sunion`, given `(a:%list,b:%list)`, prints the smallest set which contains all the elements of both `a` and `b`.
* Function `sintersection`, given `(a:%list,b:%list)`, finds the interection between  `a` and `b`.
* Function `sxunion`, given `(a:%list,b:%list)`, returns all elements in `a` or `b`, but not in both.

[Sort.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/sort.ast) defines a parameterized sort function over a list. `Sort` makes use of a user-defined order predicate on the list's elements to perform the sort. The `Quicksort` is the underlying sort algorithm. It supports the function below,
* Function `sort` can be called with three different inputs to perform its sorting output. These are: `(_,[])`, `(_,[a])`, and `(p,[pivot|rest])`.

[Stream.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/stream.ast) implements the `Stream` structure. Asteroid stream implementation is based on lists.
* Function `__init__` can be called with two different inputs: `none` and `stream:%list`. In either case, this function outputs a shallow copy of the input list.
* Function `eof` can be called with the input `none`. If `this @curr_ix == this @stream @length()`, the function returns `true`. If not, it returns `false`.
* Function `peek` can be called with the input `none`. If `this @eof()`, it returns `none`. If not, it returns `this @stream @(this @curr_ix)`.
* Function `next` can be called with the input `none`. If `this @eof()`,  it returns `none`. If not, it decides to `let this @curr_ix = this @curr_ix + 1`.
* Function `get` can be called with the input `none`, and returns `this @peek()`.
* Function `rewind` can be called with the input `none`, and then decides to `let this @curr_ix = 0`.
* Function `map`, given the input `f`, applies a given function to each element of a function.
* Function `append`, given `item`, adds said item to a stream.
* Function `__string__`, given `none`, outputs it as a string.

[Type.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/type.ast) defines type related functions and structures.

See the Built-ins section of this Reference Guide for more on the list and string related functions this module supports. Other than those, this module supports the following functions,

* Function `toreal`, given `item`, returns the input as a real number data type `('real', float(item_val[1]))`.
* Function `toboolean`, given `item`, returns the input as a Boolean value of either true or false.
* Function `isscalar`, given `item`, returns `(item is %integer)` or `(item is %real)`.
* Function `isnone`, given `x`, returns `x is %none`.

[Util.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/util.ast) defines utility functions and structures. It supports the following functions,

* Function `exit`, given `none` or `msg:%string`, imports `sys.exit(1)` to exit the program.
* Function `copy`, given Asteroid object `obj`, makes a deep copy of it.
* Function `cls`, given `none`, clears the screen.
* Function `sleep`, given `secs %if isscalar(secs)`, programs sleep for `secs` seconds.
* Function `zip`, given `(list1:%list,list2:%list)`, implements Python's zip function. It turns Python tuples into Asteroid tuples and puts them onto an output list.
* Function `unzip`, given `(list:%list)`, will unzip a list of pairs.
* Function `ascii`, given a character `item:%string`, returns the corresponding ASCII code.
* Function `achar`, given a decimal ASCII code `item:%integer`, returns the corresponding character symbol.

[Vector.ast](https://github.com/lutzhamel/asteroid/blob/ariel-asteroid-copy/code/modules/vector.ast) defines functions useful for vector arithmetic. It supports the following functions,
* Function `vop` can be called with three different inputs: `(f:%function,a:%list,b:%list)`, `(f:%function,a:%list,b %if isscalar(b))`, and `(f:%function,a %if isscalar(a),b:%list)`. In any case, this function implements actual vector arithmetic. It also implements vector/scalar arithmetic.
* Function `vadd`, given the input `(a,b)`, returns `vop(lambda with (x,y) do return x+y,a,b)`.
* Function `vsub`, given the input `(a,b)`, returns `vop(lambda with (x,y) do return x-y,a,b)`.
* Function `vmult`, given the input `(a,b)`, returns ` vop(lambda with (x,y) do return x*y,a,b)`.
* Function `dot`, given `(a:%list,b:%list)`, computes the dot product of the two lists.
