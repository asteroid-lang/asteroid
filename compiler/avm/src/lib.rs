/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Abstract Virtual Machine                                                   */
/*                                                                            */
/******************************************************************************/
#![allow(unused)]

use state::*;   //Asteroid state representation
use symtab::*;  //Asteroid symbol table
use ast::*;     //Asteroid AST representation
use support::*; //Asteroid support functions

use regex::Regex;

/******************************************************************************/
pub fn unify<'a>( term: &'a ASTNode, pattern: &'a ASTNode, state: &'a mut State, unifying: bool) -> Result<Vec<(ASTNode,ASTNode)>, (&'static str,String) >{

    //TODO
    let term_type = peek(term).unwrap();
    let pattern_type = peek(pattern).unwrap();

    if term_type == "string" { // Apply regular expression pattern match
        if pattern_type == "string" {
            // Note: a pattern needs to match the whole term.
            let ASTNode::ASTString(ASTString{id:t_id,value:t_value}) = term 
                else {panic!("Unify: expected string.")};
            let ASTNode::ASTString(ASTString{id:p_id,value:p_value}) = pattern 
                else {panic!("Unify: expected string.")};

            let mut re_str = String::from(r"^");
            re_str.push_str(p_value);
            re_str.push_str("$");
            let re = Regex::new(&re_str).unwrap();
            if re.is_match(t_value) {
                Ok( vec![] ) // Return an empty unifier
            } else {
                Err( ("PatternMatchFailed", format!("regular expression {} did not match {}",term2string(pattern).unwrap(),term2string(term).unwrap())))
            }
        } else {
            Err( ("PatternMatchFailed", format!("regular expression {} did not match {}",term2string(pattern).unwrap(),term2string(term).unwrap())))
        }

    } else if ["integer","real","boolean"].contains(&term_type) {
        if term == pattern {
            Ok( vec![] ) // Return an empty unifier
        } else {
            Err( ("PatternMatchFailed",format!("{} is not the same as {}",term2string(pattern).unwrap(),term2string(term).unwrap())))
        }
    } else if term_type == "list" || pattern_type == "list" {
        if term_type != "list" || pattern_type != "list" {
            Err( ("PatternMatchFailed",format!("term and pattern do not agree on list/tuple constructor")))
        } else {
            let ASTNode::ASTList(ASTList{id:_,length:tlen,contents:tcontents}) = term
                else {panic!("Unify: expected list.")};
            let ASTNode::ASTList(ASTList{id:_,length:plen,contents:pcontents}) = pattern
                else {panic!("Unify: expected list.")};
            if tlen != plen {
                Err( ("PatternMatchFailed",format!("term and pattern lists/tuples are not the same length")))
            } else {
                let mut unifiers = Vec::with_capacity(*tlen);
                for i in 0..*tlen {
                    unifiers.extend( unify( &tcontents[i], &pcontents[i], state, unifying ).unwrap());
                }
                Ok( unifiers )
            }
        }
    } else if !unifying && term_type == "namedpattern"{
        //Unpack a term-side name-pattern if evaluating redundant clauses
        let ASTNode::ASTNamedPattern(ASTNamedPattern{id,name,pattern:named_pattern}) = term
            else {panic!("Unify: expected named-pattern.")};

        unify(&named_pattern[0],pattern,state,unifying)
    } else if !unifying && term_type == "deref" {
        //Unpack a term-sdie first class pattern if evaluating redundant clauses
        let ASTNode::ASTDeref(ASTDeref{id,expression}) = term 
            else {panic!("Unify: expected derefernece.")};

        let ASTNode::ASTID(ASTID{id,ref name}) = expression[0]
            else {panic!("Unify: expected id.")};
        
        let new_term = state.lookup_sym(name,true).unwrap();
        
        //TODO
        //REEVALUATE- copy whole state is silly/find better solution
        unify(&new_term, pattern, &mut state.clone(), unifying)

    // ** Asteroid value level matching **
    } else if term_type == "object" && pattern_type == "object" {
        //   this can happen when we dereference a variable pointing
        //   to an object as a pattern, e.g.
        //    let o = A(1,2). -- A is a structure with 2 data members
        //   let *o = o.

        let ASTNode::ASTObject(ASTObject{id:t_id,struct_id:t_struct_id,object_memory:t_object_memory}) = term
            else {panic!("Unify: expected object.")};
        let ASTNode::ASTObject(ASTObject{id:p_id,struct_id:p_struct_id,object_memory:p_object_memory}) = pattern
            else {panic!("Unify: expected object.")};
        
        let ASTID{id:_,name:t_name} = t_struct_id;
        let ASTID{id:_,name:p_name} = p_struct_id;

        if t_name != p_name {
            Err( ("PatternMatchFailed",format!("pattern type {} and term type {} do not agree",p_name,t_name)))
        } else {
            unify( &ASTNode::ASTList(t_object_memory.clone()), &ASTNode::ASTList(p_object_memory.clone()), state, unifying )
        }
    } else if pattern_type == "string" && term_type != "string" {
        // regular expression applied to a non-string structure
        // this is possible because all data types are subtypes of string
        let str_term = term2string(term).unwrap();
        let new_str = ASTString::new(str_term).unwrap();
        unify( &ASTNode::ASTString(new_str), pattern, state, unifying)

    } else if pattern_type == "if" {

        // If we are not evaluating redundant claues
        if !unifying {
            // If we are evaluating subsumption between two different conditional patterns
            // we want to 'punt' and print a warning message.
            if term_type == "if" {
                if !state.cond_warning {
                    state.warning("Redundant pattern detection is not supported for conditional pattern expressions.");
                    state.cond_warning = true;
                }
                return Err( ("PatternMatchFailed",format!("Subsumption relatioship broken, pattern will not be rendered redundant."))) // User should never see
            }
        } 

        let ASTNode::ASTIf(ASTIf{id,cond_exp,then_exp,else_exp}) = pattern
            else {panic!("Unify: expected if expresssion.")};

        let else_type = peek(&else_exp[0]).unwrap();
        if else_type != "none" {
            return Err( ("ValueError",String::from("Conditional patterns do not support else clauses.")) )
        } 

        let  unifiers = unify(term,&then_exp[0],state,unifying).unwrap();
        
        if state.constraint_lvl > 0 {
            state.push_scope();
        }

        // evaluate the conditional expression in the
        // context of the unifiers.
        declare_unifiers(&unifiers);
        let bool_val = map2boolean(&walk(&cond_exp[0],state).unwrap()).unwrap();

        if state.constraint_lvl > 0 {
            state.pop_scope();
        }

        let ASTNode::ASTBool(ASTBool{id:_,value}) = bool_val
            else {panic!("Unify: expected boolean.")};
        
        if value {
            Ok( unifiers )
        } else {
            Err( ("PatternMatchFailed",String::from("Condtional pattern match failed.")))
        }
    
    } else if term_type == "if" {
        // We will only get here when evaluating subsumption

        // If we get here, a conditional pattern clause is placed after a non-conditonal
        // pattern clause. Therefore, we need to check if the subsume because if they do
        // the conditonal clause is redundant.
        let ASTNode::ASTIf(ASTIf{id:_,cond_exp,then_exp,else_exp}) = term
            else {panic!("Unify: expected list")};

        let else_type = peek(&else_exp[0]).unwrap();
        if else_type != "none" {
            return Err( ("ValueError",String::from("Conditional patterns do not support else clauses.")) )
        } else {
            unify( &then_exp[0], pattern, state, false )
        }
    } else if pattern_type == "typematch"{

        let ASTNode::ASTTypeMatch(ASTTypeMatch{id:_,expression}) = pattern
            else {panic!("Unify: expected typematch.")};

        let ASTNode::ASTID(ASTID{id:_,name:p_name}) = &expression[0]
            else {panic!("Unigy: expected ID.")};

        if ["string","real","integer","list","typle","bool","boolean","none"].contains(&p_name.as_str()) {
            if !unifying {
                // handle lists/head-tails subsuming each other
                if ["list","headtail"].contains(&term_type){
                    if p_name == "list" {
                        return Ok( vec![] );
                    }
                }
            }

            if p_name == term_type {
                Ok( vec![] )
            } else {
                Err(("PatternMatchFailed",format!("expected typematch {} got a term of type {}",p_name,term_type)))
            }

        } else if p_name == "function" {
            //matching function and member function values
            if ["function","memberfunctionval"].contains(&term_type) {
                Ok( vec![] )
            } else {
                Err( ("PatternMatchFailed",format!("expected typematch {} got a term of type {}",p_name,term_type)))
            }

        } else if p_name == "pattern" {
            if term_type == "quote" {
                Ok( vec![] )
            } else {
                Err( ("PatternMatchFailed",format!("expected typematch {} got a term of type {}",p_name,term_type)))
            }

        } else if term_type == "object" {
            let ASTNode::ASTObject(ASTObject{id:_,struct_id,object_memory}) = term
                else {panic!("Unify: expected object.")};
            let ASTID{id:_,name:t_name} = struct_id;

            if t_name == p_name {
                Ok( vec![] )
            } else {
                Err( ("PatternMatchFailed",format!("expected typematch {} got a term of type {}",p_name,term_type)))
            }

        } else {
            // Check if the typematch is in the symbol table
            let in_symtab = state.find_sym(p_name);

            if let None = in_symtab {
                return Err( ("PatternMatchFailed",format!("{} is not a valid type for typematch.",p_name)));
            }

            let in_symtab_type = peek( state.lookup_sym(p_name,true).unwrap()).unwrap();
            if in_symtab_type != "struct" {
                Err( ("PatternMatchFailed",format!("{} is not a type.",p_name)))
            } else {
                Err( ("PatternMatchFailed",format!("expected typematch {} got an object of type {}",p_name,term_type)))
            }
        }
        
    } else {
        Err( ("PatternMatchFailed", format!("pattern {} did not match {}",term2string(pattern).unwrap(),term2string(term).unwrap())))
    }
}
/******************************************************************************/
pub fn walk<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    match node {
        ASTNode::ASTInteger(_) => Some(node.clone()),
        ASTNode::ASTReal(_) => Some(node.clone()),
        ASTNode::ASTBool(_) => Some(node.clone()),
        ASTNode::ASTString(_) => Some(node.clone()),
        ASTNode::ASTLineInfo(_) => process_lineinfo(node, state),
        ASTNode::ASTList(_) => list_exp(node, state),
        ASTNode::ASTNone(_) => Some(node.clone()),
        ASTNode::ASTNil(_) => Some(node.clone()),
        ASTNode::ASTToList(_) => to_list_exp(node,state),
        ASTNode::ASTRawToList(_) => raw_to_list_exp(node,state),
        ASTNode::ASTHeadTail(_) => head_tail_exp(node,state),
        ASTNode::ASTRawHeadTail(_) => raw_head_tail_exp(node,state),
        ASTNode::ASTSequence(_) => sequence_exp(node,state),
        ASTNode::ASTObject(_) => Some(node.clone()),
        ASTNode::ASTEval(_) => eval_exp(node,state),
        ASTNode::ASTQuote(_) => quote_exp(node,state),
        ASTNode::ASTConstraint(_) => constraint_exp(node,state),
        ASTNode::ASTTypeMatch(_) => constraint_exp(node,state),
        ASTNode::ASTForeign(_) => Some(node.clone()),
        ASTNode::ASTID(_) => id_exp(node,state),
        ASTNode::ASTApply(_) => apply_exp(node,state),
        ASTNode::ASTIndex(_) => index_exp(node,state),
        ASTNode::ASTEscape(_) => escape_exp(node,state),
        ASTNode::ASTIs(_) => is_exp(node,state),
        ASTNode::ASTIn(_) => in_exp(node,state),
        ASTNode::ASTIf(_) => if_exp(node,state),
        ASTNode::ASTNamedPattern(_) => named_pattern_exp(node,state),
        ASTNode::ASTMemberFunctionVal(_) => Some(node.clone()),
        ASTNode::ASTDeref(_) => deref_exp(node,state),
        _ => panic!("Unknown node type in AST."),
    }    
}
/******************************************************************************/
pub fn process_lineinfo<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    match node {
        ast::ASTNode::ASTLineInfo(ASTLineInfo{ id, module, line_number }) => state.lineinfo = (module.clone(),*line_number),
        _ => panic!("lineinfo error."),
    }
    Some( node.clone() )
}
/******************************************************************************/
pub fn list_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTList( ASTList{id,length,contents} ) = node 
        else { panic!("ERROR: walk: expected list in list_exp()") };

    let mut new_contents = Vec::with_capacity(*length);
    for i in 0..*length {
        new_contents.push( walk( &contents[i], state).unwrap() );
    }
    Some( ast::ASTNode::ASTList( ASTList::new(*length,new_contents).unwrap() ) )
}
/******************************************************************************/
pub fn tuple_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTTuple( ASTTuple{id,length,contents} ) = node 
        else { panic!("ERROR: walk: expected tuple in tuple_exp()") };

    let mut new_contents = Vec::with_capacity(*length);
    for i in 0..*length {
        new_contents.push( walk( &contents[i], state).unwrap() );
    }
    Some( ast::ASTNode::ASTTuple( ASTTuple::new(*length,new_contents).unwrap() ) )
}
/******************************************************************************/
pub fn to_list_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTToList(ASTToList{id,start,stop,stride}) = node 
        else { panic!("ERROR: walk: expected to_list in to_list_exp()") }; 

    let mut start_val = 0;
    let mut stop_val = 0;
    let mut stride_val = 0;

    {
        let start = walk(&start[0],state).unwrap();
        let ast::ASTNode::ASTInteger(ASTInteger{id,value}) = start 
            else { panic!("ERROR: walk: expected integer in to_list_exp()") };
        start_val= value;
    }

    {
        let stop = walk(&stop[0],state).unwrap();
        let ast::ASTNode::ASTInteger(ASTInteger{id,value}) = stop
            else { panic!("ERROR: walk: expected integer in to_list_exp()") };
        stop_val = value;
    }

    {
        let stride = walk(&stride[0],state).unwrap();
        let ast::ASTNode::ASTInteger(ASTInteger{id,value}) = stride
            else { panic!("ERROR: walk: expected integer in to_list_exp()") };
        stride_val = value;
    }

    let len = 
        if stop_val > start_val {
            ((stop_val-start_val)/stride_val) as usize
        } else {
            ((start_val-stop_val)/stride_val) as usize
        };

    let mut newlist = Vec::with_capacity(len);

    for i in (start_val..stop_val).step_by(stride_val as usize) {
        newlist.push(ast::ASTNode::ASTInteger(ast::ASTInteger::new( i ).unwrap()));
    }

    Some( ast::ASTNode::ASTList( ASTList::new(len,newlist).unwrap() ) )
}
/******************************************************************************/
pub fn raw_to_list_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTRawToList(ASTRawToList{id,start,stop,stride}) = node 
        else { panic!("ERROR: walk: expected to_list in to_list_exp()") }; 

    Some( walk( &ast::ASTNode::ASTToList( ASTToList{id:id-1,start:start.clone(),stop:stop.clone(),stride:stride.clone()}), state).unwrap() )
}
/******************************************************************************/
pub fn head_tail_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTHeadTail(ASTHeadTail{id,head,tail}) = node 
        else { panic!("ERROR: walk: expected head-tail exp in head_tail_exp().") }; 

    let ast::ASTNode::ASTList( ASTList{id,length,ref contents} ) = tail[0]
        else { panic!("ERROR: unsupported tail type in head-tail operator.") };

    let mut new_contents = Vec::with_capacity(length);
    new_contents.push(head[0].clone());
    for content in contents {
        new_contents.push(content.clone());
    }

    Some( ast::ASTNode::ASTList( ASTList::new( length + 1, new_contents).unwrap() ) ) 
}
/******************************************************************************/
pub fn raw_head_tail_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTRawHeadTail(ASTRawHeadTail{id,head,tail}) = node 
        else { panic!("ERROR: walk: expected raw head-tail exp in raw_head_tail_exp().") }; 

    Some( walk( &ast::ASTNode::ASTHeadTail( ASTHeadTail{id:id-1,head:head.clone(),tail:tail.clone()}), state).unwrap() )
}
/******************************************************************************/
pub fn sequence_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTSequence(ASTSequence{id,first,second}) = node 
        else { panic!("ERROR: walk: expected sequence expression in sequence_exp().") };  

    let first = walk( &first[0],state).unwrap();
    let second = walk( &second[0],state).unwrap();

    Some( ast::ASTNode::ASTSequence( ASTSequence{id:*id,first:vec![first],second:vec![second]} ))
}
/******************************************************************************/
pub fn eval_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTEval(ASTEval{id,expression}) = node 
        else { panic!("ERROR: walk: expected eval expression in exal_exp().") };  

    // Note: eval is essentially a macro call - that is a function
    // call without pushing a symbol table record.  That means
    // we have to first evaluate the argument to 'eval' before
    // walking the term.  This is safe because if the arg is already
    // the actual term it will be quoted and nothing happen
    let exp_value_expand = walk(&expression[0],state).unwrap();

    // now walk the actual term..
    state.ignore_quote_on();
    let exp_val = walk( &exp_value_expand,state).unwrap();
    state.ignore_quote_off();

    Some(exp_val)
}
/******************************************************************************/
pub fn quote_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTQuote(ASTQuote{id,expression}) = node 
        else { panic!("ERROR: walk: expected quote expression in quote_exp().") };  

    // quoted code should be treated like a constant if not ignore_quote
    if state.ignore_quote {
        Some(walk(&expression[0],state).unwrap())
    } else {
        Some(node.clone())
    }
}
/******************************************************************************/
pub fn constraint_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    //let ast::ASTNode::ASTConstraint(ASTConstraint{id,expression}) = node 
    //    else { panic!("ERROR: walk: expected constraint exp in constraint_exp().") };

    panic!("Constraint patterns cannot be used as constructors.");
}
/******************************************************************************/
pub fn id_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTID(ASTID{id,name}) = node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 
    
    Some( state.lookup_sym(name,true).unwrap().clone() )
}
/******************************************************************************/
pub fn apply_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTApply(ASTApply{id,function,argument}) = node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    //TODO 
    // handle builtin operators that look like apply lists.

    // handle function application
    let f_val = walk( &function[0], state).unwrap();
    //let f_name = ;
    let arg_val = walk( &argument[0], state).unwrap();

    Some(node.clone())
}
/******************************************************************************/
pub fn index_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTIndex(ASTIndex{id,structure,index_exp}) = node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    // look at the semantics of 'structure'
    let structure_val = walk( &structure[0],state).unwrap();

    // indexing/slicing
    let result = read_at_ix(&structure_val,&index_exp[0],state).unwrap();

    Some(result)
}
/******************************************************************************/
pub fn escape_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    //TODO
    Some( node.clone() )
}
/******************************************************************************/
pub fn is_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTIs(ASTIs{id,pattern,term}) = node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    let term_val = walk(&term[0], state).unwrap();
    let unifiers = unify(&term_val,&pattern[0],state,true);

    if let Err(_) = unifiers {
        Some(ASTNode::ASTBool(ASTBool::new(false).unwrap()))
    } else {
        declare_unifiers(&unifiers.unwrap());
        Some(ASTNode::ASTBool(ASTBool::new(true).unwrap()))
    }
}
/******************************************************************************/
pub fn in_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTIn(ASTIn{id,expression,expression_list}) = node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    let exp_val = walk(&expression[0],state).unwrap();
    let exp_list_val = walk(&expression_list[0],state).unwrap();
    let ASTNode::ASTList(ASTList{id,length,contents}) = exp_list_val
        else { panic!("Right argument to in operator has to be a list.")};

    // We simply map the in operator to Rust's contains function
    if contents.contains(&exp_val) {
        Some( ASTNode::ASTBool(ASTBool::new(true).unwrap()))
    } else {
        Some( ASTNode::ASTBool(ASTBool::new(false).unwrap()))
    }
}
/******************************************************************************/
pub fn if_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTIf(ASTIf{id,cond_exp,then_exp,else_exp}) = node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    let cond_val = map2boolean(&walk( &cond_exp[0], state ).unwrap()).unwrap();
    let ASTNode::ASTBool(ASTBool{id,value}) = cond_val 
        else {panic!("Expected boolean from map2boolean.")};
    
    if value {
        walk(&then_exp[0],state)
    } else {
        walk(&else_exp[0],state)
    }
}
/*******************************************************************************
# Named patterns - when walking a named pattern we are interpreting a
# a pattern as a constructor - ignore the name                                */
pub fn named_pattern_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{
    let ast::ASTNode::ASTNamedPattern(ASTNamedPattern{id,name,pattern}) = node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    walk(&pattern[0],state)
}
/******************************************************************************/
pub fn deref_exp<'a>( node: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{

    Some( node.clone() )
}
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/*******************************************************************************
# Evaluates a set of unifiers for the presence of repeated variable
# names within a pattern. Repeated variables names within the same pattern
# are what is called a non-linear pattern, which is not currently supported
# by Asteroid.                                                                */
fn check_repeated_symbols(unifiers: Vec<(ASTNode,ASTNode)> ) -> bool {
    let len = unifiers.len();
    let mut seen = Vec::with_capacity(len);

    for i in 0..len {
        let next = peek( &unifiers[i].0 ).unwrap();

        if next == "id" {
            let ASTNode::ASTID(ASTID{id,name}) = &unifiers[i].0
                else {panic!("Unify: expected id.")};
            
            if seen.contains(&name) { // repeated symbol detected
                return true
            } else {
                seen.push(&name);
            }
        }
    }
    false // no repeats exist if we get here.
}
/******************************************************************************/
pub fn declare_unifiers( unifiers: &Vec<(ASTNode,ASTNode)> ) {
    let x = 1;
}
/******************************************************************************/
// TODO needs work
pub fn read_at_ix<'a>( structure_val: &'a ASTNode, ix: &'a ASTNode, state: &'a mut State ) -> Option<ASTNode>{

    // find the actual memory we need to access
    let struct_type = peek(structure_val).unwrap();
    let ix_type = peek(ix).unwrap();

    if struct_type == "list" || struct_type == "tuple" || struct_type == "string" {
        if struct_type == "list" && ix_type == "id" {
            let ASTNode::ASTID( ASTID{id,name}) = ix else {panic!{"Error: expected ID."}};
            //if name in list_member_functions {
                // we are looking at the function name of a list member
                // function - find the implementation and return it.
                // TODO
                return Some(structure_val.clone())
            //}
        } else if struct_type == "string" && ix_type == "id" {
            let ASTNode::ASTID( ASTID{id,name}) = ix else {panic!{"Error: expected ID."}};
            //if name in string_member_functions {
                // we are looking at the function name of a string member
                // function - find the implementation and return it.
                // TODO
                return Some(structure_val.clone())
            //}
        } else {
            //memory = structure_val[1]
            let ix_val = walk( ix, state);
        }
    } else if struct_type == "object" {
        let ASTNode::ASTObject( ASTObject{id,struct_id,object_memory}) = structure_val
            else {panic!("Error: expected object.")};
        let ASTID{id,name} = struct_id;
        let ASTList{id,length,contents} = object_memory;

        let struct_val = state.lookup_sym(name,true).unwrap();
        
        let ASTNode::ASTStruct( ASTStruct{id,member_names,struct_memory}) = struct_val 
            else {panic!("Error: expected struct.")};

        if ix_type == "id" {
            let ASTNode::ASTID(ASTID{id,name}) = ix else {panic!("Error: expected ID.")};
            //if name in member_names {
            //
            //} else 
        } else {
            let ix_val = walk( ix, state).unwrap();
        }
    } else {
        panic!("{} is not indexable.",peek(structure_val).unwrap())
    }

    if ix_type == "integer" {
        if struct_type == "string" {
            //return string 
        } else if struct_type == "object" {
            //TODO
        } else {
            //TODO
        }
    } else if ix_type == "list" {
        //TODO
        let ASTNode::ASTList( ASTList{id,length,contents} ) = ix
            else {panic!("Error: expected list.")};
        if *length == 0 {
            panic!("Index list is empty.");
        }
    } else {
        panic!("Index operation '{}' not supported.",peek(ix).unwrap());
    }

    Some(structure_val.clone())
}
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineinfo() {
        let newline = ASTLineInfo::new( String::from("test1"),123 ).unwrap();
        let mut state = State::new().unwrap();
        {
            let out1 = state.lineinfo.clone();
            assert_eq!(out1,(String::from("<input>"),1));
        }

        walk( &ASTNode::ASTLineInfo(newline),&mut state );

        {
            let out2 = state.lineinfo.clone();
            assert_eq!(out2,(String::from("test1"),123));
        }

        let newline = ASTLineInfo::new( String::from("math"), 987654321).unwrap();
        walk( &ASTNode::ASTLineInfo(newline),&mut state );

        {
            let out3 = state.lineinfo.clone();
            assert_eq!(out3,(String::from("math"), 987654321));
        }
    }
    #[test]
    fn test_list() {
        let newline1 = ASTLineInfo::new( String::from("test1"),1 ).unwrap();
        let newline2 = ASTLineInfo::new( String::from("test2"),12 ).unwrap();
        let newline3 = ASTLineInfo::new( String::from("test3"),123 ).unwrap();
        let newlist = ASTList::new(3,vec![ast::ASTNode::ASTLineInfo(newline1),
                                          ast::ASTNode::ASTLineInfo(newline2),
                                          ast::ASTNode::ASTLineInfo(newline3)]).unwrap();
        let mut state = State::new().unwrap();

        walk( &ASTNode::ASTList(newlist),&mut state);

        {
            let out1 = state.lineinfo;
            assert_eq!(out1,(String::from("test3"),123));
        }
    }
    #[test]
    fn test_to_list() {

        let int1 = ASTInteger::new(0).unwrap(); //start
        let int2 = ASTInteger::new(10).unwrap();//stop
        let int3 = ASTInteger::new(1).unwrap(); //stride
        let newlist = ASTToList::new( vec![ast::ASTNode::ASTInteger(int1)],
                                      vec![ast::ASTNode::ASTInteger(int2)],
                                      vec![ast::ASTNode::ASTInteger(int3)]).unwrap();
        let mut state = State::new().unwrap();

        let out = walk( &ASTNode::ASTToList(newlist), &mut state ).unwrap(); 
        let ast::ASTNode::ASTList( ASTList{id,length,contents} ) = out 
            else { panic!("ERROR: test: expected list in to-list") };
        assert_eq!(length,10);
        let ast::ASTNode::ASTInteger( ASTInteger{id,value} ) = contents[9] 
            else { panic!("ERROR: test: expected int in to-list") };
        assert_eq!(value,9);
        let ast::ASTNode::ASTInteger( ASTInteger{id,value} ) = contents[4] 
            else { panic!("ERROR: test: expected int in to-list") };
        assert_eq!(value,4);

        let int3 = ASTInteger::new(0).unwrap(); //start
        let int4 = ASTInteger::new(100).unwrap();//stop
        let int5 = ASTInteger::new(2).unwrap(); //stride
        let newlist = ASTRawToList::new( vec![ast::ASTNode::ASTInteger(int3)],
                                         vec![ast::ASTNode::ASTInteger(int4)],
                                         vec![ast::ASTNode::ASTInteger(int5)]).unwrap();
        let mut state = State::new().unwrap();

        let out2 = walk( &ASTNode::ASTRawToList(newlist), &mut state ).unwrap(); 
        let ast::ASTNode::ASTList( ASTList{id,length,contents} ) = out2 
            else { panic!("ERROR: test: expected list in to-list") };
        assert_eq!(length,50);
        let ast::ASTNode::ASTInteger( ASTInteger{id,value} ) = contents[9] 
            else { panic!("ERROR: test: expected int in to-list") };
        assert_eq!(value,18);
        let ast::ASTNode::ASTInteger( ASTInteger{id,value} ) = contents[4] 
            else { panic!("ERROR: test: expected int in to-list") };
        assert_eq!(value,8);
    }
    #[test]
    fn test_headtail() {

        let int1 = ASTInteger::new(1).unwrap(); 
        let int2 = ASTInteger::new(2).unwrap(); 
        let int3 = ASTInteger::new(3).unwrap();
        let int4 = ASTInteger::new(4).unwrap(); 
        let newlist = ASTList::new(3, vec![ast::ASTNode::ASTInteger(int2),
                                         ast::ASTNode::ASTInteger(int3),
                                         ast::ASTNode::ASTInteger(int4)]).unwrap();
        let mut state = State::new().unwrap();

        let ht1 = ASTHeadTail::new(vec![ASTNode::ASTInteger(int1)],vec![ASTNode::ASTList(newlist)]).unwrap();
        let out = walk( &ASTNode::ASTHeadTail(ht1), &mut state ).unwrap(); 
        let ASTNode::ASTList( ASTList{id,length,contents} ) = out
            else { panic!("ERROR: test: expected list in to-list") };
        assert_eq!(length,4);
        let ast::ASTNode::ASTInteger( ASTInteger{id,value} ) = contents[0] 
            else { panic!("ERROR: test: expected int in to-list") };
        assert_eq!(value,1);
        let ast::ASTNode::ASTInteger( ASTInteger{id,value} ) = contents[1] 
            else { panic!("ERROR: test: expected int in to-list") };
        assert_eq!(value,2);
        let ast::ASTNode::ASTInteger( ASTInteger{id,value} ) = contents[2] 
            else { panic!("ERROR: test: expected int in to-list") };
        assert_eq!(value,3);
        let ast::ASTNode::ASTInteger( ASTInteger{id,value} ) = contents[3] 
            else { panic!("ERROR: test: expected int in to-list") };
        assert_eq!(value,4);
    }
    #[test]
    fn test_unify_integers() {
        let mut state = State::new().unwrap();
        let int1 = ASTInteger::new(1).unwrap(); 
        let int2 = ASTInteger::new(2).unwrap();
        let int3 = ASTInteger::new(10).unwrap();
        let int4 = ASTInteger::new(10).unwrap();

        let result = unify(&ASTNode::ASTInteger(int1),&ASTNode::ASTInteger(int2),&mut state, true);
        let Err(_) = result else { panic!("test unify integers error.")};

        let result = unify(&ASTNode::ASTInteger(int3),&ASTNode::ASTInteger(int4),&mut state, true);
        let Ok(list) = result else { panic!("test unify integers error.")};
        assert_eq!(list.len(),0);
    }
    #[test]
    fn test_unify_strings() {
        let str1 = ASTString::new(String::from("abc")).unwrap();
        let str2 = ASTString::new(String::from("abc")).unwrap();
        let mut state = State::new().unwrap();

        let result = unify(&ASTNode::ASTString(str1),&ASTNode::ASTString(str2),&mut state, true);
        let Ok(list) = result else { panic!("test unify integers error.")};
        assert_eq!(list.len(),0);

        let str3 = ASTString::new(String::from("def")).unwrap();
        let str4 = ASTString::new(String::from("abc")).unwrap();

        let result = unify(&ASTNode::ASTString(str3),&ASTNode::ASTString(str4),&mut state, true);
        let Err(_) = result else { panic!("test unify integers error.")};

        let str5 = ASTString::new(String::from("ttt")).unwrap();
        let str6 = ASTString::new(String::from("t*")).unwrap();
        
        let result = unify(&ASTNode::ASTString(str5),&ASTNode::ASTString(str6),&mut state, true);
        let Ok(list) = result else { panic!("test unify integers error.")};
        assert_eq!(list.len(),0);
    }
}