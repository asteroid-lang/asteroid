/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Abstract Virtual Machine                                                   */
/*                                                                            */
/* (c) University of Rhode Island                                             */
/******************************************************************************/
#![allow(unused)]

use state::*;     //Asteroid state representation
use symtab::*;    //Asteroid symbol table
use ast::*;       //Asteroid AST representation
use support::*;   //Asteroid support functions

use std::rc::Rc;  
use regex::Regex; //Regular expressions
use std::collections::HashMap;
use std::cell::RefCell;

static OPERATOR_SYMBOLS: [&str; 12] = [ "__plus__", "__minus__", "__times__", "__divide__", "__or__", "__and__", "__eq__", 
                                        "__ne__", "__lt__", "__le__", "__ge__", "__gt__" ];
static BINARY_OPERATORS: [&str; 12] = [ "__plus__", "__minus__", "__times__", "__divide__", "__or__", "__and__", "__eq__", 
                                        "__ne__", "__lt__", "__le__", "__ge__", "__gt__" ];

/******************************************************************************/
pub fn unify<'a>( term: Rc<AstroNode>, pattern: Rc<AstroNode>, state: &'a mut State, unifying: bool) -> Result<Vec<(Rc<AstroNode>,Rc<AstroNode>)>, (&'static str,String) >{
   
    let term_type = peek( Rc::clone(&term) );
    let pattern_type = peek( Rc::clone(&pattern) );

    println!("Unifying: {} and {}",term_type,pattern_type);

    if term_type == "string" && (pattern_type != "id" && pattern_type != "index") { // Apply regular expression pattern match
        
        if pattern_type == "string" {
            // Note: a pattern needs to match the whole term.
            let AstroNode::AstroString(AstroString{id:t_id,value:ref t_value}) = *term 
                else {panic!("Unify: expected string.")};
            let AstroNode::AstroString(AstroString{id:p_id,value:ref p_value}) = *pattern 
                else {panic!("Unify: expected string.")};

            let mut re_str = String::from(r"^");
            re_str.push_str(&p_value);
            re_str.push_str("$");
            let re = Regex::new(&re_str).unwrap();

            if re.is_match(&t_value) {
                Ok( vec![] ) // Return an empty unifier
            } else {
                Err( ("PatternMatchFailed", format!("regular expression {} did not match {}",term2string(&pattern).unwrap(),term2string(&term).unwrap())))
            }
        } else {
            Err( ("PatternMatchFailed", format!("regular expression {} did not match {}",term2string(&pattern).unwrap(),term2string(&term).unwrap())))
        }
    } else if (term_type == "integer" || term_type == "bool" || term_type == "real") && (pattern_type == "integer" || pattern_type == "bool" || pattern_type == "real")  {

        if term_type == pattern_type && term == pattern {
            Ok( vec![] ) // Return an empty unifier
        } else {
            Err( ("PatternMatchFailed", format!("{} is not the same as {}",term2string(&pattern).unwrap(),term2string(&term).unwrap())))
        }

    } else if !unifying && term_type == "namedpattern" {

        // Unpack a term-side name-pattern if evaluating redundant clauses
        let AstroNode::AstroNamedPattern( AstroNamedPattern{id:_,name:_,pattern:ref t_pattern}) = *term
            else {panic!("Unify: expected named pattern.")};

        unify( Rc::clone( t_pattern), pattern, state, unifying )

    } else if !unifying && term_type == "deref" {

        let AstroNode::AstroDeref(AstroDeref{id:_,expression:ref t_expression}) = *term
            else {panic!("Unify: expected derefence expression.")};

        let AstroNode::AstroID(AstroID{id:_,name:ref t_name}) = **t_expression
            else {panic!("Unify: expected derefence expression.")};

        let term = state.lookup_sym( &t_name, true );

        unify( term, pattern, state, unifying )
    
    /* ** Asteroid value level matching ** */
    } else if term_type == "object" && pattern_type == "object" {
        // this can happen when we dereference a variable pointing
        // to an object as a pattern, e.g.
        //    let o = A(1,2). -- A is a structure with 2 data members
        //    let *o = o.
        let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref t_id,object_memory:ref t_data}) = *term
            else {panic!("Unify: expected object.")};
        let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref p_id,object_memory:ref p_data}) = *pattern
            else {panic!("Unify: expected object.")};

        let AstroID{id:_,name:t_name} = t_id;
        let AstroID{id:_,name:p_name} = p_id;

        if t_name != p_name {
            Err( ("PatternMatchFailed", format!("pattern type {} and term type {} do not agree.",t_name,p_name)))
        } else {
            let mut unifiers = vec![];
            for i in 0..t_data.borrow().len() {
                unifiers.append( &mut unify( Rc::clone(&t_data.borrow()[i]) , Rc::clone(&p_data.borrow()[i]),state,unifying).unwrap());
            }
            Ok(unifiers)
        }

    } else if pattern_type == "string" && term_type != "string" {

        let new_str = term2string(&term).unwrap();
        let new_term = AstroString{id:3,value:new_str};

        unify( Rc::new(AstroNode::AstroString(new_term)),pattern,state,unifying )

    } else if pattern_type == "if" {

        // If we are evaluating subsumption
        if !unifying {
            // If we are evaluating subsumption between two different conditional patterns
            // we want to 'punt' and print a warning message.
            if !state.cond_warning {
                eprintln!("Redundant pattern detection is not supported for conditional pattern expressions.");
                state.cond_warning = true;
                Ok(vec![])
            } else {
                // Otherwise if the term is not another cmatch the clauses are correctly ordered.
                Err( ("PatternMatchFailed", format!("Subsumption relatioship broken, pattern will not be rendered redundant.")))
            } 
        } else {

            let AstroNode::AstroIf(AstroIf{id:_, cond_exp:ref p_cond, then_exp:ref p_then, else_exp:ref p_else}) = *pattern
                else {panic!("Unify: expected if expression.")};

            if let AstroNode::AstroNone(AstroNone{id:_}) = **p_else {

                let unifiers = unify(term,Rc::clone(p_then),state,unifying).unwrap();
                if state.constraint_lvl > 0 {
                    state.push_scope();
                }

                // evaluate the conditional expression in the
                // context of the unifiers.
                declare_unifiers( &unifiers, state );
                let bool_val = map2boolean( &walk(Rc::clone(p_cond),state).unwrap() ).unwrap();

                if state.constraint_lvl > 0 {
                    state.pop_scope();
                }

                let AstroNode::AstroBool(AstroBool{id:_,value:b_value}) = bool_val
                    else {panic!("Unify: expected boolean.")};

                if b_value {
                    Ok( unifiers )
                } else {
                    Err(("PatternMatchFailed","Conditional pattern match failed.".to_string()))
                }   
            } else {
                Err(("ValueError","Conditional patterns do not support else clauses.".to_string()))
            }
        }

    } else if term_type == "if" {
        // We will only get here when evaluating subsumption

        // If we get here, a conditional pattern clause is placed after a non-conditonal
        // pattern clause. Therefore, we need to check if the subsume because if they do
        // the conditonal clause is redundant.
        let AstroNode::AstroIf(AstroIf{id:_, cond_exp:ref p_cond, then_exp:ref p_then, else_exp:ref p_else}) = *term
            else {panic!("Unify: expected if expression.")};

        if let AstroNode::AstroNone(AstroNone{id:_}) = **p_else {
            unify( Rc::clone( p_then ),pattern,state,unifying  )
        } else {
            Err(("ValueError","Conditional patterns do not support else clauses.".to_string()))
        }

    } else if pattern_type == "typematch" {
        
        let AstroNode::AstroTypeMatch(AstroTypeMatch{id:_,expression:ref p_exp}) = *pattern
            else {panic!("Unify: expected typematch.")};
        //let look_next = 0u32; // Indicates what index we will look into next

        let AstroNode::AstroString(AstroString{id:_,value:ref p_type}) = **p_exp
            else {panic!("Unify: expected string.")};

        if ["string","real","integer","list","tuple","boolean","none"].contains( &p_type.as_str() ) {
            if !unifying {
                if ["list","head-tail"].contains( &term_type ) {
                    if p_type == "list" {
                        return Ok( vec![] )
                    }
                }
            } 
            if p_type == term_type {
                return Ok( vec![] )
            } else {
                Err(("PatternMatchFailed",format!("Expected typematch: {}, got a term of type {}",p_type,term_type)))
            }
        } else if p_type == "function" {
            //  matching function and member function values
            if ["function-val","member-function-val"].contains( &term_type ){
                Ok( vec![] )
            } else {
                Err(("PatternMatchFailed",format!("Expected typematch: {}, got a term of type {}",p_type,term_type)))
            }
        } else if p_type == "pattern" {
            if term_type == "quote" {
                Ok( vec![] )
            } else {
                Err(("PatternMatchFailed",format!("Expected typematch: {}, got a term of type {}",p_type,term_type)))
            }
        } else if p_type == "object" {
            let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref t_id,object_memory:ref t_mem}) = *term
                else {panic!("Unify: expected object.")};
            let AstroID{id:_,name:t_type} = t_id;

            if p_type == t_type {
                Ok( vec![] )
            } else {
                Err(("PatternMatchFailed",format!("Expected typematch: {}, got a term of type {}",p_type,term_type)))
            }
        } else {
            // Check if the typematch is in the symbol table
            let in_symtab = state.find_sym(p_type);
            match in_symtab {
                None => return Err(("PatternMatchFailed",format!("{} is not a valid type for typematch",p_type))),
                Some(_) => (),
            };

            // If it is in the symbol table but not a struct, it cannot be typematched
            // because it is not a type
            if peek( state.lookup_sym( p_type,true ) ) != "struct" {
                Err(("PatternMatchFailed",format!("{} is not a type",p_type)))
            } else { 
                //Otherwhise, the typematch has failed
                Err(("PatternMatchFailed",format!("Expected typematch: {}, got a term of type {}",p_type,term_type)))
            }
        }
    } else if pattern_type == "namedpattern" {

        let AstroNode::AstroNamedPattern(AstroNamedPattern{id:_,name:ref p_name,pattern:ref p_pattern}) = *pattern
            else {panic!("Unify: expected named pattern.")};

        // name_exp can be an id or an index expression.
        let mut unifiers = unify( Rc::clone(&term), Rc::clone(p_pattern),state,unifying );

        let mut unifiers = match unifiers {
            Ok( val ) => val,
            Err( val ) => return Err(val),
        };

        unifiers.push( (Rc::new(AstroNode::AstroID(p_name.clone())), Rc::clone(&term)) );
        Ok( unifiers )

    } else if pattern_type == "none" {
        if term_type == "none" {
            Err(("PatternMatchFailed",format!("expected 'none' got '{}'",term_type)))
        } else {
            Ok( vec![] )
        }
    // NOTE: functions/foreign are allowed in terms as long as they are matched
    // by a variable in the pattern - anything else will fail
    } else if ["tolist","rawtolist","wherelist","rawwherelist","if","escape","is","in"].contains( &term_type ) {
        Err(("PatternMatchFailed",format!("term of type '{}' not allowed in pattern matching",term_type)))

    } else if ["tolist","rawtolist","wherelist","rawwherelist","if","escape","is","in","foreign","function"].contains( &pattern_type ) {
        Err(("PatternMatchFailed",format!("term of type '{}' not allowed in pattern matching",pattern_type)))

    } else if pattern_type == "quote" {

        // quotes on the pattern side can always be ignored
        let AstroNode::AstroQuote(AstroQuote{id:_,expression:ref p_exp}) = *pattern
                else {panic!("Unify: expected quote.")};

        if term_type == "quote" {
            let AstroNode::AstroQuote(AstroQuote{id:_,expression:ref t_exp}) = *term
                else {panic!("Unify: expected quote.")};

            unify(Rc::clone(&t_exp),Rc::clone(&p_exp),state,unifying)
        } else {
            unify(Rc::clone(&term),Rc::clone(&p_exp),state,unifying)
        }
    } else if term_type == "quote" && !(["id","index"].contains( &pattern_type))  {
        // ignore quote on the term if we are not trying to unify term with
        // a variable or other kind of lval
        let AstroNode::AstroQuote(AstroQuote{id:_,expression:ref t_exp}) = *term
            else {panic!("Unify: expected quote.")};

        unify( Rc::clone(&t_exp), pattern, state, unifying )

    } else if term_type == "object" && pattern_type == "apply" {

        let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref t_name,object_memory:ref t_mem}) = *term
            else {panic!("Unify: expected object.")};
        let AstroNode::AstroApply(AstroApply{id:_,function:ref p_func,argument:ref p_arg}) = *pattern
            else {panic!("Unify: expected apply.")};
        let AstroNode::AstroID(AstroID{id:_,name:ref p_id}) = **p_func
            else {panic!("Unify: expected string.")};
        let AstroID{id:_,name:t_id} = t_name;

        
        if p_id != t_id {
            Err(("PatternMatchFailed",format!("expected type '{}' got type '{}'",p_id,t_id)))
        } else if let AstroNode::AstroTuple(AstroTuple{id:_,length:len,contents:ref content}) = **p_arg {
            //unify( Rc::clone(t_mem), Rc::clone(p_arg), state, unifying )
            let mut unifiers = vec![];
            for i in 0..len {
                unifiers.append( &mut unify( Rc::clone(&t_mem.borrow()[i]) , Rc::clone(&content.borrow()[i]),state,unifying).unwrap());
            }
            Ok(unifiers)
        } else {
            unify( Rc::clone(&t_mem.borrow()[0]), Rc::new(AstroNode::AstroList(AstroList::new(1,Rc::new(RefCell::new(vec![Rc::clone(p_arg)]))))) , state, unifying )
        }
        
    } else if pattern_type == "index" {
        // list element lval access
        Ok( vec![(Rc::clone(&pattern),Rc::clone(&term))] )
    
    //} else if term_type == "id" && unifying {
        // variable in term not allowed when unifying
    //    let AstroNode::AstroID(AstroID{id:_,name:ref t_name}) = *term
    //        else {panic!("Unify: expected id.")};

    //    Err(  ("PatternMatchFailed",format!("variable '{}' in term not allowed.",t_name)))

    } else if pattern_type == "id" {

        let AstroNode::AstroID(AstroID{id:_,name:ref p_name}) = *pattern
            else {panic!("Unify: expected id.")};       

        if p_name == "_" {
            Ok( vec![] )
        } else {
            Ok( vec![(Rc::clone(&pattern),Rc::clone(&term))] )
        }

    } else if ["headtail","rawheadtail"].contains(&pattern_type) {


        let AstroNode::AstroList(AstroList{id:_,length:t_length,contents:ref t_contents}) = *term
            else {return( Err(("PatternMatchFailed",format!("head-tail operator expected type 'list' got type '{}'",peek(Rc::clone(&term))))))};
        // let AstroNode::AstroHeadTail(AstroHeadTail{id:_,ref head,ref tail}) = *pattern
        //     else {return( Err(("PatternMatchFailed",format!("Unify: expected head-tail."))))};
        let (head,tail) = match *pattern {
            AstroNode::AstroHeadTail(AstroHeadTail{id:_,ref head,ref tail}) => (head,tail),
            AstroNode::AstroRawHeadTail(AstroRawHeadTail{id:_,ref head,ref tail}) => (head,tail),
            _ => return Err(("PatternMatchFailed",format!("Unify: expected head-tail."))),
        };

        if t_length == 0 {
            return Err( ("PatternMatchFailed",format!("head-tail operator expected a non-empty list")));
        }

        let list_head = Rc::clone(&t_contents.borrow()[0]);
        let new_len = t_contents.borrow().len()- 1;
        let list_tail = Rc::new(AstroNode::AstroList(AstroList::new( new_len, Rc::new(RefCell::new(t_contents.borrow_mut().split_off(1))))));

        let mut unifiers = vec![];
        let mut unifier = match unify( Rc::clone(&list_head), Rc::clone(&head), state, unifying ) {
            Ok( x ) => x,
            Err( x ) => return Err(x),
        };
        unifiers.append( &mut unifier );
        let mut unifier = match unify( Rc::clone(&list_tail), Rc::clone(&tail), state, unifying ) {
            Ok( x ) => x,
            Err( x ) => return Err(x),
        };
        unifiers.append( &mut unifier );

        Ok(unifiers)
    
    } else if term_type == "list" || pattern_type == "list" {

        if term_type != "list" || pattern_type != "list" {
            Err( ("PatternMatchFailed", format!("term and pattern do not agree on list/tuple constructor")))
        } else {

            let AstroNode::AstroList(AstroList{id:_,length:t_length,contents:ref t_contents}) = *term
                else {panic!("Unify: expected list.")};
            let AstroNode::AstroList(AstroList{id:_,length:p_length,contents:ref p_contents}) = *pattern
                else {panic!("Unify: expected list.")};


            if t_length != p_length {
                Err( ("PatternMatchFailed", format!("term and pattern lists/tuples are not the same length")))
            } else {
                let mut unifiers = vec![];
                for i in 0..(t_length-1) {
                    let x = unify( Rc::clone( &t_contents.borrow()[i]), Rc::clone( &p_contents.borrow()[i]), state, unifying );
                    match x {
                        Ok(mut success) => unifiers.append( &mut success ),
                        Err(_) => return x,
                    }
                }
                check_repeated_symbols( &unifiers );
                Ok( unifiers )
            }
        }

    } else if pattern_type == "deref" {
        // can be an AST representing any computation
        // that produces a pattern.

        let AstroNode::AstroDeref( AstroDeref{id:_,expression:ref exp}) = *pattern
            else {panic!("Unify: expected deref")};

        let p = walk( Rc::clone(&exp),state).unwrap();

        unify(term,p,state,unifying)

    // builtin operators look like apply lists with operator names
    } else if pattern_type == "apply" {
        if term_type != "apply" {
            Err( ("PatternMatchFailed","term and pattern disagree on \'apply\' node".to_string()) )
        } else {

            // unpack the apply structures
            let AstroNode::AstroApply(AstroApply{id:_,function:ref p_func,argument:ref p_arg}) = *pattern
                else {panic!("Unify: expected apply.")};
            let AstroNode::AstroApply(AstroApply{id:_,function:ref t_func,argument:ref t_arg}) = *term
                else {panic!("Unify: expected apply.")};

            let AstroNode::AstroID(AstroID{id:_,name:ref p_id}) = **p_func
                else {panic!("Unify: expected id.")};
            let AstroNode::AstroID(AstroID{id:_,name:ref t_id}) = **t_func
                else {panic!("Unify: expected id.")};

            // make sure apply id's match
            if p_id != t_id {
                Err( ("PatternMatchFailed",format!("term '{}' does not match pattern '{}'",t_id,p_id) ))
            } else {
                // unify the args
                unify(Rc::clone(t_arg), Rc::clone(p_arg), state, unifying)
            }
        }
    } else if pattern_type == "constraint" {
        state.inc_constraint_lvl();
        unify(term,pattern,state,unifying);
        state.dec_constraint_lvl();
        Ok(vec![])
    
    } else if peek(Rc::clone(&term)) != peek(Rc::clone(&pattern)) {
        Err( ("PatternMatchFailed",format!("nodes '{}' and '{}' are not the same",peek(Rc::clone(&term)),peek(Rc::clone(&pattern)))))

    } else { 

        let mut unifier: Vec<(Rc<AstroNode>,Rc<AstroNode>)> = vec![];
        let mut len: usize;
        let mut content: Vec<Rc<AstroNode>>;

        if let AstroNode::AstroTuple(AstroTuple{id:_,length:t_len,contents:ref t_content}) = *term {
            if let AstroNode::AstroTuple(AstroTuple{id:_,length:p_len,contents:ref p_content}) = *pattern {

                for i in 0..t_len {
                    unifier.append( &mut unify( Rc::clone(&t_content.borrow()[i]),Rc::clone(&p_content.borrow()[i]),state,unifying).unwrap() );
                }
                Ok( unifier )
            } else {
                Err( ("PatternMatchFailed",format!("nodes '{}' and '{}' are not the same",peek(Rc::clone(&term)),peek(Rc::clone(&pattern)))))
            }
        } else if let AstroNode::AstroList(AstroList{id:_,length:t_len,contents:ref t_content}) = *term {
            if let AstroNode::AstroList(AstroList{id:_,length:p_len,contents:ref p_content}) = *pattern { 


                for i in 0..t_len {
                    unifier.append( &mut unify( Rc::clone(&t_content.borrow()[i]),Rc::clone(&p_content.borrow()[i]),state,unifying).unwrap() );
                }
                Ok( unifier )
            } else {
                Err( ("PatternMatchFailed",format!("nodes '{}' and '{}' are not the same",peek(Rc::clone(&term)),peek(Rc::clone(&pattern)))))
            }
        } else {
            Err( ("PatternMatchFailed",format!("nodes '{}' and '{}' are not the same",peek(Rc::clone(&term)),peek(Rc::clone(&pattern)))))
        }
    }
}


/******************************************************************************/
pub fn walk<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{ 

    //println!("Walking: {}",peek(Rc::clone(&node)));

    match *node {
        AstroNode::AstroInteger(_) => Ok(node),
        AstroNode::AstroReal(_) => Ok(node),
        AstroNode::AstroBool(_) => Ok(node),
        AstroNode::AstroString(_) => Ok(node),
        AstroNode::AstroLineInfo(_) => set_lineinfo(node, state),
        AstroNode::AstroList(_) => list_exp(node, state),
        AstroNode::AstroTuple(_) => tuple_exp(node, state),
        AstroNode::AstroNone(_) => Ok(node),
        AstroNode::AstroNil(_) => Ok(node),
        AstroNode::AstroFunction(_) => function_exp(node,state),
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
pub fn set_lineinfo<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
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

    for i in 0..length {
        let val = match walk( Rc::clone(&contents.borrow()[i]), state) {
            Ok( x ) => x,
            Err( x ) => return Err(x),
        };
        contents.borrow_mut()[i] = val;
    }
    Ok( node ) 
}
/******************************************************************************/
pub fn tuple_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroTuple( AstroTuple{id,length,ref contents} ) = *node 
        else { panic!("ERROR: walk: expected tuple in tuple_exp()") };

    for i in 0..length {
        let val = match walk( Rc::clone(&contents.borrow()[i]), state) {
            Ok( x ) => x,
            Err( x ) => return Err(x),
        };
        contents.borrow_mut()[i] = val;
    }
    Ok( node ) 
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
        newlist.push(Rc::new(AstroNode::AstroInteger(AstroInteger::new( i ))));
    }

    Ok( Rc::new(AstroNode::AstroList( AstroList::new(len,Rc::new(RefCell::new(newlist))))))
}
/******************************************************************************/
pub fn function_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroFunction(AstroFunction{id:_,ref body_list}) = *node
        else {panic!("ERROR: walk: expected function in function_exp()")};

    Ok( Rc::new(AstroNode::AstroFunctionVal(AstroFunctionVal::new(Rc::clone(body_list), Rc::new(state.symbol_table.get_config()) ))))
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
    for content in &*(contents.borrow()) {
        new_contents.push(content.to_owned());
    }

    Ok( Rc::new( AstroNode::AstroList( AstroList::new( length + 1, Rc::new(RefCell::new(new_contents)))))) 
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
        walk( Rc::clone(expression) ,state)
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
        else { panic!("ERROR: walk: expected apply expression in apply_exp().") }; 

    // handle builtin operators that look like apply lists.
    if let AstroNode::AstroID( AstroID{id:_,name:ref tag}) = **function {

        if OPERATOR_SYMBOLS.contains( &(tag.as_str()) ) {
            handle_builtins( Rc::clone(&node), state)

        } else{
            // handle function application
            let f_val = walk( Rc::clone(&function), state).unwrap();
            let f_name = tag;
            let arg_val = walk( Rc::clone(&argument), state).unwrap();

            let _type = peek( Rc::clone(&f_val));

            if _type == "functionval" {
                return handle_call( Rc::new(AstroNode::AstroNone(AstroNone::new())), Rc::clone(&f_val), Rc::clone(&arg_val), state );

            } else if _type == "struct" {
                // object constructor call

                let AstroNode::AstroStruct(AstroStruct{id:_,member_names:ref mnames,struct_memory:ref struct_mem}) = *f_val
                    else {panic!("Error: apply exp: expected struct.")};

                // create our object memory - memory cells now have initial values
                // we use structure memory as an init template
                let mut obj_memory = Rc::new(RefCell::new((struct_mem.borrow()).clone()));
                let new_id = AstroID::new(tag.to_string());
                //let new_mem = AstroNode::AstroList(AstroList::new(obj_memory.len(), Rc::new(obj_memory)).unwrap());
                let obj_ref = Rc::new(AstroNode::AstroObject(AstroObject::new(new_id,Rc::clone(&obj_memory))));

                for element in (&*mnames.borrow()) {
                    if let AstroNode::AstroID(AstroID{id:_,name:ref tag}) = *Rc::clone(&element) {
                        if tag == "__init__" {
                            // handle constructor call
                            let slot_ix = (&*mnames.borrow()).iter().position(|x| x == element);
                            let init_fval = Rc::clone( &struct_mem.borrow()[ slot_ix.unwrap() ] );
                            handle_call( Rc::clone(&obj_ref), Rc::clone(&init_fval), Rc::clone(&arg_val), state);
                            return Ok( Rc::clone(&obj_ref) )
                        }
                    } 
                }

                // the struct does not have an __init__ function but
                // we have a constructor call with args, e.g. Foo(1,2)
                // try to apply a default constructor by copying the
                // values from the arg list to the data slots of the object

                let AstroNode::AstroTuple(AstroTuple{id:_,length:len,contents:ref content}) = *arg_val
                    else {panic!("Error: apply exp: expected tuple.")};
                
                
                let data_memory = data_only( RefCell::clone(&obj_memory) );

                if content.borrow().len() != data_memory.len() {
                    return Err(("ValueError",format!("default constructor expected {} arguments got {}",content.borrow().len(),data_memory.len())));
                } else {
                    let data_ix = data_ix_list( RefCell::clone(&obj_memory) );
                    for i in 0..content.borrow().len() {
                        obj_memory.borrow_mut()[ data_ix[i] ] = Rc::clone( &content.borrow()[ i ] );
                    }
                }
                return Ok(Rc::clone(&obj_ref)); 
            }
            Ok(node) 
        }
    } else {
        // Error?
        Ok(node)
    }
}
/******************************************************************************/
pub fn handle_call<'a>( obj_ref: Rc<AstroNode>, node: Rc<AstroNode>, args: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{

    let AstroNode::AstroFunctionVal(AstroFunctionVal{id:_,body_list:ref fpointer,ref closure}) = *node
        else {panic!("ERROR: handle call: expected function value.")};

    let AstroNode::AstroID(AstroID{id:_,name:ref fname}) = **fpointer
        else {panic!("ERROR: handle_call: expected id for function name.")};

    // static scoping for functions
    // Note: we have to do this here because unifying
    // over the body patterns can introduce variable declarations,
    // think conditional pattern matching.
    let save_symtab = state.symbol_table.get_config();
    //state.symbol_table.set_config( closure.0.clone(), closure.1.clone(), closure.2 );
    state.push_scope();

    if let AstroNode::AstroNone(AstroNone{id:_}) = *obj_ref {
        ;
    } else {
        state.enter_sym( "this", obj_ref );
    }
    // execute the function
    // function calls transfer control - save our caller's lineinfo
    let old_lineinfo = state.lineinfo.clone();
    let return_value = state.dispatch_table[ fname.as_str() ]( args, state );

    //  coming back from a function call - restore caller's lineinfo
    state.lineinfo = old_lineinfo;

    // NOTE: popping the function scope is not necessary because we
    // are restoring the original symtab configuration. this is necessary
    // because a return statement might come out of a nested with statement
    //state.symbol_table.set_config(save_symtab.0, save_symtab.1, save_symtab.2);
    return_value
}
/******************************************************************************/
pub fn handle_builtins<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{

    let AstroNode::AstroApply(AstroApply{id,ref function,ref argument}) = *node 
        else { panic!("ERROR: handle_builtins: expected apply expression.") }; 
    let AstroNode::AstroID( AstroID{id:_,name:ref builtin_type} ) = **function
        else { panic!("ERROR: handle_builtins: expected id. ")};

    if BINARY_OPERATORS.contains( &builtin_type.as_str() ) {
        
        let AstroNode::AstroTuple( AstroTuple{id:_,length:2,contents:ref args}) = **argument
            else {panic!("ERROR: handle_builtins: expected tuple for args.")};

        let val_a = walk( Rc::clone(&args.borrow()[0]), state ).unwrap();
        let val_b = walk( Rc::clone(&args.borrow()[1]), state ).unwrap();
        
        if builtin_type == "__plus__" {
            
            if let AstroNode::AstroInteger( AstroInteger{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroInteger( AstroInteger::new(v1+v2))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 as f64 + v2))));
                } else if let AstroNode::AstroString( AstroString{id:_,value:ref v2}) = *val_b {
                        return Ok( Rc::new( AstroNode::AstroString(AstroString::new(v1.to_string()+v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let AstroNode::AstroReal( AstroReal{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 + v2 as f64))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 + v2))));
                } else if let AstroNode::AstroString( AstroString{id:_,value:ref v2}) = *val_b {
                    return Ok( Rc::new( AstroNode::AstroString(AstroString::new(v1.to_string()+v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let AstroNode::AstroList( AstroList{id:_,length:len1,contents:ref c1}) = *val_a {
                if let AstroNode::AstroList( AstroList{id:_,length:len2,contents:ref c2}) = *val_b {
                    let mut c3 = (**c1).clone(); // we have to do a data-clone here otherwise we edit other nodes in place
                    c3.borrow_mut().append( &mut (*c2.borrow_mut())) ;
                    return Ok( Rc::new( AstroNode::AstroList( AstroList::new(len1+len2,Rc::new( c3 )))));
                } 
                
            } else if let AstroNode::AstroString( AstroString{id:_,value:ref v1}) = *val_a {
                if let AstroNode::AstroString( AstroString{id:_,value:ref v2}) = *val_b {
                    return Ok( Rc::new( AstroNode::AstroString(AstroString::new(v1.to_owned()+v2))));
                } else if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new( AstroNode::AstroString(AstroString::new(v1.to_owned()+&v2.to_string()))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new( AstroNode::AstroString(AstroString::new(v1.to_owned()+&v2.to_string()))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else {
                return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
            }

        } else if builtin_type == "__minus__" {

            if let AstroNode::AstroInteger( AstroInteger{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroInteger( AstroInteger::new(v1 - v2))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 as f64 - v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let AstroNode::AstroReal( AstroReal{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 - v2 as f64))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 - v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only subtract real/integers
                return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
            }

        } else if builtin_type == "__times__" {

            if let AstroNode::AstroInteger( AstroInteger{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroInteger( AstroInteger::new(v1 * v2))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 as f64 * v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let AstroNode::AstroReal( AstroReal{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 * v2 as f64))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 * v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only multiply real/integers
                return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
            }    
        } else if builtin_type == "__divide__" {

            if let AstroNode::AstroInteger( AstroInteger{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroInteger( AstroInteger::new(v1 / v2))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 as f64 / v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let AstroNode::AstroReal( AstroReal{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 / v2 as f64))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroReal( AstroReal::new(v1 / v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only divide real/integers
                return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
            }    
        } else if builtin_type == "__or__" {

            let b1 = map2boolean( &val_a).unwrap();
            let b2 = map2boolean( &val_b).unwrap();
            let AstroNode::AstroBool( AstroBool{id:_,value:b1_val}) = b1
                else {panic!("handle_builtins: expected boolean.")};
            let AstroNode::AstroBool( AstroBool{id:_,value:b2_val}) = b2
                else {panic!("handle_builtins: expected boolean.")};

            return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(b1_val || b2_val))));
        } else if builtin_type == "__and__" {

            let b1 = map2boolean( &val_a).unwrap();
            let b2 = map2boolean( &val_b).unwrap();
            let AstroNode::AstroBool( AstroBool{id:_,value:b1_val}) = b1
                else {panic!("handle_builtins: expected boolean.")};
            let AstroNode::AstroBool( AstroBool{id:_,value:b2_val}) = b2
                else {panic!("handle_builtins: expected boolean.")};

            return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(b1_val && b2_val))));
        } else if builtin_type == "__gt__" {

            if let AstroNode::AstroInteger( AstroInteger{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(v1 > v2))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(v1 as f64 > v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let AstroNode::AstroReal( AstroReal{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(v1 > v2 as f64))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(v1 > v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only subtract real/integers
                return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
            }

        } else if builtin_type == "__lt__" {
         
            if let AstroNode::AstroInteger( AstroInteger{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(v1 < v2))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new((v1 as f64) < v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let AstroNode::AstroReal( AstroReal{id:_,value:v1}) = *val_a {
                if let AstroNode::AstroInteger( AstroInteger{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(v1 < v2 as f64))));
                } else if let AstroNode::AstroReal( AstroReal{id:_,value:v2}) = *val_b {
                    return Ok( Rc::new(AstroNode::AstroBool( AstroBool::new(v1 < v2))));
                } else {
                    return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only subtract real/integers
                return Err( ("ValueError", format!("Unsuppoted type {} in +", peek(Rc::clone(&val_b)))));
            }

        }
    
    

        
    }
    Ok(node)
}
/******************************************************************************/
pub fn index_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroIndex(AstroIndex{id,ref structure,ref index_exp}) = *node 
        else { panic!("ERROR: walk: expected index expression in index_exp().") }; 

    // look at the semantics of 'structure'
    let structure_val = walk(Rc::clone(&structure),state).unwrap();

    // indexing/slicing
    let result = read_at_ix(structure_val,Rc::clone(&index_exp),state).unwrap();

    
    Ok(result)
}
/******************************************************************************/
pub fn escape_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{

    let AstroNode::AstroEscape(AstroEscape{id:_,content:ref fname}) = *node
        else {panic!("escape_exp(): expected ID.")};
    
    let old_lineinfo = state.lineinfo.clone();
    let return_value = state.dispatch_table[ fname.as_str() ]( Rc::new(AstroNode::AstroNone(AstroNone::new())), state );

    //  coming back from a function call - restore caller's lineinfo
    state.lineinfo = old_lineinfo;

    return_value
}
/******************************************************************************/
pub fn is_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroIs(AstroIs{id,ref pattern,ref term}) = *node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    let term_val = walk((*term).clone(), state).unwrap();
    let unifiers = unify(term_val,(*pattern).clone(),state,true);

    if let Err(_) = unifiers {
        Ok( Rc::new( AstroNode::AstroBool(AstroBool::new(false))))
    } else {
        declare_unifiers(&unifiers.unwrap(),state);
        Ok( Rc::new( AstroNode::AstroBool(AstroBool::new(true))))
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
    if (*contents).borrow().contains(&exp_val) {
        Ok( Rc::new( AstroNode::AstroBool(AstroBool::new(true))))
    } else {
        Ok( Rc::new( AstroNode::AstroBool(AstroBool::new(false))))
    }
}
/******************************************************************************/
pub fn if_exp<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{
    let AstroNode::AstroIf(AstroIf{id,ref cond_exp,ref then_exp,ref else_exp}) = *node 
        else { panic!("ERROR: walk: expected id expression in id_exp().") }; 

    let cond_val = map2boolean(&walk( Rc::clone(&cond_exp), state ).unwrap()).unwrap();
    let AstroNode::AstroBool(AstroBool{id,value}) = cond_val 
        else {panic!("Expected boolean from map2boolean.")};
    
    if value {
        walk(Rc::clone(&then_exp),state)
    } else {
        walk(Rc::clone(&else_exp),state)
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
fn check_repeated_symbols(unifiers: &Vec<(Rc<AstroNode>,Rc<AstroNode>)> ) -> bool {
    let len = unifiers.len();
    let mut seen = Vec::with_capacity(len);

    for i in 0..len {
        let next = peek( (unifiers[i].0).clone() );

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
pub fn declare_unifiers<'a>( unifiers: &Vec<(Rc<AstroNode>,Rc<AstroNode>)>, state: &'a mut State ) -> Result<(), (&'static str,String) >{
    // walk the unifiers and bind name-value pairs into the symtab

    for (lhs,value) in unifiers {

        if let AstroNode::AstroID(AstroID{id:_,ref name}) = **lhs {
            if name == "this" {
                return Err(("ValueError","'this' is a reserved keyword.".to_string()));
            } else {
                state.enter_sym(&name,Rc::clone(value));
            }
        } else if let AstroNode::AstroIndex(AstroIndex{id:_,ref structure,ref index_exp}) = **lhs {
            // Note: structures have to be declared before index access
            // can be successful!!  They have to be declared so that there
            // is memory associated with the structure.

            // indexing/slicing
            // update the memory of the object.
            store_at_ix(Rc::clone(structure),Rc::clone(index_exp),Rc::clone(value),state);
        } else {
            return Err(("ValueError",format!("unknown unifier type '{}'",peek(Rc::clone(lhs)))));
        }
    }
    Ok(())
}
/******************************************************************************/
pub fn declare_formal_args<'a>( unifiers: &Vec<(Rc<AstroNode>,Rc<AstroNode>)>, state: &'a mut State ) -> Result<(), (&'static str,String) >{
    // unfiers is of the format: [ (pattern, term), (pattern, term),...]

    for (pattern,term) in unifiers {
        if let AstroNode::AstroID(AstroID{id:_,ref name}) = **pattern {
            if name == "this" {
                return Err(("ValueError","'this' is a reserved keyword.".to_string()));
            } else {
                state.enter_sym(&name,Rc::clone(term));
            }
        } else {
            return Err(("ValueError",format!("unknown unifier type '{}'",peek(Rc::clone(pattern)))));
        }
    }
    Ok(())
}
/******************************************************************************/
pub fn store_at_ix<'a>( structure: Rc<AstroNode>, ix: Rc<AstroNode>, value: Rc<AstroNode>, state: &'a mut State ) -> Result<(), (&'static str,String)>{

    let mut structure_val = Rc::new(AstroNode::AstroNone(AstroNone::new()));
    
    // Handle recurive application iteratively here.
    if let AstroNode::AstroIndex(AstroIndex{id:_,structure:ref s,index_exp:ref idx}) = *structure {

        let mut inner_mem = Rc::clone(s);

        // Construct a list of all of the indices
        let ix_val = walk(Rc::clone(&ix), state).unwrap();
        let AstroNode::AstroInteger(AstroInteger{id:_,value:v}) = *ix_val
            else {panic!("store_at_ix: expected integer index.")};
        let mut idx_list = vec![ v ];
        while let AstroNode::AstroIndex(AstroIndex{id:_,structure:ref s,index_exp:ref idx}) = **s {
            let AstroNode::AstroInteger(AstroInteger{id:_,value:v}) = *ix_val
                else {panic!("store_at_ix: expected integer index.")};
            idx_list.push(v);
            inner_mem = Rc::clone(s);
        }

        // Walk through the index list accessing memory until we reach the intended interior memory.
        let mut memory = walk(Rc::clone(&inner_mem),state).unwrap();
        for val in idx_list {
            memory = match *memory {
                AstroNode::AstroList( AstroList{id:_,length:l,contents:ref mem} ) => Rc::clone(&(**mem).borrow()[ val as usize ]),
                AstroNode::AstroTuple( AstroTuple{id:_,length:l,contents:ref mem} ) => Rc::clone(&(**mem).borrow()[ val as usize ]),
                _ => panic!{"store_at_ix: expected list or tuple."}
            };
        }
        structure_val = walk(Rc::clone(&memory),state).unwrap();
        
    } else {

        // look at the semantics of 'structure'
        structure_val = walk(Rc::clone(&structure),state).unwrap();
    }

    if let AstroNode::AstroList( AstroList{id:_,length:l,contents:ref mem} ) = *structure_val {

        let ix_val = walk(Rc::clone(&ix), state).unwrap();
        let AstroNode::AstroInteger(AstroInteger{id:_,value:int_val}) = *ix_val // TODO error clean up
            else {panic!("store_at_ix: expected integer.")};

        mem.borrow_mut()[int_val as usize] = Rc::clone(&value);
    
        Ok(()) 
    } else if let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref id,object_memory:ref mem}) = *structure_val {
        
        //let ix_val = walk(Rc::clone(&ix), state).unwrap();
        //println!("TYPE IS {}",peek(Rc::clone(&ix)));
        let AstroNode::AstroID(AstroID{id:_,name:ref tag}) = *ix
            else {panic!("store_at_ix: expected id.")};

        let AstroID{id:_,name:ref obj_type} = *id;
        let object_data = match walk( Rc::new(AstroNode::AstroID(id.clone())), state ) {
            Ok( val ) => val,
            Err( error ) => return Err( error ),
        };

        let AstroNode::AstroStruct(AstroStruct{id:_,member_names:ref struct_tags,struct_memory:ref struct_mem}) = *object_data
            else {panic!("store_at_ix: expected struct.")};

        // find the location in the structs memory where we want to place the new value.
        let mut found_idx = 0usize;
        let mut found = false;
        let mut curr_idx = 0usize;
        for struct_member in (*struct_tags).borrow().iter() {
            if let AstroNode::AstroID(AstroID{id:_,name:ref mem_tag}) = **struct_member {
                if mem_tag == tag {
                    found_idx = curr_idx;
                    found = true;
                }
            }
            curr_idx = curr_idx + 1;
        }
        
        //(mem.borrow_mut())[ found_idx ] = Rc::new( AstroNode::AstroNone(AstroNone::new()) );
        (mem.borrow_mut())[ found_idx ] = Rc::clone( &value );

        Ok(()) 
    } else {
        Err(("ValueError",format!("Index op not supported for '{}'",peek(structure_val))))
    }
}
/******************************************************************************/
pub fn read_at_ix<'a>( structure_val: Rc<AstroNode>, ix: Rc<AstroNode>, state: &'a mut State ) -> Result<Rc<AstroNode>, (&'static str,String)>{

    // find the actual memory we need to access
    let struct_type = peek(structure_val.clone());
    let ix_type = peek(Rc::clone(&ix));
    
    if ["list","tuple"].contains( &struct_type ) {
        if ix_type == "integer" {
            
            let AstroNode::AstroInteger(AstroInteger{id:_,value:ix_val}) = *ix
                else {panic!("read_at_ix: expected integer.")};

            let content = match *structure_val {
                AstroNode::AstroList( AstroList{id:_,length:len,contents:ref c}) => c,
                AstroNode::AstroTuple( AstroTuple{id:_,length:len,contents:ref c}) => c,
                _ => panic!("read_at_ix: expected list or tuple."),
            };

            
            return Ok( Rc::clone( &content.borrow()[ix_val as usize] ) );
        }
    } else if struct_type == "object" {

        let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref id,object_memory:ref mem}) = *structure_val
            else {panic!("read_at_ix: expected object.")};

        let AstroNode::AstroID(AstroID{id:_,name:ref tag}) = *ix
            else {panic!("read_at_ix: expected id.")};

        let AstroID{id:_,name:ref obj_type} = *id;
        let object_data = match walk( Rc::new(AstroNode::AstroID(id.clone())), state ) {
            Ok( val ) => val,
            Err( error ) => return Err( error ),
        };

        let AstroNode::AstroStruct(AstroStruct{id:_,member_names:ref struct_tags,struct_memory:ref struct_mem}) = *object_data
            else {panic!("read_at_ix: expected struct.")};

        // find the location in the structs memory where we want to place the new value.
        let mut found_idx = 0usize;
        let mut found = false;
        let mut curr_idx = 0usize;
        for struct_member in (*struct_tags).borrow().iter() {
            if let AstroNode::AstroID(AstroID{id:_,name:ref mem_tag}) = **struct_member {
                if mem_tag == tag {
                    found_idx = curr_idx;
                    found = true;
                }
            }
            curr_idx = curr_idx + 1;
        }
        
        //(mem.borrow_mut())[ found_idx ] = Rc::new( AstroNode::AstroNone(AstroNone::new()) );
        return Ok( Rc::clone( &mem.borrow_mut()[ found_idx ]) );

    
    } else if struct_type == "string" {

        let AstroNode::AstroInteger(AstroInteger{id:_,value:ix_val}) = *ix
                else {panic!("read_at_ix: expected integer.")};

        let content = match *structure_val {
            AstroNode::AstroString( AstroString{id:_,value:ref val}) => val,
            _ => panic!("read_at_ix: expected string."),
        };

        match content.chars().nth( ix_val as usize) {
            Some( character ) => return Ok(Rc::new(AstroNode::AstroString(AstroString::new(character.to_string())))),
            _                 => return Err( ("ValueError",format!("String '{}' too short for index value {}",content,ix_val)) )
        }
    }
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

/******************************************************************************/
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unify_regex() {
        let s1 = Rc::new( AstroNode::AstroString( AstroString::new(String::from("hello"))) );
        let s2 = Rc::new( AstroNode::AstroString( AstroString::new(String::from("hello"))) );
        let s3 = Rc::new( AstroNode::AstroString( AstroString::new(String::from("nothello"))) );

        let mut state = State::new().unwrap();
        let u = true;
        
        let out = unify(s1.clone(),s2,&mut state,u).unwrap();
        assert_eq!(out.len(),0); //SHOULD PASS

        let out = unify(s1,s3,&mut state,u);
        match out {
            Err(x) => (), //SHOULD BE ERR
            _ => panic!("Regex text failed"),
        }
    }
    #[test]
    fn test_unify_primitives() {
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let i3 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));

        let b1 = Rc::new( AstroNode::AstroBool( AstroBool::new(true)));
        let b2 = Rc::new( AstroNode::AstroBool( AstroBool::new(false)));
        let b3 = Rc::new( AstroNode::AstroBool( AstroBool::new(true)));

        let r1 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.1)));
        let r2 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.2)));
        let r3 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.1)));

        let mut state = State::new().unwrap();
        let u_mode = true;

        let out1 = unify(i1.clone(),i3,&mut state,u_mode).unwrap();
        let out2 = unify(b1.clone(),b3,&mut state,u_mode).unwrap();
        let out3 = unify(r1.clone(),r3,&mut state,u_mode).unwrap();

        assert_eq!(out1.len(),0); //SHOULD PASS
        assert_eq!(out2.len(),0); //SHOULD PASS
        assert_eq!(out3.len(),0); //SHOULD PASS

        let out1 = unify(i1.clone(),i2,&mut state,u_mode);
        let out2 = unify(b1.clone(),b2,&mut state,u_mode);
        let out3 = unify(r1.clone(),r2,&mut state,u_mode);

        match out1 {
            Err(x) => (), //SHOULD BE ERR
            _ => panic!("Primitive unify test failed"),
        }
        match out2 {
            Err(x) => (), //SHOULD BE ERR
            _ => panic!("rimitive unify test failed"),
        }
        match out3 {
            Err(x) => (), //SHOULD BE ERR
            _ => panic!("rimitive unify test failed"),
        }
    }
    #[test]
    fn test_unify_intlists() {
        let mut state = State::new().unwrap();
        let u_mode = true;

        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let i3 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(3)));

        let l1 = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));
        let l2 = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![i2.clone(),i3.clone()])))));
        let l3 = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![i3.clone(),i2.clone(),i1.clone()])))));
        let l4 = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));

        let out1 = unify( Rc::clone(&l1),Rc::clone(&l4),&mut state,u_mode ).unwrap(); //Should pass unwrapping
        let out2 = unify( Rc::clone(&l1),Rc::clone(&l2),&mut state,u_mode );
        let out3 = unify( Rc::clone(&l1),Rc::clone(&l3),&mut state,u_mode );

        match out2 {
            Ok(_) => panic!("test failed."),
            Err(_) => (),
        }
        match out3 {
            Ok(_) => panic!("test failed."),
            Err(_) => (),
        }

    }
    #[test]
    fn test_walk_lineinfo() {
        let newline = AstroLineInfo::new( String::from("test1"),123 );
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

        let newline = AstroLineInfo::new( String::from("math"), 987654321);
        walk( Rc::new(  AstroNode::AstroLineInfo(newline)),&mut state );

        {
            let out3 = state.lineinfo.clone();
            assert_eq!(out3,(String::from("math"), 987654321));
        }
    }
    #[test]
    fn test_unify_var_to_int() {
        // let x = 123.

        let mut state = State::new().unwrap();
        let var = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let int = Rc::new(AstroNode::AstroInteger(AstroInteger::new(123)));

        let out = declare_unifiers( &unify(int,var,&mut state,true).unwrap(), &mut state );

        let check = state.lookup_sym("x",true);
        match *check {
            AstroNode::AstroInteger(AstroInteger{id:_,value:123}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_real() {
        // let x = 1.23.

        let mut state = State::new().unwrap();
        let var = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let val = Rc::new(AstroNode::AstroReal(AstroReal::new(1.23)));

        let out = declare_unifiers( &unify(val,var,&mut state,true).unwrap(), &mut state );

        let check = state.lookup_sym("x",true);
        match *check {
            AstroNode::AstroReal(AstroReal{id:_,value:val}) if val == 1.23 => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_string() {
        // let x = "hello123".

        let mut state = State::new().unwrap();
        let var = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let val = Rc::new(AstroNode::AstroString(AstroString::new("hello123".to_string())));

        let out = declare_unifiers( &unify(val,var,&mut state,true).unwrap(), &mut state );

        let check = state.lookup_sym("x",true);
        match *check {
            AstroNode::AstroString(AstroString{id:_,value:ref val}) if val == "hello123" => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_bool() {
        // let x = false.

        let mut state = State::new().unwrap();
        let var = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let val = Rc::new(AstroNode::AstroBool(AstroBool::new(false)));

        let out = declare_unifiers( &unify(val,var,&mut state,true).unwrap(), &mut state );

        let check = state.lookup_sym("x",true);
        match *check {
            AstroNode::AstroBool(AstroBool{id:_,value:val}) if val == false =>(),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_int_thrice() {
        // let x = 2.
        // let y = 4.
        // let z = 8.

        let mut state = State::new().unwrap();
        let var1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(2)));
        let var2 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
        let val2 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(4)));
        let var3 = Rc::new(AstroNode::AstroID(AstroID::new("z".to_string())));
        let val3 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(8)));

        declare_unifiers( &unify(val1,var1,&mut state,true).unwrap(), &mut state );
        declare_unifiers( &unify(val2,var2,&mut state,true).unwrap(), &mut state );
        declare_unifiers( &unify(val3,var3,&mut state,true).unwrap(), &mut state );

        let check1 = state.lookup_sym("x",true);
        let check2 = state.lookup_sym("y",true);
        let check3 = state.lookup_sym("z",true);
        match *check1 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:2}) => (),
            _ => panic!("test failed"),
        };
        match *check2 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:4}) => (),
            _ => panic!("test failed"),
        };
        match *check3 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:8}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_varlist_to_intlist() {
        // let [x,y] = [3,4].

        let mut state = State::new().unwrap();
        let var1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(3)));
        let var2 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
        let val2 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(4))); 
        let varlist = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&var1),Rc::clone(&var2)])))));
        let vallist = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&val1),Rc::clone(&val2)])))));

        declare_unifiers( &unify(vallist,varlist,&mut state,true).unwrap(), &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:3}) => (),
            _ => panic!("test failed"),
        };
        let check2 = state.lookup_sym("y",true);
        match *check2 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:4}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_varlist_to_multilist() {
        // let [x,y,3] = ["string1",1.3334,3].

        let mut state = State::new().unwrap();
        let var1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(AstroNode::AstroString(AstroString::new("string1".to_string())));
        let var2 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
        let val2 = Rc::new(AstroNode::AstroReal(AstroReal::new(1.3334)));
        let int1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(3))); 
        let int2 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(3)));
        let varlist = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&var1),Rc::clone(&var2),Rc::clone(&int1)])))));
        let vallist = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&val1),Rc::clone(&val2),Rc::clone(&int2)])))));

        declare_unifiers( &unify(vallist,varlist,&mut state,true).unwrap(), &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            AstroNode::AstroString(AstroString{id:_,value:ref val}) if val == "string1" => (),
            _ => panic!("test failed"),
        };
        let check2 = state.lookup_sym("y",true);
        match *check2 {
            AstroNode::AstroReal(AstroReal{id:_,value:val}) if val == 1.3334 => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_vartuple_to_inttuple() {
        // let (x,y,z) = (2,3,4).

        let mut state = State::new().unwrap();
        let var1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(2)));
        let var2 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
        let val2 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(3))); 
        let var3 = Rc::new(AstroNode::AstroID(AstroID::new("z".to_string())));
        let val3 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(4))); 
        let varlist = Rc::new( AstroNode::AstroTuple( AstroTuple::new(3,Rc::new(RefCell::new(vec![Rc::clone(&var1),Rc::clone(&var2),Rc::clone(&var3)])))));
        let vallist = Rc::new( AstroNode::AstroTuple( AstroTuple::new(3,Rc::new(RefCell::new(vec![Rc::clone(&val1),Rc::clone(&val2),Rc::clone(&val3)])))));

        declare_unifiers( &unify(vallist,varlist,&mut state,true).unwrap(), &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:2}) => (),
            _ => panic!("test failed"),
        };
        let check2 = state.lookup_sym("y",true);
        match *check2 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:3}) => (),
            _ => panic!("test failed"),
        };
        let check2 = state.lookup_sym("z",true);
        match *check2 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:4}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_var() {
        // let x = 234.
        // let y = x.

        let mut state = State::new().unwrap();
        let var1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(234)));
        let var2 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));

        declare_unifiers( &unify(val1,Rc::clone(&var1),&mut state,true).unwrap(), &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:234}) => (),
            _ => panic!("test failed"),
        };

        declare_unifiers( &unify(Rc::clone(&var1),var2,&mut state,true).unwrap(), &mut state );

        let check2 = state.lookup_sym("y",true);
        match *check2 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:234}) => (),
            AstroNode::AstroInteger(AstroInteger{id:_,value:v}) => println!("{}",v),
            _ =>    println!("DEBUG: {}", peek(Rc::clone(&check2))),
        };

    }
    #[test]
    fn test_unify_int_to_namedpattern() {
        // let x:%integer = 17.

        let mut state = State::new().unwrap();
        let var1 = AstroID::new("x".to_string());
        let pmatch_type = Rc::new(AstroNode::AstroString( AstroString::new( "integer".to_string())));
        let pmatch = Rc::new(AstroNode::AstroTypeMatch(AstroTypeMatch::new(pmatch_type)));
        let p = Rc::new(AstroNode::AstroNamedPattern(AstroNamedPattern::new(var1,pmatch)));
        let val1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(17)));

        declare_unifiers( &unify(val1,p,&mut state,true).unwrap(), &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:17}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_index_to_int() {
        // let x = [1,0,3].
        // let x@1 = 2.

        let mut state = State::new().unwrap();
        let var1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(0)));
        let i3 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(3)));
        let i4 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let l1 = Rc::new( AstroNode::AstroList( AstroList::new(3,Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));
        let idx_exp = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));

        declare_unifiers( &unify(Rc::clone(&l1),Rc::clone(&var1),&mut state,true).unwrap(), &mut state );

        let idx1 = Rc::new( AstroNode::AstroIndex( AstroIndex::new( Rc::clone(&var1), Rc::clone(&idx_exp) )));

        declare_unifiers( &unify(Rc::clone(&i4),Rc::clone(&idx1),&mut state,true).unwrap(), &mut state );
        let check1 = state.lookup_sym("x",true);

        let AstroNode::AstroList(AstroList{id:_,length:3,contents:ref c}) = *check1
            else {panic!("test failed")};
        
        if let AstroNode::AstroInteger(AstroInteger{id:_,value:2}) = *c.borrow()[1] {
            ();
        } else {
            panic!("test failed");
        };
    }
    #[test]
    fn test_prog_addition_int_to_int() {
        // program
        // let a = 1 + 1.

        // python compiler:
        // set_lineinfo('prog.ast',1)
        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('integer', 1), ('integer', 1)])))
        // unifiers = unify(exp_val,('id', 'a'))
        // declare_unifiers(unifiers)

        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('integer', 1), ('integer', 1)])))
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroInteger(AstroInteger{id:_,value:2}) = *check1
            else {panic!("test failed")};
    }
    #[test]
    fn test_prog_addition_int_to_real() {
        // program
        // let a = 1 + 1.

        // python compiler:
        // set_lineinfo('prog.ast',1)
        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('integer', 1), ('real', 1.1)])))
        // unifiers = unify(exp_val,('id', 'a'))
        // declare_unifiers(unifiers)

        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('integer', 1), ('real', 1.1)])))
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let r1 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.1)));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&r1)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroReal(AstroReal{id:_,value:v}) = *check1
            else {panic!("test failed")};
        if v == 2.1 {
            ();
        } else {
            panic!("test failed");
        }
    }
    #[test]
    fn test_prog_addition_real_to_int() {
        // program
        // let a = 1 + 1.

        // python compiler:
        // set_lineinfo('prog.ast',1)
        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('real', 1.35), ('integer', 1)])))
        // unifiers = unify(exp_val,('id', 'a'))
        // declare_unifiers(unifiers)

        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple',  [('real', 1.35), ('integer', 1)])))
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let r1 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.35)));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&r1),Rc::clone(&i1)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroReal(AstroReal{id:_,value:v}) = *check1
            else {panic!("test failed")};
        if v == 2.35 {
            ();
        } else {
            panic!("test failed");
        }
    }
    #[test]
    fn test_prog_addition_real_to_real() {
        // program
        // let a = 1 + 1.

        // python compiler:
        // set_lineinfo('prog.ast',1)
        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('real', 1.35), ('real', 2.15)])))
        // unifiers = unify(exp_val,('id', 'a'))
        // declare_unifiers(unifiers)

        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple',  [('real', 1.35), ('real', 2.15)])))
        let r1 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.35)));
        let r2 = Rc::new( AstroNode::AstroReal( AstroReal::new(2.15)));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&r1),Rc::clone(&r2)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroReal(AstroReal{id:_,value:v}) = *check1
            else {panic!("test failed")};
        if v == 3.5 {
            ();
        } else {
            panic!("test failed");
        }
    }
    #[test]
    fn test_prog_addition_list_to_list() {
        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('list', [('integer', 1), ('integer', 2)]), ('list', [('integer', 3), ('integer', 4)])])))
        // unifiers = unify(exp_val,('id', 'b'))
        // declare_unifiers(unifiers)

        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let i3 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(3)));
        let i4 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(4)));
        let l1 = Rc::new( AstroNode::AstroList( AstroList::new(2,Rc::new(RefCell::new(vec![i1.clone(),i2.clone()])))));
        let l2 = Rc::new( AstroNode::AstroList( AstroList::new(2,Rc::new(RefCell::new(vec![i3.clone(),i4.clone()])))));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&l1),Rc::clone(&l2)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroList(AstroList{id:_,length:4,contents:ref c}) = *check1
            else {panic!("test failed")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:1}) = *c.borrow()[0]
            else {panic!("test failed")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:2}) = *c.borrow()[1]
            else {panic!("test failed")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:3}) = *c.borrow()[2]
            else {panic!("test failed")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:4}) = *c.borrow()[3]
            else {panic!("test failed")};
    }
    #[test]
    fn test_prog_addition_string_to_string() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( AstroNode::AstroString( AstroString::new("Hello ".to_string())));
        let s2 = Rc::new( AstroNode::AstroString( AstroString::new("World!".to_string())));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&s1),Rc::clone(&s2)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroString(AstroString{id:_,value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello World!",v);
    }
    #[test]
    fn test_prog_addition_string_to_int() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( AstroNode::AstroString( AstroString::new("Hello ".to_string())));
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(123)));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&s1),Rc::clone(&i1)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroString(AstroString{id:_,value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello 123",v);
    }
    #[test]
    fn test_prog_addition_string_to_real() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( AstroNode::AstroString( AstroString::new("Hello ".to_string())));
        let r1 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.23)));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&s1),Rc::clone(&r1)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroString(AstroString{id:_,value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello 1.23",v);
    }
    #[test]
    fn test_prog_addition_int_to_string() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( AstroNode::AstroString( AstroString::new(" Hello".to_string())));
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(123)));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&s1)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroString(AstroString{id:_,value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("123 Hello",v);
    }
    #[test]
    fn test_prog_addition_real_to_string() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( AstroNode::AstroString( AstroString::new(" Hello".to_string())));
        let r1 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.23)));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&r1),Rc::clone(&s1)])))));
        let id1 = Rc::new( AstroNode::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = walk( Rc::clone( &apply1), &mut state ).unwrap();

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( AstroNode::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true).unwrap();

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroString(AstroString{id:_,value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("1.23 Hello",v);
    }
    #[test]
    fn test_prog_firstclass_conditional_patternmatch() {
        //let POS_INT = pattern with x if x > 0.
        //let x: *POS_INT = 2.

        /*
        set_lineinfo('prog.txt',1)
        exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__gt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('id', 'x'), ('null',))))
        unifiers = unify(exp_val,('id', 'POS_INT'))
        declare_unifiers(unifiers)

        set_lineinfo('prog.txt',2)
        exp_val = walk(('integer', 2))
        unifiers = unify(exp_val,('named-pattern', ('id', 'x'), ('deref', ('id', 'POS_INT'))))
        declare_unifiers(unifiers)
        */

        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );
        
        //exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__gt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('id', 'x'), ('null',))))
        let var1 = Rc::new(AstroNode::AstroID(AstroID::new("__gt__".to_string())));
        let var2 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let var3 = Rc::new(AstroNode::AstroID(AstroID::new("POS_INT".to_string())));
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(0)));
        let null1 = Rc::new( AstroNode::AstroNone( AstroNone::new()));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&var2),Rc::clone(&i1)])))));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&var1), Rc::clone(&t1) )));
        let if1 = Rc::new( AstroNode::AstroIf( AstroIf::new( Rc::clone(&apply1), Rc::clone(&var2), Rc::clone(&null1))));
        let quote1 = Rc::new( AstroNode::AstroQuote( AstroQuote::new( Rc::clone( &if1))));
        let exp_val = walk( quote1, &mut state );

        //error handling
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( (_type,msg) ) => panic!("{}: {}",_type,msg),
        };

        //unifiers = unify(exp_val,('id', 'POS_INT'))
        let unifiers = unify( exp_val, var3, &mut state, true);

        //error handling
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( (_type,msg) ) => panic!("{}: {}",_type,msg),
        };

        //declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state);

        //set_lineinfo('prog.txt',2)
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:String::from("prog.ast"),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        //exp_val = walk(('integer', 2))
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let exp_val = walk( i2, &mut state );

        //error handling
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( (_type,msg) ) => panic!("{}: {}",_type,msg),
        };

        //unifiers = unify(exp_val,('named-pattern', ('id', 'x'), ('deref', ('id', 'POS_INT'))))
        let var3 = AstroID::new("x".to_string());
        let var4 = Rc::new(AstroNode::AstroID(AstroID::new("POS_INT".to_string())));
        let deref1 = Rc::new( AstroNode::AstroDeref(AstroDeref::new( Rc::clone(&var4) )));
        let namedp1 = Rc::new(AstroNode::AstroNamedPattern(AstroNamedPattern::new( var3, Rc::clone(&deref1))));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&namedp1), &mut state, true);

        //error handling
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( (_type,msg) ) => panic!("{}: {}",_type,msg),
        };

        //declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            AstroNode::AstroInteger(AstroInteger{id:_,value:2}) => (),
            _ => panic!("test failed"),
        };
    }
    //#[test]
    fn test_prog_function_sum() {

        // def _ast72(arg):
        //     set_lineinfo('prog.txt',3)
        //     try:
        //         unifiers = unify(arg,('named-pattern', ('id', 'n'), ('deref', ('id', 'POS_INT'))))
        //         state.symbol_table.push_scope({})
        //         declare_formal_args(unifiers)
        //         set_lineinfo('prog.txt',4)
        //         val = walk(('apply', ('id', '__plus__'), ('tuple', [('id', 'n'), ('apply', ('id', 'sum'), ('apply', ('id', '__minus__'), ('tuple', [('id', 'n'), ('integer', 1)])))])))
        //         state.symbol_table.pop_scope()
        //         return val

        //         state.symbol_table.pop_scope()
        //     except PatternMatchFailed:
        //         set_lineinfo('prog.txt',5)
        //         try:
        //             unifiers = unify(arg,('integer', 0))
        //             state.symbol_table.push_scope({})
        //             declare_formal_args(unifiers)
        //             set_lineinfo('prog.txt',6)
        //             val = walk(('integer', 0))
        //             state.symbol_table.pop_scope()
        //             return val

        //             state.symbol_table.pop_scope()
        //         except PatternMatchFailed:
        //             raise ValueError('none of the function bodies unified with actual parameters')
    
        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__gt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('id', 'x'), ('null',))))
        // unifiers = unify(exp_val,('id', 'POS_INT'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'sum'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',8)
        // exp_val = walk(('apply', ('id', 'sum'), ('integer', 5)))
        // unifiers = unify(exp_val,('id', 'y'))
        // declare_unifiers(unifiers)
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__gt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('id', 'x'), ('null',))))
        // unifiers = unify(exp_val,('id', 'POS_INT'))
        // declare_unifiers(unifiers)

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        let var1 = Rc::new(AstroNode::AstroID(AstroID::new("__gt__".to_string())));
        let var2 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let var3 = Rc::new(AstroNode::AstroID(AstroID::new("POS_INT".to_string())));
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(0)));
        let null1 = Rc::new( AstroNode::AstroNone( AstroNone::new()));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&var2),Rc::clone(&i1)])))));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&var1), Rc::clone(&t1) )));
        let if1 = Rc::new( AstroNode::AstroIf( AstroIf::new( Rc::clone(&apply1), Rc::clone(&var2), Rc::clone(&null1))));
        let quote1 = Rc::new( AstroNode::AstroQuote( AstroQuote::new( Rc::clone( &if1))));
        let exp_val = walk( quote1, &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, var3, &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'sum'))
        // declare_unifiers(unifiers)

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id8 = Rc::new(AstroNode::AstroID(AstroID::new("_ast72".to_string())));
        let id9 = Rc::new(AstroNode::AstroID(AstroID::new("sum".to_string())));
        let func1 = Rc::new(AstroNode::AstroFunction(AstroFunction::new( Rc::clone(&id8) )));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id9), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        // set_lineinfo('prog.txt',8)
        // exp_val = walk(('apply', ('id', 'sum'), ('integer', 5)))
        // unifiers = unify(exp_val,('id', 'y'))
        // declare_unifiers(unifiers)

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id10 = Rc::new(AstroNode::AstroID(AstroID::new("sum".to_string())));
        let id11 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(5)));
        let apply2 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id10), Rc::clone(&i2) )));
        let exp_val = walk( Rc::clone(&apply2), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id11), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);    
        
        let check1 = state.lookup_sym("y",true);
        let AstroNode::AstroInteger(AstroInteger{id:_,value:15}) = *check1
            else {panic!("test failed")};
    } 
    #[test]
    fn test_prog_object_construction() {
        // -- define a structure of type A
        // structure A with
        //     data a.
        //     data b.
        // end

        // let obj = A(1,2).     -- default constructor, a<-1, b<-2

        // set_lineinfo('prog.txt',2)
        // # structure def for A
        // member_list = [('data', ('id', 'a')), ('data', ('id', 'b'))]
        // struct_memory = []
        // member_names = []
        // for member_ix in range(len(member_list)):
        //     member = member_list[member_ix]
        //     if member[0] == 'data':
        //         (DATA, (ID, member_id)) = member
        //         struct_memory.append(('none', None))
        //         member_names.append(member_id)
        //     elif member[0] == 'unify':
        //         (UNIFY, (ID, member_id), function_exp) = member
        //         function_val = walk(function_exp)
        //         struct_memory.append(function_val)
        //         member_names.append(member_id)
        //     elif member[0] == 'noop':
        //         pass
        //     else:
        //         raise ValueError('unsupported struct member {}'.format(member[0]))
        // struct_type = ('struct',('member-names', ('list', member_names)),('struct-memory', ('list', struct_memory)))
        // state.symbol_table.enter_sym('A', struct_type)
     
        // set_lineinfo('prog.txt',7)
        // exp_val = walk(('apply', ('id', 'A'), ('tuple', [('integer', 1), ('integer', 2)])))
        // unifiers = unify(exp_val,('id', 'obj'))
        // declare_unifiers(unifiers)

        let mut state = State::new().unwrap();

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        // # structure def for A
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("a".to_string())));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("b".to_string())));
        let d1 = Rc::new(AstroNode::AstroData(AstroData::new(Rc::clone(&id1))));
        let d2 = Rc::new(AstroNode::AstroData(AstroData::new(Rc::clone(&id2))));
        let member_list = vec![ Rc::clone(&d1), Rc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<Rc<AstroNode>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<Rc<AstroNode>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( Rc::clone(&member) );
            if _type == "data" {
                let AstroNode::AstroData(AstroData{id:_,value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let AstroNode::AstroID(AstroID{id:_,name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( Rc::new(AstroNode::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = Rc::new(AstroNode::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", Rc::clone(&struct_type)  );

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("A".to_string())));
        let id4 = Rc::new(AstroNode::AstroID(AstroID::new("obj".to_string())));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id3), Rc::clone(&t1) )));
        let exp_val = walk( Rc::clone(&apply1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id4), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);  
        /**********************************************************************************************************************/
        let check1 = state.lookup_sym("obj",true);
        
        let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref id,object_memory:ref mem}) = *check1
            else {panic!("test failed.")};
        let AstroID{id:_,name:ref tag} = *id;
        assert_eq!( tag, "A" );
 
        let AstroNode::AstroInteger(AstroInteger{id:_,value:v1}) = *(mem.borrow()[0])
            else {panic!("test failed")}; 
        assert_eq!( v1,1 );

        let AstroNode::AstroInteger(AstroInteger{id:_,value:v2}) = *(mem.borrow()[1])
            else {panic!("test failed")}; 
        assert_eq!( v2,2 );

    }
    #[test]
    fn test_prog_while_loop() {

        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('integer', 0))
        // unifiers = unify(exp_val,('id', 'ctr'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',2)
        // while map2boolean(walk(('apply', ('id', '__lt__'), ('tuple', [('id', 'ctr'), ('integer', 100)]))))[1]:
        //    set_lineinfo('prog.txt',3)
        //    exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('id', 'ctr'), ('integer', 1)])))
        //    unifiers = unify(exp_val,('id', 'ctr'))
        //    declare_unifiers(unifiers)
      
        // set_lineinfo('prog.txt',4)

        let mut state = State::new().unwrap();

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(0)));
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("ctr".to_string())));
        let exp_val = walk( Rc::clone(&i1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);  

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(100)));
        let i3 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("__lt__".to_string())));
        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("__plus__".to_string())));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&id1),Rc::clone(&i2)])))));
        let t2 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&id1),Rc::clone(&i3)])))));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id2), Rc::clone(&t1) )));
        let apply2 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id3), Rc::clone(&t2) )));

        while let Some(AstroNode::AstroBool(AstroBool{id:_,value:true})) = map2boolean( &walk(Rc::clone(&apply1), &mut state ).unwrap()) {

            let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, &mut state );

            let exp_val = walk( Rc::clone(&apply2), &mut state);
            let exp_val = match exp_val {
                Ok( val ) => val,
                Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
            };

            let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

            let unifiers = match unifiers {
                Ok( val ) => val,
                Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
            };

            let check1 = state.lookup_sym("ctr",true);
            let AstroNode::AstroInteger(AstroInteger{id:_,value:v}) = *check1 else {panic!("test failed.")};

            declare_unifiers( &unifiers, &mut state); 
        }

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:4}));
        set_lineinfo(  new_lineinfo, &mut state );

        let check1 = state.lookup_sym("ctr",true);
        let AstroNode::AstroInteger(AstroInteger{id:_,value:100}) = *check1 
            else {panic!("test failed.")};

            
    }
    #[test]
    fn test_prog_object_constructor() {
        // structure Circle with
        //     data radius.
        //     data diameter.
        //     function __init__ with (radius) do -- constructor
        //         let this@radius = radius.
        //         let this@diameter = 2 * radius.
        //     end
        // end
        // let a = Circle( 2 ).

        // def _ast72(arg):
        // set_lineinfo('prog.txt',4)
        // try:
        //    unifiers = unify(arg,('id', 'radius'))
        //    state.symbol_table.push_scope({})
        //    declare_formal_args(unifiers)
        //    set_lineinfo('prog.txt',5)
        //    exp_val = walk(('id', 'radius'))
        //    unifiers = unify(exp_val,('index', ('id', 'this'), ('id', 'radius')))
        //    declare_unifiers(unifiers)
  
        //    set_lineinfo('prog.txt',6)
        //    exp_val = walk(('apply', ('id', '__times__'), ('tuple', [('integer', 2), ('id', 'radius')])))
        //    unifiers = unify(exp_val,('index', ('id', 'this'), ('id', 'diameter')))
        //    declare_unifiers(unifiers)
  
        //    state.symbol_table.pop_scope()
        // except PatternMatchFailed:
        //    raise ValueError('none of the function bodies unified with actual parameters')
  

        // set_lineinfo('prog.txt',1)
        // # structure def for Circle
        // member_list = [('data', ('id', 'radius')), ('data', ('id', 'diameter')), ('unify', ('id', '__init__'), ('function-exp', ('implementation', '_ast72')))]
        // struct_memory = []
        // member_names = []
        // for member_ix in range(len(member_list)):
        //     member = member_list[member_ix]
        //     if member[0] == 'data':
        //         (DATA, (ID, member_id)) = member
        //         struct_memory.append(('none', None))
        //         member_names.append(member_id)
        //     elif member[0] == 'unify':
        //         (UNIFY, (ID, member_id), function_exp) = member
        //         function_val = walk(function_exp)
        //         struct_memory.append(function_val)
        //         member_names.append(member_id)
        //     elif member[0] == 'noop':
        //         pass
        //     else:
        //         raise ValueError('unsupported struct member {}'.format(member[0]))
        // struct_type = ('struct',('member-names', ('list', member_names)),('struct-memory', ('list', struct_memory)))
        // state.symbol_table.enter_sym('Circle', struct_type)
     
        // set_lineinfo('prog.txt',10)
        // exp_val = walk(('apply', ('id', 'Circle'), ('integer', 2)))
        // unifiers = unify(exp_val,('id', 'a'))
        // declare_unifiers(unifiers)
    
        fn _ast72<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result< Rc<AstroNode>, (&'static str,String)> {
            
            let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:4}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(AstroNode::AstroID(AstroID::new("radius".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:5}));
                set_lineinfo(  new_lineinfo, state );

                let exp_val = walk( Rc::clone(&id1), state );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                let id2 = Rc::new(AstroNode::AstroID(AstroID::new("this".to_string())));
                let id3 = Rc::new(AstroNode::AstroID(AstroID::new("diameter".to_string())));
                let index1 = Rc::new(AstroNode::AstroIndex(AstroIndex::new( Rc::clone(&id2), Rc::clone(&id1))));
                let index2 = Rc::new(AstroNode::AstroIndex(AstroIndex::new( Rc::clone(&id2), Rc::clone(&id3))));

                let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&index1), state, true);

                let unifiers = match unifiers {
                    Ok( val ) => val,
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                declare_unifiers( &unifiers, state);

                let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:6}));
                set_lineinfo(  new_lineinfo, state );

                let id4 = Rc::new(AstroNode::AstroID(AstroID::new("__times__".to_string())));
                let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
                let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&id1)])))));
                let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id4), Rc::clone(&t1))));

                let exp_val = walk( Rc::clone(&apply1), state );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&index2), state, true);

                let unifiers = match unifiers {
                    Ok( val ) => val,
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                declare_unifiers( &unifiers, state);

                state.pop_scope();

                return Ok(Rc::new(AstroNode::AstroNone(AstroNone::new())));
            } else {
                return Err(("ValueError",format!("none of the function bodies unified with actual parameters")));
            }
        }

        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("__init__") , _ast72 );
        
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:4}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("radius".to_string())));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("diameter".to_string())));
        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("__init__".to_string())));
        let data1 = Rc::new(AstroNode::AstroData(AstroData::new(Rc::clone(&id1))));
        let data2 = Rc::new(AstroNode::AstroData(AstroData::new(Rc::clone(&id2))));
        let func1 = Rc::new(AstroNode::AstroFunction(AstroFunction::new( Rc::clone(&id3))));
        let unify1 = Rc::new(AstroNode::AstroUnify(AstroUnify::new( Rc::clone(&id3), Rc::clone(&func1))));

        let member_list = vec![ Rc::clone(&data1), Rc::clone(&data2), Rc::clone(&unify1) ];
        let mut struct_memory: RefCell<Vec<Rc<AstroNode>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<Rc<AstroNode>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( Rc::clone(&member) );
            if _type == "data" {
                let AstroNode::AstroData(AstroData{id:_,value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let AstroNode::AstroID(AstroID{id:_,name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( Rc::new(AstroNode::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "unify" {
                let AstroNode::AstroUnify(AstroUnify{id:_,term:ref id_node,pattern:ref function_exp}) = *member
                    else {panic!("ERROR: object construction: expection unify node.")};
                let function_val = match walk( Rc::clone(&function_exp), &mut state ) {
                    Ok( val ) => val,
                    Err ( (_type,msg) ) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };
                struct_memory.borrow_mut().push( Rc::clone( &function_val ));
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "noop" {
                ;// pass
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }

        let struct_type = Rc::new(AstroNode::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "Circle", Rc::clone(&struct_type)  );

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo(AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:10}));
        set_lineinfo(  new_lineinfo, &mut state );   

        // exp_val = walk(('apply', ('id', 'Circle'), ('integer', 2)))
        // unifiers = unify(exp_val,('id', 'a'))
        // declare_unifiers(unifiers)
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("Circle".to_string())));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("a".to_string())));
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&i1))));

        let exp_val = walk( Rc::clone(&apply1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        /**************************************************************************************************/
        //assert
        let check1 = state.lookup_sym("a",true);
        let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref id,object_memory:ref mem}) = *check1
            else {panic!("test failed")};

        let AstroID{id:_,name:ref tag} = *id;

        assert_eq!( tag,"Circle" );

        let AstroNode::AstroInteger(AstroInteger{id:_,value:2}) = *(*(mem.borrow()))[0]
            else {panic!("test failed")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:4}) = *(*(mem.borrow()))[1]
            else {panic!("test failed")};
    } 
    #[test]
    fn test_prog_modify_list() {

        //ASTEROID
        //let x = [1,2,3].
        //let x@1 = 4.

        //PYTHON
        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('list', [('integer', 1), ('integer', 2), ('integer', 3)]))
        // unifiers = unify(exp_val,('id', 'x'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('integer', 4))
        // unifiers = unify(exp_val,('index', ('id', 'x'), ('integer', 1)))
        // declare_unifiers(unifiers)

        //RUST
        let mut state = State::new().unwrap();
        
        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let i1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(1)));
        let i2 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(2)));
        let i3 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(3)));
        let l1 = Rc::new(AstroNode::AstroList(AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2),Rc::clone(&i3)])))));
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
     
        let exp_val = walk( Rc::clone(&l1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let i4 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(4)));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let i5 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(1)));
        let idx1 = Rc::new(AstroNode::AstroIndex(AstroIndex::new(Rc::clone(&id2),Rc::clone(&i5))));

        let exp_val = walk( Rc::clone(&i4), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&idx1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("x",true);
        let AstroNode::AstroList(AstroList{id:_,length:len,ref contents}) = *check1    
            else {panic!("test failed.")};

        let AstroNode::AstroInteger(AstroInteger{id:_,value:1}) = *contents.borrow()[0]
            else {panic!("test failed.")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:4}) = *contents.borrow()[1]
            else {panic!("test failed.")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:3}) = *contents.borrow()[2]
            else {panic!("test failed.")};
    }
    #[test]
    fn test_prog_modify_interior_list() {

        // ASTEROID
        // let x = [1,[2,3,4],5].
        // let x@1@1 = "hello".

        // PYTHON
        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('list', [('integer', 1), ('list', [('integer', 2), ('integer', 3), ('integer', 4)]), ('integer', 5)]))
        // unifiers = unify(exp_val,('id', 'x'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('string', 'hello'))
        // unifiers = unify(exp_val,('index', ('index', ('id', 'x'), ('integer', 1)), ('integer', 1)))
        // declare_unifiers(unifiers)
     
        // RUST
        let mut state = State::new().unwrap();

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let i1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(1)));
        let i2 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(2)));
        let i3 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(3)));
        let i4 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(4)));
        let i5 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(5)));
        let l1 = Rc::new(AstroNode::AstroList(AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&i2),Rc::clone(&i3),Rc::clone(&i4)])))));
        let l2 = Rc::new(AstroNode::AstroList(AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&l1),Rc::clone(&i5)])))));
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( Rc::clone(&l2), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let s1 = Rc::new(AstroNode::AstroString(AstroString::new("hello".to_string())));
        let i6 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(1)));
        let i7 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(1)));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let idx1 = Rc::new(AstroNode::AstroIndex(AstroIndex::new(Rc::clone(&id2),Rc::clone(&i6))));
        let idx2 = Rc::new(AstroNode::AstroIndex(AstroIndex::new(Rc::clone(&idx1),Rc::clone(&i7))));

        let exp_val = walk( Rc::clone(&s1), &mut state );
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&idx2), &mut state, true);
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("x",true);
        let AstroNode::AstroList(AstroList{id:_,length:3,ref contents}) = *check1    
            else {panic!("test failed.")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:1}) = *contents.borrow()[0]
            else {panic!("test failed.")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:5}) = *contents.borrow()[2]
            else {panic!("test failed.")};
        let AstroNode::AstroList(AstroList{id:_,length:l,contents:ref inner_contents}) = *contents.borrow()[1]    
            else {panic!("test failed.")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:2}) = *inner_contents.borrow()[0]
            else {panic!("test failed.")};
        let AstroNode::AstroString(AstroString{id:_,value:ref v}) = *inner_contents.borrow()[1] 
            else {panic!("error")};
        let AstroNode::AstroInteger(AstroInteger{id:_,value:4}) = *inner_contents.borrow()[2]
            else {panic!("test failed.")};
        assert_eq!(v,"hello");
    }
    #[test]
    fn test_prog_headtail_function() {
        // Asteroid
        // let x = [1,2,3].
        // function f
	    //     with [x|tail] do
		//         return x.
        // end.
        // let y = f(x).

        // Python
        // def _ast72(arg):
        // set_lineinfo('prog.txt',3)
        // try:
        //    unifiers = unify(arg,('head-tail', ('id', 'x'), ('id', 'tail')))
        //    state.symbol_table.push_scope({})
        //    declare_formal_args(unifiers)
        //    set_lineinfo('prog.txt',4)
        //    val = walk(('id', 'x'))
        //    state.symbol_table.pop_scope()
        //    return val
  
        //    state.symbol_table.pop_scope()
        // except PatternMatchFailed:
        //    raise ValueError('none of the function bodies unified with actual parameters')
  
        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('list', [('integer', 1), ('integer', 2), ('integer', 3)]))
        // unifiers = unify(exp_val,('id', 'x'))
        // declare_unifiers(unifiers)

        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'f'))
        // declare_unifiers(unifiers)

        // set_lineinfo('prog.txt',5)
        // set_lineinfo('prog.txt',6)
        // exp_val = walk(('apply', ('id', 'f'), ('id', 'x')))
        // unifiers = unify(exp_val,('id', 'y'))
        // declare_unifiers(unifiers)


        // Rust
        fn _ast72<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result< Rc<AstroNode>, (&'static str,String)> {

            println!("into function with {}!",peek(Rc::clone(&node)));
            
            let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
            let id2 = Rc::new(AstroNode::AstroID(AstroID::new("tail".to_string())));
            let ht1 = Rc::new(AstroNode::AstroHeadTail(AstroHeadTail::new(Rc::clone(&id1),Rc::clone(&id2))));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&ht1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                let id3 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));

                let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:4}));
                set_lineinfo(  new_lineinfo, state );

                let exp_val = walk( Rc::clone(&id3), state );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                state.pop_scope();

                return Ok( exp_val )
            } else {
                return Err(("ValueError",format!("none of the function bodies unified with actual parameters")));
            }
        }


        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );  

        let i1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(1)));
        let i2 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(2)));
        let i3 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(3)));
        let l1 = Rc::new(AstroNode::AstroList(AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2),Rc::clone(&i3)])))));
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( Rc::clone(&l1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("_ast72".to_string())));
        let func1 = Rc::new(AstroNode::AstroFunction(AstroFunction::new(Rc::clone(&id2))));
        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("f".to_string())));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id3), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:5}));
        set_lineinfo(  new_lineinfo, &mut state ); 

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:6}));
        set_lineinfo(  new_lineinfo, &mut state ); 

        let id4 = Rc::new(AstroNode::AstroID(AstroID::new("f".to_string())));
        let id5 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let id6 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
        let apply1 = Rc::new(AstroNode::AstroApply(AstroApply::new(Rc::clone(&id4),Rc::clone(&id5))));

        let exp_val = walk( Rc::clone(&apply1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id6), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("y",true);

        let AstroNode::AstroInteger(AstroInteger{id:_,value:1}) = *check1
            else {panic!("test failed.")};
        // let AstroNode::AstroList(AstroList{id:_,length:3,ref contents}) = *check1    
        //     else {panic!("test failed.")};
        // let AstroNode::AstroInteger(AstroInteger{id:_,value:1}) = *contents.borrow()[0]
        //     else {panic!("test failed.")};

    }
    #[test]
    fn test_prog_twoheads_headtail_function() {
        // Asteroid
        // let x = [1,2,3].
        // function f
	    //     with [x|y|tail] do
		//         return x + y.
        // end.
        // let z = f(x).

        // Python
        // def _ast72(arg):
        // set_lineinfo('prog.txt',3)
        // try:
        //    unifiers = unify(arg,('head-tail', ('id', 'x'), ('raw-head-tail', ('id', 'y'), ('id', 'tail'))))
        //    state.symbol_table.push_scope({})
        //    declare_formal_args(unifiers)
        //    set_lineinfo('prog.txt',4)
        //    val = walk(('apply', ('id', '__plus__'), ('tuple', [('id', 'x'), ('id', 'y')])))
        //    state.symbol_table.pop_scope()
        //    return val
        //    state.symbol_table.pop_scope()
        // except PatternMatchFailed:
        //    raise ValueError('none of the function bodies unified with actual parameters')
  
  
        //    state.symbol_table.pop_scope()
        // except PatternMatchFailed:
        //    raise ValueError('none of the function bodies unified with actual parameters')
  
        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('list', [('integer', 1), ('integer', 2), ('integer', 3)]))
        // unifiers = unify(exp_val,('id', 'x'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'f'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',5)
        // set_lineinfo('prog.txt',6)
        // exp_val = walk(('apply', ('id', 'f'), ('id', 'x')))
        // unifiers = unify(exp_val,('id', 'z'))
        // declare_unifiers(unifiers)
     


        // Rust
        fn _ast72<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result< Rc<AstroNode>, (&'static str,String)> {

            println!("into function with {}!",peek(Rc::clone(&node)));
            
            let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
            let id2 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
            let id3 = Rc::new(AstroNode::AstroID(AstroID::new("tail".to_string())));
            let rht1 = Rc::new(AstroNode::AstroRawHeadTail(AstroRawHeadTail::new(Rc::clone(&id2),Rc::clone(&id3))));
            let ht1 = Rc::new(AstroNode::AstroHeadTail(AstroHeadTail::new(Rc::clone(&id1),Rc::clone(&rht1))));


            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&ht1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                let id4 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
                let id5 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
                let tup1 = Rc::new(AstroNode::AstroTuple(AstroTuple::new(2,Rc::new(RefCell::new(vec![ Rc::clone(&id4),Rc::clone(&id5) ])))));
                let id6 = Rc::new(AstroNode::AstroID(AstroID::new("__plus__".to_string())));
                let apply1 = Rc::new(AstroNode::AstroApply(AstroApply::new(Rc::clone(&id6),Rc::clone(&tup1))));

                let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:4}));
                set_lineinfo(  new_lineinfo, state );

                let exp_val = walk( Rc::clone(&apply1), state );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                state.pop_scope();

                return Ok( exp_val )
            } else {
                return Err(("ValueError",format!("none of the function bodies unified with actual parameters")));
            }
        }


        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );  

        let i1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(1)));
        let i2 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(2)));
        let i3 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(3)));
        let l1 = Rc::new(AstroNode::AstroList(AstroList::new(3,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2),Rc::clone(&i3)])))));
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( Rc::clone(&l1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("_ast72".to_string())));
        let func1 = Rc::new(AstroNode::AstroFunction(AstroFunction::new(Rc::clone(&id2))));
        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("f".to_string())));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id3), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:5}));
        set_lineinfo(  new_lineinfo, &mut state ); 

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:6}));
        set_lineinfo(  new_lineinfo, &mut state ); 

        let id4 = Rc::new(AstroNode::AstroID(AstroID::new("f".to_string())));
        let id5 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let id6 = Rc::new(AstroNode::AstroID(AstroID::new("z".to_string())));
        let apply1 = Rc::new(AstroNode::AstroApply(AstroApply::new(Rc::clone(&id4),Rc::clone(&id5))));

        let exp_val = walk( Rc::clone(&apply1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id6), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("z",true);


        let AstroNode::AstroInteger(AstroInteger{id:_,value:3}) = *check1
            else {panic!("test failed.")};
    }
    #[test]
    fn test_prog_modify_object() {
        // -- define a structure of type A
        // structure A with
        //     data a.
        //     data b.
        // end

        // let obj = A(1,2).     -- default constructor, a<-1, b<-2
        // let obj@b = 4.

        // set_lineinfo('prog.txt',2)
        // # structure def for A
        // member_list = [('data', ('id', 'a')), ('data', ('id', 'b'))]
        // struct_memory = []
        // member_names = []
        // for member_ix in range(len(member_list)):
        //     member = member_list[member_ix]
        //     if member[0] == 'data':
        //         (DATA, (ID, member_id)) = member
        //         struct_memory.append(('none', None))
        //         member_names.append(member_id)
        //     elif member[0] == 'unify':
        //         (UNIFY, (ID, member_id), function_exp) = member
        //         function_val = walk(function_exp)
        //         struct_memory.append(function_val)
        //         member_names.append(member_id)
        //     elif member[0] == 'noop':
        //         pass
        //     else:
        //         raise ValueError('unsupported struct member {}'.format(member[0]))
        // struct_type = ('struct',('member-names', ('list', member_names)),('struct-memory', ('list', struct_memory)))
        // state.symbol_table.enter_sym('A', struct_type)
    
        // set_lineinfo('prog.txt',7)
        // exp_val = walk(('apply', ('id', 'A'), ('tuple', [('integer', 1), ('integer', 2)])))
        // unifiers = unify(exp_val,('id', 'obj'))
        // declare_unifiers(unifiers)
    
        // set_lineinfo('prog.txt',8)
        // exp_val = walk(('integer', 4))
        // unifiers = unify(exp_val,('index', ('id', 'obj'), ('id', 'b')))
        // declare_unifiers(unifiers)
        

     
        
        let mut state = State::new().unwrap();

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        // # structure def for A
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("a".to_string())));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("b".to_string())));
        let d1 = Rc::new(AstroNode::AstroData(AstroData::new(Rc::clone(&id1))));
        let d2 = Rc::new(AstroNode::AstroData(AstroData::new(Rc::clone(&id2))));
        let member_list = vec![ Rc::clone(&d1), Rc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<Rc<AstroNode>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<Rc<AstroNode>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( Rc::clone(&member) );
            if _type == "data" {
                let AstroNode::AstroData(AstroData{id:_,value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let AstroNode::AstroID(AstroID{id:_,name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( Rc::new(AstroNode::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = Rc::new(AstroNode::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", Rc::clone(&struct_type)  );

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("A".to_string())));
        let id4 = Rc::new(AstroNode::AstroID(AstroID::new("obj".to_string())));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id3), Rc::clone(&t1) )));
        let exp_val = walk( Rc::clone(&apply1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id4), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);  

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i3 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(4)));
        let id5 = Rc::new(AstroNode::AstroID(AstroID::new("obj".to_string())));
        let id6 = Rc::new(AstroNode::AstroID(AstroID::new("b".to_string())));
        let idx1 = Rc::new(AstroNode::AstroIndex(AstroIndex::new( Rc::clone(&id5), Rc::clone(&id6))));

        let exp_val = match walk( Rc::clone(&i3), &mut state) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = match unify( exp_val, Rc::clone(&idx1), &mut state, true) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);  

        /**********************************************************************************************************************/
        let check1 = state.lookup_sym("obj",true);
        
        let AstroNode::AstroObject(AstroObject{id:_,struct_id:ref id,object_memory:ref mem}) = *check1
            else {panic!("test failed.")};
        let AstroID{id:_,name:ref tag} = *id;
        assert_eq!( tag, "A" );
 
        let AstroNode::AstroInteger(AstroInteger{id:_,value:v1}) = *(mem.borrow()[0])
            else {panic!("test failed")}; 
        assert_eq!( v1,1 );

        let AstroNode::AstroInteger(AstroInteger{id:_,value:v2}) = *(mem.borrow()[1])
            else {panic!("test failed")}; 
        assert_eq!( v2,4 );
    }
    #[test]
    fn test_prog_access_object_field() {
        // -- define a structure of type A
        // structure A with
        //     data a.
        //     data b.
        // end

        // let obj = A(1,2).     -- default constructor, a<-1, b<-2
        // let z = obj@b.

        // set_lineinfo('prog.txt',2)
        // # structure def for A
        // member_list = [('data', ('id', 'a')), ('data', ('id', 'b'))]
        // struct_memory = []
        // member_names = []
        // for member_ix in range(len(member_list)):
        //     member = member_list[member_ix]
        //     if member[0] == 'data':
        //         (DATA, (ID, member_id)) = member
        //         struct_memory.append(('none', None))
        //         member_names.append(member_id)
        //     elif member[0] == 'unify':
        //         (UNIFY, (ID, member_id), function_exp) = member
        //         function_val = walk(function_exp)
        //         struct_memory.append(function_val)
        //         member_names.append(member_id)
        //     elif member[0] == 'noop':
        //         pass
        //     else:
        //         raise ValueError('unsupported struct member {}'.format(member[0]))
        // struct_type = ('struct',('member-names', ('list', member_names)),('struct-memory', ('list', struct_memory)))
        // state.symbol_table.enter_sym('A', struct_type)
     
        // set_lineinfo('prog.txt',7)
        // exp_val = walk(('apply', ('id', 'A'), ('tuple', [('integer', 1), ('integer', 2)])))
        // unifiers = unify(exp_val,('id', 'obj'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',8)
        // exp_val = walk(('index', ('id', 'obj'), ('id', 'b')))
        // unifiers = unify(exp_val,('id', 'z'))
        // declare_unifiers(unifiers)

        let mut state = State::new().unwrap();

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        // # structure def for A
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("a".to_string())));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("b".to_string())));
        let d1 = Rc::new(AstroNode::AstroData(AstroData::new(Rc::clone(&id1))));
        let d2 = Rc::new(AstroNode::AstroData(AstroData::new(Rc::clone(&id2))));
        let member_list = vec![ Rc::clone(&d1), Rc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<Rc<AstroNode>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<Rc<AstroNode>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( Rc::clone(&member) );
            if _type == "data" {
                let AstroNode::AstroData(AstroData{id:_,value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let AstroNode::AstroID(AstroID{id:_,name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( Rc::new(AstroNode::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = Rc::new(AstroNode::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", Rc::clone(&struct_type)  );

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2)));
        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("A".to_string())));
        let id4 = Rc::new(AstroNode::AstroID(AstroID::new("obj".to_string())));
        let t1 = Rc::new( AstroNode::AstroTuple( AstroTuple::new(2,Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let apply1 = Rc::new( AstroNode::AstroApply( AstroApply::new( Rc::clone(&id3), Rc::clone(&t1) )));
        let exp_val = walk( Rc::clone(&apply1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id4), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);  

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i3 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(4)));
        let id5 = Rc::new(AstroNode::AstroID(AstroID::new("obj".to_string())));
        let id6 = Rc::new(AstroNode::AstroID(AstroID::new("b".to_string())));
        let id7 = Rc::new(AstroNode::AstroID(AstroID::new("z".to_string())));
        let idx1 = Rc::new(AstroNode::AstroIndex(AstroIndex::new( Rc::clone(&id5), Rc::clone(&id6))));

        let exp_val = match walk( Rc::clone(&idx1), &mut state) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id7), &mut state, true) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);  
     
        let check1 = state.lookup_sym("z",true);

        let AstroNode::AstroInteger(AstroInteger{id:_,value:v1}) = *check1
            else {panic!("test failed")}; 
        assert_eq!( v1,2 );

    }
    #[test]
    fn test_unify_index_string() {
        // Asteroid

        // let x = "abcdefg".
        // let y = x@1.

        //Python

        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('string', 'abcdefg'))
        // unifiers = unify(exp_val,('id', 'x'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('index', ('id', 'x'), ('integer', 1)))
        // unifiers = unify(exp_val,('id', 'y'))
        // declare_unifiers(unifiers)

        //Rust

        let mut state = State::new().unwrap();

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        // # structure def for A
        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let s1 = Rc::new(AstroNode::AstroString(AstroString::new("abcdefg".to_string())));

        let exp_val = match walk( Rc::clone(&s1), &mut state) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id1), &mut state, true) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state); 

        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));
        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
        let i1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(1)));
        let idx1 = Rc::new(AstroNode::AstroIndex(AstroIndex::new( Rc::clone(&id2), Rc::clone(&i1) )));

        let exp_val = match walk( Rc::clone(&idx1), &mut state) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id3), &mut state, true) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state); 

        /***********************************************************************************************/
        let check1 = state.lookup_sym("y",true);

        let AstroNode::AstroString(AstroString{id:_,value:ref v1}) = *check1
            else {panic!("test failed")}; 
        assert_eq!( v1,"b" );
    }
    #[test]
    fn test_prog_escape_func() {
        // Asteroid
        // function times_two with x do return escape
        // "
        // let AstroNode::AstroInteger(AstroInteger{id:_,value:val}) = *state.lookup_sym( \"x\" ) 
        //     else {return Err((\"ValueError\",\"times_two() expected a single integer.\"))};
        
        // return Rc::new(AstroNode::AstroInteger(AstroInteger::new(2*val)));
        // "
        // end
        // let y = times_two( 15 ).


        // Python
        // def _ast72():
            // let AstroNode::AstroInteger(AstroInteger{id:_,value:val}) = *state.lookup_sym( "x" )
            //   else {return Err(("ValueError","times_two() expected a single integer."))};
            // return Ok(Rc::new(AstroNode::AstroInteger(AstroInteger::new(2*val))));
            // avm.avm.__retval__ = __retval__
  
        // def _ast73(arg):
        //     set_lineinfo('prog.txt',1)
        //     try:
        //     unifiers = unify(arg,('id', 'x'))
        //     state.symbol_table.push_scope({})
        //     declare_formal_args(unifiers)
        //     set_lineinfo('prog.txt',1)
        //     val = walk(('escape', ('implementation', '_ast72')))
        //     state.symbol_table.pop_scope()
        //     return val
    
        //     state.symbol_table.pop_scope()
        //     except PatternMatchFailed:
        //     raise ValueError('none of the function bodies unified with actual parameters')

        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('function-exp', ('implementation', '_ast73')))
        // unifiers = unify(exp_val,('id', 'times_two'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',9)
        // exp_val = walk(('apply', ('id', 'times_two'), ('integer', 15)))
        // unifiers = unify(exp_val,('id', 'y'))
        // declare_unifiers(unifiers)
     
        // Rust

        fn _ast72<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result< Rc<AstroNode>, (&'static str,String)> {
            let AstroNode::AstroInteger(AstroInteger{id:_,value:val}) = *state.lookup_sym( "x", true )
              else {return Err(("ValueError",format!("times_two() expected a single integer.")))};
            return Ok(Rc::new(AstroNode::AstroInteger(AstroInteger::new(2*val))));
        }
        fn _ast73<'a>( node: Rc<AstroNode>, state: &'a mut State ) -> Result< Rc<AstroNode>, (&'static str,String)> {

            let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(AstroNode::AstroID(AstroID::new("x".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
                set_lineinfo(  new_lineinfo, state );

                let id2 = Rc::new(AstroNode::AstroID(AstroID::new("_ast72".to_string())));
                let esc1 = Rc::new(AstroNode::AstroEscape(AstroEscape::new( "_ast72".to_string() )));

                let exp_val = match walk( Rc::clone(&esc1), state) {
                    Ok( val ) => val,
                    Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
                };

                state.push_scope();

                return Ok( exp_val )
            } else {
                return Err(("ValueError","none of the function bodies unified with actual parameters".to_string()))
            }
            
        }

        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast72") , _ast72 );
        state.dispatch_table.insert( String::from("_ast73") , _ast73 );

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id1 = Rc::new(AstroNode::AstroID(AstroID::new("_ast73".to_string())));
        let id2 = Rc::new(AstroNode::AstroID(AstroID::new("times_two".to_string())));
        let func1 = Rc::new(AstroNode::AstroFunction(AstroFunction::new( Rc::clone(&id1) )));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = unify( exp_val, Rc::clone(&id2), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(AstroNode::AstroLineInfo( AstroLineInfo{id:0,module:"prog.ast".to_string(),line_number:9}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id3 = Rc::new(AstroNode::AstroID(AstroID::new("times_two".to_string())));
        let id4 = Rc::new(AstroNode::AstroID(AstroID::new("y".to_string())));
        let i1 = Rc::new(AstroNode::AstroInteger(AstroInteger::new(15)));
        let apply1 = Rc::new(AstroNode::AstroApply(AstroApply::new( Rc::clone(&id3), Rc::clone(&i1) )));

        let exp_val = match  walk( Rc::clone(&apply1), &mut state) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id4), &mut state, true) {
            Ok( val ) => val,
            Err((_type,msg)) => panic!("{}: {}: {}: {}",_type,state.lineinfo.0,state.lineinfo.1,msg),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("y",true);

        let AstroNode::AstroInteger(AstroInteger{id:_,value:v}) = *check1
            else {panic!("test failed")}; 
        assert_eq!(30,v);
    }
}