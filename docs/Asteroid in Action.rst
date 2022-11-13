



..
   *** DO NOT EDIT; MACHINE GENERATED ***



.. highlight:: none

Asteroid in Action
==================

This document was inspired by Andrew Shitov's excellent book `Using Raku: 100 Programming Challenges Solved with the Brand-New Raku Programming Language <https://andrewshitov.com/wp-content/uploads/2020/01/Using-Raku.pdf>`_.  Here we use Asteroid to solve these programming challenges.

Section: Using Strings
----------------------

Challenge: Hello, World!
^^^^^^^^^^^^^^^^^^^^^^^^

> Print ‘Hello, World!’

The canonical ``Hello, World!`` program.  The easiest way to write this in Asteroid is,
::

  load system io.

  io @println "Hello, World!".


Output::

    Hello, World!



Challenge: Greet a person
^^^^^^^^^^^^^^^^^^^^^^^^^

> Ask a user for their name and greet them by printing ‘Hello, <Name\>!’

Here is our first solution using a separate function for each of the steps,
::

  load system io.

  io @print ("Enter your name: ").
  let name = io @input().
  io @print ("Hello, "+name+"!").



Letting the function ``input`` do the prompting,
::

  load system io.

  let name = io @input("Enter your name: ").
  io @println ("Hello, "+name+"!").


Doing everything in one step,
::

  load system io.

  io @println ("Hello, "+io @input("Enter your name: ")+"!").


Challenge: String length
^^^^^^^^^^^^^^^^^^^^^^^^

> Print the length of a string.

In order to print the length of a string we can use the function ``len`` available in the ``util`` module,
::

  load system io.

  load "util".
  io @println (len("Hello!")).


Output::

    6


We can also use the string member function ``length`` in order to compute the length of the string,
::

  load system io.

  io @println ("Hello!" @length()).


Output::

    6


Challenge: Unique digits
^^^^^^^^^^^^^^^^^^^^^^^^

> Print unique digits from a given integer number.

In order to accomplish this we take advantage of the string ``explode`` function and the ``sort`` function on lists.
Finally we use the ``reduce`` function to map a list with repeated digits to a list with unique digits,
::

  load system io.

  function unique with (x,y) do
      if not (x @member(y)) do
          return x @append(y).
      else do
          return x.
      end
  end

  let digits = "332211" @explode()
                        @sort()
                        @reduce(unique,[]).
  io @println digits.
  assert(digits == ["1","2","3"]).


Output::

    [1,2,3]


Probably the most noteworthy characteric about this program is the ``reduce`` function.  The ``reduce`` function applies a binary function to a list.  The first argument of the binary function acts like an accumulator, and the second argument gets instantiated with the elements of the list to be processed.  In our function ``unique``, the variable ``x`` is the accumulator with an initial value of ``[]``.  The function tests whether the element ``y`` is in the list.  If it is not, then it adds it to the list. Otherwise, it just returns the accumulator unchanged.

Section: Modifying String Data
------------------------------

Challenge: Reverse a string
^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Print a string in the reversed order from right to left.

We use the ``explode`` function to turn a string into a list of characters. Then, we reverse the list and turn it back into a string using the ``join`` function,
::

  load system io.

  let str = "Hello, World!" @explode()
                            @reverse()
                            @join("").
  io @println str.
  assert(str == "!dlroW ,olleH").


Output::

    !dlroW ,olleH


Challenge: Removing blanks from a string
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Remove leading, trailing, and double spaces from a given string.
::

  load system io.
  let str = "   Hello  ,   World    !   " @trim()
                                          @replace("  ","").
  io @println str.
  assert(str == "Hello, World!").


Output::

    Hello, World!


Challenge: Camel case
^^^^^^^^^^^^^^^^^^^^^

> Create a camel-case identifier from a given phrase.

In this task, we will form the ``CamelCase`` variable for names from a given phrase.
Names created in this style are built of several words, each of which starts
with a capital letter.
::

  load system io.

  function title with w do
      let letter_list = w @tolower()
                          @explode().
      let first_letter = letter_list @0
                                     @toupper().
      if letter_list @length() > 1 do
          let title_case = ([first_letter] + letter_list @[1 to letter_list@length()-1]) @join("").
      else
          let title_case = first_letter.
      end
      return title_case.
  end

  let str = "once upon a time".
  let camel_str = str @split()
                      @map(title)
                      @join("").
  io @println camel_str.
  assert(camel_str == "OnceUponATime").


Output::

    OnceUponATime


Challenge: Incrementing filenames
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Generate a list of filenames like file1.txt, file2.txt, etc.
::

  load system io.

  let root = "file".
  let ext = ".txt".

  for i in 1 to 5 do
      io @println (root+i+ext).
  end


Output::

    file1.txt
    file2.txt
    file3.txt
    file4.txt
    file5.txt


Challenge: Random passwords
^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Generate a random string that can be used as a password.

In our solution we take advantage of Asteroid's ``Pick`` object.  The ``Pick`` object maintains a list of items that we can randomly select from using the ``pick`` member function.  As input to the ``Pick`` object, we compute a bunch of lists of characters that are useful for password construction.  The function ``achar`` converts a decimal ASCII code to a single character string.
::

  load system io.
  load system type.
  load system util.
  load system pick.
  load system random.

  random @seed(42).

  -- make up lists of symbols useful for password construction
  let int_list = [0 to 9] @map(type @tostring).
  let lc_list = [97 to 122] @map(util @achar). -- lower case characters
  let uc_list = [65 to 90] @map(util @achar). --upper case characters
  let sp_list = ["!","_","#","$","%","*"].
  -- build the overall pick list of symbols
  let pick_list = int_list+lc_list+uc_list+sp_list.

  -- generate the password and print it.
  let pwd = pick @pick pick_list @pick(15)
                           @join("").
  io @println pwd.

  assert (pwd == "e3zvshdbS43brt#")


Output::

    e3zvshdbS43brt#


Challenge: DNA-to-RNA transcription
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Convert the given DNA sequence to a compliment RNA.

We’ll not dig deep into the biology aspect of the problem. For us, it is important that the DNA is a string containing the four letters A, C, G, and T,
and the RNA is a string of A, C, G, and U. The transformation from DNA
to RNA happens according to the following table:
::

    DNA: A C G T
    RNA: U G C A

We will solve this programming problem using Asteroid's first-class patterns. We could have solved this with just testing equality on DNA characters. However, using first-class patterns is more general and can be applied to problems with a more structured mapping relationship.
::

  load system io.

  let dna2rna_table =
      [
        ("A","U"),
        ("C","G"),
        ("G","C"),
        ("T","A")
      ].

  function dna2rna with x do
      for (dna,rna) in dna2rna_table do
          if x is *dna do
              return rna.
          end
      end
      throw Error("unknown dna char "+x).
  end

  let dna_seq = "ACCATCAGTC".
  let rna_seq = dna_seq @explode()
                        @map(dna2rna)
                        @join("").
  io @println rna_seq.

  assert(rna_seq == "UGGUAGUCAG").


Output::

    UGGUAGUCAG


Challenge: Caesar cipher
^^^^^^^^^^^^^^^^^^^^^^^^

> Encode a message using the Caesar cipher technique.

The Caesar code is a simple method of transcoding the letters of the message
so that each letter is replaced with the letter that occurs in the alphabet N
positions earlier or later.
For example, if N is 4, then the letter e becomes a, f is transformed to b,
etc. The alphabet is looped so that z becomes v, and letters a to d become
w to z.
::

  load system io.
  load system util.

  let achar = util @achar.
  let ascii = util @ascii.

  let encode_table = [119 to 122] @map(achar) + [97 to 118] @map(achar).

  function encode with (v:%string) if len(v) == 1 do
      -- only lowercase letters are encoded
      if not (ascii(v) in [97 to 122]) do
          return v.
      else
          return encode_table @(ascii(v)-ascii("a")).
      end
  end

  function decode with (v:%string) if len(v) == 1 do
      -- only lowercase letters are decoded
      if not (ascii(v) in [97 to 122]) do
          return v.
      else
          return encode_table @(ascii(v)-ascii("w")+4).
      end
  end

  let message = "hello, world!"
  let secret = message @explode()
                       @map(encode)
                       @join("").
  io @println secret.

  assert (secret == "dahhk, sknhz!")

  let decoded_msg = secret @explode()
                           @map(decode)
                           @join("").
  io @println decoded_msg.

  assert (decoded_msg == "hello, world!")


Output::

    dahhk, sknhz!
    hello, world!


Section: Text Analysis
----------------------

Challenge: Plural endings
^^^^^^^^^^^^^^^^^^^^^^^^^

> Put a noun in the correct form — singular or plural — depending on the number next to it.

In program outputs, it is often required to print some number followed by a noun, for example::

    10 files found

If there is only one file, then the phrase should be ``1 file found`` instead.
::

  load system io.

  for n in 0 to 5 do
      io @println (n+" file"+("s " if n>1 or n==0 else " ")+"found").
  end


Output::

    0 files found
    1 file found
    2 files found
    3 files found
    4 files found
    5 files found


Challenge: The most frequent word
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Find the most frequent word in the given text.

In our solution we use a hash table to count the number of word occurences.
::


  load system io.
  load system util.
  load system hash.

  -- text generated at 'https://www.lipsum.com/'
  let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed
  accumsan magna quis risus commodo, et pellentesque dui cursus. Sed quis risus
  libero. Cras et mattis libero, eget varius nisi. Phasellus ultrices, augue non
  dictum eleifend, nunc elit blandit velit, a viverra risus enim in tellus.
  Maecenas quis ante eget turpis rhoncus rhoncus eget ut mauris. Suspendisse nec
  erat sed nunc tempus hendrerit. Nunc dictum nunc molestie eleifend tempus.
  Praesent cursus lorem diam, sed mattis velit vehicula scelerisque. Nunc iaculis
  rhoncus ante. Etiam quam nisi, fermentum et euismod a, vulputate eu elit.
  Suspendisse tincidunt ligula quis interdum blandit. Quisque sed aliquam tellus.
  Pellentesque ac lacus pulvinar, ornare purus ac, viverra ex. Donec quis pharetra
  dolor.

  In ac massa tortor. Cras sagittis luctus scelerisque. Morbi a neque sed tortor
  ultrices dapibus. Mauris pretium vitae massa non auctor. Cras egestas ex ante,
  ac ullamcorper ante dignissim eget. Fusce bibendum justo ut enim luctus, id
  volutpat diam lacinia. Mauris sit amet ante risus.

  Nullam rhoncus ultricies dui. Etiam vel metus vehicula, pellentesque felis ut,
  suscipit nunc. Sed nec interdum lorem. Maecenas odio erat, vestibulum nec
  dapibus id, commodo vitae libero. Nulla sed urna sit amet nunc commodo finibus
  sed vel elit. Aliquam euismod feugiat nisi quis placerat. Aliquam libero nisl,
  ultrices non est at, sagittis hendrerit dui. Quisque id sem lorem. Nam ultricies
  metus id ultrices molestie. Pellentesque elementum consequat nibh, nec convallis
  lorem ullamcorper in. Etiam vitae mi tellus. Etiam accumsan massa sit amet dolor
  tincidunt iaculis. Nam ullamcorper blandit sem id bibendum. Quisque elementum
  ipsum ac sapien blandit vehicula."

  -- get rid of punctuation, turn to lower case, and split into words.
  -- Note: we could have employed richer regular expressions to clean up the text here
  let wl = text @replace("\.","")
                @replace(",","")
                @tolower()
                @split().

  -- put the words into a hash table, the value is the count of the words
  let ht = hash @hash().
  for w in wl do
      if not ht @get(w) do
          ht @insert(w,1).
      else do
          ht @insert(w,ht @get(w)+1).
      end
  end

  -- get the contents of hash table and find the most frequent word
  let (keys,values) = util @unzip(ht@aslist()).
  let values_sorted = values @copy()
                             @sort(true).
  let most_frequent_word = keys @(values @index(values_sorted @0)).
  io @println most_frequent_word.

  assert (most_frequent_word == "sed").


Output::

    sed


Challenge: The longest common substring
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Find the longest common substring in the given two strings.

Let us limit ourselves with finding only the first longest substring. If there
are more common substrings of the same length, then the rest are ignored.
There are two loops (see also Task 17, The longest palindrome) over the first
string (``stra``). These use the index method to search for the substring in the
second string (``strb``).
::

  load system io.

  let stra = "the quick brown fox jumps over the lazy dog".
  let strb = "what does the fox say?".
  let common = "".

  for startix in 0 to stra @length()-1 do
      for endix in startix to stra @length()-1 do
          let s = stra @[startix to endix].
          if strb @index(s) and s @length() > common @length() do
              let common = s.
          end
      end
  end

  if common do
      io @println ("The longest common substring is '"+common+"'.").
  else do
      io @println ("There are no common substrings.").
  end

  assert (common == " fox ").


Output::

    The longest common substring is ' fox '.


Challenge: Anagram test
^^^^^^^^^^^^^^^^^^^^^^^

> Tell if the two words are anagrams of each other.

An anagram is a word, phrase, or name formed by rearranging the letters of another, such as ``cinema``, formed from ``iceman``.
::

  load system io.

  let str1 = "cinema".
  let str2 = "iceman".

  function normalize with str do
      return str @explode()
                 @sort()
                 @join("").
  end

  if normalize(str1) == normalize(str2) do
      io @println "Anagrams".
  else do
      io @println "Not anagrams".
  end

  assert (normalize(str1) == normalize(str2)).


Output::

    Anagrams


Challenge: Palindrome test
^^^^^^^^^^^^^^^^^^^^^^^^^^

> Check if the entered string is palindromic.

A palindrome is a string that can be read from both ends: left to right or right
to left.
::

  load system io.

  let str = "Was it a rat I saw?".

  function clean with str:%string do
      return str @tolower()
                 @replace("[^a-z]","").
  end

  -- only keep lower case letters
  let clean_str = clean(str).

  -- check if it is palidromic
  if clean_str == clean_str @flip() do
      io @println "Palindromic".
  else do
      io @println "Not palindromic".
  end

  assert (clean_str == clean_str @flip()).


Output::

    Palindromic


Challenge: The longest palindrome
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Find the longest palindromic substring in the given string.

The main idea behind the solution is to scan the string with a window of
varying width. In other words, starting from a given character, test all the
substrings of any length possible at that position.
Now, extract the substring and do the check similar to the solution of Task
16, Palindrome test. Here, we have to be careful to check the palindrome
without taking into account the non-letter characters, but saving the result as
part of the original string.
::

  load system io.

  let str = "Hello, World!".

  function clean with str:%string do
      return str @tolower()
                 @replace("[^a-z]","").
  end

  function palindrome_test with str:%string do
      let clean_str = clean(str).
      if clean_str == clean_str @flip() do
          return true.
      else do
          return false.
      end
  end

  -- create the moving window over the string
  let longest_palindrome = "".

  for i in 0 to str @length()-2 do
      for j in i+1 to str @length()-1 do
          let str1 = str @[i to j].
          if palindrome_test(str1) and
             str1 @length() > longest_palindrome @length() do
              let longest_palindrome = str1.
          end
      end
  end

  io @println longest_palindrome.


Output::

    o, Wo


Challenge: Finding duplicate texts
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Find duplicate fragments in the same text.

We do this by finding and hashing N-grams after the appropriate preprocessing.  We will use ``N=3``.
::

  load system io.
  load system hash.

  -- text from "www.lipsum.com"

  let str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed
  malesuada sapien nec neque suscipit, non rutrum arcu scelerisque. Nam feugiat
  sapien porta ipsum accumsan, eget maximus diam volutpat. Pellentesque elementum
  in orci quis pretium. Donec dignissim nunc lectus, id ornare urna varius ut.
  Praesent semper faucibus vehicula. Aliquam luctus sapien at lorem malesuada,
  eget suscipit felis facilisis. Suspendisse velit lectus, mollis sit amet tempor
  eget, faucibus ut nulla. Vestibulum et elementum dolor, a vehicula ipsum. Morbi
  ut fringilla nisi. Fusce congue rutrum orci nec porta. Ut laoreet justo vel
  turpis sodales vehicula. Nulla porttitor nisl id odio eleifend sodales.

  Suspendisse blandit tristique enim id laoreet. Etiam vel aliquet dui, quis
  tempus magna. Donec blandit volutpat felis egestas tincidunt. Integer placerat
  luctus mi non pharetra. Donec aliquet nisl orci, egestas elementum nunc bibendum
  a. Morbi nec risus aliquet, viverra nunc in, molestie odio. Curabitur
  pellentesque, ante eget dictum aliquam, felis leo bibendum libero, vel bibendum
  lorem velit eget ex. Lorem ipsum dolor sit amet, consectetur adipiscing elit.
  Vestibulum pretium tellus quis ante vulputate, pretium tincidunt ipsum dapibus.
  Praesent congue, ipsum ut sagittis tempus, lacus nisi dapibus dui, aliquam porta
  metus odio ut neque. Aliquam vitae faucibus dolor. Nulla iaculis lorem non
  mauris viverra, ut malesuada nibh aliquam. Nam bibendum sit amet massa in
  dignissim. Nam posuere nunc ante, at viverra diam rhoncus vel.

  Aliquam mollis sagittis nulla. Maecenas faucibus eu dui eget accumsan.
  Suspendisse sit amet fermentum sapien. Nunc vitae mi nibh. Mauris condimentum
  vestibulum imperdiet. Quisque at vehicula dui. Integer sit amet volutpat arcu.
  Maecenas efficitur leo tortor, non ullamcorper magna tempor non. Sed efficitur
  quis metus ut pulvinar. Proin nunc felis, congue sit amet nibh placerat,
  tincidunt mattis nunc. Duis efficitur lacus a orci porttitor, sed molestie risus
  tempor.

  Sed tincidunt ipsum at urna sollicitudin feugiat. Ut mollis orci quis massa
  dictum facilisis. Maecenas non elementum mauris. Sed rutrum orci faucibus,
  tristique nunc nec, mattis ante. Pellentesque habitant morbi tristique senectus
  et netus et malesuada fames ac turpis egestas. In hac habitasse platea dictumst.
  Morbi pellentesque dolor sit amet nunc tincidunt, ut rutrum ante vulputate.
  Nullam pretium, mi sed condimentum luctus, ipsum nunc dictum lorem, vel
  ultricies nibh mi ut sem. Nam volutpat id libero eget mollis.

  Vestibulum eget velit eros. Phasellus sit amet vestibulum odio, vel malesuada
  quam. Mauris dictum erat eu ligula mollis laoreet. Phasellus ut ante auctor,
  hendrerit ipsum et, fermentum magna. Etiam nec eros elementum, consectetur nibh
  ac, ullamcorper ligula. Aliquam sed porttitor sapien. Nulla tincidunt, turpis
  vitae venenatis aliquet, quam purus elementum diam, in tincidunt orci diam sed
  nulla. Cras pellentesque non diam quis sollicitudin. Duis suscipit lectus dui,
  eu varius metus pretium sit amet.

  Nulla eu ex velit. Ut non justo semper, gravida erat quis, vehicula est.
  Suspendisse nunc dui, iaculis id purus sit amet, rutrum commodo lacus. Aenean
  consequat turpis a est vestibulum, ac accumsan nibh dapibus. Nam blandit
  scelerisque lectus, eu pellentesque arcu ornare non. Fusce ac gravida diam. Ut
  in fringilla eros. Sed metus augue, porta quis vehicula at, pellentesque et
  mauris. Duis sodales lacus sit amet condimentum placerat. In blandit tristique
  nulla eget malesuada. Sed congue finibus neque at semper. Etiam pellentesque
  egestas urna, ut lobortis odio euismod et. Phasellus aliquet quam purus, quis
  ullamcorper sem mollis eu.

  Mauris quis ullamcorper nisi. Aenean quam nulla, sodales eu faucibus in, mattis
  a nulla. Nullam pulvinar pretium justo eu mattis. Aliquam rutrum ipsum vitae leo
  maximus ultrices. Donec ut pulvinar nisi. Sed pharetra, turpis dictum lobortis
  egestas, quam massa venenatis enim, dapibus efficitur dolor mauris eu felis.
  Donec vulputate ultrices justo sit amet condimentum. Donec id posuere nulla. In
  vestibulum mi in lectus commodo dignissim. Quisque vestibulum egestas arcu sit
  amet finibus. Proin commodo aliquet neque quis maximus.

  Nulla facilisi. Sed gravida aliquet diam in congue. Mauris vehicula justo ac
  sollicitudin laoreet. Mauris enim mi, auctor id magna eget, feugiat sollicitudin
  leo. Vivamus ornare ornare commodo. Suspendisse ut dui quis enim porta pretium.
  Praesent vitae lacus fermentum, posuere orci ac, imperdiet massa. Nulla
  hendrerit id nisl sed maximus. Vivamus commodo lacus eu condimentum bibendum.
  Suspendisse porttitor sem eget dolor aliquet congue. Pellentesque tristique
  augue at quam hendrerit dignissim. Aenean a congue dui. Vestibulum ante ipsum
  primis in faucibus orci luctus et ultrices posuere cubilia curae; Integer ante
  lacus, commodo et enim sed, auctor egestas metus.

  Aliquam a urna id risus tincidunt rutrum. Nunc facilisis, tortor ac suscipit
  aliquam, ante neque tincidunt mi, nec ullamcorper lectus ligula vel urna.
  Suspendisse lobortis at felis sit amet facilisis. Pellentesque velit lacus,
  porttitor vitae eros rutrum, convallis blandit erat. Pellentesque nec mi
  viverra, volutpat dui in, rutrum lacus. Ut non venenatis leo. Praesent
  sollicitudin magna porttitor lorem elementum molestie non a turpis. Suspendisse
  potenti.

  Donec malesuada iaculis laoreet. Nunc ut volutpat ante, ut consequat tortor.
  Phasellus posuere, ipsum quis dignissim iaculis, nisl felis ullamcorper ligula,
  quis placerat sem sapien nec ante. Cras suscipit ut magna nec lacinia. Donec
  ipsum nibh, imperdiet non aliquam eu, maximus id ante. Pellentesque vitae felis
  felis. Aliquam et diam sed nulla volutpat vestibulum molestie non lacus.
  Praesent porta et lacus auctor fermentum. In hac habitasse platea dictumst.
  Aliquam erat volutpat. Etiam at ligula orci. Class aptent taciti sociosqu ad
  litora torquent per conubia nostra, per inceptos himenaeos."

  let word_list = str @tolower()
                      @replace("[^a-z0-9_]"," ")
                      @split().
  let ht = hash @hash().

  -- create N-grams
  for i in 0 to word_list @length()-3 do
      -- Note: make this code more general
      let n_gram = [word_list@i, word_list@(i+1), word_list @(i+2)] @join(" ").
      -- put the N-gram into a hash table, the value is the count of the N-gram in the text.
      if not ht @get(n_gram) do
          ht @insert(n_gram,1).
      else do
          ht @insert(n_gram,ht @get(n_gram)+1).
      end
  end

  for ((n_gram,cnt) if cnt > 1) in ht @aslist() do
      io @println (n_gram+": "+cnt).
  end


Output::

    lorem ipsum dolor: 2
    ipsum dolor sit: 2
    dolor sit amet: 3
    sit amet consectetur: 2
    amet consectetur adipiscing: 2
    consectetur adipiscing elit: 2
    in hac habitasse: 2
    hac habitasse platea: 2
    habitasse platea dictumst: 2
    aliquet quam purus: 2
    diam sed nulla: 2
    sit amet condimentum: 2



Section: Using Numbers
----------------------

Challenge: Pi
^^^^^^^^^^^^^

> Print the value of pi.
::

  load system io.
  load system math. -- definition of pi

  io @println (math @pi).


Output::

    3.141592653589793

Other constants are also available.
::

  load system io.
  load system math.

  io @println (math @e).
  io @println (math @tau). -- tau=2*pi

  assert (math @tau == 2 * math @pi)


Output::

    2.718281828459045
    6.283185307179586


Challenge: Factorial!
^^^^^^^^^^^^^^^^^^^^^

> Print the factorial of a given number.

By definition, the factorial of a positive integer number N is a product of all the integers numbering from 1 to N, including N. Our first solution is based on the direct implementation of the definition above using the list ``reduce`` function.
::

  load system io.

  let n = 3.
  let fact = [1 to n] @reduce(lambda with (a,b) do return a*b).
  io @println fact.
  assert (fact == 6).


Output::

    6


Our second solution uses the recursive definition of factorial,
::

         | 1       if  x = 0,
    x! = | x(x-1)! if  x > 0,
         | undef   if  x < 0,

where :math:`x \in Int`.
Here, each case specifies what value the function should return if
the predicate applied to the input is true.  The last case is of some interest because it states that the function is undefined for negative integers.
::

  load system io.

  let POS_INT = pattern with (x:%integer) if x > 0.
  let NEG_INT = pattern with (x:%integer) if x < 0.

  function fact
      with 0 do
          return 1
      with n:*POS_INT do
          return n * fact (n-1).
      with n:*NEG_INT do
          throw Error("factorial is not defined for "+n).
      end

  io @println ("The factorial of 3 is: " + fact (3)).
  assert (fact(3) == 6).


Output::

    The factorial of 3 is: 6


Challenge: Fibonacci numbers
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Print the Nth Fibonacci number.

Fibonacci numbers are defined by the recurring formula:
::

    f_n = f_{n-1} + f_{n-2}

You can assign two values at a time (**Challenge: Swap two values**). You can use that technique for calculating the next Fibonacci number from the previous two. To bootstrap the algorithm, the two first values are needed. In one of the definitions of the Fibonacci row, the first two values are both 1.

Here we give an iterative solutions.  It is clear that there exists a trivial recursive solution by implementing the above formula.
::

  load system io.

  let n = 10. -- compute the 10th Fib number

  let (f_1,f_2) = (1,1).
  for i in 3 to n do
      let (f_1,f_2) = (f_1+f_2,f_1).
  end

  io @println f_1.
  assert (f_1 == 55)


Output::

    55


Challenge: Print squares
^^^^^^^^^^^^^^^^^^^^^^^^

> Print the squares of the numbers 1 through 10.

Of course this is straightforward, with a ``for-loop`` over a list.  Here we show another solution using the list ``map`` function.
::

  load system io.

  let sq = [1 to 10] @map(lambda with x do return x*x).

  io @println sq.

  assert (sq == [1,4,9,16,25,36,49,64,81,100])


Output::

    [1,4,9,16,25,36,49,64,81,100]


Challenge: Powers of two
^^^^^^^^^^^^^^^^^^^^^^^^

> Print the first ten powers of two.

Just as in the previous challenge, we skip the naive loop solution and give a solution using the ``map`` function.
::

  load system io.
  load system math.

  let p2 = [0 to 9] @map(lambda with x do return math @pow(2,x)).

  io @println p2.

  assert (p2 == [1,2,4,8,16,32,64,128,256,512])


Output::

    [1,2,4,8,16,32,64,128,256,512]


Challenge: Odd and even numbers
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Print the first ten odd numbers. Print the first ten even numbers.

We start with printing the first ten odd numbers,
::

  load system io.
  load system math.

  let odd = []
  for (n if math @mod(n,2) =/= 0) in 1 to 10 do
      let odd = odd + [n].
  end

  io @println odd.
  assert(odd == [1,3,5,7,9])


Output::

    [1,3,5,7,9]


Now the even numbers,
::

  load system io.
  load system math.

  let even = []
  for (n if math @mod(n,2) == 0) in 1 to 10 do
      let even = even + [n].
  end

  io @println even.
  assert(even == [2,4,6,8,10])


Output:
::

    [2,4,6,8,10]


Challenge: Compare numbers approximately
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Compare the two non-integer values approximately.

Comparing non-integer numbers (which are represented as floating-point numbers) is often a task that requires approximate comparison.  In Asteroid this can be accomplished with the ``isclose`` function availabel in the ``math`` module.
::

  load system io.
  load system math.

  -- not equal under the default tolerance of 1E-09
  assert (not math @isclose(2.0,2.00001)).

  -- equal under the user defined tolerance of 0.0001
  assert (math @isclose(2.0,2.00001,0.0001)).



Challenge: Prime numbers
^^^^^^^^^^^^^^^^^^^^^^^^

> Decide if the given number is a prime number.

Prime numbers are those that can be divided only by 1, and by themselves.
::

  load system io.
  load system math.

  function isprime with x do
      if x >= 2 do
          for y in range(2,x) do
              if not math @mod(x,y) do
                  return false.
              end
          end
      else do
          return false.
      end
      return true.
  end

  io @println (isprime 17).
  io @println (isprime 15).

  assert (isprime(17)).
  assert (not isprime(15)).


Output:
::

    true
    false


Challenge: List of prime numbers
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Print the list of the first ten prime numbers.
::

  load system io.
  load system math.

  function isprime with x do
      if x >= 2 do
          for y in range(2,x) do
              if not math @mod(x,y) do
                  return false.
              end
          end
      else do
          return false.
      end
      return true.
  end

  let cnt = 0.
  for (n if isprime(n)) in 1 to 1000000 do
      io @println n.
      let cnt = cnt+1.
      if cnt == 10 do
          break.
      end
  end


Output:
::

    2
    3
    5
    7
    11
    13
    17
    19
    23
    29


Challenge: Prime factors
^^^^^^^^^^^^^^^^^^^^^^^^

> Find the prime factors of a given number.

Prime factors are the prime numbers that divide the given integer number exactly.
::

  load system io.
  load system math.

  function isprime with x do
      if x >= 2 do
          for y in range(2,x) do
              if not math @mod(x,y) do
                  return false.
              end
          end
      else do
          return false.
      end
      return true.
  end

  function primes with x do
      let lp = [].
      for (n if isprime(n)) in 1 to x do
          let lp = lp+[n].
      end
      return lp.
  end

  let n = 165.
  let factors = [].
  let primes_list =  primes(n).
  let ix = 0.

  while n > 1 do
      let factor = primes_list @ix.
      let ix = ix+1.
      if not math @mod(n,factor) do
          let ix = 0.
          let n = n/factor.
          let factors = factors+[factor].
      end
  end
  io @println factors.

  assert (factors == [3,5,11])


Output:
::

    [3,5,11]


Challenge: Reducing a fraction
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Compose a fraction from the two given integers — numerator and denominator — and reduce it to lowest terms.

5/15 and 16/280 are examples of fractions that can be reduced. The final results of this task are 1/3 and 2/35. Generally, the algorithm of reducing a fraction requires searching for the greatest common divisor, and then dividing both numerator and denominator by that number.  For our solution we use the function ``gcd`` available in the ``math`` module.
::

  load system io.
  load system math.

  -- fraction a/b
  let a = 16.
  let b = 280.

  -- reduce fraction
  let gcd_val = math @gcd(a,b).
  let numerator = a/gcd_val.
  let denominator = b/gcd_val.
  io @println numerator.
  io @println denominator.

  -- show that original and reduced fraction are the same value
  assert (a/b == numerator/denominator).


Output:
::

    2
    35


Challenge: Divide by zero
^^^^^^^^^^^^^^^^^^^^^^^^^

> Do something with the division by zero.

Asteroid is an eager language, that is, expressions are evaluated as early as possible.  We can trap division-by-zero errors using a try-catch block.
::

  load system io.

  try
      io @println (42/0).
  catch Exception(_,m) do
      io @println m.
  end
  io @println "We are still alive...".


Output:
::

    integer division or modulo by zero
    We are still alive...


Section: Random Numbers
-----------------------

Challenge: Generating random numbers
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Generate a random number between 0 and N.

Asteroid has two random number generation functions: ``random()`` generates a random real value in the interval $[0.0,1.0)$ and ``randint(a,b)`` that generates a random value in the interval $[a,b]$.  The type of the random value generated depends on the type of the values a and b specifying the interval.
::

  load system io.
  load system random.
  load system util.
  load system type.

  let randint = random @randint.

  random @seed(42).

  io @println (random @random()).          -- random value in [0.0,1.0)
  io @println (randint(0.0,1.0)).  -- random value in [0.0,1.0]
  io @println (randint(0,1)).      -- always 0 or 1

  -- generating a random number in the appropriate interval
  let n = 10.
  io @println (randint(0.0,type @toreal(n))).
  io @println (randint(0,n)).


Output:
::

    0.6394267984578837
    0.025010755222666936
    1
    2.4489185380347624
    2


Challenge: Neumann’s random generator
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Implement Von Neumann’s random number generator (also known as Middle-square method).

This algorithm is a simple method of generating short sequences of four-digit random integers. The method has its drawbacks, but for us, it is an interesting algorithmic task. The recipe has these steps:

1. Take a number between 0 and 9999.
2. Calculate the square of it.
3. If necessary, add leading zeros to make the number 8-digit.
4. Take the middle four digits.
5. Repeat from step 2.

To illustrate it with an example, let’s take the number 1234 as the seed. On step 2, it becomes 1522756; after step 3, 01522756. Finally, step 4 extracts the number 5227.
::

  load system io.
  load system util.
  load system type.

  let n = 1234.
  let sq = n*n.
  let sq_str = type @tostring(sq).
  if sq_str @length() < 8 do
      let prefix = [1 to 8-sq_str@length()] @map(lambda with _ do return "0")
                                            @join("").
      let sq_str = prefix + sq_str.
  end
  let rstr = sq_str @[2 to 5].
  let rval = type @tointeger(rstr).
  io @println rval.

  assert (rval == 5227)


Output:
::

    5227


Challenge: Histogram of random numbers
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Test the quality of the random generator by using a histogram to visualise the distribution.

The quality of the built-in generator of random numbers fully depends on the algorithm the developers of the compiler used. As a user, you cannot do much to change the existing generator, but you can always test if it delivers numbers uniformly distributed across the whole interval.

In our solution, we generate 10 random integers between 0 and 9. We then count how many times each of the integers have been generated.  If it is a decent random number generator, all numbers should have been generated roughly an equal number of times.
::

  load system io.
  load system random.

  let hist = [0 to 9] @map(lambda with _ do return 0).

  for _ in range(10000) do
      let ix = random @randint(0,9).
      let hist @ix = hist @ix +1
  end

  io @println hist.


Output:
::

    [944,1032,1015,968,981,986,1014,1058,989,1013]


Section: Mathematical Problems
------------------------------

Challenge: Distance between two points
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Calculate the distance between the two points on a surface.

There are two points on a surface, each with their own coordinates, x and y. The task is to find the distance between these two points.
A straightforward solution would be to use the Pythagorean theorem:
::

  load system io.
  load system math.

  let x = [10, 3].
  let y = [9, 1].
  let d = (math @sqrt(math @pow(x@0-y@0,2) + math @pow(x@1-y@1,2))).
  io @println d.

  assert (d == 2.23606797749979)


Output:
::

    2.23606797749979


Another approach is using the math identity,
::

    ||a|| = sqrt(a . a)


where ``.`` represents the dot product. In our case ``a`` would be the distance vector between points ``x`` and ``y``,
::

  load system io.
  load system math.
  load system vector.

  let x = [10, 3].
  let y = [9, 1].
  let a = vector @sub(x,y).
  let d = math @sqrt(vector @dot(a,a)).
  io @println d.

  assert (d == 2.23606797749979)


Output:
::

    2.23606797749979


The interesting part about the second approach is that it is completely dimension independent.  Note that except for the definition of the vectors $x$ and $y$ dimension never plays a part in the definition of the program.

Challenge: Standard deviation
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> For the given data, calculate the standard deviation value (sigma).

Standard deviation is a statistical term that shows how compact data distribution is. The formula is the following:

.. math::

    \sigma = \sqrt\frac{1}{n-1}\sum_i(\bar{x} - x_i)^2

where :math:`n` is the number of elements in the array :math:`x`; :math:`\bar{x}` is its average value (**Challenge: Average on an array**).
::

  load system io.
  load system math.

  let values = [727.7, 1086.5, 1091.0, 1361.3, 1490.5, 1956.1].

  let avg = values @reduce(lambda with (x,y) do return x+y) / values @length().
  let diff_sq = values @map(lambda with x do return math @pow(x-avg,2)).
  let numerator = diff_sq @reduce(lambda with (x,y) do return x+y).
  let denominator = values @length() -1.
  let sigma = math @sqrt(numerator/denominator).
  io @println sigma.

  assert (sigma == 420.96248961952256)


Output:
::

    420.96248961952256


Challenge: Polar coordinates
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Convert the Cartesian coordinates to polar and backward.

Polar coordinates are a convenient way of representing points on a surface with the two values: distance from the centre of coordinates, and the angle between the vector and the pole axis.
The conversion formulae between the Cartesian and polar systems, which is valid for **positive** ``x`` and ``y``, are the following:
::

    x = r cos(psi)
    y = r sin(psi)
    r = sqrt(x^2 + y^2)
    psi = arctan(x/y)

These expressions can be implemented as-is in the code:
::

  load system io.
  load system math.

  -- define common math functions locally so the
  -- formulas are easy to read
  let cos = math @cos.
  let sin = math @sin.
  let sqrt = math @sqrt.
  let pow = math @pow.
  let atan = math @atan.

  function polar_to_cartesian with (r,psi) do
      -- return a tuple: (x,y)
      return (r*cos(psi),r*sin(psi)).
  end

  function cartesian_to_polar with (x,y) do
      -- return a tuple: (r,psi)
      return (sqrt(pow(x,2)+pow(y,2)),atan(y/x)).
  end

  let (r,psi) = cartesian_to_polar(1,2).
  let (x,y) = polar_to_cartesian(r,psi).

  io @println (x,y).

  -- show that the recovered coordinates are the same
  -- we started with
  assert (math @isclose(1,x,0.0001) and math @isclose(2,y,0.0001)).


Output:
::

    (1.0000000000000002,2.0)


For the **negative** ``x`` and ``y``, the Cartesian-to-polar conversion is a bit more complicated. Depending on the quadrant of the point, the ``psi`` value is bigger
or smaller than ``pi``. When ``x`` is zero, it is either ``-pi/2`` or ``pi/2``.
All these variants can be implemented by using ``with`` clauses and conditional matching, as demonstrated below:
::

  load system io.
  load system math.
  load system util.
  load system type.

  -- define common math functions locally so the
  -- formulas are easy to read
  let cos = math @cos.
  let sin = math @sin.
  let sqrt = math @sqrt.
  let pow = math @pow.
  let atan = math @atan.
  let pi = math @pi.
  let toreal = type @toreal.

  function polar_to_cartesian with (r,psi) do
      -- return a tuple: (x,y)
      return (r*cos(psi),r*sin(psi)).
  end

  function cartesian_to_polar with (x,y) do
      return (sqrt(pow(x,2)+pow(y,2)),cartesian_to_psi(x,y)).
  end

  function cartesian_to_psi
      with (x,y) if x > 0  do
          return atan(toreal(y)/x).
      with (x,y) if x < 0 and y >= 0 do
          return atan(toreal(y)/x)+pi.
      with (x,y) if x < 0 and y < 0 do
          return atan(toreal(y)/x)-pi.
      with (x,y) if x == 0 and y > 0 do
          return pi/2.
      with (x,y) if x == 0 and y < 0 do
          return -pi/2.
      with (x,y) if x == 0 and y == 0 do
          return none.
      end

  let (r,psi) = cartesian_to_polar(-3,5).
  let (x,y) = polar_to_cartesian(r,psi).

  io @println (x,y).

  -- show that the recovered coordinates are the same
  -- we started with
  assert (math @isclose(-3,x,0.0001) and math @isclose(5,y,0.0001)).


Output:
::

    (-2.999999999999999,5.000000000000001)


Challenge: Monte Carlo method
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Calculate the area of a circle of radius 1 using the Monte Carlo method.

The Monte Carlo method is a statistical method of calculating data whose formula is not known. The idea is to generate a big number of random numbers and see how many of them satisfy the condition.

To calculate the area of a circle with a radius of 1, pairs of random numbers between −1 and 1 are generated. These pairs represent the points in the square in the center of coordinates with sides of length 2. The area of the square is thus 4. If the distance between the random point and the center of the square is less than 1, then this point is located inside the circle of that radius. Counting the number of points that landed inside the circle and the number of points outside the circle gives the approximate value of the area of the circle, as soon as the area of the square is known. Here is the program.
::

  load system io.
  load system math.
  load system random.

  let sqrt = math @sqrt.
  let pow = math @pow.
  let randint = random @randint.

  random @seed(42).

  let inside = 0.
  let n = 10000.
  for _ in 1 to n do
      let point = (randint(-1.0,1.0),randint(-1.0,1.0)).
      if sqrt(pow(point@0,2)+pow(point@1,2)) <= 1.0 do
          let inside = inside+1.
      end
  end
  let area = 4.0 * inside / n.
  io @println area.

  assert (area == 3.1392).


Output:
::

    3.1392


Challenge: Guess the number
^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Write a program that generates a random integer number between 0 and 10, asks the user to guess it, and says if the entered value is too small or too big.

First, a random number needs to be generated. Then the program must
ask for the initial guess and enter the loop, which compares the guess with the generated number.
::

  load system io.
  load system random.
  load system util.
  load system type.

  random @seed(42).

  let n = random @randint(0,10).
  let guess = type @tointeger(io @input("Guess my number between 0 and 10: ")).
  while guess =/= n do
      if guess < n do
          io @println "Too small.".
      elif guess > n  do
           io @println "Too big.".
      end
      let guess = type @tointeger(io @input("Try again: ")).
  end
  io @println "Yes, this is it!".


Challenge: Binary to integer
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Convert a binary number to a decimal integer.

In Asteroid this is straightforward using the built-in ``tointeger`` function, passing it a string representation of the binary number and the base.
::

  load system io.
  load system type.

  let bin = "101101".
  let int = type @tointeger(bin,2).
  io @println int.

  assert (int == 45).


Output:
::

    45


Challenge: Integer as binary, octal, and hex
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Print a given integer number in the binary, octal, and hexadecimal representations.

In Asteroid this is easily done with the ``tobase`` function.
::

  load system io.
  load system type.

  let tobase = type @tobase.
  let tointeger = type @tointeger.

  let val = 42.

  io @println (tobase(val,2)).  -- bin
  io @println (tobase(val,8)).  -- oct
  io @println (tobase(val,16)). -- hex

  -- make sure that conversions are correct in both directions
  assert (tointeger(tobase(val,2),2) == val).
  assert (tointeger(tobase(val,8),8) == val).
  assert (tointeger(tobase(val,16),16) == val).


Output:
::

    101010
    52
    2A


Challenge: Sum of digits
^^^^^^^^^^^^^^^^^^^^^^^^

> Calculate the sum of digits of a given number.

Pretty straightforward using string and list manipulation.
::

  load system io.
  load system type.

  let number = 139487854.


  let s = type @tostring number @explode()
                                @map(type @tointeger)
                                @reduce(lambda with (x,y) do return x+y).
  io @println s.

  assert (s == 49).


Output:
::

    49


Challenge: Bit counter
^^^^^^^^^^^^^^^^^^^^^^

> Count the number of bits set to 1 in a binary representation of a positive integer number.

If we remove all the zeros from a binary number, then we are left with only ``1`` characters which we can then count.
::

  load system io.

  let bits = "1010101" @replace("0","")
                       @length().
  io @println bits.

  assert (bits == 4).


Output:
::

    4


Challenge: Compose the largest number
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Given the list of integers, compose the largest possible number by concatenating them.

The easiest way to achieve that is to treat the numbers as strings, sort them alphabetically in descending order, concatenate the pieces to a single string, and get the resulting integer.
::

  load system io.
  load system type.

  let a = type @tointeger([67, 8, 1, 5, 45] @map(type @tostring) @sort(true) @join("")).
  io @println a.

  assert (a == 8675451).


Output:
::

    8675451


Challenge: Convert to Roman numerals
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Convert an integer number to a Roman numerals string.

Roman numbers are not a direct translation of the decimal system. In this task, we assume that the number is not more than 3999, which is the maximum a regular Roman number can reach.

Let’s use the algorithm that keeps the table of pre-calculated sequences of Roman letters. This is so that we don’t have to check when III becomes IV, or when another I appears after V, etc.

In the program below, there are four such sequences: for thousands, hundreds, tens, and ones. The program iterates over the digits of the number in the decimal representation and chooses one of the values from the array of lists stored in the ``roman_hash`` table.
::

  load system io.
  load system math.
  load system util.
  load system hash.
  load system type.

  let roman_hash = hash @hash().
  roman_hash @insert(1000,["","M","MM","MMM"]).
  roman_hash @insert(100,["","C","CC","CCC","CD","D","DC","DCC","DCCC","CM"]).
  roman_hash @insert(10,["","X","XX","XXX","XL","L","LX","LXX","LXXX","XC"]).
  roman_hash @insert(1,["","I","II","III","IV","V","VI","VII","VIII","IX"]).

  let n = 2018.
  let p10 = range(type @tostring n @length()) @map(lambda with x do return math @pow(10,x))
                                              @reverse().
  let digits = type @tostring n @explode()
                                @map(type @tointeger).
  let z = util @zip(digits, p10).
  io @println z.
  let roman = "".
  for (d,p) in z do
      let roman = roman + roman_hash @get(p) @d.
  end
  io @println roman.

  assert (roman == "MMXVIII")


Output:
::

    [(2,1000),(0,100),(1,10),(8,1)]
    MMXVIII


Challenge: Spelling numbers
^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Write an integer number below one million in words.

Human languages have many inconsistencies, especially in the most frequent constructs. Spelling numbers seems to be a simple task, but due to a number of small differences, the resulting program is quite big.

The program is listed on the next page. Let’s discuss the algorithm first.

Take a number; for example, 987,654. The rules for spelling out the groups of three digits, 987 and 654, are the same. For the first group, the word thousand must be added.

Now, examine a group of three digits. The first digit is the number of hundreds, and it has to be spelled only if it is not zero. If it is not zero, then we spell the digit and add the word hundred.

Now, remove the leftmost digit, and we’ve got two digits left. If the remaining two digits form the number from 1 to 20, then it can be directly converted to the corresponding name. The names for the numbers from 0 to 10 are obviously different. The names for the numbers from 11 to 19 have some commonalities, but is it still easier to directly prepare the names for all of them.

For the larger numbers (21 to 99), there are two cases. If the number is dividable by 10 then a name for 20, 30, 40, etc. is taken. If not, then the name is built of the name of tens and the name for units, joined with a hyphen, such as forty-five.

The zero name appears only in the case when the given number is zero.
::

  load system io.
  load system math.

  let mod = math @mod.

  let names = ["zero","one","two","three","four","five","six","seven","eight","nine",
               "ten","eleven","twelve","thirteen","fourteen","fifteen",
               "sixteen","seventeen","eighteen","nineteen","twenty","thirty",
               "forty","fifty","sixty","seventy","eighty","ninety"].

  function spell_number
      with (n:%integer) if n < 20 do
          return names @n.
      with (n:%integer) if n < 100 do
          let r = names @(n / 10 + 18).
          let r = r + ("-" + names @(mod(n,10))) if mod(n,10) else "".
          return r.
      with (n:%integer) if n < 1000 do
          return spell_part(n,100,"hundred").
      with (n:%integer) if n < 1000000 do
          return spell_part(n,1000,"thousand").
      end

  function spell_part
      with (n:%integer,base:%integer,name:%string) do
          let r = spell_number(n/base) + " " + name.
          return r + " " + spell_number(mod(n,base)) if mod(n,base) else r.
      end

  io @println (spell_number 15).
  io @println (spell_number 75).
  io @println (spell_number 987654).
  io @println (spell_number 1001).


Output:
::

    fifteen
    seventy-five
    nine hundred eighty-seven thousand six hundred fifty-four
    one thousand one


Section: Manipulating Lists and Arrays
--------------------------------------

Challenge: Swap two values
^^^^^^^^^^^^^^^^^^^^^^^^^^

> Swap the values of two variables.

In Asteroid, there is no need to use temporary variables to swap the values of two variables. Just use tuples on both sides of the equation:
::

    let (b,a) = (a,b).

Consider the complete program:
::

  load system io.

  let (a,b) = (10,20).
  let (b,a) = (a,b).
  io @println ("a = "+a,"b = "+b).

  assert ((a,b) is (20,10)).


Output:
::

    (a = 20,b = 10)


This program prints the swapped values:
::

    (a = 20,b = 10)

This approach also works with elements of an array:
::

  load system io.

  let a = [3,5,7,4].
  let (a@2,a@3) = (a@3,a@2).
  io @println a.

  assert (a is [3,5,4,7]).


Output:
::

    [3,5,4,7]


Challenge: Reverse a list
^^^^^^^^^^^^^^^^^^^^^^^^^

> Print the given list in reverse order.
::

  load system io.

  let a = [10, 20, 30, 40, 50].
  io @println (a @reverse()).

  assert(a == [50,40,30,20,10]).


Output:
::

    [50,40,30,20,10]


Challenge: Rotate a list
^^^^^^^^^^^^^^^^^^^^^^^^

> Move all elements of an array N positions to the left or to the right.

Asteroid does not have a built-in ``rotate`` function. However, such a function is easily constructed through slicing lists (see ``vix`` below).
::

  load system io.
  load system math.

  function rotate with (l:%list,i:%integer) do
      let n = l @length().
      let vix = range n @map(lambda with x do return math @mod(x+i,n)).
      return l @vix.
  end


  let a = [1, 3, 5, 7, 9, 11, 13, 15].
  let b = rotate(a,3).
  let c = rotate(a,-3).
  io @println a.
  io @println b.
  io @println c.

  assert(b == [7,9,11,13,15,1,3,5] and c == [11,13,15,1,3,5,7,9]).


Output:
::

    [1,3,5,7,9,11,13,15]
    [7,9,11,13,15,1,3,5]
    [11,13,15,1,3,5,7,9]


Challenge: Randomize an array
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Shuffle the elements of an array in random order.

This is easily accomplished with the built-in ``shuffle``.
::

  load system io.
  load system random.

  random @seed(42).
  let b = [1 to 20] @shuffle().
  io @println b.

  assert(b == [20,6,15,5,10,14,16,19,7,13,18,11,2,12,3,17,8,9,1,4]).


Output:
::

    [20,6,15,5,10,14,16,19,7,13,18,11,2,12,3,17,8,9,1,4]


Challenge: Incrementing array elements
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Increment each element in an array.

For this we use Asteroid's ``vector`` module, which can handle incrementing a vector with a scalar.
::

  load system io.
  load system vector.

  let a = [1 to 10].
  let b = vector @add(a,1).
  io @println b.

  assert(b == [2,3,4,5,6,7,8,9,10,11]).


Output:
::

    [2,3,4,5,6,7,8,9,10,11]


Challenge: Adding up two arrays
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Take two arrays and create a new one whose elements are the sums of the corresponding items of the initial arrays.

Again, here we take advantage of Asteroid's ``vector`` module.  Note that the two vectors have to be of the same length in order to add them together.
::

  load system io.
  load system vector.

  let a = [10 to 20].
  let b = [30 to 40].
  let c = vector @add(a,b).
  io @println c.

  assert(c == [40,42,44,46,48,50,52,54,56,58,60]).


Output:
::

    [40,42,44,46,48,50,52,54,56,58,60]


The vector module defines a function called ``op`` that allows you to combine two vectors using any arbitrary binary function.  Rewriting the above program using ``op``,
::

  load system io.
  load system vector.

  let a = [10 to 20].
  let b = [30 to 40].
  let c = vector @op((lambda with (x,y) do return x+y),a,b).
  io @println c.

  assert(c == [40,42,44,46,48,50,52,54,56,58,60]).


Output:
::

    [40,42,44,46,48,50,52,54,56,58,60]


As we said above, any arbitrary binary function. Consider the relational operator ``<`` expressed as a lambda function,
::

  load system io.
  load system vector.
  load system random.

  random @seed(42).

  let a = [1 to 10] @shuffle().
  let b = [1 to 10] @shuffle().
  let c = vector @op((lambda with (x,y) do return x<y),a,b).
  io @println c.

  assert(c == [false,true,false,false,false,true,false,false,true,true]).


Output:
::

    [false,true,false,false,false,true,false,false,true,true]


Challenge: Exclusion of two arrays
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> From the given two arrays, find the elements of the first array which do not
appear in the second one.

Here we use Asteroid's ``set`` module.
::

  load system io.
  load system set.

  let a = [1 to 10].
  let b = [5 to 15].
  let c = set @diff(a,b).
  io @println c.

  assert(c @sort() == [1,2,3,4]).


Output:
::

    [2,3,1,4]


Section: Information Retrieval
------------------------------

Challenge: Sum of the elements of an array
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Find the sum of the elements of an array of integers.
::

  load system io.

  let a = [4, 6, 8, 1, 0, 58, 1, 34, 7, 4, 2].
  let s = a @reduce(lambda with (x,y) do return x+y).
  io @println s.

  assert (s == 125).


Output:
::

    125


If summing up elements that are greater than 10,
::

  load system io.

  let a = [4, 6, 8, 1, 0, 58, 1, 34, 7, 4, 2].
  let f = (lambda with (x,y) do return x+(y if y > 10 else 0)).
  let s = a @reduce(f,0).
  io @println s.

  assert (s == 92).


Output:
::

    92


Challenge: Average of an array
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Find the average value of the given array of numbers.
::

  load system io.

  let a = [7, 11, 34, 50, 200].
  let avg = a @reduce(lambda with (x,y) do return x+y)/a @length().
  io @println avg.

  assert (avg == 60).


Output:
::

    60


Challenge: Is an element in a list?
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Tell if the given value is in the list.
::

  load system io.

  let array = [10, 14, 0, 15, 17, 20, 30, 35].
  let x = 17.
  io @println ((x+" is in the list") if array @member(x) else (x+" is not in the list")).


Output:
::

    17 is in the list


We can also use a reduction function to solve this,
::

  load system io.

  let array = [10, 14, 0, 15, 17, 20, 30, 35].
  let x = 17.

  if array @reduce(lambda with (acc,i) do return true if i==x else acc,false) do
      io @println (x+" is in the list").
  else
      io @println (x+" is not in the list").
  end


Output:
::

    17 is in the list


Challenge: First odd number
^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Find the first odd number in a list of integers.

The easiest way to do this is with a reduction,
::

  load system io.
  load system math.
  load system util.
  load system type.

  let mod = math @mod.

  let array = [2, 4, 18, 9, 16, 7, 10].
  let odd = array @reduce(lambda with (acc,i) do return i if type @isnone(acc) and mod(i,2) else acc,none).
  io @println odd.


Output:
::

    9


Challenge: Take every second element
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Form a new array by picking every second element from the original array.
::

  load system io.
  load system math.

  let array = [20 to 30] @filter(lambda with x do return math @mod(x,2)).
  io @println array.

  assert (array == [21,23,25,27,29]).


Output:
::

    [21,23,25,27,29]


We can use an index vector to accomplish  the same thing,
::

  load system io.
  load system math.

  let a = [20 to 30].
  let array = a @[1 to a @length()-1 step 2] .
  io @println array.

  assert (array == [21,23,25,27,29]).


Output:
::

    [21,23,25,27,29]


Challenge: Number of occurrences in an array
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Count how many times a particular element appears in the array.
::

  load system io.
  load system math.

  let dt = ["apple",
            "pear",
            "grape",
            "lemon",
            "peach",
            "apple",
            "banana",
            "grape",
            "pineapple",
            "avocado"].
            
  let cnt = dt @count("grape").
  io @println cnt.

  assert (cnt == 2).


Output:
::

    2


Challenge: Finding unique elements
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Print all unique elements of the given array.

Converting a list to a set will remove all duplicate elements in the list.
::

  load system io.
  load system set.

  function unique with lst:%list do
      return set @toset lst @sort().
  end

  let a = unique([2, 3, 7, 4, 5, 5, 6, 2, 10, 7]).

  io @println a.

  assert (a == [2,3,4,5,6,7,10])

Output:
::

    [2,3,4,5,6,7,10]


Challenge: Minimum and maximum
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Find the minimum and the maximum numbers in the given list of integers.
::

  load system io.

  function max with lst:%list do
      return lst @sort(true) @0.
  end

  function min with lst:%list do
      return lst @sort() @0.
  end

  let v = [7, 6, 12, 3, 4, 10, 2, 5, 15, 6, 7, 8, 9, 3].
  
  let a = max v.
  let b = min v.

  io @println a.
  io @println b.

  assert (a == 15 and b == 2).


Output:
::

    15
    2


Challenge: Increasing sequences
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Check if the given array contains increasing (or decreasing) numbers.
::

  load system io.
  load system type.

  let a = [3, 7, 19, 20, 34].
  let b = type @toboolean(a @reduce(lambda with (x,y) do return y if x<y else false)).

  io @println b.

  assert (b).


Output:
::

    true

Section: Multi-Dimensional Data
-------------------------------

Challenge: Transpose a matrix
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Take a matrix and print its transposed version.

In Asteroid a matrix can be represented by nested lists, like so,
::

    let m = [[1,2],
             [3,4]].

The transpose of this matrix is,
::

    let m = [[1,3],
             [2,4]].

In a square matrix computing the transpose is just a matter of swapping around the elements.  However, here we will solve the more general problem for non-square matrices,
::

    let m = [[1,2],
             [3,4],
             [5,6]].

with its transpose,
::

    let m = [[1,3,5],
             [2,4,6]].

The procedure:
::

  load system io.

  function transpose with m do
      -- figure out the dimensions
      let xdim = m @0 @length().
      let ydim = m @length().

      -- reserve space for the transpose
      -- first we do the ydim of new matrix
      let mt = range(xdim).
      for y in mt do
          let mt @y = range(ydim).
      end

      -- swap the elements
      for x in range(xdim) do
          for y in range(ydim) do
              let mt @x @y = m @y @x.
          end
      end

      return mt.
  end

  function print_matrix with m do
      io @println "".
      for r in m do
          for e in r do
              io @print (e + " ").
          end
          io @println ("").
      end
      io @println "".
  end

  let m = [[1,2],
           [3,4]].

  let mt = transpose(m).

  io @println ("The transpose of:").
  print_matrix m.
  io @println ("is:").
  print_matrix mt.
  io @println ("").

  let m = [[1,2],
           [3,4],
           [5,6]].

  let mt = transpose(m).

  io @println ("The transpose of:").
  print_matrix m.
  io @println ("is:").
  print_matrix mt.
  io @println ("").

  assert(mt == [[1,3,5],[2,4,6]]).


Output:
::

    The transpose of:

    1 2
    3 4

    is:

    1 3
    2 4


    The transpose of:

    1 2
    3 4
    5 6

    is:

    1 3 5
    2 4 6


Challenge: Sort hashes by parameter
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> Sort a list of hashes using data in their values.

This task is commonly performed to sort items where the sortable parameter is one of the values in the hash. For example, sorting a list of people by age.
::

  load system io.
  load system hash.
  load system sort.
  load system random.

  let randint = random @randint.

  random @seed(42).

  -- hash of names with ages
  let ht = hash @hash().
  ht @insert("Billie",randint(20,50)).
  ht @insert("Joe",randint(20,50)).
  ht @insert("Pete",randint(20,50)).
  ht @insert("Brandi",randint(20,50)).

  -- export the hash as a list of pairs
  let lst = ht @aslist().

  -- define our order predicate on a
  -- list of pairs where the second
  -- component holds the order info
  function pairs with ((_,x),(_,y)) do
      return true if x < y else false.
  end

  -- print out the sorted list
  io @println (sort @sort(pairs,lst)).

  assert (sort @sort(pairs,lst) == [("Pete",20),("Joe",23),("Billie",40),("Brandi",43)])


Output:
::

    [(Pete,20),(Joe,23),(Billie,40),(Brandi,43)]


Challenge: Count hash values
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

> For a given hash, count the number of occurrences of each of its values.

For example, a hash is a collection mapping a car’s license plate to the colour of the car or a passport number to the name of the street where the person lives. In the first example, the task is to count how many cars of each colour there are. In the second example, we have to say how many people live on each street. But let’s simply count the colours of fruit.
::

  load system io.
  load system hash.
  load system sort.

  let fruit_hash = hash @hash().
  fruit_hash @insert("apple","red").
  fruit_hash @insert("avocado","green").
  fruit_hash @insert("banana","yellow").
  fruit_hash @insert("grapefruit","orange").
  fruit_hash @insert("grapes","green").
  fruit_hash @insert("kiwi","green").
  fruit_hash @insert("lemon","yellow").
  fruit_hash @insert("orange","orange").
  fruit_hash @insert("pear","green").
  fruit_hash @insert("plum","purple").

  let fruit_lst = fruit_hash @aslist().

  let color_hash = hash @hash().
  for (_,color) in fruit_lst do
      if not color_hash @get(color) do
          color_hash @insert(color,1).
      else
          color_hash @insert(color, color_hash @get(color) +1).
      end
  end
  let color_lst = color_hash @aslist().

  function pairs with ((_,x),(_,y)) do
      return true if x < y else false.
  end

  io @println (sort @sort(pairs,color_lst)).


Output:
::

    [(red,1),(purple,1),(yellow,2),(orange,2),(green,4)]


Challenge: Product table
^^^^^^^^^^^^^^^^^^^^^^^^

> Generate and print the product table for the values from 1 to 10.

We will do this with an outer loop  and a ``map`` function.
::

  load system io.
  load system type.

  function format with v do
      let maxlen = 3.
      let vstr = type @tostring v.
      return [1 to maxlen-len(vstr)] @map(lambda with _ do return " ") @join("") + vstr.
  end

  for i in 1 to 10 do
      io @println ([1 to 10] @map(lambda with x do return format(i*x)) @join(" ")).
  end


Output:
::

      1   2   3   4   5   6   7   8   9  10
      2   4   6   8  10  12  14  16  18  20
      3   6   9  12  15  18  21  24  27  30
      4   8  12  16  20  24  28  32  36  40
      5  10  15  20  25  30  35  40  45  50
      6  12  18  24  30  36  42  48  54  60
      7  14  21  28  35  42  49  56  63  70
      8  16  24  32  40  48  56  64  72  80
      9  18  27  36  45  54  63  72  81  90
     10  20  30  40  50  60  70  80  90 100


Challenge: Pascal triangle
^^^^^^^^^^^^^^^^^^^^^^^^^^

> Generate the numbers of the Pascal triangle and print them.

The Pascal triangle is a sequence of rows of integers. It starts with a single 1 on the top row, and each following row has one number more, starting and ending with 1, while all of the other items are the sums of the two elements above it in the previous row. It is quite obvious from the illustration:
::

           1
          1 1
         1 2 1
        1 3 3 1
       1 4 6 4 1
     1 5 10 10 5 1
    1 6 15 20 15 6 1

To calculate the values of the next row, you may want to iterate over the values of the current row and make the sums with the numbers next to it. Let us use the functional style that the language offers.
Consider the fourth row, for example: 1 3 3 1. To make the fifth row, you can shift all the values by one position to the right and add them up to the current row:
::

      13310
    + 01331
    -------
    14641

We can easily accomplish this with our ``vector`` module. Given the vector of the fourth row,
::

    [1,3,3,1]

we create two new vectors,
::

    [1,3,3,1,0]

and
::

    [0,1,3,3,1]

We then add them together,
::

    vector @add([1,3,3,1,0],[0,1,3,3,1]) = [1,4,6,4,1]

The only thing that is left to do is to iterate appropiately and format the output.
::

  load system io.
  load system vector.
  load system util.
  load system type.

  let triangle = [[1]].
  let ix = 0.

  for i in 1 to 6 do
      let v = triangle @ix.
      let v1 = [0] + v.
      let v2 = v + [0].
      let new_v = vector @add(v1,v2).
      let triangle = triangle + [new_v].
      let ix = ix + 1.
  end

  for r in triangle do
      io @println (r @map(lambda with v do return type @tostring v) @join(" ")).
  end


Output:
::

    1
    1 1
    1 2 1
    1 3 3 1
    1 4 6 4 1
    1 5 10 10 5 1
    1 6 15 20 15 6 1


The program prints the first seven rows of the Pascal triangle. The rows are not centred, and are aligned to the left side.
As an extra exercise, modify the program so that it prints the triangle as it is shown at the beginning of this task. For example, you can first generate rows and keep them in a separate array and then, knowing the length of the longest string, add some spaces in front of the rows before printing them.

