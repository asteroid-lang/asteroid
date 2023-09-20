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
 
use regex::Regex; //Regular expressions
use std::process; //exit()
use std::collections::HashMap;
use std::rc::Rc; 
use std::cell::RefCell;

static OPERATOR_SYMBOLS: [&str; 12] = [ "__plus__", "__minus__", "__times__", "__divide__", "__or__", "__and__", "__eq__", 
                                        "__ne__", "__lt__", "__le__", "__ge__", "__gt__" ];
static BINARY_OPERATORS: [&str; 12] = [ "__plus__", "__minus__", "__times__", "__divide__", "__or__", "__and__", "__eq__", 
                                        "__ne__", "__lt__", "__le__", "__ge__", "__gt__" ];

/******************************************************************************/
pub fn unify<'a>( term: Rc<Node>, pattern: Rc<Node>, state: &'a mut State, unifying: bool) -> Result<Vec<(Rc<Node>,Rc<Node>)>, Error >{
   
    let term_type = peek( Rc::clone(&term) );
    let pattern_type = peek( Rc::clone(&pattern) );

    //println!("Unifying: {} and {}",term_type,pattern_type);

    if term_type == "string" && (pattern_type != "id" && pattern_type != "index") { // Apply regular expression pattern match
        
        if pattern_type == "string" {
            // Note: a pattern needs to match the whole term.
            let Node::AstroString(AstroString{value:ref t_value}) = *term 
                else {return(Err(Error::VMError("Unify: expected string.".to_string())))};
            let Node::AstroString(AstroString{value:ref p_value}) = *pattern 
                else {return(Err(Error::VMError("Unify: expected string.".to_string())))};

            let mut re_str = String::from(r"^");
            re_str.push_str(&p_value);
            re_str.push_str("$");
            let re = Regex::new(&re_str).unwrap();

            if re.is_match(&t_value) {
                Ok( vec![] ) // Return an empty unifier
            } else {
                Err( Error::PatternMatchFailed( format!("regular expression {} did not match {}",term2string(&pattern).unwrap(),term2string(&term).unwrap())))
            }
        } else {
            Err( Error::PatternMatchFailed( format!("regular expression {} did not match {}",term2string(&pattern).unwrap(),term2string(&term).unwrap())))
        }
    } else if (term_type == "integer" || term_type == "bool" || term_type == "real") && (pattern_type == "integer" || pattern_type == "bool" || pattern_type == "real")  {

        if term_type == pattern_type && term == pattern {
            Ok( vec![] ) // Return an empty unifier
        } else {
            Err( Error::PatternMatchFailed( format!("{} is not the same as {}",term2string(&pattern).unwrap(),term2string(&term).unwrap())))
        }

    } else if !unifying && term_type == "namedpattern" {

        // Unpack a term-side name-pattern if evaluating redundant clauses
        let Node::AstroNamedPattern( AstroNamedPattern{name:_,pattern:ref t_pattern}) = *term
            else {return(Err(Error::VMError("Unify: expected named pattern.".to_string())))};

        unify( Rc::clone( t_pattern), pattern, state, unifying )

    } else if !unifying && term_type == "deref" {

        let Node::AstroDeref(AstroDeref{expression:ref t_expression}) = *term
            else {return(Err(Error::VMError("Unify: expected derefence expression.".to_string())))};

        let Node::AstroID(AstroID{name:ref t_name}) = **t_expression
            else {return(Err(Error::VMError("Unify: expected derefence expression.".to_string())))};

        let term = state.lookup_sym( &t_name, true );

        unify( term, pattern, state, unifying )
    
    /* ** Asteroid value level matching ** */
    } else if term_type == "object" && pattern_type == "object" {
        // this can happen when we dereference a variable pointing
        // to an object as a pattern, e.g.
        //    let o = A(1,2). -- A is a structure with 2 data members
        //    let *o = o.
        let Node::AstroObject(AstroObject{struct_id:ref t_id,object_memory:ref t_data}) = *term
            else {return(Err(Error::VMError("Unify: expected object.".to_string())))};
        let Node::AstroObject(AstroObject{struct_id:ref p_id,object_memory:ref p_data}) = *pattern
            else {return(Err(Error::VMError("Unify: expected object.".to_string())))};

        let AstroID{name:t_name} = t_id;
        let AstroID{name:p_name} = p_id;

        if t_name != p_name {
            Err( Error::PatternMatchFailed( format!("pattern type {} and term type {} do not agree.",t_name,p_name)))
        } else {
            let mut unifiers = vec![];
            for i in 0..t_data.borrow().len() {
                let mut unifier = match unify( Rc::clone(&t_data.borrow()[i]) , Rc::clone(&p_data.borrow()[i]),state,unifying) {
                    Ok( val ) => val,
                    Err( e ) => return Err( e )
                };
                unifiers.append( &mut unifier );
            }
            Ok(unifiers)
        }

    } else if pattern_type == "string" && term_type != "string" {

        let new_str = term2string(&term).unwrap();
        let new_term = AstroString{value:new_str};

        unify( Rc::new(Node::AstroString(new_term)),pattern,state,unifying )

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
                Err( Error::PatternMatchFailed( format!("Subsumption relatioship broken, pattern will not be rendered redundant.")))
            } 
        } else {

            let Node::AstroIf(AstroIf{ cond_exp:ref p_cond, then_exp:ref p_then, else_exp:ref p_else}) = *pattern
                else {return(Err(Error::VMError("Unify: expected if expression.".to_string())))};

            if let Node::AstroNone(AstroNone{}) = **p_else {

                let unifiers = match unify(term,Rc::clone(p_then),state,unifying) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
                if state.constraint_lvl > 0 {
                    state.push_scope();
                }

                // evaluate the conditional expression in the
                // context of the unifiers.
                declare_unifiers( &unifiers, state );
                let bool_val = match walk(Rc::clone(p_cond),state) {
                    Ok( val ) => map2boolean(&val),
                    Err( e ) => return Err(e),
                };

                if state.constraint_lvl > 0 {
                    state.pop_scope();
                }

                let Node::AstroBool(AstroBool{value:b_value}) = bool_val
                    else {return(Err(Error::VMError("Unify: expected boolean.".to_string())))};

                if b_value {
                    Ok( unifiers )
                } else {
                    Err( Error::PatternMatchFailed( "Conditional pattern match failed.".to_string()))
                }   
            } else {
                Err( Error::ValueError("Conditional patterns do not support else clauses.".to_string()))
            }
        }

    } else if term_type == "if" {
        // We will only get here when evaluating subsumption

        // If we get here, a conditional pattern clause is placed after a non-conditonal
        // pattern clause. Therefore, we need to check if the subsume because if they do
        // the conditonal clause is redundant.
        let Node::AstroIf(AstroIf{cond_exp:ref p_cond, then_exp:ref p_then, else_exp:ref p_else}) = *term
            else {return(Err(Error::VMError("Unify: expected if expression.".to_string())))};

        if let Node::AstroNone(AstroNone{}) = **p_else {
            unify( Rc::clone( p_then ),pattern,state,unifying  )
        } else {
            Err( Error::ValueError("Conditional patterns do not support else clauses.".to_string()))
        }

    } else if pattern_type == "typematch" {
        
        let Node::AstroTypeMatch(AstroTypeMatch{expression:ref p_exp}) = *pattern
            else {return(Err(Error::VMError("Unify: expected typematch.".to_string())))};
        //let look_next = 0u32; // Indicates what index we will look into next

        let Node::AstroString(AstroString{value:ref p_type}) = **p_exp
            else {return(Err(Error::VMError("Unify: expected string.".to_string())))};

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
                Err( Error::PatternMatchFailed( format!("Expected typematch: {}, got a term of type {}",p_type,term_type)) )
            }
        } else if p_type == "function" {
            //  matching function and member function values
            if ["function-val","member-function-val"].contains( &term_type ){
                Ok( vec![] )
            } else {
                Err( Error::PatternMatchFailed( format!("Expected typematch: {}, got a term of type {}",p_type,term_type)) ) 
            }
        } else if p_type == "pattern" {
            if term_type == "quote" {
                Ok( vec![] )
            } else {
                Err( Error::PatternMatchFailed( format!("Expected typematch: {}, got a term of type {}",p_type,term_type)) )
            }
        } else if p_type == "object" {
            let Node::AstroObject(AstroObject{struct_id:ref t_id,object_memory:ref t_mem}) = *term
                else {return(Err(Error::VMError("Unify: expected object.".to_string())))};
            let AstroID{name:t_type} = t_id;

            if p_type == t_type {
                Ok( vec![] )
            } else {
                Err( Error::PatternMatchFailed( format!("Expected typematch: {}, got a term of type {}",p_type,term_type)))
            }
        } else {
            // Check if the typematch is in the symbol table
            let in_symtab = state.find_sym(p_type);
            match in_symtab {
                None => return Err( Error::PatternMatchFailed( format!("{} is not a valid type for typematch",p_type))),
                Some(_) => (),
            };

            // If it is in the symbol table but not a struct, it cannot be typematched
            // because it is not a type
            if peek( state.lookup_sym( p_type,true ) ) != "struct" {
                Err( Error::PatternMatchFailed( format!("{} is not a type",p_type)) )
            } else { 
                //Otherwhise, the typematch has failed
                Err( Error::PatternMatchFailed( format!("Expected typematch: {}, got a term of type {}",p_type,term_type)))
            }
        }
    } else if pattern_type == "namedpattern" {

        let Node::AstroNamedPattern(AstroNamedPattern{name:ref p_name,pattern:ref p_pattern}) = *pattern
            else {return(Err(Error::VMError("Unify: expected named pattern.".to_string())))};

        // name_exp can be an id or an index expression.
        let mut unifiers = unify( Rc::clone(&term), Rc::clone(p_pattern),state,unifying );

        let mut unifiers = match unifiers {
            Ok( val ) => val,
            Err( val ) => return Err(val),
        };

        unifiers.push( (Rc::new(Node::AstroID(p_name.clone())), Rc::clone(&term)) );
        Ok( unifiers )

    } else if pattern_type == "none" {
        if term_type == "none" {
            Err( Error::PatternMatchFailed( format!("expected 'none' got '{}'",term_type)))
        } else {
            Ok( vec![] )
        }
    // NOTE: functions/foreign are allowed in terms as long as they are matched
    // by a variable in the pattern - anything else will fail
    } else if ["tolist","rawtolist","wherelist","rawwherelist","if","escape","is","in"].contains( &term_type ) {
        Err( Error::PatternMatchFailed( format!("term of type '{}' not allowed in pattern matching",term_type)))

    } else if ["tolist","rawtolist","wherelist","rawwherelist","if","escape","is","in","foreign","function"].contains( &pattern_type ) {
        Err( Error::PatternMatchFailed( format!("term of type '{}' not allowed in pattern matching",pattern_type)))

    } else if pattern_type == "quote" {

        // quotes on the pattern side can always be ignored
        let Node::AstroQuote(AstroQuote{expression:ref p_exp}) = *pattern
                else {return(Err(Error::VMError("Unify: expected quote.".to_string())))};

        if term_type == "quote" {
            let Node::AstroQuote(AstroQuote{expression:ref t_exp}) = *term
                else {return(Err(Error::VMError("Unify: expected quote.".to_string())))};

            unify(Rc::clone(&t_exp),Rc::clone(&p_exp),state,unifying)
        } else {
            unify(Rc::clone(&term),Rc::clone(&p_exp),state,unifying)
        }
    } else if term_type == "quote" && !(["id","index"].contains( &pattern_type))  {
        // ignore quote on the term if we are not trying to unify term with
        // a variable or other kind of lval
        let Node::AstroQuote(AstroQuote{expression:ref t_exp}) = *term
            else {return(Err(Error::VMError("Unify: expected quote.".to_string())))};

        unify( Rc::clone(&t_exp), pattern, state, unifying )

    } else if term_type == "object" && pattern_type == "apply" {

        let Node::AstroObject(AstroObject{struct_id:ref t_name,object_memory:ref t_mem}) = *term
            else {return(Err(Error::VMError("Unify: expected object.".to_string())))};
        let Node::AstroApply(AstroApply{function:ref p_func,argument:ref p_arg}) = *pattern
            else {return(Err(Error::VMError("Unify: expected apply.".to_string())))};
        let Node::AstroID(AstroID{name:ref p_id}) = **p_func
            else {return(Err(Error::VMError("Unify: expected string.".to_string())))};
        let AstroID{name:t_id} = t_name;

        
        if p_id != t_id {
            Err( Error::PatternMatchFailed( format!("expected type '{}' got type '{}'",p_id,t_id)) )
        } else if let Node::AstroTuple(AstroTuple{contents:ref content}) = **p_arg {
            //unify( Rc::clone(t_mem), Rc::clone(p_arg), state, unifying )
            let mut unifiers = vec![];
            for i in 0..content.borrow().len() {
                let mut unifier = match unify( Rc::clone(&t_mem.borrow()[i]) , Rc::clone(&content.borrow()[i]),state,unifying) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
                unifiers.append( &mut unifier);
            }
            Ok(unifiers)
        } else {
            unify( Rc::clone(&t_mem.borrow()[0]), Rc::new(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(p_arg)]))))) , state, unifying )
        }
        
    } else if pattern_type == "index" {
        // list element lval access
        Ok( vec![(Rc::clone(&pattern),Rc::clone(&term))] )
    
    //} else if term_type == "id" && unifying {
        // variable in term not allowed when unifying
    //    let Node::AstroID(AstroID{name:ref t_name}) = *term
    //        else {panic!("Unify: expected id.")};

    //    Err(  ("PatternMatchFailed",format!("variable '{}' in term not allowed.",t_name)))

    } else if pattern_type == "id" {

        let Node::AstroID(AstroID{name:ref p_name}) = *pattern
            else {return(Err(Error::VMError("Unify: expected id.".to_string())))};       

        if p_name == "_" {
            Ok( vec![] )
        } else {
            Ok( vec![(Rc::clone(&pattern),Rc::clone(&term))] )
        }

    } else if ["headtail","rawheadtail"].contains(&pattern_type) {


        let Node::AstroList(AstroList{contents:ref t_contents}) = *term
            else {return( Err(Error::PatternMatchFailed( format!("head-tail operator expected type 'list' got type '{}'",peek(Rc::clone(&term))))))};

        let (head,tail) = match *pattern {
            Node::AstroHeadTail(AstroHeadTail{ref head,ref tail}) => (head,tail),
            Node::AstroRawHeadTail(AstroRawHeadTail{ref head,ref tail}) => (head,tail),
            _ => return Err(Error::PatternMatchFailed( format!("Unify: expected head-tail."))),
        };

        if t_contents.borrow().len() == 0 {
            return Err(Error::PatternMatchFailed( format!("head-tail operator expected a non-empty list")));
        }

        let list_head = Rc::clone(&t_contents.borrow()[0]);
        let list_tail = Rc::new(Node::AstroList(AstroList::new( Rc::new(RefCell::new(t_contents.borrow_mut().split_off(1))))));

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
            Err(Error::PatternMatchFailed( format!("term and pattern do not agree on list/tuple constructor")))
        } else {

            let Node::AstroList(AstroList{contents:ref t_contents}) = *term
                else {return(Err(Error::VMError("Unify: expected list.".to_string())))};
            let Node::AstroList(AstroList{contents:ref p_contents}) = *pattern
                else {return(Err(Error::VMError("Unify: expected list.".to_string())))};


            if t_contents.borrow().len() != p_contents.borrow().len() {
                Err(Error::PatternMatchFailed( format!("term and pattern lists/tuples are not the same length")))
            } else {
                let mut unifiers = vec![];
                for i in 0..(t_contents.borrow().len()) {
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

        let Node::AstroDeref( AstroDeref{expression:ref exp}) = *pattern
            else {return(Err(Error::VMError("Unify: expected deref".to_string())))};

        let p = match walk( Rc::clone(&exp),state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };

        unify(term,p,state,unifying)

    // builtin operators look like apply lists with operator names
    } else if pattern_type == "apply" {
        if term_type != "apply" {
            Err(Error::PatternMatchFailed("term and pattern disagree on \'apply\' node".to_string()) )
        } else {

            // unpack the apply structures
            let Node::AstroApply(AstroApply{function:ref p_func,argument:ref p_arg}) = *pattern
                else {return(Err(Error::VMError("Unify: expected apply.".to_string())))};
            let Node::AstroApply(AstroApply{function:ref t_func,argument:ref t_arg}) = *term
                else {return(Err(Error::VMError("Unify: expected apply.".to_string())))};

            let Node::AstroID(AstroID{name:ref p_id}) = **p_func
                else {return(Err(Error::VMError("Unify: expected id.".to_string())))};
            let Node::AstroID(AstroID{name:ref t_id}) = **t_func
                else {return(Err(Error::VMError("Unify: expected id.".to_string())))};

            // make sure apply id's match
            if p_id != t_id {
                Err(Error::PatternMatchFailed(format!("term '{}' does not match pattern '{}'",t_id,p_id) ))
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
        Err(Error::PatternMatchFailed(format!("nodes '{}' and '{}' are not the same",peek(Rc::clone(&term)),peek(Rc::clone(&pattern)))))

    } else { 

        let mut unifiers: Vec<(Rc<Node>,Rc<Node>)> = vec![];
        let mut len: usize;
        let mut content: Vec<Rc<Node>>;

        if let Node::AstroTuple(AstroTuple{contents:ref t_content}) = *term {
            if let Node::AstroTuple(AstroTuple{contents:ref p_content}) = *pattern {

                for i in 0..t_content.borrow().len() {
                    let mut unifier = match unify( Rc::clone(&t_content.borrow()[i]),Rc::clone(&p_content.borrow()[i]),state,unifying) {
                        Ok( val ) => val,
                        Err( e ) => return Err(e),
                    };
                    unifiers.append( &mut unifier );
                }
                Ok( unifiers )
            } else {
                Err(Error::PatternMatchFailed(format!("nodes '{}' and '{}' are not the same",peek(Rc::clone(&term)),peek(Rc::clone(&pattern)))))
            }
        } else if let Node::AstroList(AstroList{contents:ref t_content}) = *term {
            if let Node::AstroList(AstroList{contents:ref p_content}) = *pattern { 


                for i in 0..t_content.borrow().len() {
                    let mut unifier = match unify( Rc::clone(&t_content.borrow()[i]),Rc::clone(&p_content.borrow()[i]),state,unifying) {
                        Ok( val ) => val,
                        Err( e ) => return Err(e),
                    };
                    unifiers.append( &mut unifier  );
                }
                Ok( unifiers )
            } else {
                Err(Error::PatternMatchFailed(format!("nodes '{}' and '{}' are not the same",peek(Rc::clone(&term)),peek(Rc::clone(&pattern)))))
            }
        } else {
            Err(Error::PatternMatchFailed(format!("nodes '{}' and '{}' are not the same",peek(Rc::clone(&term)),peek(Rc::clone(&pattern)))))
        }
    }
}


/******************************************************************************/
pub fn walk<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{ 

    //println!("Walking: {}",peek(Rc::clone(&node)));

    match *node {
        Node::AstroInteger(_) => Ok(node),
        Node::AstroReal(_) => Ok(node),
        Node::AstroBool(_) => Ok(node),
        Node::AstroString(_) => Ok(node),
        Node::AstroLineInfo(_) => set_lineinfo(node, state),
        Node::AstroList(_) => list_exp(node, state),
        Node::AstroTuple(_) => tuple_exp(node, state),
        Node::AstroNone(_) => Ok(node),
        Node::AstroNil(_) => Ok(node),
        Node::AstroFunction(_) => function_exp(node,state),
        Node::AstroToList(_) => to_list_exp(node,state),
        Node::AstroRawToList(_) => raw_to_list_exp(node,state),
        Node::AstroHeadTail(_) => head_tail_exp(node,state),
        Node::AstroRawHeadTail(_) => raw_head_tail_exp(node,state),
        Node::AstroSequence(_) => sequence_exp(node,state),
        Node::AstroObject(_) => Ok(node),
        Node::AstroEval(_) => eval_exp(node,state),
        Node::AstroQuote(_) => quote_exp(node,state),
        Node::AstroConstraint(_) => constraint_exp(node,state),
        Node::AstroTypeMatch(_) => constraint_exp(node,state),
        Node::AstroForeign(_) => Ok(node),
        Node::AstroID(_) => id_exp(node,state),
        Node::AstroApply(_) => apply_exp(node,state),
        Node::AstroIndex(_) => index_exp(node,state),
        Node::AstroEscape(_) => escape_exp(node,state),
        Node::AstroIs(_) => is_exp(node,state),
        Node::AstroIn(_) => in_exp(node,state),
        Node::AstroIf(_) => if_exp(node,state),
        Node::AstroNamedPattern(_) => named_pattern_exp(node,state),
        Node::AstroMemberFunctionVal(_) => Ok(node),
        Node::AstroDeref(_) => deref_exp(node,state),
        _ => return(Err(Error::VMError("Unknown node type in walk function.".to_string()))),
    }    
}
/******************************************************************************/
pub fn set_lineinfo<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    match *node {
        Node::AstroLineInfo(AstroLineInfo{ref module,line_number}) => state.lineinfo = (module.clone(),line_number),
        _ => return(Err(Error::VMError("lineinfo error.".to_string()))),
    }
    Ok( node )
}
/******************************************************************************/
pub fn list_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroList( AstroList{ref contents} ) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected list in list_exp()".to_string()))) };

    let len = contents.borrow().len();
    for i in 0..len {
        let val = match walk( Rc::clone(&contents.borrow()[i]), state) {
            Ok( x ) => x,
            Err( x ) => return Err(x),
        };
        contents.borrow_mut()[i] = val;
    }
    Ok( node ) 
}
/******************************************************************************/
pub fn tuple_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroTuple( AstroTuple{ref contents} ) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected tuple in tuple_exp()".to_string()))) };

    let len = contents.borrow().len();
    for i in 0..len {
        let val = match walk( Rc::clone(&contents.borrow()[i]), state) {
            Ok( x ) => x,
            Err( x ) => return Err(x),
        };
        contents.borrow_mut()[i] = val;
    }
    Ok( node ) 
}
/******************************************************************************/
pub fn to_list_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroToList(AstroToList{ref start,ref stop,ref stride}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected to_list in to_list_exp()".to_string()))) }; 

    let mut start_val;
    let mut stop_val;
    let mut stride_val;

    {
        let start = match walk(start.clone(),state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let Node::AstroInteger(AstroInteger{value}) = *start 
            else { return(Err(Error::VMError("ERROR: walk: expected integer in to_list_exp()".to_string()))) };
        start_val= value;
    }

    {
        let stop = match walk(stop.clone(),state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let Node::AstroInteger(AstroInteger{value}) = *stop
            else { return(Err(Error::VMError("ERROR: walk: expected integer in to_list_exp()".to_string()))) };
        stop_val = value;
    }

    {
        let stride = match walk(stride.clone(),state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let Node::AstroInteger(AstroInteger{value}) = *stride
            else { return(Err(Error::VMError("ERROR: walk: expected integer in to_list_exp()".to_string()))) };
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
        newlist.push(Rc::new(Node::AstroInteger(AstroInteger::new( i ))));
    }

    Ok( Rc::new(Node::AstroList( AstroList::new(Rc::new(RefCell::new(newlist))))))
}
/******************************************************************************/
pub fn function_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroFunction(AstroFunction{ref body_list}) = *node
        else {return(Err(Error::VMError("ERROR: walk: expected function in function_exp()".to_string())))};

    Ok( Rc::new(Node::AstroFunctionVal(AstroFunctionVal::new(Rc::clone(body_list), Rc::new(state.symbol_table.get_config()) ))))
}
/******************************************************************************/
pub fn raw_to_list_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroRawToList(AstroRawToList{ref start,ref stop,ref stride}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected to_list in to_list_exp()".to_string()))) }; 

    walk( Rc::new( Node::AstroToList( AstroToList{start:(*start).clone(),stop:(*stop).clone(),stride:(*stride).clone()} )), state)
}
/******************************************************************************/
pub fn head_tail_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroHeadTail(AstroHeadTail{ref head,ref tail}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected head-tail exp in head_tail_exp().".to_string()))) }; 

    let Node::AstroList( AstroList{ref contents} ) = **tail
        else { return(Err(Error::VMError("ERROR: unsupported tail type in head-tail operator.".to_string()))) };

    let mut new_contents = Vec::with_capacity(contents.borrow().len());
    new_contents.push(head.to_owned());
    for content in &*(contents.borrow()) {
        new_contents.push(content.to_owned());
    }

    Ok( Rc::new( Node::AstroList( AstroList::new( Rc::new(RefCell::new(new_contents)))))) 
}
/******************************************************************************/
pub fn raw_head_tail_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroRawHeadTail(AstroRawHeadTail{ref head,ref tail}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected raw head-tail exp in raw_head_tail_exp().".to_string()))) }; 

    walk( Rc::new( Node::AstroHeadTail( AstroHeadTail{head:head.to_owned(),tail:tail.to_owned()})), state)
}
/******************************************************************************/
pub fn sequence_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroSequence(AstroSequence{ref first,ref second}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected sequence expression in sequence_exp().".to_string()))) };  

    let first = match walk( Rc::clone(&first),state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let second = match walk( Rc::clone(&second),state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    Ok( Rc::new( Node::AstroSequence( AstroSequence{first:first,second:second})))
}
/******************************************************************************/
pub fn eval_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroEval(AstroEval{ref expression}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected eval expression in exal_exp().".to_string()))) };  

    // Note: eval is essentially a macro call - that is a function
    // call without pushing a symbol table record.  That means
    // we have to first evaluate the argument to 'eval' before
    // walking the term.  This is safe because if the arg is already
    // the actual term it will be quoted and nothing happen
    let exp_value_expand = match walk( (*expression).clone(),state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    // now walk the actual term..
    state.ignore_quote_on();
    let exp_val = match walk( exp_value_expand,state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    state.ignore_quote_off();

    Ok(exp_val)
}
/******************************************************************************/
pub fn quote_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroQuote(AstroQuote{ref expression}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected quote expression in quote_exp().".to_string()))) };  

    // quoted code should be treated like a constant if not ignore_quote
    if state.ignore_quote {
        walk( Rc::clone(expression) ,state)
    } else {
        Ok( node )
    }
}
/******************************************************************************/
pub fn constraint_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    //let Node::AstroConstraint(AstroConstraint{id,expression}) = node 
    //    else { panic!("ERROR: walk: expected constraint exp in constraint_exp().") };

    return(Err(Error::VMError("Constraint patterns cannot be used as constructors.".to_string())));
}
/******************************************************************************/
pub fn id_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error> {
    let Node::AstroID(AstroID{ref name}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected id expression in id_exp().".to_string()))) }; 
    
    Ok( state.lookup_sym(name,true).clone() )
}
/******************************************************************************/
pub fn apply_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroApply(AstroApply{ref function,ref argument}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected apply expression in apply_exp().".to_string()))) }; 

    // handle builtin operators that look like apply lists.
    if let Node::AstroID( AstroID{name:ref tag}) = **function {

        if OPERATOR_SYMBOLS.contains( &(tag.as_str()) ) {
            handle_builtins( Rc::clone(&node), state)

        } else{
            // handle function application
            let f_val = match walk( Rc::clone(&function), state) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
            let f_name = tag;
            let arg_val = match  walk( Rc::clone(&argument), state) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };

            let _type = peek( Rc::clone(&f_val));

            if _type == "functionval" {
                return handle_call( Rc::new(Node::AstroNone(AstroNone::new())), Rc::clone(&f_val), Rc::clone(&arg_val), state );

            } else if _type == "struct" {
                // object constructor call

                let Node::AstroStruct(AstroStruct{member_names:ref mnames,struct_memory:ref struct_mem}) = *f_val
                    else {return(Err(Error::VMError("Error: apply exp: expected struct.".to_string())))};

                // create our object memory - memory cells now have initial values
                // we use structure memory as an init template
                let mut obj_memory = Rc::new(RefCell::new((struct_mem.borrow()).clone()));
                let new_id = AstroID::new(tag.to_string());
                //let new_mem = Node::AstroList(AstroList::new(obj_memory.len(), Rc::new(obj_memory)).unwrap());
                let obj_ref = Rc::new(Node::AstroObject(AstroObject::new(new_id,Rc::clone(&obj_memory))));

                for element in (&*mnames.borrow()) {
                    if let Node::AstroID(AstroID{name:ref tag}) = *Rc::clone(&element) {
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

                let Node::AstroTuple(AstroTuple{contents:ref content}) = *arg_val
                    else {return(Err(Error::VMError("Error: apply exp: expected tuple.".to_string())))};
                
                
                let data_memory = data_only( RefCell::clone(&obj_memory) );

                if content.borrow().len() != data_memory.len() {
                    return Err(Error::ValueError(format!("default constructor expected {} arguments got {}",content.borrow().len(),data_memory.len())));
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
pub fn handle_call<'a>( obj_ref: Rc<Node>, node: Rc<Node>, args: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{

    let Node::AstroFunctionVal(AstroFunctionVal{body_list:ref fpointer,ref closure}) = *node
        else {return(Err(Error::VMError("ERROR: handle call: expected function value.".to_string())))};

    let Node::AstroID(AstroID{name:ref fname}) = **fpointer
        else {return(Err(Error::VMError("ERROR: handle_call: expected id for function name.".to_string())))};

    // static scoping for functions
    // Note: we have to do this here because unifying
    // over the body patterns can introduce variable declarations,
    // think conditional pattern matching.
    let save_symtab = state.symbol_table.get_config();
    //state.symbol_table.set_config( closure.0.clone(), closure.1.clone(), closure.2 );
    state.push_scope();

    if let Node::AstroNone(AstroNone{}) = *obj_ref {
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
pub fn handle_builtins<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{

    let Node::AstroApply(AstroApply{ref function,ref argument}) = *node 
        else { return(Err(Error::VMError("ERROR: handle_builtins: expected apply expression.".to_string()))) }; 
    let Node::AstroID( AstroID{name:ref builtin_type} ) = **function
        else { return(Err(Error::VMError("ERROR: handle_builtins: expected id. ".to_string())))};

    if BINARY_OPERATORS.contains( &builtin_type.as_str() ) {
        
        let Node::AstroTuple( AstroTuple{contents:ref args}) = **argument
            else {return(Err(Error::VMError("ERROR: handle_builtins: expected tuple for args.".to_string())))};

        let val_a = match walk( Rc::clone(&args.borrow()[0]), state ) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let val_b = match walk( Rc::clone(&args.borrow()[1]), state ) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        
        if builtin_type == "__plus__" {
            
            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroInteger( AstroInteger::new(v1+v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 as f64 + v2))));
                } else if let Node::AstroString( AstroString{value:ref v2}) = *val_b {
                        return Ok( Rc::new( Node::AstroString(AstroString::new(v1.to_string()+v2))));
                } else {
                    return Err(Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 + v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 + v2))));
                } else if let Node::AstroString( AstroString{value:ref v2}) = *val_b {
                    return Ok( Rc::new( Node::AstroString(AstroString::new(v1.to_string()+v2))));
                } else {
                    return Err(Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroList( AstroList{contents:ref c1}) = *val_a {
                if let Node::AstroList( AstroList{contents:ref c2}) = *val_b {
                    let mut c3 = (**c1).clone(); // we have to do a data-clone here otherwise we edit other nodes in place
                    c3.borrow_mut().append( &mut (*c2.borrow_mut())) ;
                    return Ok( Rc::new( Node::AstroList( AstroList::new(Rc::new( c3 )))));
                } 
                
            } else if let Node::AstroString( AstroString{value:ref v1}) = *val_a {
                if let Node::AstroString( AstroString{value:ref v2}) = *val_b {
                    return Ok( Rc::new( Node::AstroString(AstroString::new(v1.to_owned()+v2))));
                } else if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new( Node::AstroString(AstroString::new(v1.to_owned()+&v2.to_string()))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new( Node::AstroString(AstroString::new(v1.to_owned()+&v2.to_string()))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else {
                return Err( Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
            }

        } else if builtin_type == "__minus__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroInteger( AstroInteger::new(v1 - v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 as f64 - v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 - v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 - v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only subtract real/integers
                return Err( Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
            }

        } else if builtin_type == "__times__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroInteger( AstroInteger::new(v1 * v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 as f64 * v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 * v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 * v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only multiply real/integers
                return Err( Error::ValueError( format!("Unsupported type {} in +", peek(Rc::clone(&val_b)))));
            }    
        } else if builtin_type == "__divide__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    if v2 == 0 { // Divison by 0 check
                        return Err(Error::ArithmeticError("Division by zero".to_string()));
                    } else {
                        return Ok( Rc::new(Node::AstroInteger( AstroInteger::new(v1 / v2))));
                    }
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    if v2 == 0.0 { // Divison by 0 check
                        return Err( Error::ArithmeticError("Division by zero".to_string()));
                    } else {
                        return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 as f64 / v2))));
                    }
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in /", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    if v2 == 0 { // Divison by 0 check
                        return Err(Error::ArithmeticError("Division by zero".to_string()));
                    } else {
                        return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 / v2 as f64))));
                    }
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    if v2 == 0.0 { // Divison by 0 check
                        return Err(Error::ArithmeticError("Division by zero".to_string()));
                    } else {
                        return Ok( Rc::new(Node::AstroReal( AstroReal::new(v1 / v2))));
                    }
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in /", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only divide real/integers
                return Err( Error::ValueError( format!("Unsupported type {} in /", peek(Rc::clone(&val_b)))));
            }    
        } else if builtin_type == "__or__" {

            let b1 = map2boolean( &val_a);
            let b2 = map2boolean( &val_b);
            let Node::AstroBool( AstroBool{value:b1_val}) = b1
                else {return(Err(Error::VMError("handle_builtins: expected boolean.".to_string())))};
            let Node::AstroBool( AstroBool{value:b2_val}) = b2
                else {return(Err(Error::VMError("handle_builtins: expected boolean.".to_string())))};

            return Ok( Rc::new(Node::AstroBool( AstroBool::new(b1_val || b2_val))));
        } else if builtin_type == "__and__" {

            let b1 = map2boolean( &val_a);
            let b2 = map2boolean( &val_b);
            let Node::AstroBool( AstroBool{value:b1_val}) = b1
                else {return(Err(Error::VMError("handle_builtins: expected boolean.".to_string())))};
            let Node::AstroBool( AstroBool{value:b2_val}) = b2
                else {return(Err(Error::VMError("handle_builtins: expected boolean.".to_string())))};

            return Ok( Rc::new(Node::AstroBool( AstroBool::new(b1_val && b2_val))));
        } else if builtin_type == "__gt__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 > v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 as f64 > v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in >", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 > v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 > v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in >", peek(Rc::clone(&val_b)))));
                }

            } else { 
                return Err( Error::ValueError( format!("Unsupported type {} in >", peek(Rc::clone(&val_b)))));
            }

        } else if builtin_type == "__lt__" {
         
            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 < v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new((v1 as f64) < v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in <", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 < v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 < v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in <", peek(Rc::clone(&val_b)))));
                }

            } else { 
                return Err( Error::ValueError( format!("Unsupported type {} in <", peek(Rc::clone(&val_b)))));
            }

        } else if builtin_type == "__le__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 <= v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new((v1 as f64) <= v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in <=", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 <= v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 <= v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in <=", peek(Rc::clone(&val_b)))));
                }

            } else { 
                return Err( Error::ValueError( format!("Unsupported type {} in <=", peek(Rc::clone(&val_b)))));
            }

        } else if builtin_type == "__ge__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 >= v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 as f64 >= v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in >=", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 >= v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 >= v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in >=", peek(Rc::clone(&val_b)))));
                }

            } else { // We can only subtract real/integers
                return Err( Error::ValueError( format!("Unsupported type {} in >=", peek(Rc::clone(&val_b)))));
            }
        } else if builtin_type == "__eq__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 == v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 as f64 == v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in ==", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 == v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 == v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in ==", peek(Rc::clone(&val_b)))));
                }

            } else { // TODO
                return Err( Error::ValueError( format!("Unsupported type {} in ==", peek(Rc::clone(&val_b)))));
            }
        } else if builtin_type == "__ne__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 != v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 as f64 != v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in =/=", peek(Rc::clone(&val_b)))));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 != v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( Rc::new(Node::AstroBool( AstroBool::new(v1 != v2))));
                } else {
                    return Err( Error::ValueError( format!("Unsupported type {} in =/=", peek(Rc::clone(&val_b)))));
                }

            } else { // TODO
                return Err( Error::ValueError( format!("Unsupported type {} in =/=", peek(Rc::clone(&val_b)))));
            }
        }
    

        
    }
    Ok(node)
}
/******************************************************************************/
pub fn index_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroIndex(AstroIndex{ref structure,ref index_exp}) = *node 
        else {return(Err(Error::VMError("ERROR: walk: expected index expression in index_exp().".to_string()))) }; 

    // look at the semantics of 'structure'
    let structure_val =  match walk(Rc::clone(&structure),state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    // indexing/slicing
    let result = match read_at_ix(structure_val,Rc::clone(&index_exp),state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    Ok(result)
}
/******************************************************************************/
pub fn escape_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{

    let Node::AstroEscape(AstroEscape{content:ref fname}) = *node
        else {return(Err(Error::VMError("escape_exp(): expected ID.".to_string())))};
    
    let old_lineinfo = state.lineinfo.clone();
    let return_value = state.dispatch_table[ fname.as_str() ]( Rc::new(Node::AstroNone(AstroNone::new())), state );

    //  coming back from a function call - restore caller's lineinfo
    state.lineinfo = old_lineinfo;

    return_value
}
/******************************************************************************/
pub fn is_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroIs(AstroIs{ref pattern,ref term}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected id expression in id_exp().".to_string()))) }; 

    let term_val = match walk((*term).clone(), state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let unifiers = unify(term_val,(*pattern).clone(),state,true);

    if let Err(_) = unifiers {
        Ok( Rc::new( Node::AstroBool(AstroBool::new(false))))
    } else {
        let unifiers = match unifiers {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        declare_unifiers(&unifiers,state);
        Ok( Rc::new( Node::AstroBool(AstroBool::new(true))))
    }
}
/******************************************************************************/
pub fn in_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroIn(AstroIn{ref expression,ref expression_list}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected id expression in id_exp().".to_string()))) }; 

    let exp_val = match walk((*expression).clone(),state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let exp_list_val = match walk((*expression_list).clone(),state) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let Node::AstroList(AstroList{ref contents}) = *exp_list_val
        else { return(Err(Error::VMError("Right argument to in operator has to be a list.".to_string())))};

    // We simply map the in operator to Rust's contains function
    if (*contents).borrow().contains( &exp_val ) {
        Ok( Rc::new( Node::AstroBool(AstroBool::new(true))))
    } else {
        Ok( Rc::new( Node::AstroBool(AstroBool::new(false))))
    }
}
/******************************************************************************/
pub fn if_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroIf(AstroIf{ref cond_exp,ref then_exp,ref else_exp}) = *node 
        else { return(Err(Error::VMError("ERROR: walk: expected id expression in id_exp().".to_string()))) }; 

    
    let cond_val = match walk( Rc::clone(&cond_exp), state ) {
        Ok( val ) => map2boolean(&val),
        Err( e ) => return Err(e),
    };
    
    let Node::AstroBool(AstroBool{value}) = cond_val 
        else {return(Err(Error::VMError("Expected boolean from map2boolean.".to_string())))};
    
    if value {
        walk(Rc::clone(&then_exp),state)
    } else {
        walk(Rc::clone(&else_exp),state)
    }
}
/*******************************************************************************
# Named patterns - when walking a named pattern we are interpreting a
# a pattern as a constructor - ignore the name                                */
pub fn named_pattern_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{
    let Node::AstroNamedPattern(AstroNamedPattern{ref name,ref pattern}) =* node 
        else { return(Err(Error::VMError("ERROR: walk: expected id expression in id_exp().".to_string()))) }; 

    walk((*pattern).clone(),state)
}
/******************************************************************************/
pub fn deref_exp<'a>( node: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{

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
fn check_repeated_symbols(unifiers: &Vec<(Rc<Node>,Rc<Node>)> ) -> bool {
    let len = unifiers.len();
    let mut seen = Vec::with_capacity(len);

    for i in 0..len {
        let next = peek( (unifiers[i].0).clone() );

        if next == "id" {
            let Node::AstroID(AstroID{ref name}) = *unifiers[i].0
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
pub fn declare_unifiers<'a>( unifiers: &Vec<(Rc<Node>,Rc<Node>)>, state: &'a mut State ) -> Result<(), Error >{
    // walk the unifiers and bind name-value pairs into the symtab

    for (lhs,value) in unifiers {

        if let Node::AstroID(AstroID{ref name}) = **lhs {
            if name == "this" {
                return Err( Error::ValueError("'this' is a reserved keyword.".to_string()));
            } else {
                state.enter_sym(&name,Rc::clone(value));
            }
        } else if let Node::AstroIndex(AstroIndex{ref structure,ref index_exp}) = **lhs {
            // Note: structures have to be declared before index access
            // can be successful!!  They have to be declared so that there
            // is memory associated with the structure.

            // indexing/slicing
            // update the memory of the object.
            store_at_ix(Rc::clone(structure),Rc::clone(index_exp),Rc::clone(value),state);
        } else {
            return Err( Error::ValueError(format!("unknown unifier type '{}'",peek(Rc::clone(lhs)))));
        }
    }
    Ok(())
}
/******************************************************************************/
pub fn declare_formal_args<'a>( unifiers: &Vec<(Rc<Node>,Rc<Node>)>, state: &'a mut State ) -> Result<(), Error >{
    // unfiers is of the format: [ (pattern, term), (pattern, term),...]

    for (pattern,term) in unifiers {
        if let Node::AstroID(AstroID{ref name}) = **pattern {
            if name == "this" {
                return Err( Error::ValueError("'this' is a reserved keyword.".to_string()));
            } else {
                state.enter_sym(&name,Rc::clone(term));
            }
        } else {
            return Err( Error::ValueError(format!("unknown unifier type '{}'",peek(Rc::clone(pattern)))));
        }
    }
    Ok(())
}
/******************************************************************************/
pub fn store_at_ix<'a>( structure: Rc<Node>, ix: Rc<Node>, value: Rc<Node>, state: &'a mut State ) -> Result<(), Error>{

    let mut structure_val = Rc::new(Node::AstroNone(AstroNone::new()));
    
    // Handle recurive application iteratively here.
    if let Node::AstroIndex(AstroIndex{structure:ref s,index_exp:ref idx}) = *structure {

        let mut inner_mem = Rc::clone(s);

        // Construct a list of all of the indices
        let ix_val = match walk(Rc::clone(&ix), state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let Node::AstroInteger(AstroInteger{value:v}) = *ix_val
            else {return(Err(Error::VMError("store_at_ix: expected integer index.".to_string())))};
        let mut idx_list = vec![ v ];
        while let Node::AstroIndex(AstroIndex{structure:ref s,index_exp:ref idx}) = **s {
            let Node::AstroInteger(AstroInteger{value:v}) = *ix_val
                else {return(Err(Error::VMError("store_at_ix: expected integer index.".to_string())))};
            idx_list.push(v);
            inner_mem = Rc::clone(s);
        }

        // Walk through the index list accessing memory until we reach the intended interior memory.
        let mut memory = match walk(Rc::clone(&inner_mem),state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        for val in idx_list {
            memory = match *memory {
                Node::AstroList( AstroList{contents:ref mem} ) => Rc::clone(&(**mem).borrow()[ val as usize ]),
                Node::AstroTuple( AstroTuple{contents:ref mem} ) => Rc::clone(&(**mem).borrow()[ val as usize ]),
                _ => return(Err(Error::VMError("store_at_ix: expected list or tuple.".to_string())))
            };
        }
        structure_val = match walk(Rc::clone(&memory),state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        
    } else {

        // look at the semantics of 'structure'
        structure_val = match walk(Rc::clone(&structure),state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
    }

    if let Node::AstroList( AstroList{contents:ref mem} ) = *structure_val {

        let ix_val = match walk(Rc::clone(&ix), state) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let Node::AstroInteger(AstroInteger{value:int_val}) = *ix_val // TODO error clean up
            else {return(Err(Error::VMError("store_at_ix: expected integer.".to_string())))};

        mem.borrow_mut()[int_val as usize] = Rc::clone(&value);
    
        Ok(()) 
    } else if let Node::AstroObject(AstroObject{struct_id:ref id,object_memory:ref mem}) = *structure_val {
        
        //let ix_val = walk(Rc::clone(&ix), state).unwrap();
        //println!("TYPE IS {}",peek(Rc::clone(&ix)));
        let Node::AstroID(AstroID{name:ref tag}) = *ix
            else {return(Err(Error::VMError("store_at_ix: expected id.".to_string())))};

        let AstroID{name:ref obj_type} = *id;
        let object_data = match walk( Rc::new(Node::AstroID(id.clone())), state ) {
            Ok( val ) => val,
            Err( error ) => return Err( error ),
        };

        let Node::AstroStruct(AstroStruct{member_names:ref struct_tags,struct_memory:ref struct_mem}) = *object_data
            else {return(Err(Error::VMError("store_at_ix: expected struct.".to_string())))};

        // find the location in the structs memory where we want to place the new value.
        let mut found_idx = 0usize;
        let mut found = false;
        let mut curr_idx = 0usize;
        for struct_member in (*struct_tags).borrow().iter() {
            if let Node::AstroID(AstroID{name:ref mem_tag}) = **struct_member {
                if mem_tag == tag {
                    found_idx = curr_idx;
                    found = true;
                }
            }
            curr_idx = curr_idx + 1;
        }
        
        //(mem.borrow_mut())[ found_idx ] = Rc::new( Node::AstroNone(AstroNone::new()) );
        (mem.borrow_mut())[ found_idx ] = Rc::clone( &value );

        Ok(()) 
    } else {
        Err( Error::ValueError(format!("Index op not supported for '{}'",peek(structure_val))))
    }
}
/******************************************************************************/
pub fn read_at_ix<'a>( structure_val: Rc<Node>, ix: Rc<Node>, state: &'a mut State ) -> Result<Rc<Node>, Error>{

    // find the actual memory we need to access
    let struct_type = peek(structure_val.clone());
    let ix_type = peek(Rc::clone(&ix));
    
    if ["list","tuple"].contains( &struct_type ) {
        if ix_type == "integer" {
            
            let Node::AstroInteger(AstroInteger{value:ix_val}) = *ix
                else {return(Err(Error::VMError("read_at_ix: expected integer.".to_string())))};

            let content = match *structure_val {
                Node::AstroList( AstroList{contents:ref c}) => c,
                Node::AstroTuple( AstroTuple{contents:ref c}) => c,
                _ => return(Err(Error::VMError("read_at_ix: expected list or tuple.".to_string()))),
            };

            
            return Ok( Rc::clone( &content.borrow()[ix_val as usize] ) );
        }
    } else if struct_type == "object" {

        let Node::AstroObject(AstroObject{struct_id:ref id,object_memory:ref mem}) = *structure_val
            else {return(Err(Error::VMError("read_at_ix: expected object.".to_string())))};

        let Node::AstroID(AstroID{name:ref tag}) = *ix
            else {return(Err(Error::VMError("read_at_ix: expected id.".to_string())))};

        let AstroID{name:ref obj_type} = *id;
        let object_data = match walk( Rc::new(Node::AstroID(id.clone())), state ) {
            Ok( val ) => val,
            Err( error ) => return Err( error ),
        };

        let Node::AstroStruct(AstroStruct{member_names:ref struct_tags,struct_memory:ref struct_mem}) = *object_data
            else {return(Err(Error::VMError("read_at_ix: expected struct.".to_string())))};

        // find the location in the structs memory where we want to place the new value.
        let mut found_idx = 0usize;
        let mut found = false;
        let mut curr_idx = 0usize;
        for struct_member in (*struct_tags).borrow().iter() {
            if let Node::AstroID(AstroID{name:ref mem_tag}) = **struct_member {
                if mem_tag == tag {
                    found_idx = curr_idx;
                    found = true;
                }
            }
            curr_idx = curr_idx + 1;
        }
        
        return Ok( Rc::clone( &mem.borrow_mut()[ found_idx ]) );

    } else if struct_type == "string" {

        let Node::AstroInteger(AstroInteger{value:ix_val}) = *ix
                else {return(Err(Error::VMError("read_at_ix: expected integer.".to_string())))};

        let content = match *structure_val {
            Node::AstroString( AstroString{value:ref val}) => val,
            _ => return(Err(Error::VMError("read_at_ix: expected string.".to_string()))),
        };

        match content.chars().nth( ix_val as usize) {
            Some( character ) => return Ok(Rc::new(Node::AstroString(AstroString::new(character.to_string())))),
            _                 => return Err( Error::ValueError(format!("String '{}' too short for index value {}",content,ix_val)) )
        }
    }

    Ok(structure_val.clone())
}
/******************************************************************************/
pub fn exit<'a>( error: Error , state: &'a mut State ) -> ! {
    println!("Asteroid encountered an error.");
    match error {
        Error::ValueError( msg ) => println!("Error Type: {}\nError Location:\n\tFile: {}\n\tLine: {}\nError Message: {}","Value Error",state.lineinfo.0,state.lineinfo.1,msg),
        Error::NonLinearPattern( msg ) => println!("Error Type: {}\nError Location: File: {} Line: {}\nError Message: {}","Non-Linear Pattern",state.lineinfo.0,state.lineinfo.1,msg),
        Error::FileNotFound( msg ) => println!("Error Type: {}\nError Location: File: {} Line: {}\nError Message: {}","File Not Found",state.lineinfo.0,state.lineinfo.1,msg),
        Error::RedundantPatternFound( msg ) => println!("Error Type: {}\nError Location: File: {} Line: {}\nError Message: {}","Overlapping Pattern",state.lineinfo.0,state.lineinfo.1,msg),
        Error::ArithmeticError( msg ) => println!("Error Type: {}\nError Location: File: {} Line: {}\nError Message: {}","Arithmetic Error",state.lineinfo.0,state.lineinfo.1,msg),
        Error::PatternMatchFailed( msg ) => println!("Error Type: {}\nError Location: File: {} Line: {}\nError Message: {}","Pattern Match Failed",state.lineinfo.0,state.lineinfo.1,msg),
        Error::VMError( msg) => println!("An internal compiler error occurred.\nError Location: File: {} Line: {}\nError Message: {}",state.lineinfo.0,state.lineinfo.1,msg),
    }
    println!("Exiting...");
    process::exit(1);
}
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
/******************************************************************************/



















#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unify_regex() {
        let s1 = Rc::new( Node::AstroString( AstroString::new(String::from("hello"))) );
        let s2 = Rc::new( Node::AstroString( AstroString::new(String::from("hello"))) );
        let s3 = Rc::new( Node::AstroString( AstroString::new(String::from("nothello"))) );

        let mut state = State::new().unwrap();
        let u = true;
        
        let out = match unify(s1.clone(),s2,&mut state,u) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };
        assert_eq!(out.len(),0); //SHOULD PASS

        let out = unify(s1,s3,&mut state,u);
        match out {
            Err(x) => (), //SHOULD BE ERR
            _ => panic!("Regex text failed"),
        }
    }
    #[test]
    fn test_unify_primitives() {
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let i3 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));

        let b1 = Rc::new( Node::AstroBool( AstroBool::new(true)));
        let b2 = Rc::new( Node::AstroBool( AstroBool::new(false)));
        let b3 = Rc::new( Node::AstroBool( AstroBool::new(true)));

        let r1 = Rc::new( Node::AstroReal( AstroReal::new(1.1)));
        let r2 = Rc::new( Node::AstroReal( AstroReal::new(1.2)));
        let r3 = Rc::new( Node::AstroReal( AstroReal::new(1.1)));

        let mut state = State::new().unwrap();
        let u_mode = true;

        let out1 = match unify(i1.clone(),i3,&mut state,u_mode){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };
        let out2 = match unify(b1.clone(),b3,&mut state,u_mode){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };
        let out3 = match unify(r1.clone(),r3,&mut state,u_mode){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

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

        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let i3 = Rc::new( Node::AstroInteger( AstroInteger::new(3)));

        let l1 = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));
        let l2 = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i2.clone(),i3.clone()])))));
        let l3 = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i3.clone(),i2.clone(),i1.clone()])))));
        let l4 = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));

        let out1 = match unify( Rc::clone(&l1),Rc::clone(&l4),&mut state,u_mode) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };
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

        walk( Rc::new( Node::AstroLineInfo(newline)),&mut state );

        {
            let out2 = state.lineinfo.clone();
            assert_eq!(out2,(String::from("test1"),123));
        }

        let newline = AstroLineInfo::new( String::from("math"), 987654321);
        walk( Rc::new(  Node::AstroLineInfo(newline)),&mut state );

        {
            let out3 = state.lineinfo.clone();
            assert_eq!(out3,(String::from("math"), 987654321));
        }
    }
    #[test]
    fn test_unify_var_to_int() {
        // let x = 123.

        let mut state = State::new().unwrap();
        let var = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let int = Rc::new(Node::AstroInteger(AstroInteger::new(123)));

        let unifier = match unify(int,var,&mut state,true) {
            Ok( val ) => val,
            Err(e) => panic!("error!"),
        };

        let out = declare_unifiers( &unifier, &mut state );

        let check = state.lookup_sym("x",true);
        match *check {
            Node::AstroInteger(AstroInteger{value:123}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_real() {
        // let x = 1.23.

        let mut state = State::new().unwrap();
        let var = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let val = Rc::new(Node::AstroReal(AstroReal::new(1.23)));

        let unifier = match unify(val,var,&mut state,true) {
            Ok(val) => val,
            Err(e) => panic!("error!"),
        };

        let out = declare_unifiers( &unifier, &mut state );

        let check = state.lookup_sym("x",true);
        match *check {
            Node::AstroReal(AstroReal{value:val}) if val == 1.23 => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_string() {
        // let x = "hello123".

        let mut state = State::new().unwrap();
        let var = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let val = Rc::new(Node::AstroString(AstroString::new("hello123".to_string())));

        let unifiers = match unify(val,var,&mut state,true) {
            Ok(val) => val,
            Err(e) => panic!("error!"),
        };

        let out = declare_unifiers( &unifiers, &mut state );

        let check = state.lookup_sym("x",true);
        match *check {
            Node::AstroString(AstroString{value:ref val}) if val == "hello123" => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_bool() {
        // let x = false.

        let mut state = State::new().unwrap();
        let var = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let val = Rc::new(Node::AstroBool(AstroBool::new(false)));

        let unifiers = match unify(val,var,&mut state,true){
            Ok(val) => val,
            Err(e) => panic!("Error"),
        };

        let out = declare_unifiers( &unifiers, &mut state );

        let check = state.lookup_sym("x",true);
        match *check {
            Node::AstroBool(AstroBool{value:val}) if val == false =>(),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_int_thrice() {
        // let x = 2.
        // let y = 4.
        // let z = 8.

        let mut state = State::new().unwrap();
        let var1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
        let var2 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
        let val2 = Rc::new(Node::AstroInteger(AstroInteger::new(4)));
        let var3 = Rc::new(Node::AstroID(AstroID::new("z".to_string())));
        let val3 = Rc::new(Node::AstroInteger(AstroInteger::new(8)));

        let unifiers = match unify(val1,var1,&mut state,true){
            Ok(val) => val,
            Err(e) => panic!("Error!")
        };
        declare_unifiers( &unifiers, &mut state );

        let unifiers = match unify(val2,var2,&mut state,true){
            Ok(val) => val,
            Err(e) => panic!("Error!")
        };
        declare_unifiers( &unifiers, &mut state );

        let unifiers = match unify(val3,var3,&mut state,true){
            Ok(val) => val,
            Err(e) => panic!("Error!")
        };
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("x",true);
        let check2 = state.lookup_sym("y",true);
        let check3 = state.lookup_sym("z",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:2}) => (),
            _ => panic!("test failed"),
        };
        match *check2 {
            Node::AstroInteger(AstroInteger{value:4}) => (),
            _ => panic!("test failed"),
        };
        match *check3 {
            Node::AstroInteger(AstroInteger{value:8}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_varlist_to_intlist() {
        // let [x,y] = [3,4].

        let mut state = State::new().unwrap();
        let var1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(Node::AstroInteger(AstroInteger::new(3)));
        let var2 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
        let val2 = Rc::new(Node::AstroInteger(AstroInteger::new(4))); 
        let varlist = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&var1),Rc::clone(&var2)])))));
        let vallist = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&val1),Rc::clone(&val2)])))));

        let unifiers = match unify(vallist,varlist,&mut state,true) {
            Ok(val) => val,
            Err(e) => panic!("error!"),
        };

        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:3}) => (),
            _ => panic!("test failed"),
        };
        let check2 = state.lookup_sym("y",true);
        match *check2 {
            Node::AstroInteger(AstroInteger{value:4}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_varlist_to_multilist() {
        // let [x,y,3] = ["string1",1.3334,3].

        let mut state = State::new().unwrap();
        let var1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(Node::AstroString(AstroString::new("string1".to_string())));
        let var2 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
        let val2 = Rc::new(Node::AstroReal(AstroReal::new(1.3334)));
        let int1 = Rc::new(Node::AstroInteger(AstroInteger::new(3))); 
        let int2 = Rc::new(Node::AstroInteger(AstroInteger::new(3)));
        let varlist = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&var1),Rc::clone(&var2),Rc::clone(&int1)])))));
        let vallist = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&val1),Rc::clone(&val2),Rc::clone(&int2)])))));

        let unifiers = match unify(vallist,varlist,&mut state,true){
            Ok(val) => val,
            Err(e) => panic!("error!")
        };

        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroString(AstroString{value:ref val}) if val == "string1" => (),
            _ => panic!("test failed"),
        };
        let check2 = state.lookup_sym("y",true);
        match *check2 {
            Node::AstroReal(AstroReal{value:val}) if val == 1.3334 => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_vartuple_to_inttuple() {
        // let (x,y,z) = (2,3,4).

        let mut state = State::new().unwrap();
        let var1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
        let var2 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
        let val2 = Rc::new(Node::AstroInteger(AstroInteger::new(3))); 
        let var3 = Rc::new(Node::AstroID(AstroID::new("z".to_string())));
        let val3 = Rc::new(Node::AstroInteger(AstroInteger::new(4))); 
        let varlist = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&var1),Rc::clone(&var2),Rc::clone(&var3)])))));
        let vallist = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&val1),Rc::clone(&val2),Rc::clone(&val3)])))));

        let unifiers = match unify(vallist,varlist,&mut state,true){
            Ok(val) => val,
            Err(e) => panic!("error!")
        };
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:2}) => (),
            _ => panic!("test failed"),
        };
        let check2 = state.lookup_sym("y",true);
        match *check2 {
            Node::AstroInteger(AstroInteger{value:3}) => (),
            _ => panic!("test failed"),
        };
        let check2 = state.lookup_sym("z",true);
        match *check2 {
            Node::AstroInteger(AstroInteger{value:4}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_var() {
        // let x = 234.
        // let y = x.

        let mut state = State::new().unwrap();
        let var1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = Rc::new(Node::AstroInteger(AstroInteger::new(234)));
        let var2 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));

        let unifiers = match unify(val1,Rc::clone(&var1),&mut state,true){
            Ok(val) => val,
            Err(e) => panic!("error!")
        };

        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:234}) => (),
            _ => panic!("test failed"),
        };

        let unifiers = match unify(Rc::clone(&var1),var2,&mut state,true){
            Ok(val) => val,
            Err(e) => panic!("error!")
        };

        declare_unifiers( &unifiers, &mut state );

        let check2 = state.lookup_sym("y",true);
        match *check2 {
            Node::AstroInteger(AstroInteger{value:234}) => (),
            Node::AstroInteger(AstroInteger{value:v}) => println!("{}",v),
            _ =>    println!("DEBUG: {}", peek(Rc::clone(&check2))),
        };

    }
    #[test]
    fn test_unify_int_to_namedpattern() {
        // let x:%integer = 17.

        let mut state = State::new().unwrap();
        let var1 = AstroID::new("x".to_string());
        let pmatch_type = Rc::new(Node::AstroString( AstroString::new( "integer".to_string())));
        let pmatch = Rc::new(Node::AstroTypeMatch(AstroTypeMatch::new(pmatch_type)));
        let p = Rc::new(Node::AstroNamedPattern(AstroNamedPattern::new(var1,pmatch)));
        let val1 = Rc::new(Node::AstroInteger(AstroInteger::new(17)));

        let unifiers = match unify(val1,p,&mut state,true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:17}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_index_to_int() {
        // let x = [1,0,3].
        // let x@1 = 2.

        let mut state = State::new().unwrap();
        let var1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(0)));
        let i3 = Rc::new( Node::AstroInteger( AstroInteger::new(3)));
        let i4 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let l1 = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));
        let idx_exp = Rc::new( Node::AstroInteger( AstroInteger::new(1)));

        let unifiers = match unify(Rc::clone(&l1),Rc::clone(&var1),&mut state,true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state );

        let idx1 = Rc::new( Node::AstroIndex( AstroIndex::new( Rc::clone(&var1), Rc::clone(&idx_exp) )));

        let unifiers = match unify(Rc::clone(&i4),Rc::clone(&idx1),&mut state,true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state );
        let check1 = state.lookup_sym("x",true);

        let Node::AstroList(AstroList{contents:ref c}) = *check1
            else {panic!("test failed")};
        
        if let Node::AstroInteger(AstroInteger{value:2}) = *c.borrow()[1] {
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
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('integer', 1), ('integer', 1)])))
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroInteger(AstroInteger{value:2}) = *check1
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
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('integer', 1), ('real', 1.1)])))
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let r1 = Rc::new( Node::AstroReal( AstroReal::new(1.1)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&r1)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroReal(AstroReal{value:v}) = *check1
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
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple',  [('real', 1.35), ('integer', 1)])))
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let r1 = Rc::new( Node::AstroReal( AstroReal::new(1.35)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&r1),Rc::clone(&i1)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroReal(AstroReal{value:v}) = *check1
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
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple',  [('real', 1.35), ('real', 2.15)])))
        let r1 = Rc::new( Node::AstroReal( AstroReal::new(1.35)));
        let r2 = Rc::new( Node::AstroReal( AstroReal::new(2.15)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&r1),Rc::clone(&r2)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };
        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };
        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroReal(AstroReal{value:v}) = *check1
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
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let i3 = Rc::new( Node::AstroInteger( AstroInteger::new(3)));
        let i4 = Rc::new( Node::AstroInteger( AstroInteger::new(4)));
        let l1 = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i1.clone(),i2.clone()])))));
        let l2 = Rc::new( Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i3.clone(),i4.clone()])))));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&l1),Rc::clone(&l2)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroList(AstroList{contents:ref c}) = *check1
            else {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{value:1}) = *c.borrow()[0]
            else {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{value:2}) = *c.borrow()[1]
            else {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{value:3}) = *c.borrow()[2]
            else {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{value:4}) = *c.borrow()[3]
            else {panic!("test failed")};
    }
    #[test]
    fn test_prog_addition_string_to_string() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( Node::AstroString( AstroString::new("Hello ".to_string())));
        let s2 = Rc::new( Node::AstroString( AstroString::new("World!".to_string())));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&s1),Rc::clone(&s2)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello World!",v);
    }
    #[test]
    fn test_prog_addition_string_to_int() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( Node::AstroString( AstroString::new("Hello ".to_string())));
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(123)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&s1),Rc::clone(&i1)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello 123",v);
    }
    #[test]
    fn test_prog_addition_string_to_real() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( Node::AstroString( AstroString::new("Hello ".to_string())));
        let r1 = Rc::new( Node::AstroReal( AstroReal::new(1.23)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&s1),Rc::clone(&r1)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello 1.23",v);
    }
    #[test]
    fn test_prog_addition_int_to_string() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( Node::AstroString( AstroString::new(" Hello".to_string())));
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(123)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&s1)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("123 Hello",v);
    }
    #[test]
    fn test_prog_addition_real_to_string() {
        // rust compiler:
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        
        let s1 = Rc::new( Node::AstroString( AstroString::new(" Hello".to_string())));
        let r1 = Rc::new( Node::AstroReal( AstroReal::new(1.23)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&r1),Rc::clone(&s1)])))));
        let id1 = Rc::new( Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&t1))));
        let exp_val = match walk( Rc::clone( &apply1), &mut state ){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = Rc::new( Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
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
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );
        
        //exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__gt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('id', 'x'), ('null',))))
        let var1 = Rc::new(Node::AstroID(AstroID::new("__gt__".to_string())));
        let var2 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let var3 = Rc::new(Node::AstroID(AstroID::new("POS_INT".to_string())));
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(0)));
        let null1 = Rc::new( Node::AstroNone( AstroNone::new()));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&var2),Rc::clone(&i1)])))));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&var1), Rc::clone(&t1) )));
        let if1 = Rc::new( Node::AstroIf( AstroIf::new( Rc::clone(&apply1), Rc::clone(&var2), Rc::clone(&null1))));
        let quote1 = Rc::new( Node::AstroQuote( AstroQuote::new( Rc::clone( &if1))));
        let exp_val = walk( quote1, &mut state );

        //error handling
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        //unifiers = unify(exp_val,('id', 'POS_INT'))
        let unifiers = unify( exp_val, var3, &mut state, true);

        //error handling
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        //declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state);

        //set_lineinfo('prog.txt',2)
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        //exp_val = walk(('integer', 2))
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let exp_val = walk( i2, &mut state );

        //error handling
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        //unifiers = unify(exp_val,('named-pattern', ('id', 'x'), ('deref', ('id', 'POS_INT'))))
        let var3 = AstroID::new("x".to_string());
        let var4 = Rc::new(Node::AstroID(AstroID::new("POS_INT".to_string())));
        let deref1 = Rc::new( Node::AstroDeref(AstroDeref::new( Rc::clone(&var4) )));
        let namedp1 = Rc::new(Node::AstroNamedPattern(AstroNamedPattern::new( var3, Rc::clone(&deref1))));
        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&namedp1), &mut state, true);

        //error handling
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        //declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:2}) => (),
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

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        let var1 = Rc::new(Node::AstroID(AstroID::new("__gt__".to_string())));
        let var2 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let var3 = Rc::new(Node::AstroID(AstroID::new("POS_INT".to_string())));
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(0)));
        let null1 = Rc::new( Node::AstroNone( AstroNone::new()));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&var2),Rc::clone(&i1)])))));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&var1), Rc::clone(&t1) )));
        let if1 = Rc::new( Node::AstroIf( AstroIf::new( Rc::clone(&apply1), Rc::clone(&var2), Rc::clone(&null1))));
        let quote1 = Rc::new( Node::AstroQuote( AstroQuote::new( Rc::clone( &if1))));
        let exp_val = walk( quote1, &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, var3, &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'sum'))
        // declare_unifiers(unifiers)

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id8 = Rc::new(Node::AstroID(AstroID::new("_ast72".to_string())));
        let id9 = Rc::new(Node::AstroID(AstroID::new("sum".to_string())));
        let func1 = Rc::new(Node::AstroFunction(AstroFunction::new( Rc::clone(&id8) )));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id9), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        // set_lineinfo('prog.txt',8)
        // exp_val = walk(('apply', ('id', 'sum'), ('integer', 5)))
        // unifiers = unify(exp_val,('id', 'y'))
        // declare_unifiers(unifiers)

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id10 = Rc::new(Node::AstroID(AstroID::new("sum".to_string())));
        let id11 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(5)));
        let apply2 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id10), Rc::clone(&i2) )));
        let exp_val = walk( Rc::clone(&apply2), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id11), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);    
        
        let check1 = state.lookup_sym("y",true);
        let Node::AstroInteger(AstroInteger{value:15}) = *check1
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

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        // # structure def for A
        let id1 = Rc::new(Node::AstroID(AstroID::new("a".to_string())));
        let id2 = Rc::new(Node::AstroID(AstroID::new("b".to_string())));
        let d1 = Rc::new(Node::AstroData(AstroData::new(Rc::clone(&id1))));
        let d2 = Rc::new(Node::AstroData(AstroData::new(Rc::clone(&id2))));
        let member_list = vec![ Rc::clone(&d1), Rc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<Rc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<Rc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( Rc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( Rc::new(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = Rc::new(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", Rc::clone(&struct_type)  );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let id3 = Rc::new(Node::AstroID(AstroID::new("A".to_string())));
        let id4 = Rc::new(Node::AstroID(AstroID::new("obj".to_string())));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id3), Rc::clone(&t1) )));
        let exp_val = walk( Rc::clone(&apply1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id4), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);  
        /**********************************************************************************************************************/
        let check1 = state.lookup_sym("obj",true);
        
        let Node::AstroObject(AstroObject{struct_id:ref id,object_memory:ref mem}) = *check1
            else {panic!("test failed.")};
        let AstroID{name:ref tag} = *id;
        assert_eq!( tag, "A" );
 
        let Node::AstroInteger(AstroInteger{value:v1}) = *(mem.borrow()[0])
            else {panic!("test failed")}; 
        assert_eq!( v1,1 );

        let Node::AstroInteger(AstroInteger{value:v2}) = *(mem.borrow()[1])
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

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(0)));
        let id1 = Rc::new(Node::AstroID(AstroID::new("ctr".to_string())));
        let exp_val = walk( Rc::clone(&i1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);  

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(100)));
        let i3 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let id2 = Rc::new(Node::AstroID(AstroID::new("__lt__".to_string())));
        let id3 = Rc::new(Node::AstroID(AstroID::new("__plus__".to_string())));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&id1),Rc::clone(&i2)])))));
        let t2 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&id1),Rc::clone(&i3)])))));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id2), Rc::clone(&t1) )));
        let apply2 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id3), Rc::clone(&t2) )));

        let mut loop_val = match walk(Rc::clone(&apply1), &mut state ) {
            Ok( val ) => val,
            Err(e) => panic!("Error"),
        };
        while let Node::AstroBool(AstroBool{value:true}) = map2boolean( &loop_val) {

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, &mut state );

            let exp_val = walk( Rc::clone(&apply2), &mut state);
            let exp_val = match exp_val {
                Ok( val ) => val,
                Err( e ) => exit(e, &mut state),
            };

            let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

            let unifiers = match unifiers {
                Ok( val ) => val,
                Err( e ) => exit(e, &mut state),
            };

            let check1 = state.lookup_sym("ctr",true);
            let Node::AstroInteger(AstroInteger{value:v}) = *check1 else {panic!("test failed.")};

            declare_unifiers( &unifiers, &mut state); 

            loop_val = match walk(Rc::clone(&apply1), &mut state ) {
                Ok( val ) => val,
                Err(e) => panic!("Error"),
            };
        }

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
        set_lineinfo(  new_lineinfo, &mut state );

        let check1 = state.lookup_sym("ctr",true);
        let Node::AstroInteger(AstroInteger{value:100}) = *check1 
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
    
        fn _ast72<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error> {
            
            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("radius".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state),
                };

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
                set_lineinfo(  new_lineinfo, state );

                let exp_val = walk( Rc::clone(&id1), state );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state),
                };

                let id2 = Rc::new(Node::AstroID(AstroID::new("this".to_string())));
                let id3 = Rc::new(Node::AstroID(AstroID::new("diameter".to_string())));
                let index1 = Rc::new(Node::AstroIndex(AstroIndex::new( Rc::clone(&id2), Rc::clone(&id1))));
                let index2 = Rc::new(Node::AstroIndex(AstroIndex::new( Rc::clone(&id2), Rc::clone(&id3))));

                let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&index1), state, true);

                let unifiers = match unifiers {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state),
                };

                declare_unifiers( &unifiers, state);

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
                set_lineinfo(  new_lineinfo, state );

                let id4 = Rc::new(Node::AstroID(AstroID::new("__times__".to_string())));
                let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
                let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&id1)])))));
                let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id4), Rc::clone(&t1))));

                let exp_val = walk( Rc::clone(&apply1), state );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state),
                };

                let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&index2), state, true);

                let unifiers = match unifiers {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state),
                };

                declare_unifiers( &unifiers, state);

                state.pop_scope();

                return Ok(Rc::new(Node::AstroNone(AstroNone::new())));
            } else {
                return Err(Error::ValueError(format!("none of the function bodies unified with actual parameters")));
            }
        }

        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("__init__") , _ast72 );
        
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let id1 = Rc::new(Node::AstroID(AstroID::new("radius".to_string())));
        let id2 = Rc::new(Node::AstroID(AstroID::new("diameter".to_string())));
        let id3 = Rc::new(Node::AstroID(AstroID::new("__init__".to_string())));
        let data1 = Rc::new(Node::AstroData(AstroData::new(Rc::clone(&id1))));
        let data2 = Rc::new(Node::AstroData(AstroData::new(Rc::clone(&id2))));
        let func1 = Rc::new(Node::AstroFunction(AstroFunction::new( Rc::clone(&id3))));
        let unify1 = Rc::new(Node::AstroUnify(AstroUnify::new( Rc::clone(&id3), Rc::clone(&func1))));

        let member_list = vec![ Rc::clone(&data1), Rc::clone(&data2), Rc::clone(&unify1) ];
        let mut struct_memory: RefCell<Vec<Rc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<Rc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( Rc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( Rc::new(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "unify" {
                let Node::AstroUnify(AstroUnify{term:ref id_node,pattern:ref function_exp}) = *member
                    else {panic!("ERROR: object construction: expection unify node.")};
                let function_val = match walk( Rc::clone(&function_exp), &mut state ) {
                    Ok( val ) => val,
                    Err ( e ) => panic!("error!"),
                };
                struct_memory.borrow_mut().push( Rc::clone( &function_val ));
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "noop" {
                ;// pass
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }

        let struct_type = Rc::new(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "Circle", Rc::clone(&struct_type)  );

        let new_lineinfo = Rc::new(Node::AstroLineInfo(AstroLineInfo{module:"prog.ast".to_string(),line_number:10}));
        set_lineinfo(  new_lineinfo, &mut state );   

        // exp_val = walk(('apply', ('id', 'Circle'), ('integer', 2)))
        // unifiers = unify(exp_val,('id', 'a'))
        // declare_unifiers(unifiers)
        let id1 = Rc::new(Node::AstroID(AstroID::new("Circle".to_string())));
        let id2 = Rc::new(Node::AstroID(AstroID::new("a".to_string())));
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id1), Rc::clone(&i1))));

        let exp_val = walk( Rc::clone(&apply1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( Rc::clone(&exp_val), Rc::clone(&id2), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        /**************************************************************************************************/
        //assert
        let check1 = state.lookup_sym("a",true);
        let Node::AstroObject(AstroObject{struct_id:ref id,object_memory:ref mem}) = *check1
            else {panic!("test failed")};

        let AstroID{name:ref tag} = *id;

        assert_eq!( tag,"Circle" );

        let Node::AstroInteger(AstroInteger{value:2}) = *(*(mem.borrow()))[0]
            else {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{value:4}) = *(*(mem.borrow()))[1]
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
        
        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
        let i2 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
        let i3 = Rc::new(Node::AstroInteger(AstroInteger::new(3)));
        let l1 = Rc::new(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2),Rc::clone(&i3)])))));
        let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
     
        let exp_val = walk( Rc::clone(&l1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let i4 = Rc::new(Node::AstroInteger(AstroInteger::new(4)));
        let id2 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let i5 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
        let idx1 = Rc::new(Node::AstroIndex(AstroIndex::new(Rc::clone(&id2),Rc::clone(&i5))));

        let exp_val = walk( Rc::clone(&i4), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&idx1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("x",true);
        let Node::AstroList(AstroList{ref contents}) = *check1    
            else {panic!("test failed.")};

        let Node::AstroInteger(AstroInteger{value:1}) = *contents.borrow()[0]
            else {panic!("test failed.")};
        let Node::AstroInteger(AstroInteger{value:4}) = *contents.borrow()[1]
            else {panic!("test failed.")};
        let Node::AstroInteger(AstroInteger{value:3}) = *contents.borrow()[2]
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

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
        let i2 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
        let i3 = Rc::new(Node::AstroInteger(AstroInteger::new(3)));
        let i4 = Rc::new(Node::AstroInteger(AstroInteger::new(4)));
        let i5 = Rc::new(Node::AstroInteger(AstroInteger::new(5)));
        let l1 = Rc::new(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&i2),Rc::clone(&i3),Rc::clone(&i4)])))));
        let l2 = Rc::new(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&l1),Rc::clone(&i5)])))));
        let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( Rc::clone(&l2), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );   

        let s1 = Rc::new(Node::AstroString(AstroString::new("hello".to_string())));
        let i6 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
        let i7 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
        let id2 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let idx1 = Rc::new(Node::AstroIndex(AstroIndex::new(Rc::clone(&id2),Rc::clone(&i6))));
        let idx2 = Rc::new(Node::AstroIndex(AstroIndex::new(Rc::clone(&idx1),Rc::clone(&i7))));

        let exp_val = walk( Rc::clone(&s1), &mut state );
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&idx2), &mut state, true);
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("x",true);
        let Node::AstroList(AstroList{ref contents}) = *check1    
            else {panic!("test failed.")};
        let Node::AstroInteger(AstroInteger{value:1}) = *contents.borrow()[0]
            else {panic!("test failed.")};
        let Node::AstroInteger(AstroInteger{value:5}) = *contents.borrow()[2]
            else {panic!("test failed.")};
        let Node::AstroList(AstroList{contents:ref inner_contents}) = *contents.borrow()[1]    
            else {panic!("test failed.")};
        let Node::AstroInteger(AstroInteger{value:2}) = *inner_contents.borrow()[0]
            else {panic!("test failed.")};
        let Node::AstroString(AstroString{value:ref v}) = *inner_contents.borrow()[1] 
            else {panic!("error")};
        let Node::AstroInteger(AstroInteger{value:4}) = *inner_contents.borrow()[2]
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
        fn _ast72<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error> {

            println!("into function with {}!",peek(Rc::clone(&node)));
            
            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
            let id2 = Rc::new(Node::AstroID(AstroID::new("tail".to_string())));
            let ht1 = Rc::new(Node::AstroHeadTail(AstroHeadTail::new(Rc::clone(&id1),Rc::clone(&id2))));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&ht1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state),
                };

                let id3 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
                set_lineinfo(  new_lineinfo, state );

                let exp_val = walk( Rc::clone(&id3), state );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state),
                };

                state.pop_scope();

                return Ok( exp_val )
            } else {
                return Err(Error::ValueError(format!("none of the function bodies unified with actual parameters")));
            }
        }


        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );  

        let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
        let i2 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
        let i3 = Rc::new(Node::AstroInteger(AstroInteger::new(3)));
        let l1 = Rc::new(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2),Rc::clone(&i3)])))));
        let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( Rc::clone(&l1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id2 = Rc::new(Node::AstroID(AstroID::new("_ast72".to_string())));
        let func1 = Rc::new(Node::AstroFunction(AstroFunction::new(Rc::clone(&id2))));
        let id3 = Rc::new(Node::AstroID(AstroID::new("f".to_string())));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id3), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
        set_lineinfo(  new_lineinfo, &mut state ); 

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
        set_lineinfo(  new_lineinfo, &mut state ); 

        let id4 = Rc::new(Node::AstroID(AstroID::new("f".to_string())));
        let id5 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let id6 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
        let apply1 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id4),Rc::clone(&id5))));

        let exp_val = walk( Rc::clone(&apply1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id6), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("y",true);

        let Node::AstroInteger(AstroInteger{value:1}) = *check1
            else {panic!("test failed.")};
        // let Node::AstroList(AstroList{ref contents}) = *check1    
        //     else {panic!("test failed.")};
        // let Node::AstroInteger(AstroInteger{value:1}) = *contents.borrow()[0]
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
        fn _ast72<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error> {

            println!("into function with {}!",peek(Rc::clone(&node)));
            
            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
            let id2 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
            let id3 = Rc::new(Node::AstroID(AstroID::new("tail".to_string())));
            let rht1 = Rc::new(Node::AstroRawHeadTail(AstroRawHeadTail::new(Rc::clone(&id2),Rc::clone(&id3))));
            let ht1 = Rc::new(Node::AstroHeadTail(AstroHeadTail::new(Rc::clone(&id1),Rc::clone(&rht1))));


            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&ht1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state),
                };

                let id4 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
                let id5 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
                let tup1 = Rc::new(Node::AstroTuple(AstroTuple::new(Rc::new(RefCell::new(vec![ Rc::clone(&id4),Rc::clone(&id5) ])))));
                let id6 = Rc::new(Node::AstroID(AstroID::new("__plus__".to_string())));
                let apply1 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id6),Rc::clone(&tup1))));

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
                set_lineinfo(  new_lineinfo, state );

                let exp_val = walk( Rc::clone(&apply1), state );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state),
                };

                state.pop_scope();

                return Ok( exp_val )
            } else {
                return Err(Error::ValueError(format!("none of the function bodies unified with actual parameters")));
            }
        }


        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );  

        let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
        let i2 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
        let i3 = Rc::new(Node::AstroInteger(AstroInteger::new(3)));
        let l1 = Rc::new(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2),Rc::clone(&i3)])))));
        let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( Rc::clone(&l1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id1), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id2 = Rc::new(Node::AstroID(AstroID::new("_ast72".to_string())));
        let func1 = Rc::new(Node::AstroFunction(AstroFunction::new(Rc::clone(&id2))));
        let id3 = Rc::new(Node::AstroID(AstroID::new("f".to_string())));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id3), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
        set_lineinfo(  new_lineinfo, &mut state ); 

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
        set_lineinfo(  new_lineinfo, &mut state ); 

        let id4 = Rc::new(Node::AstroID(AstroID::new("f".to_string())));
        let id5 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let id6 = Rc::new(Node::AstroID(AstroID::new("z".to_string())));
        let apply1 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id4),Rc::clone(&id5))));

        let exp_val = walk( Rc::clone(&apply1), &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id6), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("z",true);


        let Node::AstroInteger(AstroInteger{value:3}) = *check1
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

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        // # structure def for A
        let id1 = Rc::new(Node::AstroID(AstroID::new("a".to_string())));
        let id2 = Rc::new(Node::AstroID(AstroID::new("b".to_string())));
        let d1 = Rc::new(Node::AstroData(AstroData::new(Rc::clone(&id1))));
        let d2 = Rc::new(Node::AstroData(AstroData::new(Rc::clone(&id2))));
        let member_list = vec![ Rc::clone(&d1), Rc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<Rc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<Rc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( Rc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( Rc::new(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = Rc::new(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", Rc::clone(&struct_type)  );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let id3 = Rc::new(Node::AstroID(AstroID::new("A".to_string())));
        let id4 = Rc::new(Node::AstroID(AstroID::new("obj".to_string())));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id3), Rc::clone(&t1) )));
        let exp_val = walk( Rc::clone(&apply1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id4), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);  

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i3 = Rc::new( Node::AstroInteger( AstroInteger::new(4)));
        let id5 = Rc::new(Node::AstroID(AstroID::new("obj".to_string())));
        let id6 = Rc::new(Node::AstroID(AstroID::new("b".to_string())));
        let idx1 = Rc::new(Node::AstroIndex(AstroIndex::new( Rc::clone(&id5), Rc::clone(&id6))));

        let exp_val = match walk( Rc::clone(&i3), &mut state) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = match unify( exp_val, Rc::clone(&idx1), &mut state, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);  

        /**********************************************************************************************************************/
        let check1 = state.lookup_sym("obj",true);
        
        let Node::AstroObject(AstroObject{struct_id:ref id,object_memory:ref mem}) = *check1
            else {panic!("test failed.")};
        let AstroID{name:ref tag} = *id;
        assert_eq!( tag, "A" );
 
        let Node::AstroInteger(AstroInteger{value:v1}) = *(mem.borrow()[0])
            else {panic!("test failed")}; 
        assert_eq!( v1,1 );

        let Node::AstroInteger(AstroInteger{value:v2}) = *(mem.borrow()[1])
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

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        // # structure def for A
        let id1 = Rc::new(Node::AstroID(AstroID::new("a".to_string())));
        let id2 = Rc::new(Node::AstroID(AstroID::new("b".to_string())));
        let d1 = Rc::new(Node::AstroData(AstroData::new(Rc::clone(&id1))));
        let d2 = Rc::new(Node::AstroData(AstroData::new(Rc::clone(&id2))));
        let member_list = vec![ Rc::clone(&d1), Rc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<Rc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<Rc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( Rc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( Rc::new(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( Rc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = Rc::new(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", Rc::clone(&struct_type)  );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
        let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(2)));
        let id3 = Rc::new(Node::AstroID(AstroID::new("A".to_string())));
        let id4 = Rc::new(Node::AstroID(AstroID::new("obj".to_string())));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id3), Rc::clone(&t1) )));
        let exp_val = walk( Rc::clone(&apply1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id4), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);  

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i3 = Rc::new( Node::AstroInteger( AstroInteger::new(4)));
        let id5 = Rc::new(Node::AstroID(AstroID::new("obj".to_string())));
        let id6 = Rc::new(Node::AstroID(AstroID::new("b".to_string())));
        let id7 = Rc::new(Node::AstroID(AstroID::new("z".to_string())));
        let idx1 = Rc::new(Node::AstroIndex(AstroIndex::new( Rc::clone(&id5), Rc::clone(&id6))));

        let exp_val = match walk( Rc::clone(&idx1), &mut state) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id7), &mut state, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);  
     
        let check1 = state.lookup_sym("z",true);

        let Node::AstroInteger(AstroInteger{value:v1}) = *check1
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

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        // # structure def for A
        let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let s1 = Rc::new(Node::AstroString(AstroString::new("abcdefg".to_string())));

        let exp_val = match walk( Rc::clone(&s1), &mut state) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id1), &mut state, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state); 

        let id2 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let id3 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
        let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
        let idx1 = Rc::new(Node::AstroIndex(AstroIndex::new( Rc::clone(&id2), Rc::clone(&i1) )));

        let exp_val = match walk( Rc::clone(&idx1), &mut state) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id3), &mut state, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state); 

        /***********************************************************************************************/
        let check1 = state.lookup_sym("y",true);

        let Node::AstroString(AstroString{value:ref v1}) = *check1
            else {panic!("test failed")}; 
        assert_eq!( v1,"b" );
    }
    #[test]
    fn test_prog_escape_func() {
        // Asteroid
        // function times_two with x do return escape
        // "
        // let Node::AstroInteger(AstroInteger{value:val}) = *state.lookup_sym( \"x\" ) 
        //     else {return Err((\"ValueError\",\"times_two() expected a single integer.\"))};
        
        // return Rc::new(Node::AstroInteger(AstroInteger::new(2*val)));
        // "
        // end
        // let y = times_two( 15 ).


        // Python
        // def _ast72():
            // let Node::AstroInteger(AstroInteger{value:val}) = *state.lookup_sym( "x" )
            //   else {return Error::ValueError("times_two() expected a single integer."))};
            // return Ok(Rc::new(Node::AstroInteger(AstroInteger::new(2*val))));
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

        fn _ast72<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error> {
            let Node::AstroInteger(AstroInteger{value:val}) = *state.lookup_sym( "x", true )
              else {return Err(Error::ValueError(format!("times_two() expected a single integer.")))};
            return Ok(Rc::new(Node::AstroInteger(AstroInteger::new(2*val))));
        }
        fn _ast73<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error> {

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state),
                };

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
                set_lineinfo(  new_lineinfo, state );

                let id2 = Rc::new(Node::AstroID(AstroID::new("_ast72".to_string())));
                let esc1 = Rc::new(Node::AstroEscape(AstroEscape::new( "_ast72".to_string() )));

                let exp_val = match walk( Rc::clone(&esc1), state) {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state),
                };

                state.push_scope();

                return Ok( exp_val )
            } else {
                return Err( Error::ValueError("none of the function bodies unified with actual parameters".to_string()) )
            }
            
        }

        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast72") , _ast72 );
        state.dispatch_table.insert( String::from("_ast73") , _ast73 );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id1 = Rc::new(Node::AstroID(AstroID::new("_ast73".to_string())));
        let id2 = Rc::new(Node::AstroID(AstroID::new("times_two".to_string())));
        let func1 = Rc::new(Node::AstroFunction(AstroFunction::new( Rc::clone(&id1) )));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id2), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:9}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id3 = Rc::new(Node::AstroID(AstroID::new("times_two".to_string())));
        let id4 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));
        let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(15)));
        let apply1 = Rc::new(Node::AstroApply(AstroApply::new( Rc::clone(&id3), Rc::clone(&i1) )));

        let exp_val = match  walk( Rc::clone(&apply1), &mut state) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id4), &mut state, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("y",true);

        let Node::AstroInteger(AstroInteger{value:v}) = *check1
            else {panic!("test failed")}; 
        assert_eq!(30,v);
    }#[test]
    fn test_prog_function() {
        //Asteroid
        // function reduce with (a,b) do -- pattern match the actual argument
        //     return a*b.
        // end

        // let r = reduce (2,4).  -- function call via juxtaposition
        // assert (r == 8).

        //Python
        // def _ast72(arg):
        // set_lineinfo('prog.txt',1)
        // try:
        //    unifiers = unify(arg,('tuple', [('id', 'a'), ('id', 'b')]))
        //    state.symbol_table.push_scope({})
        //    declare_formal_args(unifiers)
        //    set_lineinfo('prog.txt',2)
        //    val = walk(('apply', ('id', '__times__'), ('tuple', [('id', 'a'), ('id', 'b')])))
        //    state.symbol_table.pop_scope()
        //    return val
  
        //    state.symbol_table.pop_scope()
        // except PatternMatchFailed:
        //    raise ValueError('none of the function bodies unified with actual parameters')
  


        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'reduce'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',5)
        // exp_val = walk(('apply', ('id', 'reduce'), ('tuple', [('integer', 2), ('integer', 4)])))
        // unifiers = unify(exp_val,('id', 'r'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',6)
        // exp_val = walk(('apply', ('id', '__eq__'), ('tuple', [('id', 'r'), ('integer', 8)])))
        // assert exp_val[1], 'assert failed'
     

        //Rust
        fn _ast72<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error> {

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("a".to_string())));
            let id2 = Rc::new(Node::AstroID(AstroID::new("b".to_string())));
            let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&id1),Rc::clone(&id2)])))));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&t1), state, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state),
                };

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
                set_lineinfo(  new_lineinfo, state );

                let id3 = Rc::new(Node::AstroID(AstroID::new("a".to_string())));
                let id4 = Rc::new(Node::AstroID(AstroID::new("b".to_string())));
                let t2 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&id3),Rc::clone(&id4)])))));
                let id5 = Rc::new(Node::AstroID(AstroID::new("__times__".to_string())));
                let apply1 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id5),Rc::clone(&t2))));

                let val = walk(Rc::clone(&apply1),state);

                state.pop_scope();

                return val;
            } else {
                return Err(Error::PatternMatchFailed("None of the function bodies unified with actual parameters.".to_string()));
            }
        }

        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id1 = Rc::new(Node::AstroID(AstroID::new("_ast72".to_string())));
        let id2 = Rc::new(Node::AstroID(AstroID::new("reduce".to_string())));
        let func1 = Rc::new(Node::AstroFunction(AstroFunction::new( Rc::clone(&id1) )));

        let exp_val = match walk( Rc::clone(&func1), &mut state) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id2), &mut state, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id3 = Rc::new(Node::AstroID(AstroID::new("reduce".to_string()))); 
        let id4 = Rc::new(Node::AstroID(AstroID::new("r".to_string()))); 
        let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
        let i2 = Rc::new(Node::AstroInteger(AstroInteger::new(4)));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i1),Rc::clone(&i2)])))));
        let apply1 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id3),Rc::clone(&t1))));

        let exp_val = match walk( Rc::clone(&apply1), &mut state) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = match unify( exp_val, Rc::clone(&id4), &mut state, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i3 = Rc::new(Node::AstroInteger(AstroInteger::new(8)));
        let id5 = Rc::new(Node::AstroID(AstroID::new("r".to_string()))); 
        let id6 = Rc::new(Node::AstroID(AstroID::new("__eq__".to_string()))); 
        let t2 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i3),Rc::clone(&id5)])))));
        let apply1 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id6),Rc::clone(&t2))));
        let exp_val = match walk( Rc::clone(&apply1), &mut state) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

 
        let Node::AstroBool(AstroBool{ value:true }) = *exp_val else {panic!("test failed")};
    }
    #[test]
    fn test_prog_factorial() {
        // //Asteroid
        // let pos_int = pattern (x:%integer) if x > 0.
        // let neg_int = pattern (x:%integer) if x < 0.

        // function fact
        //     with 0 do
        //         return 1
        //     with n:*pos_int do
        //         return n * fact (n-1).
        //     with *neg_int do
        //         throw Error("fact undefined for negative values").
        // end

        // let x = fact 10.

        //Python

        // def _ast72(arg):
        // set_lineinfo('prog.txt',14)
        // try:
        //    unifiers = unify(arg,('integer', 0))
        //    state.symbol_table.push_scope({})
        //    declare_formal_args(unifiers)
        //    set_lineinfo('prog.txt',15)
        //    val = walk(('integer', 1))
        //    state.symbol_table.pop_scope()
        //    return val
  
        //    state.symbol_table.pop_scope()
        // except PatternMatchFailed:
        //    set_lineinfo('prog.txt',16)
        //    try:
        //       unifiers = unify(arg,('deref', ('id', 'pos_int')))
        //       state.symbol_table.push_scope({})
        //       declare_formal_args(unifiers)
        //       set_lineinfo('prog.txt',17)
        //       val = walk(('integer', 1))
        //       state.symbol_table.pop_scope()
        //       return val
  
        //       state.symbol_table.pop_scope()
        //    except PatternMatchFailed:
        //       set_lineinfo('prog.txt',18)
        //       try:
        //          unifiers = unify(arg,('deref', ('id', 'neg_int')))
        //          state.symbol_table.push_scope({})
        //          declare_formal_args(unifiers)
        //          set_lineinfo('prog.txt',19)
        //          val = walk(('integer', -1))
        //          state.symbol_table.pop_scope()
        //          return val
  
        //          state.symbol_table.pop_scope()
        //       except PatternMatchFailed:
        //          raise ValueError('none of the function bodies unified with actual parameters')
  


        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__gt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('named-pattern', ('id', 'x'), ('typematch', 'integer')), ('null',))))
        // unifiers = unify(exp_val,('id', 'pos_int'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__lt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('named-pattern', ('id', 'x'), ('typematch', 'integer')), ('null',))))
        // unifiers = unify(exp_val,('id', 'neg_int'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',4)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'fact'))
        // declare_unifiers(unifiers)

        // set_lineinfo('prog.txt',13)
        // exp_val = walk(('apply', ('id', 'fact'), ('integer', 10)))
        // unifiers = unify(exp_val,('id', 'x'))
        // declare_unifiers(unifiers)

     
        //Rust
        fn _ast72<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error>{

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
            set_lineinfo(  new_lineinfo, state );
        
            let id1 = AstroID::new("n".to_string());
            let id2 = Rc::new(Node::AstroID(AstroID::new("pos_int".to_string())));
            let id3 = Rc::new(Node::AstroID(AstroID::new("neg_int".to_string())));  
            let deref1 = Rc::new(Node::AstroDeref(AstroDeref::new(Rc::clone(&id2))));
            let deref2 = Rc::new(Node::AstroDeref(AstroDeref::new(Rc::clone(&id3))));
            let namedp1 = Rc::new(Node::AstroNamedPattern(AstroNamedPattern::new(id1,Rc::clone(&deref1))));

            let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(0)));
        
            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&i1), state, true ) {

                state.push_scope();

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
                set_lineinfo(  new_lineinfo, state );

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let i2 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
                let result = walk( Rc::clone(&i2), state );
                state.pop_scope();
                result

            } else if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&namedp1), state, true ) {

                state.push_scope();

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
                set_lineinfo(  new_lineinfo, state );

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let id4 = Rc::new(Node::AstroID(AstroID::new("__times__".to_string())));
                let id5 = Rc::new(Node::AstroID(AstroID::new("n".to_string())));
                let id6 = Rc::new(Node::AstroID(AstroID::new("n".to_string())));
                let id7 = Rc::new(Node::AstroID(AstroID::new("__minus__".to_string())));
                let id8 = Rc::new(Node::AstroID(AstroID::new("fact".to_string())));
                let i3 = Rc::new( Node::AstroInteger( AstroInteger::new(1)));
                let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&id5),Rc::clone(&i3)])))));
                let apply1 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id7),Rc::clone(&t1))));
                let apply2 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id8),Rc::clone(&apply1))));
                let t2 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&id6),Rc::clone(&apply2)])))));
                let apply3 = Rc::new(Node::AstroApply(AstroApply::new(Rc::clone(&id4),Rc::clone(&t2))));

                let result = walk( Rc::clone(&apply3), state );
                state.pop_scope();
                result

            } else if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&deref2), state, true ) {

                state.push_scope();

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
                set_lineinfo(  new_lineinfo, state );

                let out1 = declare_formal_args( &unifiers, state );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                state.pop_scope();
                Err(Error::ValueError("fact undefined for negative values".to_string()))

            } else {
                return Err(Error::PatternMatchFailed("None of the function bodies unified with actual parameters.".to_string()));
            }
        }

        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        let var1 = Rc::new(Node::AstroID(AstroID::new("__gt__".to_string())));
        let var2 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let var3 = Rc::new(Node::AstroID(AstroID::new("pos_int".to_string())));
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(0)));
        let null1 = Rc::new( Node::AstroNone( AstroNone::new()));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&var2),Rc::clone(&i1)])))));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&var1), Rc::clone(&t1) )));
        let if1 = Rc::new( Node::AstroIf( AstroIf::new( Rc::clone(&apply1), Rc::clone(&var2), Rc::clone(&null1))));
        let quote1 = Rc::new( Node::AstroQuote( AstroQuote::new( Rc::clone( &if1))));
        let exp_val = walk( quote1, &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, var3, &mut state, true);
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state );

        let var1 = Rc::new(Node::AstroID(AstroID::new("__lt__".to_string())));
        let var2 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let var3 = Rc::new(Node::AstroID(AstroID::new("neg_int".to_string())));
        let i1 = Rc::new( Node::AstroInteger( AstroInteger::new(0)));
        let null1 = Rc::new( Node::AstroNone( AstroNone::new()));
        let t1 = Rc::new( Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&var2),Rc::clone(&i1)])))));
        let apply1 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&var1), Rc::clone(&t1) )));
        let if1 = Rc::new( Node::AstroIf( AstroIf::new( Rc::clone(&apply1), Rc::clone(&var2), Rc::clone(&null1))));
        let quote1 = Rc::new( Node::AstroQuote( AstroQuote::new( Rc::clone( &if1))));
        let exp_val = walk( quote1, &mut state );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, var3, &mut state, true);
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
        set_lineinfo(  new_lineinfo, &mut state );

        let id8 = Rc::new(Node::AstroID(AstroID::new("_ast72".to_string())));
        let id9 = Rc::new(Node::AstroID(AstroID::new("fact".to_string())));
        let func1 = Rc::new(Node::AstroFunction(AstroFunction::new( Rc::clone(&id8) )));
        let exp_val = walk( Rc::clone(&func1), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id9), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:13}));
        set_lineinfo(  new_lineinfo, &mut state );

        let i5 = Rc::new( Node::AstroInteger( AstroInteger::new(10)));
        let id10 = Rc::new(Node::AstroID(AstroID::new("fact".to_string())));
        let id11 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
        let apply2 = Rc::new( Node::AstroApply( AstroApply::new( Rc::clone(&id10), Rc::clone(&i5) )));
        let exp_val = walk( Rc::clone(&apply2), &mut state);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        let unifiers = unify( exp_val, Rc::clone(&id11), &mut state, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state),
        };

        declare_unifiers( &unifiers, &mut state);

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:3628800}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_prog_try_catch() {
        // Asteroid
        // try
        //     let y = 1.
        //     let x = 1/0.
        //     let z = 2/2.
        // catch Exception("ValueError", s) do
        //     let x = 3.
        // catch Exception("FileNotFound", s) do
        //     let x = 4.
        // catch Exception("ArithmeticError", s) do
        //     let x = 5.
        // catch Exception("PatternMatchFailed", s) do
        //     let x = 6.
        // catch Exception("RedundantPatternFound", s) do
        //     let x = 7.
        // catch Exception("NonLinearPattern", s) do
        //     let x = 8.
        // end

        // Python
        // set_lineinfo('prog.txt',1)
        // try:
        //    set_lineinfo('prog.txt',2)
        //    exp_val = walk(('integer', 1))
        //    unifiers = unify(exp_val,('id', 'y'))
        //    declare_unifiers(unifiers)
     
        //    set_lineinfo('prog.txt',3)
        //    exp_val = walk(('apply', ('id', '__divide__'), ('tuple', [('integer', 1), ('integer', 0)])))
        //    unifiers = unify(exp_val,('id', 'x'))
        //    declare_unifiers(unifiers)
     
        //    set_lineinfo('prog.txt',4)
        //    exp_val = walk(('apply', ('id', '__divide__'), ('tuple', [('integer', 2), ('integer', 2)])))
        //    unifiers = unify(exp_val,('id', 'z'))
        //    declare_unifiers(unifiers)
     
        // except ThrowValue as inst:
        //    except_val = inst.value
        //    inst_val = inst
        // except PatternMatchFailed as inst:
        //    except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'PatternMatchFailed'),('string', inst.value)])))
        //    inst_val = inst
        // except RedundantPatternFound as inst:
        //    except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'RedundantPatternFound'),('string', str(inst))])))
        //    inst_val = inst
        // except NonLinearPatternError as inst:
        //    except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'NonLinearPatternError'),('string', str(inst))])))
        //    inst_val = inst
        // except ArithmeticError as inst:
        //    except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'ArithmeticError'),('string', str(inst))])))
        //    inst_val = inst
        // except FileNotFoundError as inst:
        //    except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'FileNotFound'),('string', str(inst))])))
        //    inst_val = inst
        // except Exception as inst:
        //    except_val = ('object',('struct-id', ('id', 'Exception')),('object-memory',('list',[('string', 'SystemError'),('string', str(inst))])))
        //    inst_val = inst
        // else:
        //    except_val = None
        // if except_val:
        //    exception_handled = False
        //    try:
        //       unifiers = unify(except_val, ('apply', ('id', 'Exception'), ('tuple', [('string', 'ValueError'), ('id', 's')])))
        //    except PatternMatchFailed:
        //       try:
        //          unifiers = unify(except_val, ('apply', ('id', 'Exception'), ('tuple', [('string', 'FileNotFound'), ('id', 's')])))
        //       except PatternMatchFailed:
        //          try:
        //             unifiers = unify(except_val, ('apply', ('id', 'Exception'), ('tuple', [('string', 'ArithmeticError'), ('id', 's')])))
        //          except PatternMatchFailed:
        //             try:
        //                unifiers = unify(except_val, ('apply', ('id', 'Exception'), ('tuple', [('string', 'PatternMatchFailed'), ('id', 's')])))
        //             except PatternMatchFailed:
        //                try:
        //                   unifiers = unify(except_val, ('apply', ('id', 'Exception'), ('tuple', [('string', 'RedundantPatternFound'), ('id', 's')])))
        //                except PatternMatchFailed:
        //                   try:
        //                      unifiers = unify(except_val, ('apply', ('id', 'Exception'), ('tuple', [('string', 'NonLinearPattern'), ('id', 's')])))
        //                   except PatternMatchFailed:
        //                      pass
        //                   else:
        //                      declare_unifiers(unifiers)
        //                      set_lineinfo('prog.txt',16)
        //                      exp_val = walk(('integer', 8))
        //                      unifiers = unify(exp_val,('id', 'x'))
        //                      declare_unifiers(unifiers)
     
        //                      exception_handled = True
        //                else:
        //                   declare_unifiers(unifiers)
        //                   set_lineinfo('prog.txt',14)
        //                   exp_val = walk(('integer', 7))
        //                   unifiers = unify(exp_val,('id', 'x'))
        //                   declare_unifiers(unifiers)
     
        //                   exception_handled = True
        //             else:
        //                declare_unifiers(unifiers)
        //                set_lineinfo('prog.txt',12)
        //                exp_val = walk(('integer', 6))
        //                unifiers = unify(exp_val,('id', 'x'))
        //                declare_unifiers(unifiers)
     
        //                exception_handled = True
        //          else:
        //             declare_unifiers(unifiers)
        //             set_lineinfo('prog.txt',10)
        //             exp_val = walk(('integer', 5))
        //             unifiers = unify(exp_val,('id', 'x'))
        //             declare_unifiers(unifiers)
     
        //             exception_handled = True
        //       else:
        //          declare_unifiers(unifiers)
        //          set_lineinfo('prog.txt',8)
        //          exp_val = walk(('integer', 4))
        //          unifiers = unify(exp_val,('id', 'x'))
        //          declare_unifiers(unifiers)
     
        //          exception_handled = True
        //    else:
        //       declare_unifiers(unifiers)
        //       set_lineinfo('prog.txt',6)
        //       exp_val = walk(('integer', 3))
        //       unifiers = unify(exp_val,('id', 'x'))
        //       declare_unifiers(unifiers)
     
        //       exception_handled = True
        //    if not exception_handled:
        //       raise inst_val
     
        // Rust
        fn _try1_catch1<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error>{

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("s".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
                set_lineinfo(  new_lineinfo, state );

                let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(3)));
                let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

                let exp_val = match walk( Rc::clone(&i1), state) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let unifiers = match unify( exp_val, Rc::clone(&id1), state, true) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
        
                declare_unifiers( &unifiers, state);

                return Ok(Rc::new(Node::AstroNone(AstroNone::new())));

            } else {
                return Err( Error::PatternMatchFailed("pattern match failed.".to_string()) );
            }
        }
        fn _try1_catch2<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error>{

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:7}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("s".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
                set_lineinfo(  new_lineinfo, state );

                let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(4)));
                let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

                let exp_val = match walk( Rc::clone(&i1), state) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let unifiers = match unify( exp_val, Rc::clone(&id1), state, true) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
        
                declare_unifiers( &unifiers, state);

                return Ok(Rc::new(Node::AstroNone(AstroNone::new())));

            } else {
                return Err( Error::PatternMatchFailed("pattern match failed.".to_string()) );
            }
        };
        fn _try1_catch3<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error>{

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:9}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("s".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:10}));
                set_lineinfo(  new_lineinfo, state );

                let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(5)));
                let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

                let exp_val = match walk( Rc::clone(&i1), state) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let unifiers = match unify( exp_val, Rc::clone(&id1), state, true) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
        
                declare_unifiers( &unifiers, state);

                return Ok(Rc::new(Node::AstroNone(AstroNone::new())));

            } else {
                return Err( Error::PatternMatchFailed("pattern match failed.".to_string()) );
            }
        };
        fn _try1_catch4<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error>{

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:11}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("s".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:12}));
                set_lineinfo(  new_lineinfo, state );

                let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(6)));
                let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

                let exp_val = match walk( Rc::clone(&i1), state) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let unifiers = match unify( exp_val, Rc::clone(&id1), state, true) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
        
                declare_unifiers( &unifiers, state);

                return Ok(Rc::new(Node::AstroNone(AstroNone::new())));

            } else {
                return Err( Error::PatternMatchFailed("pattern match failed.".to_string()) );
            }
        };
        fn _try1_catch5<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error>{

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:13}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("s".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:14}));
                set_lineinfo(  new_lineinfo, state );

                let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(7)));
                let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

                let exp_val = match walk( Rc::clone(&i1), state) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let unifiers = match unify( exp_val, Rc::clone(&id1), state, true) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
        
                declare_unifiers( &unifiers, state);

                return Ok(Rc::new(Node::AstroNone(AstroNone::new())));

            } else {
                return Err( Error::PatternMatchFailed("pattern match failed.".to_string()) );
            }
        };
        fn _try1_catch6<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error>{

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:15}));
            set_lineinfo(  new_lineinfo, state );

            let id1 = Rc::new(Node::AstroID(AstroID::new("s".to_string())));

            if let Ok( unifiers ) = unify( Rc::clone(&node), Rc::clone(&id1), state, true ) {

                let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:16}));
                set_lineinfo(  new_lineinfo, state );

                let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(8)));
                let id1 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));

                let exp_val = match walk( Rc::clone(&i1), state) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let unifiers = match unify( exp_val, Rc::clone(&id1), state, true) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
        
                declare_unifiers( &unifiers, state);

                return Ok(Rc::new(Node::AstroNone(AstroNone::new())));

            } else {
                return Err( Error::PatternMatchFailed("pattern match failed.".to_string()) );
            }
        };
        fn _try1<'a>( node: Rc<Node>, state: &'a mut State ) -> Result< Rc<Node>, Error>{

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
            set_lineinfo(  new_lineinfo, state );

            let i1 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
            let id1 = Rc::new(Node::AstroID(AstroID::new("y".to_string())));

            // check for exceptions/errors
            let exp_val = match walk( Rc::clone(&i1), state) {
                Ok( val ) => val,
                Err( Error::ValueError(e) ) => return _try1_catch1( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::FileNotFound(e) ) => return _try1_catch2( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::ArithmeticError(e) ) => return _try1_catch3( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::PatternMatchFailed(e) ) => return _try1_catch4( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::RedundantPatternFound(e) ) => return _try1_catch5( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::NonLinearPattern(e) ) => return _try1_catch6( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( e ) => return Err(e),
            };


            // check for exceptions/errors
            let unifiers = match unify( exp_val, Rc::clone(&id1), state, true) {
                Ok( val ) => val,
                Err( Error::ValueError(e) ) => return _try1_catch1( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::FileNotFound(e) ) => return _try1_catch2( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::ArithmeticError(e) ) => return _try1_catch3( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::PatternMatchFailed(e) ) => return _try1_catch4( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::RedundantPatternFound(e) ) => return _try1_catch5( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::NonLinearPattern(e) ) => return _try1_catch6( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( e ) => return Err(e),
            };

            declare_unifiers( &unifiers, state);

            let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, state );

            let i2 = Rc::new(Node::AstroInteger(AstroInteger::new(1)));
            let i3 = Rc::new(Node::AstroInteger(AstroInteger::new(0)));
            let t1 = Rc::new(Node::AstroTuple(AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i2),Rc::clone(&i3)])))));
            let id2 = Rc::new(Node::AstroID(AstroID::new("x".to_string())));
            let id3 = Rc::new(Node::AstroID(AstroID::new("__divide__".to_string())));
            let apply1 = Rc::new(Node::AstroApply(AstroApply::new( Rc::clone(&id3), Rc::clone(&t1) )));

            // check for exceptions/errors
            let exp_val = match walk( Rc::clone(&apply1), state) {
                Ok( val ) => val,
                Err( Error::ValueError(e) ) => return _try1_catch1( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::FileNotFound(e) ) => return _try1_catch2( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::ArithmeticError(e) ) => return _try1_catch3( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::PatternMatchFailed(e) ) => return _try1_catch4( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::RedundantPatternFound(e) ) => return _try1_catch5( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::NonLinearPattern(e) ) => return _try1_catch6( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( e ) => return Err(e),
            };

            // check for exceptions/errors
            let unifiers = match unify( exp_val, Rc::clone(&id2), state, true) {
                Ok( val ) =>  val,
                Err( Error::ValueError(e) ) => return _try1_catch1( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::FileNotFound(e) ) => return _try1_catch2( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::ArithmeticError(e) ) => return _try1_catch3( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::PatternMatchFailed(e) ) => return _try1_catch4( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::RedundantPatternFound(e) ) => return _try1_catch5( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::NonLinearPattern(e) ) => return _try1_catch6( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( e ) => return Err(e),
            };
    
            declare_unifiers( &unifiers, state);

            let i4 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
            let i5 = Rc::new(Node::AstroInteger(AstroInteger::new(2)));
            let t2 = Rc::new(Node::AstroTuple(AstroTuple::new(Rc::new(RefCell::new(vec![Rc::clone(&i4),Rc::clone(&i5)])))));
            let id4 = Rc::new(Node::AstroID(AstroID::new("z".to_string())));
            let id5 = Rc::new(Node::AstroID(AstroID::new("__divide__".to_string())));
            let apply2 = Rc::new(Node::AstroApply(AstroApply::new( Rc::clone(&id3), Rc::clone(&t1) )));

            // check for exceptions/errors
            let exp_val = match walk( Rc::clone(&apply2), state) {
                Ok( val ) => val,
                Err( Error::ValueError(e) ) => return _try1_catch1( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::FileNotFound(e) ) => return _try1_catch2( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::ArithmeticError(e) ) => return _try1_catch3( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::PatternMatchFailed(e) ) => return _try1_catch4( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::RedundantPatternFound(e) ) => return _try1_catch5( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::NonLinearPattern(e) ) => return _try1_catch6( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( e ) => return Err(e),
            };

            // check for exceptions/errors
            let unifiers = match unify( exp_val, Rc::clone(&id2), state, true) {
                Ok( val ) =>  val,
                Err( Error::ValueError(e) ) => return _try1_catch1( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::FileNotFound(e) ) => return _try1_catch2( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::ArithmeticError(e) ) => return _try1_catch3( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::PatternMatchFailed(e) ) => return _try1_catch4( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::RedundantPatternFound(e) ) => return _try1_catch5( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( Error::NonLinearPattern(e) ) => return _try1_catch6( Rc::new(Node::AstroString(AstroString::new(e))), state),
                Err( e ) => return Err(e),
            };
    
            declare_unifiers( &unifiers, state);

            return Ok(Rc::new(Node::AstroNone(AstroNone::new())));
        }

        let mut state = State::new().unwrap();
        //state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = Rc::new(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state );

        match _try1( Rc::new(Node::AstroNone(AstroNone::new())), &mut state ) {
            Ok(_) => (),
            Err(e) => exit( e, &mut state),
        };

        let check1 = state.lookup_sym("x",true);

        let Node::AstroInteger(AstroInteger{value:v}) = *check1
            else {panic!("test failed")}; 
        assert_eq!(5,v);

    }
}