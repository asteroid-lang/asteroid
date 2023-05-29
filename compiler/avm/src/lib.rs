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

use std::rc::Rc;
use regex::Regex;

/******************************************************************************/
pub fn unify<'a>( term: Rc<AstroNode>, pattern: Rc<AstroNode>, state: &'a mut State, unifying: bool) -> Result<Vec<(Rc<AstroNode>,Rc<AstroNode>)>, (&'static str,String) >{
   
    let term_type = peek(term).unwrap();
    let pattern_type = peek(pattern).unwrap();

    /**
    if term_type == "string" { // Apply regular expression pattern match
        if pattern_type == "string" {
            // Note: a pattern needs to match the whole term.
            let AstroNode::AstroString(AstroString{id:t_id,value:t_value}) = term 
                else {panic!("Unify: expected string.")};
            let AstroNode::AstroString(AstroString{id:p_id,value:p_value}) = pattern 
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
            let AstroNode::AstroList(AstroList{id:_,length:tlen,contents:tcontents}) = term
                else {panic!("Unify: expected list.")};
            let AstroNode::AstroList(AstroList{id:_,length:plen,contents:pcontents}) = pattern
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
        let AstroNode::AstroNamedPattern(AstroNamedPattern{id,name,pattern:named_pattern}) = term
            else {panic!("Unify: expected named-pattern.")};

        unify(named_pattern,pattern,state,unifying)
    } else if !unifying && term_type == "deref" {
        //Unpack a term-sdie first class pattern if evaluating redundant clauses
        let AstroNode::AstroDeref(AstroDeref{id,expression}) = term 
            else {panic!("Unify: expected derefernece.")};

        let AstroNode::AstroID(AstroID{id,ref name}) = **expression
            else {panic!("Unify: expected id.")};
        
        let new_term = state.lookup_sym(name,true);
        
        //TODO
        //REEVALUATE- copy whole state is silly/find better solution
        unify(&new_term, pattern, &mut state.clone(), unifying)

    // ** Astroeroid value level matching **
    } else if term_type == "object" && pattern_type == "object" {
        //   this can happen when we dereference a variable pointing
        //   to an object as a pattern, e.g.
        //    let o = A(1,2). -- A is a structure with 2 data members
        //   let *o = o.

        let AstroNode::AstroObject(AstroObject{id:t_id,struct_id:t_struct_id,object_memory:t_object_memory}) = term
            else {panic!("Unify: expected object.")};
        let AstroNode::AstroObject(AstroObject{id:p_id,struct_id:p_struct_id,object_memory:p_object_memory}) = pattern
            else {panic!("Unify: expected object.")};
        
        let AstroID{id:_,name:t_name} = t_struct_id;
        let AstroID{id:_,name:p_name} = p_struct_id;

        if t_name != p_name {
            Err( ("PatternMatchFailed",format!("pattern type {} and term type {} do not agree",p_name,t_name)))
        } else {
            unify( &AstroNode::AstroList(t_object_memory.clone()), &AstroNode::AstroList(p_object_memory.clone()), state, unifying )
        }
    } else if pattern_type == "string" && term_type != "string" {
        // regular expression applied to a non-string structure
        // this is possible because all data types are subtypes of string
        let str_term = term2string(term).unwrap();
        let new_str = AstroString::new(str_term).unwrap();
        unify( &AstroNode::AstroString(new_str), pattern, state, unifying)

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

        let AstroNode::AstroIf(AstroIf{id,cond_exp,then_exp,else_exp}) = pattern
            else {panic!("Unify: expected if expresssion.")};

        let else_type = peek(else_exp).unwrap();
        if else_type != "none" {
            return Err( ("ValueError",String::from("Conditional patterns do not support else clauses.")) )
        } 

        let  unifiers = unify(term,then_exp,state,unifying).unwrap();
        
        if state.constraint_lvl > 0 {
            state.push_scope();
        }

        // evaluate the conditional expression in the
        // context of the unifiers.
        declare_unifiers(&unifiers);
        let bool_val = map2boolean(&walk(cond_exp,state).unwrap()).unwrap();

        if state.constraint_lvl > 0 {
            state.pop_scope();
        }

        let AstroNode::AstroBool(AstroBool{id:_,value}) = bool_val
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
        let AstroNode::AstroIf(AstroIf{id:_,cond_exp,then_exp,else_exp}) = term
            else {panic!("Unify: expected list")};

        let else_type = peek(else_exp).unwrap();
        if else_type != "none" {
            return Err( ("ValueError",String::from("Conditional patterns do not support else clauses.")) )
        } else {
            unify( then_exp, pattern, state, false )
        }
    } else if pattern_type == "typematch"{

        let AstroNode::AstroTypeMatch(AstroTypeMatch{id:_,expression}) = pattern
            else {panic!("Unify: expected typematch.")};

        let AstroNode::AstroID(AstroID{id:_,name:ref p_name}) = **expression
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
            let AstroNode::AstroObject(AstroObject{id:_,struct_id,object_memory}) = term
                else {panic!("Unify: expected object.")};
            let AstroID{id:_,name:t_name} = struct_id;

            if t_name == p_name {
                Ok( vec![] )
            } else {
                Err( ("PatternMatchFailed",format!("expected typematch {} got a term of type {}",p_name,term_type)))
            }

        } else {
            // Check if the typematch is in the symbol table
            let in_symtab = state.find_sym(&p_name);

            if let None = in_symtab {
                return Err( ("PatternMatchFailed",format!("{} is not a valid type for typematch.",p_name)));
            }

            let in_symtab_type = peek( state.lookup_sym(&p_name,true)).unwrap();
            if in_symtab_type != "struct" {
                Err( ("PatternMatchFailed",format!("{} is not a type.",p_name)))
            } else {
                Err( ("PatternMatchFailed",format!("expected typematch {} got an object of type {}",p_name,term_type)))
            }
        }
        
    } else {
        Err( ("PatternMatchFailed", format!("pattern {} did not match {}",term2string(pattern).unwrap(),term2string(term).unwrap())))
    }
    **/
    return Err( ("test", "test_".to_string()) );
}
/******************************************************************************/
pub fn walk<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{ 
    match *node {
        AstroNode::AstroInteger(_) => Ok(node),
        AstroNode::AstroReal(_) => Ok(node),
        AstroNode::AstroBool(_) => Ok(node),
        AstroNode::AstroString(_) => Ok(node),
        AstroNode::AstroLineInfo(_) => process_lineinfo(node, state),
        AstroNode::AstroList(_) => list_exp(node, state),
        AstroNode::AstroNone(_) => Ok(node),
        AstroNode::AstroNil(_) => Ok(node),
        AstroNode::AstroToList(_) => to_list_exp(node,state),
        AstroNode::AstroRawToList(_) => raw_to_list_exp(node,state),
        AstroNode::AstroHeadTail(_) => head_tail_exp(node,state),
        AstroNode::AstroRawHeadTail(_) => raw_head_tail_exp(node,state),
        AstroNode::AstroSequence(_) => sequence_exp(node,state),
        AstroNode::AstroObject(_) => Ok(node),
        AstroNode::AstroEval(_) => eval_exp(node,state),
        AstroNode::AstroQuote(_) => quote_exp(node,state),
        AstroNode::AstroConstraint(_) => constraint_exp(node,state),
        AstroNode::AstroTypeMatch(_) => constraint_exp(node,state),
        AstroNode::AstroForeign(_) => Ok(node),
        AstroNode::AstroID(_) => id_exp(node,state),
        AstroNode::AstroApply(_) => apply_exp(node,state),
        AstroNode::AstroIndex(_) => index_exp(node,state),
        AstroNode::AstroEscape(_) => escape_exp(node,state),
        AstroNode::AstroIs(_) => is_exp(node,state),
        AstroNode::AstroIn(_) => in_exp(node,state),
        AstroNode::AstroIf(_) => if_exp(node,state),
        AstroNode::AstroNamedPattern(_) => named_pattern_exp(node,state),
        AstroNode::AstroMemberFunctionVal(_) => Ok(node),
        AstroNode::AstroDeref(_) => deref_exp(node,state),
        _ => panic!("Unknown node type in walk function."),
    }    
}
/******************************************************************************/
pub fn process_lineinfo<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    match *node {
        AstroNode::AstroLineInfo(AstroLineInfo{id,ref module,line_number}) => state.lineinfo = (module.clone(),line_number),
        _ => panic!("lineinfo error."),
    }
    Ok( node )
}
/******************************************************************************/
pub fn list_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroList( AstroList{id,length,ref contents} ) = *node 
        else { panic!("ERROR: walk: expected list in list_exp()") };

    let mut new_contents = Vec::with_capacity(length);
    for i in 0..length {
        new_contents.push(  walk( contents[i].clone(), state).unwrap());
    }
    Ok( Rc::new( AstroNode::AstroList( AstroList::new(length,new_contents).unwrap())))
}
/******************************************************************************/
pub fn tuple_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroTuple( AstroTuple{id,length,ref contents} ) = *node 
        else { panic!("ERROR: walk: expected tuple in tuple_exp()") };

    let mut new_contents = Vec::with_capacity(length);
    for i in 0..length {
        new_contents.push( walk( contents[i].clone(), state).unwrap() );
    }
    Ok( Rc::new( AstroNode::AstroTuple( AstroTuple::new(length,new_contents).unwrap())))
}
/******************************************************************************/
pub fn to_list_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroToList(AstroToList{id,ref start,ref stop,ref stride}) = *node 
        else { panic!("ERROR: walk: expected to_list in to_list_exp()") }; 

    let mut start_val;
    let mut stop_val;
    let mut stride_val;

    {
        let start = walk(start.clone(),state).unwrap();
        let AstroNode::AstroInteger(AstroInteger{id,value}) = *start 
            else { panic!("ERROR: walk: expected integer in to_list_exp()") };
        start_val= value;
    }

    {
        let stop = walk(stop.clone(),state).unwrap();
        let AstroNode::AstroInteger(AstroInteger{id,value}) = *stop
            else { panic!("ERROR: walk: expected integer in to_list_exp()") };
        stop_val = value;
    }

    {
        let stride = walk(stride.clone(),state).unwrap();
        let AstroNode::AstroInteger(AstroInteger{id,value}) = *stride
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
        newlist.push(Rc::new(AstroNode::AstroInteger(AstroInteger::new( i ).unwrap())));
    }

    Ok( Rc::new(AstroNode::AstroList( AstroList::new(len,newlist).unwrap())))
}
/******************************************************************************/
pub fn raw_to_list_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroRawToList(AstroRawToList{id,ref start,ref stop,ref stride}) = *node 
        else { panic!("ERROR: walk: expected to_list in to_list_exp()") }; 

    walk( Rc::new( AstroNode::AstroToList( AstroToList{id:id-1,start:(*start).clone(),stop:(*stop).clone(),stride:(*stride).clone()} )), state)
}
/******************************************************************************/
pub fn head_tail_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroHeadTail(AstroHeadTail{id,ref head,ref tail}) = *node 
        else { panic!("ERROR: walk: expected head-tail exp in head_tail_exp().") }; 

    let AstroNode::AstroList( AstroList{id,length,ref contents} ) = **tail
        else { panic!("ERROR: unsupported tail type in head-tail operator.") };

    let mut new_contents = Vec::with_capacity(length);
    new_contents.push(head.to_owned());
    for content in contents {
        new_contents.push(content.to_owned());
    }

    Ok( Rc::new( AstroNode::AstroList( AstroList::new( length + 1, new_contents).unwrap()))) 
}
/******************************************************************************/
pub fn raw_head_tail_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroRawHeadTail(AstroRawHeadTail{id,ref head,ref tail}) = *node 
        else { panic!("ERROR: walk: expected raw head-tail exp in raw_head_tail_exp().") }; 

    walk( Rc::new( AstroNode::AstroHeadTail( AstroHeadTail{id:id-1,head:head.to_owned(),tail:tail.to_owned()})), state)
}
/******************************************************************************/
pub fn sequence_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroSequence(AstroSequence{id,ref first,ref second}) = *node 
        else { panic!("ERROR: walk: expected sequence expression in sequence_exp().") };  

    let first = walk( (*first).clone(),state).unwrap();
    let second = walk( (*second).clone(),state).unwrap();

    Ok( Rc::new( AstroNode::AstroSequence( AstroSequence{id:id,first:first,second:second})))
}
/******************************************************************************/
pub fn eval_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroEval(AstroEval{id,ref expression}) = *node 
        else { panic!("ERROR: walk: expected eval expression in exal_exp().") };  

    // Note: eval is essentially a macro call - that is a function
    // call without pushing a symbol table record.  That means
    // we have to first evaluate the argument to 'eval' before
    // walking the term.  This is safe because if the arg is already
    // the actual term it will be quoted and nothing happen
    let exp_value_expand = walk( (*expression).clone(),state).unwrap();

    // now walk the actual term..
    state.ignore_quote_on();
    let exp_val = walk( exp_value_expand,state).unwrap();
    state.ignore_quote_off();

    Ok(exp_val)
}
/******************************************************************************/
pub fn quote_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroQuote(AstroQuote{id,ref expression}) = *node 
        else { panic!("ERROR: walk: expected quote expression in quote_exp().") };  

    // quoted code should be treated like a constant if not ignore_quote
    if state.ignore_quote {
        walk((*expression).clone(),state)
    } else {
        Ok( node )
    }
}
/******************************************************************************/
pub fn constraint_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    //let AstroNode::AstroConstraint(AstroConstraint{id,expression}) = node 
    //    else { panic!("ERROR: walk: expected constraint exp in constraint_exp().") };

    panic!("Constraint patterns cannot be used as constructors.");
}
/******************************************************************************/
pub fn id_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)> {
    let AstroNode::AstroID(AstroID{id,ref name}) = *node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 
    
    Ok( state.lookup_sym(name,true).clone() )
}
/******************************************************************************/
pub fn apply_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroApply(AstroApply{id,ref function,ref argument}) = *node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    //TODO 
    // handle builtin operators that look like apply lists.

    // handle function application
    let f_val = walk( (*function).clone(), state).unwrap();
    //let f_name = ;
    let arg_val = walk( (*argument).clone(), state).unwrap();

    Ok(node)
}
/******************************************************************************/
pub fn index_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroIndex(AstroIndex{id,ref structure,ref index_exp}) = *node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    // look at the semantics of 'structure'
    let structure_val = walk((*structure).clone(),state).unwrap();

    // indexing/slicing
    let result = read_at_ix(structure_val,(*index_exp).clone(),state).unwrap();

    Ok(result)
}
/******************************************************************************/
pub fn escape_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    //TODO
    Ok( node )
}
/******************************************************************************/
pub fn is_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroIs(AstroIs{id,ref pattern,ref term}) = *node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    let term_val = walk((*term).clone(), state).unwrap();
    let unifiers = unify(term_val,(*pattern).clone(),state,true);

    if let Err(_) = unifiers {
        Ok( Rc::new( AstroNode::AstroBool(AstroBool::new(false).unwrap())))
    } else {
        declare_unifiers(&unifiers.unwrap());
        Ok( Rc::new( AstroNode::AstroBool(AstroBool::new(true).unwrap())))
    }
}
/******************************************************************************/
pub fn in_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroIn(AstroIn{id,ref expression,ref expression_list}) = *node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    let exp_val = walk((*expression).clone(),state).unwrap();
    let exp_list_val = walk((*expression_list).clone(),state).unwrap();
    let AstroNode::AstroList(AstroList{id,length,ref contents}) = *exp_list_val
        else { panic!("Right argument to in operator has to be a list.")};

    // We simply map the in operator to Rust's contains function
    if (*contents).contains(&exp_val) {
        Ok( Rc::new( AstroNode::AstroBool(AstroBool::new(true).unwrap())))
    } else {
        Ok( Rc::new( AstroNode::AstroBool(AstroBool::new(false).unwrap())))
    }
}
/******************************************************************************/
pub fn if_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroIf(AstroIf{id,ref cond_exp,ref then_exp,ref else_exp}) = *node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    let cond_val = map2boolean(&walk( (*cond_exp).clone(), state ).unwrap()).unwrap();
    let AstroNode::AstroBool(AstroBool{id,value}) = cond_val 
        else {panic!("Expected boolean from map2boolean.")};
    
    if value {
        walk((*then_exp).clone(),state)
    } else {
        walk((*else_exp).clone(),state)
    }
}
/*******************************************************************************
# Named patterns - when walking a named pattern we are interpreting a
# a pattern as a constructor - ignore the name                                */
pub fn named_pattern_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroNamedPattern(AstroNamedPattern{id,ref name,ref pattern}) =* node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    walk((*pattern).clone(),state)
}
/******************************************************************************/
pub fn deref_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{

    Ok( node )
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
# by Astroeroid.                                                                */
fn check_repeated_symbols(unifiers: Vec<(Rc<AstroNode>,Rc<AstroNode>)> ) -> bool {
    let len = unifiers.len();
    let mut seen = Vec::with_capacity(len);

    for i in 0..len {
        let next = peek( (unifiers[i].0).clone() ).unwrap();

        if next == "id" {
            let AstroNode::AstroID(AstroID{id,ref name}) = *unifiers[i].0
                else {panic!("Unify: expected id.")};
            
            if seen.contains(name) { // repeated symbol detected
                return true
            } else {
                seen.push((*name).clone()); // NOT AN RC CLONE
                                            // but just string(variable name)
            }
        }
    }
    false // no repeats exist if we get here.
}
/******************************************************************************/
pub fn declare_unifiers( unifiers: &Vec<(Rc<AstroNode>,Rc<AstroNode>)> ) {
    let x = 1;
}
/******************************************************************************/
// TODO needs work
pub fn read_at_ix<'a>( structure_val: Rc<AstroNode>, ix: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{

    // find the actual memory we need to access
    let struct_type = peek(structure_val.clone()).unwrap();
    let ix_type = peek(ix).unwrap();

    /**

    if struct_type == "list" || struct_type == "tuple" || struct_type == "string" {
        if struct_type == "list" && ix_type == "id" {
            let AstroNode::AstroID( AstroID{id,name}) = ix else {panic!{"Error: expected ID."}};
            //if name in list_member_functions {
                // we are looking at the function name of a list member
                // function - find the implementation and return it.
                // TODO
                return Some(structure_val.clone())
            //}
        } else if struct_type == "string" && ix_type == "id" {
            let AstroNode::AstroID( AstroID{id,name}) = ix else {panic!{"Error: expected ID."}};
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
        let AstroNode::AstroObject( AstroObject{id,struct_id,object_memory}) = structure_val
            else {panic!("Error: expected object.")};
        let AstroID{id,name} = struct_id;
        let AstroList{id,length,contents} = object_memory;

        let struct_val = state.lookup_sym(name,true).unwrap();
        
        let AstroNode::AstroStruct( AstroStruct{id,member_names,struct_memory}) = struct_val 
            else {panic!("Error: expected struct.")};

        if ix_type == "id" {
            let AstroNode::AstroID(AstroID{id,name}) = ix else {panic!("Error: expected ID.")};
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
        let AstroNode::AstroList( AstroList{id,length,contents} ) = ix
            else {panic!("Error: expected list.")};
        if *length == 0 {
            panic!("Index list is empty.");
        }
    } else {
        panic!("Index operation '{}' not supported.",peek(ix).unwrap());
    }

    **/
    Ok(structure_val.clone())
}
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineinfo() {
        let newline = AstroLineInfo::new( String::from("test1"),123 ).unwrap();
        let mut state = State::new().unwrap();
        {
            let out1 = state.lineinfo.clone();
            assert_eq!(out1,(String::from("<input>"),1));
        }

        walk( Rc::new( AstroNode::AstroLineInfo(newline)),&mut state );

        {
            let out2 = state.lineinfo.clone();
            assert_eq!(out2,(String::from("test1"),123));
        }

        let newline = AstroLineInfo::new( String::from("math"), 987654321).unwrap();
        walk( Rc::new(  AstroNode::AstroLineInfo(newline)),&mut state );

        {
            let out3 = state.lineinfo.clone();
            assert_eq!(out3,(String::from("math"), 987654321));
        }
    }
    // #[test]
    // fn test_list() {
    //     let newline1 = AstroLineInfo::new( String::from("test1"),1 ).unwrap();
    //     let newline2 = AstroLineInfo::new( String::from("test2"),12 ).unwrap();
    //     let newline3 = AstroLineInfo::new( String::from("test3"),123 ).unwrap();
    //     let newlist = AstroList::new(3,vec![AstroNode::AstroLineInfo(newline1),
    //                                       AstroNode::AstroLineInfo(newline2),
    //                                       AstroNode::AstroLineInfo(newline3)]).unwrap();
    //     let mut state = State::new().unwrap();

    //     walk( &AstroNode::AstroList(newlist),&mut state);

    //     {
    //         let out1 = state.lineinfo;
    //         assert_eq!(out1,(String::from("test3"),123));
    //     }
    // }
    // #[test]
    // fn test_to_list() {

    //     let int1 = AstroInteger::new(0).unwrap(); //start
    //     let int2 = AstroInteger::new(10).unwrap();//stop
    //     let int3 = AstroInteger::new(1).unwrap(); //stride
    //     let newlist = AstroToList::new( vec![AstroNode::AstroInteger(int1)],
    //                                   vec![AstroNode::AstroInteger(int2)],
    //                                   vec![AstroNode::AstroInteger(int3)]).unwrap();
    //     let mut state = State::new().unwrap();

    //     let out = walk( &AstroNode::AstroToList(newlist), &mut state ).unwrap(); 
    //     let AstroNode::AstroList( AstroList{id,length,contents} ) = out 
    //         else { panic!("ERROR: test: expected list in to-list") };
    //     assert_eq!(length,10);
    //     let AstroNode::AstroInteger( AstroInteger{id,value} ) = contents[9] 
    //         else { panic!("ERROR: test: expected int in to-list") };
    //     assert_eq!(value,9);
    //     let AstroNode::AstroInteger( AstroInteger{id,value} ) = contents[4] 
    //         else { panic!("ERROR: test: expected int in to-list") };
    //     assert_eq!(value,4);

    //     let int3 = AstroInteger::new(0).unwrap(); //start
    //     let int4 = AstroInteger::new(100).unwrap();//stop
    //     let int5 = AstroInteger::new(2).unwrap(); //stride
    //     let newlist = AstroRawToList::new( vec![AstroNode::AstroInteger(int3)],
    //                                      vec![AstroNode::AstroInteger(int4)],
    //                                      vec![AstroNode::AstroInteger(int5)]).unwrap();
    //     let mut state = State::new().unwrap();

    //     let out2 = walk( &AstroNode::AstroRawToList(newlist), &mut state ).unwrap(); 
    //     let AstroNode::AstroList( AstroList{id,length,contents} ) = out2 
    //         else { panic!("ERROR: test: expected list in to-list") };
    //     assert_eq!(length,50);
    //     let AstroNode::AstroInteger( AstroInteger{id,value} ) = contents[9] 
    //         else { panic!("ERROR: test: expected int in to-list") };
    //     assert_eq!(value,18);
    //     let AstroNode::AstroInteger( AstroInteger{id,value} ) = contents[4] 
    //         else { panic!("ERROR: test: expected int in to-list") };
    //     assert_eq!(value,8);
    // }
    // #[test]
    // fn test_headtail() {

    //     let int1 = AstroInteger::new(1).unwrap(); 
    //     let int2 = AstroInteger::new(2).unwrap(); 
    //     let int3 = AstroInteger::new(3).unwrap();
    //     let int4 = AstroInteger::new(4).unwrap(); 
    //     let newlist = AstroList::new(3, vec![AstroNode::AstroInteger(int2),
    //                                      AstroNode::AstroInteger(int3),
    //                                      AstroNode::AstroInteger(int4)]).unwrap();
    //     let mut state = State::new().unwrap();

    //     let ht1 = AstroHeadTail::new(vec![AstroNode::AstroInteger(int1)],vec![AstroNode::AstroList(newlist)]).unwrap();
    //     let out = walk( &AstroNode::AstroHeadTail(ht1), &mut state ).unwrap(); 
    //     let AstroNode::AstroList( AstroList{id,length,contents} ) = out
    //         else { panic!("ERROR: test: expected list in to-list") };
    //     assert_eq!(length,4);
    //     let AstroNode::AstroInteger( AstroInteger{id,value} ) = contents[0] 
    //         else { panic!("ERROR: test: expected int in to-list") };
    //     assert_eq!(value,1);
    //     let AstroNode::AstroInteger( AstroInteger{id,value} ) = contents[1] 
    //         else { panic!("ERROR: test: expected int in to-list") };
    //     assert_eq!(value,2);
    //     let AstroNode::AstroInteger( AstroInteger{id,value} ) = contents[2] 
    //         else { panic!("ERROR: test: expected int in to-list") };
    //     assert_eq!(value,3);
    //     let AstroNode::AstroInteger( AstroInteger{id,value} ) = contents[3] 
    //         else { panic!("ERROR: test: expected int in to-list") };
    //     assert_eq!(value,4);
    // }
    // #[test]
    // fn test_unify_integers() {
    //     let mut state = State::new().unwrap();
    //     let int1 = AstroInteger::new(1).unwrap(); 
    //     let int2 = AstroInteger::new(2).unwrap();
    //     let int3 = AstroInteger::new(10).unwrap();
    //     let int4 = AstroInteger::new(10).unwrap();

    //     let result = unify(Rc::new(AstroNode::AstroInteger(int1)),Rc::new(AstroNode::AstroInteger(int2)),&mut state, true);
    //     let Err(_) = result else { panic!("test unify integers error.")};

    //     let result = unify(Rc::new(AstroNode::AstroInteger(int3)),Rc::new(AstroNode::AstroInteger(int4)),&mut state, true);
    //     let Ok(list) = result else { panic!("test unify integers error.")};
    //     assert_eq!(list.len(),0);
    // }
    // #[test]
    // fn test_unify_strings() {
    //     let str1 = AstroString::new(String::from("abc")).unwrap();
    //     let str2 = AstroString::new(String::from("abc")).unwrap();
    //     let mut state = State::new().unwrap();

    //     let result = unify(Rc::new(AstroNode::AstroString(str1)),Rc::new(AstroNode::AstroString(str2)),&mut state, true);
    //     let Ok(list) = result else { panic!("test unify integers error.")};
    //     assert_eq!(list.len(),0);

    //     let str3 = AstroString::new(String::from("def")).unwrap();
    //     let str4 = AstroString::new(String::from("abc")).unwrap();

    //     let result = unify(Rc::new(AstroNode::AstroString(str3)),Rc::new(AstroNode::AstroString(str4)),&mut state, true);
    //     let Err(_) = result else { panic!("test unify integers error.")};

    //     let str5 = AstroString::new(String::from("ttt")).unwrap();
    //     let str6 = AstroString::new(String::from("t*")).unwrap();
        
    //     let result = unify(Rc::new(AstroNode::AstroString(str5)),Rc::new(AstroNode::AstroString(str6)),&mut state, true);
    //     let Ok(list) = result else { panic!("test unify integers error.")};
    //     assert_eq!(list.len(),0);
    // }
}