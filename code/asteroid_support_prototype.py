# prototype module for detecting redundant patterns in the Asteroid programming language.

from asteroid_walk import unify, PatternMatchFailed
from asteroid_support import assert_match, head_tail_length

##############################################################################################
# This exception is used when a pattern has been identified as being 'useless',
# or reundant. This exception is formatted to pack both offending patterns
# information into a single formatted message to the user informing them of
# where and what caused this error.
##############################################################################################
class RedundantPatternFound(Exception):
    """Exception raised for detection of redundant pattern in function declaration.

    Attributes:
        pattern1 -- The pattern with the higher order of precedence (comparer).
        line1    -- The line number location of pattern 1 in its file.
        pattern2 -- The pattern with the lower order of precedence  (comparee).
        line2    -- The line number location of pattern 2 in its file.
        function -- The name of the function where the redundancy was found.
        file     -- The name of the file where the function is from. 
    """
    def __init__(self, pattern1, pattern2,function_name,location1,location2):
        self.pattern1 = pattern1
        self.line1 = str(location1[1] - 1) #patterns dont have line #, so we
                                            #compensate here by using the first line
                                            # of the patterns body, then moving back
                                            # 1 line(minus 1)
        self.pattern2 = pattern2
        self.line2 = str(location2[1] - 1)
        self.file = location1[0]
        self.function = function_name
        self.message = "Redundant Pattern Detected\n"
        self.message += "\tFunction: " + self.function.value + " from file " + self.file
        self.message += "\n\tPattern: " + term2string(self.pattern1) + " on line " + self.line1 
        self.message += "\n\twill consume all matches for" 
        self.message += "\n\tPattern: " + term2string(self.pattern2) + " on line " + self.line2
        super().__init__(self.message)    

###########################################################################################
# Evaluates the presence of redundant, or 'useless', pattern clauses in an Asteroid function:
#
# A redundant, or 'useless', pattern is defined as a pattern which can never be matched
# due to a preceeding pattern consuming all intended pattern matches.
#
# Consider the following Asteroid function:
#
# function testFunction
#   with (x,y) do
#       return 1.
#   orwith (x,1) do
#      return 2.
#   end function.
#
# In the above function, the pattern (x,1) can never be reached as the preceeding pattern (x,y)
# will consume all intended matches. Therefore, it is redundant.
#
# Function check_redundancy takes in a functions body list during parsing.
# This body list contains a functions patterns along with the associated bodies for each
# pattern. This function then evaluates if patterns exist within the passed in function that
# are redundant. If so, a warning is printed to the console identifing the offending
# pattern(s)
#       
################################################################################################
def check_redundancy( body_list, id_tok ):

    #Node type assertions
    #or "Make sure we are walking down the right part of the tree"
    (BODY_LIST, function_bodies ) = body_list
    assert_match(BODY_LIST,'body-list')
    (LIST, bodies) = function_bodies
    assert_match(LIST,'list')
    

    #compare every pattern with the patterns that follow it
    for i in range(len(bodies)):
        
        #get the pattern with the higher level of precedence 
        (BODY_H,(PTRN,ptrn_h),stmts_h) = bodies[i]
        assert_match(BODY_H,'body')
        assert_match(PTRN,'pattern')
        
        for j in range(i + 1, len(bodies)):
            
            #get the pattern with the lower level of precedence
            (BODY_L,(PTRN,ptrn_l),stmts_l) = bodies[j]
            assert_match(BODY_L,'body')
            assert_match(PTRN,'pattern')

            #DEBUGGING
            ###(pattern,code) = body
            #print("COMPARE: ")
            #print(ptrn_l)
            #print("TO: ")
            #print(ptrn_h)
            
            #Here we get line numbers in case we throw an error
            # we have to do a little 'tree walking' to get to the
            # line #, hence all the unpacking.
            (STMT_LIST,(LIST,LINE_LIST)) = stmts_l
            first_line_l = LINE_LIST[0]
            (LINE_INFO,location_l) = first_line_l

            (STMT_LIST,(LIST,LINE_LIST)) = stmts_h
            first_line_h = LINE_LIST[0]
            (LINE_INFO,location_h) = first_line_h

            #Compare the patterns to determine if the pattern with the
            #higher level of precedence will render the pattern with
            #the lower level of precedence useless/redundant by calling
            #on the unify function to evaluate the subsumption relationship
            #between the two patterns.
            try:
                unify( ptrn_l, ptrn_h , False )
            except PatternMatchFailed:
                pass
            else:
                raise RedundantPatternFound( ptrn_h , ptrn_l , id_tok, location_h, location_l )
##############################################################################################
# Modified version of term2string used for testing / formatted output of the redundancy check
# TODO I don't know if this will reworked. But I am keeping it in for formatting testing output.
def term2string(term):

    TYPE = term[0]

    if TYPE in ['id', 'integer', 'real', 'string']:
        val = term[1]
        return str(val)

    elif TYPE in ['boolean', 'none']:
        val = term[1]
        return str(val).lower()

    elif TYPE in ['list', 'tuple']:
        val = term[1]
        term_string = '[' if TYPE == 'list' else '('
        l = len(val)
        for i in range(l):
            term_string += term2string(val[i])
            if i != l-1:
                term_string += ','
        if l == 1 and TYPE == 'tuple': # proper 1-tuple notation
            term_string += ','
        term_string += ']' if TYPE == 'list' else ')'
        return term_string

    elif TYPE == 'object':
        (OBJECT,
         (CLASS_ID, (ID, class_id)),
         (OBJECT_MEMORY, (LIST, object_memory))) = term
        term_string = class_id + '('
        for ix in range(0, len(object_memory)):
            term_string += term2string(object_memory[ix])
            term_string += ', ' if ix != len(object_memory)-1 else ''
        term_string += ')'
        return term_string

    elif TYPE == 'function':
        # TODO: decide whether it makes sense to print out functions
        return '(function ...)'

    elif TYPE == 'apply-list':
        (LIST, apply_list) = term[1]
        term_string = term2string(apply_list[0])
        for ix in range(1, len(apply_list)):
            if apply_list[ix][0] not in ['tuple', 'apply_list']:
                term_string += '('
            term_string += term2string(apply_list[ix])
            if apply_list[ix][0] not in ['tuple', 'apply_list']:
                term_string += ')'
        return term_string

    elif TYPE == 'quote':
        val = term[1]
        return "'" + term2string(val)

    elif TYPE == 'nil':
        return ''
    
    ############################################# Newly added #########
    elif TYPE == 'head-tail':
        length = head_tail_length(term)
        term_string = "["
        for ix in range(1,length):
            
            #update output text with each entry in head-tail list
            term_string += term2string(term[1])

            #step down the head-tail tree
            term = term[2]
            
            #Insert head-tail entry delimiter
            term_string+= "|"
            
            #Catch the last entry
            if (ix == (length-1)):
                term_string += term2string(term)
                
        #Put the head-tail list delimiter on the end and then return
        term_string += "]"
        return term_string
    
    elif TYPE == 'named-pattern':
        
        (NAMED_PATTERN,ID,pattern) = term
        term_string = ID[1] + ':'
        
        return term_string + term2string(pattern)
    
    elif TYPE == 'typeclass':
        (TYPECLASS,cType) = term
        term_string = cType
        return term_string    
    
    elif TYPE == 'typematch':
        (TYPECLASS,cType) = term
        term_string = cType
        return term_string   
        
    else:
        print(term)
        raise ValueError(
            "unknown type '{}' in term2string"
            .format(TYPE))

