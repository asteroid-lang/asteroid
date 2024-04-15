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
 
use regex::Regex;    //Regular expressions
use shared_arena::*; //Arena for AST nodes

use std::process;              // exit()
use std::collections::HashMap; // states's symbol table
use std::rc::Rc;               // state
use std::cell::RefCell;        // state

static OPERATOR_SYMBOLS: [&str; 12] = [ "__plus__", "__minus__", "__times__", "__divide__", "__or__", "__and__", "__eq__", 
                                        "__ne__", "__lt__", "__le__", "__ge__", "__gt__" ];
static BINARY_OPERATORS: [&str; 12] = [ "__plus__", "__minus__", "__times__", "__divide__", "__or__", "__and__", "__eq__", 
                                        "__ne__", "__lt__", "__le__", "__ge__", "__gt__" ];

/******************************************************************************/
// Constructs a new standard exception of type "kind" with the message "msg"
pub fn new_exception<'a>( kind: String, msg: String, state: &'a mut State, memory: &'a mut Arena<Node> ) -> ArenaRc<Node> {
    memory.alloc_rc( Node::AstroApply( AstroApply::new( memory.alloc_rc(Node::AstroID(AstroID::new("Exception".to_string()))), ArenaRc::clone(&memory.alloc_rc(Node::AstroTuple(AstroTuple::new(Rc::new(RefCell::new(vec![  memory.alloc_rc(Node::AstroString(AstroString::new(kind.to_owned()))), memory.alloc_rc(Node::AstroString(AstroString::new(msg.to_owned()))) ] )))))))))
}
/******************************************************************************/
pub fn unify_string_to_string<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    // Note: a pattern needs to match the whole term.
    let Node::AstroString(AstroString{value:ref t_value}) = *term 
        else {return( Err(new_exception("VMError".to_string(), "unify: expected string.".to_string(), state, memory  )))};
    let Node::AstroString(AstroString{value:ref p_value}) = *pattern 
        else {return( Err(new_exception("VMError".to_string(), "unify: expected string.".to_string(), state, memory ) ))};

    let mut re_str = String::from(r"^");
    re_str.push_str(&p_value);
    re_str.push_str("$");
    let re = Regex::new(&re_str).unwrap();

    if re.is_match(&t_value) {
        Ok( vec![] ) // Return an empty unifier
    } else {
        Err( new_exception("PatternMatchFailed".to_string(), format!("regular expression {} did not match {}",term2string(&pattern).unwrap(),term2string(&term).unwrap()), state, memory ))
    }
}
/******************************************************************************/
pub fn unify_string_to_other<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    let new_str = term2string(&term).unwrap();
    let new_term = AstroString{value:new_str};

    unify( memory.alloc_rc(Node::AstroString(new_term)),pattern,state, memory,unifying )
}
/******************************************************************************/
pub fn unify_primitive_to_primitive<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{

    if *term == *pattern {
        Ok( vec![] ) // Return an empty unifier
    } else {
        Err( new_exception("PatternMatchFailed".to_string(), format!("{} is not the same as {}",term2string(&pattern).unwrap(),term2string(&term).unwrap()), state, memory ))
    }
}
/******************************************************************************/
pub fn subsume_namedpattern<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{

    // Unpack a term-side name-pattern if evaluating redundant clauses
    let Node::AstroNamedPattern( AstroNamedPattern{name:_,pattern:ref t_pattern}) = *term
        else {return( Err(new_exception("VMError".to_string(), "unify: expected named pattern.".to_string(), state, memory )))};

    unify( ArenaRc::clone( t_pattern), pattern, state, memory, unifying )
}
/******************************************************************************/
pub fn subsume_deref<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{

    let Node::AstroDeref(AstroDeref{expression:ref t_expression}) = *term
        else {return( Err(new_exception("VMError".to_string(), "unify: expected derefence expression.".to_string(), state, memory )))};
    let Node::AstroID(AstroID{name:ref t_name}) = **t_expression
        else {return( Err(new_exception("VMError".to_string(), "unify: expected derefence expression.".to_string(), state, memory )))};

    let term = state.lookup_sym( &t_name, true );

    unify( term, pattern, state, memory, unifying )
}
/******************************************************************************/
/* Asteroid value level matching */
pub fn unify_object_to_object<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{

    // this can happen when we dereference a variable pointing
    // to an object as a pattern, e.g.
    //    let o = A(1,2). -- A is a structure with 2 data members
    //    let *o = o.
    let Node::AstroObject(AstroObject{struct_id:ref t_id,object_memory:ref t_data}) = *term
        else {return( Err(new_exception("VMError".to_string(), "divison by zero".to_string(), state, memory )))};
    let Node::AstroObject(AstroObject{struct_id:ref p_id,object_memory:ref p_data}) = *pattern
        else {return( Err(new_exception("VMError".to_string(), "divison by zero".to_string(), state, memory )))};

    let AstroID{name:t_name} = t_id;
    let AstroID{name:p_name} = p_id;

    if t_name != p_name {
        Err( new_exception("PatternMatchFailed".to_string(), format!("pattern type {} and term type {} do not agree.",t_name,p_name), state, memory ))
    } else {
        let mut unifiers = vec![];
        for i in 0..(t_data.borrow().len()-1) {
            let mut unifier = match unify( ArenaRc::clone(&t_data.borrow()[i]) , ArenaRc::clone(&p_data.borrow()[i]),state, memory,unifying) {
                Ok( val ) => val,
                Err( e ) => return Err( e )
            };
            unifiers.append( &mut unifier );
        }
        Ok(unifiers)
    }

}
/******************************************************************************/
pub fn unify_if<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    let Node::AstroIf(AstroIf{ cond_exp:ref p_cond, then_exp:ref p_then, else_exp:ref p_else}) = *pattern
        else {return( Err(new_exception("VMError".to_string(), "unify: expected if expression.".to_string(), state, memory)))};

    if let Node::AstroNone(AstroNone{}) = **p_else {

        let unifiers = match unify(term,ArenaRc::clone(p_then),state,memory,unifying) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        if state.constraint_lvl > 0 {
            state.push_scope();
        }

        // evaluate the conditional expression in the
        // context of the unifiers.
        state.push_scope();
        declare_unifiers( &unifiers, state, memory );
        let x = match walk(ArenaRc::clone(p_cond),state, memory) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };

        let bool_val = match walk(ArenaRc::clone(p_cond),state, memory) {
            Ok( val ) => map2boolean(&val),
            Err( e ) => return Err(e),
        };
        state.pop_scope();

        if state.constraint_lvl > 0 {
            state.pop_scope();
        }

        let Node::AstroBool(AstroBool{value:b_value}) = bool_val
            else {return( Err(new_exception("VMError".to_string(), "unify: expected boolean.".to_string(), state, memory )))};

        if b_value {
            Ok( unifiers )
        } else {
            Err( new_exception("PatternMatchFailed".to_string(), "Conditional pattern match failed.".to_string(), state, memory  ))
        }   
    } else {
        Err( new_exception( "ValueError".to_string(), "Conditional patterns do not support else clauses.".to_string(), state, memory  ))
    }
}
/******************************************************************************/
pub fn subsume_if<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    // If we are evaluating subsumption between two different conditional patterns
    // we want to 'punt' and print a warning message.
    if !state.cond_warning {
        eprintln!("Redundant pattern detection is not supported for conditional pattern expressions.");
        state.cond_warning = true;
        Ok(vec![])
    } else {
        // Otherwise if the term is not another cmatch the clauses are correctly ordered.
        Err( new_exception("PatternMatchFailed".to_string(), format!("Subsumption relatioship broken, pattern will not be rendered redundant."), state, memory ))
    } 
}
/******************************************************************************/
pub fn subsume_conditional<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{

    // If we get here, a conditional pattern clause is placed after a non-conditonal
    // pattern clause. Therefore, we need to check if the subsume because if they do
    // the conditonal clause is redundant.

    let Node::AstroIf(AstroIf{cond_exp:ref p_cond, then_exp:ref p_then, else_exp:ref p_else}) = *term
        else {return( Err(new_exception("VMError".to_string(), "unify: expected if expression.".to_string(), state, memory )))};

    if let Node::AstroNone(AstroNone{}) = **p_else {
        unify( ArenaRc::clone( p_then ),pattern,state, memory,unifying  )
    } else {
        Err( new_exception("ValueError".to_string(), "Conditional patterns do not support else clauses.".to_string(), state, memory  ))
    }

}
/******************************************************************************/
pub fn unify_typematch<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{

    let Node::AstroTypeMatch(AstroTypeMatch{expression:ref p_exp}) = *pattern
        else {return( Err(new_exception("VMError".to_string(), "unify: expected typematch.".to_string(), state, memory )))};
    let Node::AstroString(AstroString{value:ref p_type}) = **p_exp
        else {return( Err(new_exception("VMError".to_string(), "unify: expected string.".to_string(), state, memory )) )};

    let mut term_type = peek( ArenaRc::clone(&term) );
    let pattern_type = peek( ArenaRc::clone(&pattern) );

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
        } else if "id" == term_type || "index" == term_type {
            println!("YUP!!");
            let _temp = match walk(ArenaRc::clone(&term),state,memory) {
                Ok(val) => val,
                Err(e) => return Err(e),
            };
            term_type = peek( ArenaRc::clone(&_temp) );
            if p_type == term_type {
                return Ok( vec![] )
            } else {
                Err( new_exception("PatternMatchFailed".to_string(), format!("Expected typematch: {}, got a term of type {}",p_type,term_type), state, memory ))
            }
        } else {
            Err( new_exception("PatternMatchFailed".to_string(), format!("Expected typematch: {}, got a term of type {}",p_type,term_type), state, memory ))
        }
    } else if p_type == "function" {
        //  matching function and member function values
        if ["function-val","member-function-val"].contains( &term_type ){
            Ok( vec![] )
        } else {
            Err( new_exception("PatternMatchFailed".to_string(), format!("Expected typematch: {}, got a term of type {}",p_type,term_type), state, memory ))
        }
    } else if p_type == "pattern" {
        if term_type == "quote" {
            Ok( vec![] )
        } else {
            Err( new_exception("PatternMatchFailed".to_string(), format!("Expected typematch: {}, got a term of type {}",p_type,term_type), state, memory ))
        }
    } else if p_type == "object" {
        let Node::AstroObject(AstroObject{struct_id:ref t_id,object_memory:ref t_mem}) = *term
            else {return( Err(new_exception("VMError".to_string(), "unify: expected object.".to_string(),state,memory)))};
        let AstroID{name:t_type} = t_id;

        if p_type == t_type {
            Ok( vec![] )
        } else {
            Err( new_exception("PatternMatchFailed".to_string(), format!("Expected typematch: {}, got a term of type {}",p_type,term_type), state, memory ))
        }
    } else {
        // Check if the typematch is in the symbol table
        let in_symtab = state.find_sym(p_type);
        match in_symtab {
            None => return  Err( new_exception("PatternMatchFailed".to_string(), format!("{} is not a valid type for typematch",p_type), state, memory )),
            Some(_) => (),
        };

        // If it is in the symbol table but not a struct, it cannot be typematched
        // because it is not a type
        if peek( state.lookup_sym( p_type,true ) ) != "struct" {
            Err( new_exception("PatternMatchFailed".to_string(), format!("{} is not a type",p_type),state,memory))
        } else { 
            //Otherwhise, the typematch has failed
            Err( new_exception("PatternMatchFailed".to_string(), format!("Expected typematch: {}, got a term of type {}",p_type,term_type), state, memory ))
        }
    }
}
/******************************************************************************/
pub fn unify_namedpattern<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    let Node::AstroNamedPattern(AstroNamedPattern{name:ref p_name,pattern:ref p_pattern}) = *pattern
        else {return( Err(new_exception("VMError".to_string(), "unify: expected named pattern.".to_string(), state, memory )))};

    // name_exp can be an id or an index expression.
    let mut unifiers = unify( ArenaRc::clone(&term), ArenaRc::clone(p_pattern),state, memory,unifying );

    let mut unifiers = match unifiers {
        Ok( val ) => val,
        Err( val ) => return Err(val),
    };

    unifiers.push( (memory.alloc_rc(Node::AstroID(p_name.clone())), ArenaRc::clone(&term)) );
    Ok( unifiers )
}
/******************************************************************************/
pub fn unify_none<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    let term_type = peek( ArenaRc::clone(&term) );
    if term_type == "none" {
        Err( new_exception("PatternMatchFailed".to_string(), format!("expected 'none' got '{}'",term_type), state, memory ))
    } else {
        Ok( vec![] )
    }
}
/******************************************************************************/
pub fn unify_quote<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    // quotes on the pattern side can always be ignored
    let term_type = peek( ArenaRc::clone(&term) );
    let Node::AstroQuote(AstroQuote{expression:ref p_exp}) = *pattern
        else {return( Err(new_exception("VMError".to_string(), "unify: expected quote.".to_string(), state, memory )))};

    if term_type == "quote" {
        let Node::AstroQuote(AstroQuote{expression:ref t_exp}) = *term
            else {return( Err(new_exception("VMError".to_string(), "unify: expected quote.".to_string(), state, memory )))};

        unify(ArenaRc::clone(&t_exp),ArenaRc::clone(&p_exp),state, memory,unifying)
    } else {
        unify(ArenaRc::clone(&term),ArenaRc::clone(&p_exp),state, memory,unifying)
    }
}
/******************************************************************************/
pub fn unify_term_quote<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    // ignore quote on the term if we are not trying to unify term with
    // a variable or other kind of lval
    let Node::AstroQuote(AstroQuote{expression:ref t_exp}) = *term
        else {return( Err(new_exception("VMError".to_string(), "unify: expected quote.".to_string(), state, memory )))};

    unify( ArenaRc::clone(&t_exp), pattern, state, memory, unifying )
}
/******************************************************************************/
pub fn unify_object_to_apply<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    let Node::AstroObject(AstroObject{struct_id:ref t_name,object_memory:ref t_mem}) = *term
        else {return( Err(new_exception("VMError".to_string(), "unify: expected object.".to_string(), state, memory )))};
    let Node::AstroApply(AstroApply{function:ref p_func,argument:ref p_arg}) = *pattern
        else {return( Err(new_exception("VMError".to_string(), "unify: expected apply.".to_string(), state, memory )))};
    let Node::AstroID(AstroID{name:ref p_id}) = **p_func
        else {return( Err(new_exception("VMError".to_string(), "unify: expected string.".to_string(), state, memory )) )};
    let AstroID{name:t_id} = t_name;

    
    if p_id != t_id {
        Err( new_exception("PatternMatchFailed".to_string(), format!("expected type '{}' got type '{}'",p_id,t_id),state,memory))
    } else if let Node::AstroTuple(AstroTuple{contents:ref content}) = **p_arg {
        //unify( ArenaRc::clone(t_mem), ArenaRc::clone(p_arg), state, unifying )
        let mut unifiers = vec![];
        for i in 0..content.borrow().len() {
            let mut unifier = match unify( ArenaRc::clone(&t_mem.borrow()[i]) , ArenaRc::clone(&content.borrow()[i]),state, memory,unifying) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
            unifiers.append( &mut unifier);
        }
        Ok(unifiers)
    } else {
        unify( ArenaRc::clone(&t_mem.borrow()[0]), memory.alloc_rc(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(p_arg)]))))) , state, memory, unifying )
    }
}
/******************************************************************************/
pub fn unify_index<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{

    let p = match walk(ArenaRc::clone(&pattern),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    //let Node::AstroIndex(AstroIndex{ref structure,ref index_exp}) = *p else{panic!("problem!")};
    //let Node::AstroIndex(AstroIndex{ref structure,ref index_exp}) = *index_exp1 else{panic!("problem!")};

    Ok( vec![(ArenaRc::clone(&pattern),ArenaRc::clone(&term))] )
}
/******************************************************************************/
pub fn unify_id<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    let Node::AstroID(AstroID{name:ref p_name}) = *pattern
        else {return( Err(new_exception("VMError".to_string(), "unify: expected id.".to_string(), state, memory )))};    

    if p_name == "_" {
        Ok( vec![] )
    } else {
        Ok( vec![(ArenaRc::clone(&pattern),ArenaRc::clone(&term))] )
    }
}
/******************************************************************************/
pub fn unify_headtail<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    let Node::AstroList(AstroList{contents:ref t_contents}) = *term
        else {return( Err(new_exception("PatternMatchFailed".to_string(), format!("head-tail operator expected type 'list' got type '{}'",peek(ArenaRc::clone(&term))), state, memory )))};

    let (head,tail) = match *pattern {
        Node::AstroHeadTail(AstroHeadTail{ref head,ref tail}) => (head,tail),
        Node::AstroRawHeadTail(AstroRawHeadTail{ref head,ref tail}) => (head,tail),
        _ => return Err( new_exception("PatternMatchFailed".to_string(), format!("Unify: expected head-tail."), state, memory )),
    };

    if t_contents.borrow().len() == 0 {
        return Err( new_exception("PatternMatchFailed".to_string(), format!("head-tail operator expected a non-empty list"),state,memory));
    }

    let list_head = ArenaRc::clone(&t_contents.borrow()[0]);
    let list_tail = memory.alloc_rc(Node::AstroList(AstroList::new( Rc::new(RefCell::new(t_contents.borrow_mut().split_off(1))))));

    let mut unifiers = vec![];
    let mut unifier = match unify( ArenaRc::clone(&list_head), ArenaRc::clone(&head), state, memory, unifying ) {
        Ok( x ) => x,
        Err( x ) => return Err(x),
    };
    unifiers.append( &mut unifier );
    let mut unifier = match unify( ArenaRc::clone(&list_tail), ArenaRc::clone(&tail), state, memory, unifying ) {
        Ok( x ) => x,
        Err( x ) => return Err(x),
    };
    unifiers.append( &mut unifier );

    Ok(unifiers)
}
/******************************************************************************/
pub fn unify_list<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    
    let mut term_type = peek( ArenaRc::clone(&term) );
    let pattern_type = peek( ArenaRc::clone(&pattern) );


    if term_type != "list" || pattern_type != "list" {
        Err( new_exception("PatternMatchFailed".to_string(), format!("term and pattern do not agree on list/tuple constructor"),state,memory))
    } else {
        
        let Node::AstroList(AstroList{contents:ref t_contents}) = *term
            else {return( Err(new_exception("VMError".to_string(), "unify: expected list.".to_string(), state, memory )))};
        let Node::AstroList(AstroList{contents:ref p_contents}) = *pattern
            else {return( Err(new_exception("VMError".to_string(), "unify: expected list.".to_string(), state, memory )))};

        if t_contents.borrow().len() != p_contents.borrow().len() {
            Err( new_exception("PatternMatchFailed".to_string(), format!("term and pattern lists/tuples are not the same length"), state, memory ))
        } else {
            let mut unifiers = vec![];
            for i in 0..(t_contents.borrow().len()) {
                let x = unify( ArenaRc::clone( &t_contents.borrow()[i]), ArenaRc::clone( &p_contents.borrow()[i]), state, memory, unifying );
                match x {
                    Ok(mut success) => unifiers.append( &mut success ),
                    Err(_) => return x,
                }
            }
            check_repeated_symbols( &unifiers );
            Ok( unifiers )
        }
    }
}
/******************************************************************************/
pub fn unify_deref<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    // can be an AST representing any computation
    // that produces a pattern.
    let Node::AstroDeref( AstroDeref{expression:ref exp}) = *pattern
        else {return( Err(new_exception("VMError".to_string(), "unify: expected deref".to_string(), state, memory )))};

    let p = match walk( ArenaRc::clone(&exp),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    unify(term,p,state,memory,unifying)
}
/******************************************************************************/
pub fn unify_apply<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    
    if peek( ArenaRc::clone(&term) ) != "apply" {
        Err( new_exception("PatternMatchFailed".to_string(), "term and pattern disagree on \'apply\' node".to_string(),state,memory ))
    } else {

        // unpack the apply structures
        let Node::AstroApply(AstroApply{function:ref p_func,argument:ref p_arg}) = *pattern
            else {return( Err(new_exception("VMError".to_string(), "unify: expected apply.".to_string(), state, memory )))};
        let Node::AstroApply(AstroApply{function:ref t_func,argument:ref t_arg}) = *term
            else {return( Err(new_exception("VMError".to_string(), "unify: expected apply.".to_string(), state, memory )))};

        let Node::AstroID(AstroID{name:ref p_id}) = **p_func
            else {return( Err(new_exception("VMError".to_string(), "unify: expected id.".to_string(), state, memory )))};
        let Node::AstroID(AstroID{name:ref t_id}) = **t_func
            else {return( Err(new_exception("VMError".to_string(), "unify: expected id.".to_string(), state, memory )))};

        // make sure apply id's match
        if p_id != t_id {
            Err( new_exception("PatternMatchFailed".to_string(), format!("term '{}' does not match pattern '{}'",t_id,p_id), state, memory ))
        } else {
            // unify the args
            unify(ArenaRc::clone(t_arg), ArenaRc::clone(p_arg), state, memory, unifying)
        }
    }
}
/******************************************************************************/
pub fn unify_constraint<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    state.inc_constraint_lvl();
    unify(term,pattern,state,memory,unifying);
    state.dec_constraint_lvl();
    Ok(vec![])
}
/******************************************************************************/
pub fn unify_tuple<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    let mut unifiers: Vec<(ArenaRc<Node>,ArenaRc<Node>)> = vec![];
    let mut len: usize;
    let mut content: Vec<ArenaRc<Node>>;

    if let Node::AstroTuple(AstroTuple{contents:ref t_content}) = *term {
        if let Node::AstroTuple(AstroTuple{contents:ref p_content}) = *pattern {

            for i in 0..t_content.borrow().len() {
                let mut unifier = match unify( ArenaRc::clone(&t_content.borrow()[i]),ArenaRc::clone(&p_content.borrow()[i]),state,memory,unifying) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };
                unifiers.append( &mut unifier );
            }
            Ok( unifiers )

        } else {
            Err( new_exception("PatternMatchFailed".to_string(), format!("nodes '{}' and '{}' are not the same",peek(ArenaRc::clone(&term)),peek(ArenaRc::clone(&pattern))), state, memory ))
        }

    } else {
        Err( new_exception("PatternMatchFailed".to_string(), format!("nodes '{}' and '{}' are not the same",peek(ArenaRc::clone(&term)),peek(ArenaRc::clone(&pattern))), state, memory ))
    }
}
/******************************************************************************/
pub fn unify_pairs<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{

    // unpack nodes
    let Node::AstroPair(AstroPair{first:ref term_first,second:ref term_second}) = *term else {
        return Err( new_exception("PatternMatchFailed".to_string(), format!("nodes '{}' and '{}' are not the same",peek(ArenaRc::clone(&term)),peek(ArenaRc::clone(&pattern))), state, memory ))
    };
    let Node::AstroPair(AstroPair{first:ref pattern_first,second:ref pattern_second}) = *pattern else {
        return Err( new_exception("PatternMatchFailed".to_string(), format!("nodes '{}' and '{}' are not the same",peek(ArenaRc::clone(&term)),peek(ArenaRc::clone(&pattern))), state, memory ))
    };

    // construct unifier list
    let mut unifiers = vec![];
    let mut unifier = match unify( ArenaRc::clone(&term_first),ArenaRc::clone(&pattern_first),state,memory,unifying) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    unifiers.append( &mut unifier );

    let mut unifier = match unify( ArenaRc::clone(&term_second),ArenaRc::clone(&pattern_second),state,memory,unifying) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    unifiers.append( &mut unifier );

    Ok( unifiers ) 
}
/******************************************************************************/

pub fn unify<'a>( term: ArenaRc<Node>, pattern: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node>, unifying: bool) -> Result<Vec<(ArenaRc<Node>,ArenaRc<Node>)>, ArenaRc<Node>>{
    
    let term_type = peek( ArenaRc::clone(&term) );
    let pattern_type = peek( ArenaRc::clone(&pattern) );

    //println!("Unifying: term: {} to pattern: {}",term_type,pattern_type);

    return match (term_type,pattern_type,unifying) {

        (_,"id",_) =>               unify_id(term,pattern,state,memory,unifying),
        ("namedpattern",_,false) => subsume_namedpattern(term,pattern,state,memory,unifying),
        ("deref",_,false) =>        subsume_deref(term,pattern,state,memory,unifying),
        ("object","object",_) =>    unify_object_to_object(term,pattern,state,memory,unifying),
        ("string","string",_) =>    unify_string_to_string(term,pattern,state,memory,unifying),
        ("string","index",_) =>     unify_index(term,pattern,state,memory,unifying),
        (_,"namedpattern",_) =>     unify_namedpattern(term,pattern,state,memory,unifying),
        ("pair","pair",_) =>        unify_pairs(term,pattern,state,memory,unifying),
        (_,"typematch",_) =>        unify_typematch(term,pattern,state,memory,unifying),
        ("string",_,_) =>           Err( new_exception("PatternMatchFailed".to_string(), format!("term of type '{}' not allowed in pattern matching",term_type),state,memory)),
        (_,"string",_) =>           unify_string_to_other(term,pattern,state,memory,unifying),
        (_,"if",true) =>            unify_if(term,pattern,state,memory,unifying),
        (_,"if",false) =>           subsume_if(term,pattern,state,memory,unifying),
        ("integer","integer",_) =>  unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("integer","real",_) =>     unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("integer","bool",_) =>     unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("real","integer",_) =>     unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("real","real",_) =>        unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("real","bool",_) =>        unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("bool","integer",_) =>     unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("bool","real",_) =>        unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("bool","bool",_) =>        unify_primitive_to_primitive(term,pattern,state,memory,unifying),
        ("if",_,_) =>               subsume_conditional(term,pattern,state,memory,unifying),
        (_,"namedpattern",_) =>     unify_namedpattern(term,pattern,state,memory,unifying),
        (_,"none",_) =>             unify_none(term,pattern,state,memory,unifying),
        ("tolist",_,_) =>           Err( new_exception("PatternMatchFailed".to_string(), format!("term of type '{}' not allowed in pattern matching",term_type),state,memory)),
        ("rawtolist",_,_) =>        Err( new_exception("PatternMatchFailed".to_string(), format!("term of type '{}' not allowed in pattern matching",term_type),state,memory)),
        ("wherelist",_,_) =>        Err( new_exception("PatternMatchFailed".to_string(), format!("term of type '{}' not allowed in pattern matching",term_type),state,memory)),
        ("rawwherelist",_,_) =>     Err( new_exception("PatternMatchFailed".to_string(), format!("term of type '{}' not allowed in pattern matching",term_type),state,memory)),
        ("escape",_,_) =>           Err( new_exception("PatternMatchFailed".to_string(), format!("term of type '{}' not allowed in pattern matching",term_type),state,memory)),
        ("is",_,_) =>               Err( new_exception("PatternMatchFailed".to_string(), format!("term of type '{}' not allowed in pattern matching",term_type),state,memory)),
        ("in",_,_) =>               Err( new_exception("PatternMatchFailed".to_string(), format!("term of type '{}' not allowed in pattern matching",term_type),state,memory)),
        (_,"tolist",_) =>           Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"rawtolist",_) =>        Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"wherelist",_) =>        Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"rawwherelist",_) =>     Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"escape",_) =>           Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"is",_) =>               Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"in",_) =>               Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"foreign",_) =>          Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"function",_) =>         Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' not allowed in pattern matching",pattern_type),state,memory)),
        (_,"quote",_) =>            unify_quote(term,pattern,state,memory,unifying),
        ("quote","id",_) =>         Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' cannot be matched againt '{}'",pattern_type,term_type),state,memory)),
        ("quote","index",_) =>      Err( new_exception("PatternMatchFailed".to_string(), format!("pattern of type '{}' cannot be matched againt '{}'",pattern_type,term_type),state,memory)),
        ("quote",_,_) =>            unify_term_quote(term,pattern,state,memory,unifying),
        ("object","apply",_) =>     unify_object_to_apply(term,pattern,state,memory,unifying),
        (_,"index",_) =>            unify_index(term,pattern,state,memory,unifying),
        (_,"headtail",_) =>         unify_headtail(term,pattern,state,memory,unifying),
        (_,"rawheadtail",_) =>      unify_headtail(term,pattern,state,memory,unifying),
        (_,"list",_) =>             unify_list(term,pattern,state,memory,unifying),
        ("list",_,_) =>             unify_list(term,pattern,state,memory,unifying),
        (_,"deref",_) =>            unify_deref(term,pattern,state,memory,unifying),
        (_,"apply",_) =>            unify_apply(term,pattern,state,memory,unifying),
        (_,"constraint",_) =>       unify_constraint(term,pattern,state,memory,unifying),
        (_,_,_) =>                  unify_tuple(term,pattern,state,memory,unifying)
    };
}

/******************************************************************************/
pub fn walk<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{ 

    //println!("Walking: {}",peek(ArenaRc::clone(&node)));

    match *node {
        Node::AstroInteger(_) => Ok(node),
        Node::AstroReal(_) => Ok(node),
        Node::AstroBool(_) => Ok(node),
        Node::AstroString(_) => Ok(node),
        Node::AstroLineInfo(_) => set_lineinfo(node, state,memory),
        Node::AstroList(_) => list_exp(node, state, memory),
        Node::AstroTuple(_) => tuple_exp(node, state, memory),
        Node::AstroNone(_) => Ok(node),
        Node::AstroNil(_) => Ok(node),
        Node::AstroFunction(_) => function_exp(node,state, memory),
        Node::AstroToList(_) => to_list_exp(node,state, memory),
        Node::AstroRawToList(_) => raw_to_list_exp(node,state, memory),
        Node::AstroHeadTail(_) => head_tail_exp(node,state, memory),
        Node::AstroRawHeadTail(_) => raw_head_tail_exp(node,state, memory),
        Node::AstroSequence(_) => sequence_exp(node,state, memory),
        Node::AstroObject(_) => Ok(node),
        Node::AstroEval(_) => eval_exp(node,state, memory),
        Node::AstroQuote(_) => quote_exp(node,state, memory),
        Node::AstroConstraint(_) => constraint_exp(node,state, memory),
        Node::AstroTypeMatch(_) => constraint_exp(node,state, memory),
        Node::AstroForeign(_) => Ok(node),
        Node::AstroID(_) => id_exp(node,state, memory),
        Node::AstroApply(_) => apply_exp(node,state, memory),
        Node::AstroIndex(_) => index_exp(node,state, memory),
        Node::AstroEscape(_) => escape_exp(node,state, memory),
        Node::AstroIs(_) => is_exp(node,state, memory),
        Node::AstroIn(_) => in_exp(node,state, memory),
        Node::AstroIf(_) => if_exp(node,state, memory),
        Node::AstroNamedPattern(_) => named_pattern_exp(node,state, memory),
        Node::AstroMemberFunctionVal(_) => Ok(node),
        Node::AstroDeref(_) => deref_exp(node,state, memory),
        Node::AstroPair(_) => pair_exp(node,state, memory),
        _ => return ( Err( new_exception("VMError".to_string(), "walk: unknown node type".to_string(), state, memory ))),
    }    
}
/******************************************************************************/
pub fn set_lineinfo<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    match *node {
        Node::AstroLineInfo(AstroLineInfo{ref module,line_number}) => state.lineinfo = (module.clone(),line_number),
        _ => return( Err(new_exception("VMError".to_string(), "set_lineinfo error.".to_string(),state,memory ))),
    }
    Ok( node )
}
/******************************************************************************/
pub fn list_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroList( AstroList{ref contents} ) = *node 
        else {return( Err(new_exception("VMError".to_string(), "walk: expected list in list_exp()".to_string(), state, memory )))};

    let len = contents.borrow().len();
    for i in 0..len {
        let val = match walk( ArenaRc::clone(&contents.borrow()[i]), state, memory) {
            Ok( x ) => x,
            Err( x ) => return Err(x),
        };
        contents.borrow_mut()[i] = val;
    }
    Ok( node ) 
}
/******************************************************************************/
pub fn tuple_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroTuple( AstroTuple{ref contents} ) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected tuple in tuple_exp()".to_string(), state, memory )))
    };

    let len = contents.borrow().len();
    for i in 0..len {
        let val = match walk( ArenaRc::clone(&contents.borrow()[i]), state, memory) {
            Ok( x ) => x,
            Err( x ) => return Err(x),
        };
        contents.borrow_mut()[i] = val;
    }
    Ok( node ) 
}
/******************************************************************************/
pub fn pair_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroPair( AstroPair{ ref first, ref second } ) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected pair in pair_exp()".to_string(), state, memory )))
    };

    let first = match walk( ArenaRc::clone(first),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let second = match walk( ArenaRc::clone(second),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    Ok( memory.alloc_rc( Node::AstroPair(AstroPair::new( ArenaRc::clone(&first), ArenaRc::clone(&second) )) ) ) 
}
/******************************************************************************/
pub fn to_list_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroToList(AstroToList{ref start,ref stop,ref stride}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected to_list in to_list_exp()".to_string(), state, memory )))
    };

    let mut start_val;
    let mut stop_val;
    let mut stride_val;

    {
        let start = match walk(start.clone(),state,memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };

        let Node::AstroInteger(AstroInteger{value}) = *start else {
            return( Err(new_exception("VMError".to_string(), "walk: expected integer in to_list_exp()".to_string(), state, memory )))
        };
        start_val= value;
    }

    {
        let stop = match walk(stop.clone(),state,memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };

        let Node::AstroInteger(AstroInteger{value}) = *stop else {
            return( Err(new_exception("VMError".to_string(), "walk: expected integer in to_list_exp()".to_string(), state, memory )))
        };
        stop_val = value;
    }

    {
        let stride = match walk(stride.clone(),state,memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };

        let Node::AstroInteger(AstroInteger{value}) = *stride else {
            return( Err(new_exception("VMError".to_string(), "walk: expected integer in to_list_exp()".to_string(), state, memory )))
        };
        stride_val = value;
    }

    let len = 
        if stop_val > start_val {
            ((stop_val-start_val)/stride_val) as usize
        } else {
            ((start_val-stop_val)/stride_val) as usize
        };

    let mut newlist = Vec::with_capacity(len);

    for i in (start_val..=stop_val).step_by(stride_val as usize) {
        newlist.push(memory.alloc_rc(Node::AstroInteger(AstroInteger::new( i ))));
    }

    Ok( memory.alloc_rc(Node::AstroList( AstroList::new(Rc::new(RefCell::new(newlist))))))
}
/******************************************************************************/
pub fn function_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroFunction(AstroFunction{ref body_list}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected function in function_exp()".to_string(), state, memory )))
    };

    Ok( memory.alloc_rc(Node::AstroFunctionVal(AstroFunctionVal::new(ArenaRc::clone(body_list), Rc::new( state.symbol_table.get_closure()) ))))
}
/******************************************************************************/
pub fn raw_to_list_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroRawToList(AstroRawToList{ref start,ref stop,ref stride}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected to_list in to_list_exp()".to_string(), state, memory )))
    };

    walk( memory.alloc_rc(  Node::AstroToList( AstroToList{start:(*start).clone(),stop:(*stop).clone(),stride:(*stride).clone()} )), state, memory)
}
/******************************************************************************/
pub fn head_tail_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroHeadTail(AstroHeadTail{ref head,ref tail}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected head-tail exp in head_tail_exp().".to_string(), state, memory )))
    };

    let Node::AstroList( AstroList{ref contents} ) = **tail else {
        return( Err(new_exception("VMError".to_string(), "unsupported tail type in head-tail operator.".to_string(), state, memory )))
    };

    let mut new_contents = Vec::with_capacity(contents.borrow().len());
    new_contents.push(head.to_owned());
    for content in &*(contents.borrow()) {
        new_contents.push(content.to_owned());
    }

    Ok( memory.alloc_rc(  Node::AstroList( AstroList::new( Rc::new(RefCell::new(new_contents))))))
}
/******************************************************************************/
pub fn raw_head_tail_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroRawHeadTail(AstroRawHeadTail{ref head,ref tail}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected raw head-tail exp in raw_head_tail_exp().".to_string(), state, memory )))
    };

    walk( memory.alloc_rc(  Node::AstroHeadTail( AstroHeadTail{head:head.to_owned(),tail:tail.to_owned()})), state, memory)
}
/******************************************************************************/
pub fn sequence_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroSequence(AstroSequence{ref first,ref second}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected sequence expression in sequence_exp().".to_string(), state, memory )))
    };

    let first = match walk( ArenaRc::clone(&first),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let second = match walk( ArenaRc::clone(&second),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    Ok( memory.alloc_rc(  Node::AstroSequence( AstroSequence{first:first,second:second})))
}
/******************************************************************************/
pub fn eval_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroEval(AstroEval{ref expression}) = *node 
        else {return( Err(new_exception("VMError".to_string(), "walk: expected eval expression in exal_exp().".to_string(), state, memory )))};

    // Note: eval is essentially a macro call - that is a function
    // call without pushing a symbol table record.  That means
    // we have to first evaluate the argument to 'eval' before
    // walking the term.  This is safe because if the arg is already
    // the actual term it will be quoted and nothing happen
    let exp_value_expand = match walk( (*expression).clone(),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    // now walk the actual term..
    state.ignore_quote_on();
    let exp_val = match walk( exp_value_expand,state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    state.ignore_quote_off();

    Ok(exp_val)
}
/******************************************************************************/
pub fn quote_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroQuote(AstroQuote{ref expression}) = *node 
        else {return( Err(new_exception("VMError".to_string(), "walk: expected quote expression in quote_exp()".to_string(), state, memory )))};

    // quoted code should be treated like a constant if not ignore_quote
    if state.ignore_quote {
        walk( ArenaRc::clone(expression) ,state,memory)
    } else {
        Ok( node )
    }
}
/******************************************************************************/
pub fn constraint_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    //let Node::AstroConstraint(AstroConstraint{id,expression}) = node 
    //    else { panic!("ERROR: walk: expected constraint exp in constraint_exp().") };

    return( Err(new_exception("VMError".to_string(), "constraint patterns cannot be used as constructors.".to_string(), state, memory  )));
}
/******************************************************************************/
pub fn id_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>> {
    let Node::AstroID(AstroID{ref name}) = *node 
        else {return( Err(new_exception("VMError".to_string(), "walk: expected id expression in id_exp().".to_string(), state, memory )))};
    
    Ok( state.lookup_sym(name,true).clone() )
}
/******************************************************************************/
pub fn apply_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroApply(AstroApply{ref function,ref argument}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected apply expression in apply_exp().".to_string(), state, memory )))
    };
    
    // handle builtin operators that look like apply lists.
    if let Node::AstroID( AstroID{name:ref tag}) = **function {

        if OPERATOR_SYMBOLS.contains( &tag.as_str() ) {
            handle_builtins( ArenaRc::clone(&node), state, memory)
        } else{
            //println!("APPLYING {}!",tag);
            // handle function application
            let f_val = match walk( ArenaRc::clone(&function), state,memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
            let f_name = tag;
            let arg_val = match  walk( ArenaRc::clone(&argument), state,memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };

            let _type = peek( ArenaRc::clone(&f_val));

            if _type == "functionval" {
                return handle_call( memory.alloc_rc(Node::AstroNone(AstroNone::new())), ArenaRc::clone(&f_val), ArenaRc::clone(&arg_val), state, memory );

            } else if _type == "struct" {

                // object constructor call
                let Node::AstroStruct(AstroStruct{member_names:ref mnames,struct_memory:ref struct_mem}) = *f_val else {
                    return( Err(new_exception("VMError".to_string(), "apply exp: expected struct.".to_string(), state, memory )))
                };

                // create our object memory - memory cells now have initial values
                // we use structure memory as an init template
                let mut obj_memory = Rc::new(RefCell::new((struct_mem.borrow()).clone()));
                let new_id = AstroID::new(tag.to_string());
                //let new_mem = Node::AstroList(AstroList::new(obj_memory.len(), memory.alloc_rc( obj_memory)).unwrap());
                let obj_ref = memory.alloc_rc(Node::AstroObject(AstroObject::new(new_id,Rc::clone(&obj_memory))));

                for element in (&*mnames.borrow()) {
                    if let Node::AstroID(AstroID{name:ref tag}) = *ArenaRc::clone(&element) {
                        if tag == "__init__" {
                            // handle constructor call
                            let slot_ix = (&*mnames.borrow()).iter().position(|x| x == element);
                            let init_fval = ArenaRc::clone( &struct_mem.borrow()[ slot_ix.unwrap() ] );
                            handle_call( ArenaRc::clone(&obj_ref), ArenaRc::clone(&init_fval), ArenaRc::clone(&arg_val), state,memory);
                            return Ok( ArenaRc::clone(&obj_ref) )
                        }
                    } 
                }

                // the struct does not have an __init__ function but
                // we have a constructor call with args, e.g. Foo(1,2)
                // try to apply a default constructor by copying the
                // values from the arg list to the data slots of the object

                if let Node::AstroTuple(AstroTuple{contents:ref content}) = *arg_val {
                
                    let data_memory = data_only( RefCell::clone(&obj_memory) );

                    if content.borrow().len() != data_memory.len() {
                        return Err( new_exception("ValueError".to_string(), format!("default constructor expected {} arguments got {}",content.borrow().len(),data_memory.len()), state, memory ));
                    } else {
                        let data_ix = data_ix_list( RefCell::clone(&obj_memory) );
                        for i in 0..content.borrow().len() {
                            obj_memory.borrow_mut()[ data_ix[i] ] = ArenaRc::clone( &content.borrow()[ i ] );
                        }
                    }
                    return Ok(ArenaRc::clone(&obj_ref)); 
                } else if let Node::AstroPair(AstroPair{ref first,ref second}) = *arg_val {

                    let data_memory = data_only( RefCell::clone(&obj_memory) );

                    if 2 != data_memory.len() {
                        return Err( new_exception("ValueError".to_string(), format!("default constructor expected {} arguments got {}",2,data_memory.len()), state, memory ));
                    } else {
                        let data_ix = data_ix_list( RefCell::clone(&obj_memory) );
                        obj_memory.borrow_mut()[ data_ix[0] ] = ArenaRc::clone( &first );
                        obj_memory.borrow_mut()[ data_ix[1] ] = ArenaRc::clone( &second );
                    }
                    return Ok(ArenaRc::clone(&obj_ref));
                } else {
                    return( Err(new_exception("VMError".to_string(), "apply exp: expected tuple.".to_string(), state, memory )))
                };
            }
            Ok(node) 
        }
    } else if let Node::AstroIndex(AstroIndex{structure:ref s,index_exp:ref idx}) = **function {

        
        let f_val = match walk( ArenaRc::clone(&idx), state, memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let arg_val = match  walk( ArenaRc::clone(&s), state, memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };

        let _type = peek( ArenaRc::clone(&f_val));

        if _type == "functionval" {
            return handle_call( memory.alloc_rc(Node::AstroNone(AstroNone::new())), ArenaRc::clone(&f_val), ArenaRc::clone(&arg_val), state, memory );
        }


        Ok(node)
    } else if let Node::AstroApply(AstroApply{function:ref s,argument:ref idx}) = **function {

        let result = match walk( ArenaRc::clone(function), state, memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        

        apply_exp(result,state,memory)
    } else {
        // Error?
        Ok( node )
    }
}
/******************************************************************************/
pub fn handle_call<'a>( obj_ref: ArenaRc<Node>, node: ArenaRc<Node>, args: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    // unpack
    let Node::AstroFunctionVal(AstroFunctionVal{body_list:ref fpointer,ref closure}) = *node else {
        return( Err(new_exception("VMError".to_string(), "handle call: expected function value.".to_string(), state, memory )))
    };
    let Node::AstroID(AstroID{name:ref fname}) = **fpointer else {
        return( Err(new_exception("VMError".to_string(), "handle_call: expected id for function name.".to_string(), state, memory )))
    };

    // static scoping for functions
    // Note: we have to do this here because unifying
    // over the body patterns can introduce variable declarations,
    // think conditional pattern matching.
    let save_symtab = state.symbol_table.get_config();
    state.symbol_table.set_config( Rc::clone( &closure.0 ),Rc::clone( &closure.1 ), closure.2 );
    // state.push_scope(); 

    if let Node::AstroNone(AstroNone{}) = *obj_ref {
        ;
    } else {
        state.enter_sym( "this", obj_ref );
    }
    // execute the function
    // function calls transfer control - save our caller's lineinfo
    let old_lineinfo = state.lineinfo.clone();
    let return_value = state.dispatch_table[ fname.as_str() ]( args, state, memory );

    //  coming back from a function call - restore caller's lineinfo
    state.lineinfo = old_lineinfo;

    // NOTE: popping the function scope is not necessary because we
    // are restoring the original symtab configuration. this is necessary
    // because a return statement might come out of a nested with statement
    state.symbol_table.set_config(save_symtab.0, save_symtab.1, save_symtab.2);
    
    return_value
}
/******************************************************************************/
pub fn handle_builtins<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{

    let Node::AstroApply(AstroApply{ref function,ref argument}) = *node else {
        return( Err(new_exception("VMError".to_string(), "handle_builtins: expected apply expression.".to_string(), state, memory )))
    };
    let Node::AstroID( AstroID{name:ref builtin_type} ) = **function else {
        return( Err(new_exception("VMError".to_string(), "handle_builtins: expected id.".to_string(), state, memory )))
    };

    if BINARY_OPERATORS.contains( &builtin_type.as_str() ) {
        
        let Node::AstroPair( AstroPair{ref first,ref second}) = **argument else {
            return( Err(new_exception("VMError".to_string(), "handle_builtins: expected tuple for args.".to_string(), state, memory )))
        };

        let val_a = match walk( ArenaRc::clone(&first), state, memory ) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let val_b = match walk( ArenaRc::clone(&second), state, memory ) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        
        if builtin_type == "__plus__" {
            
            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroInteger( AstroInteger::new(v1+v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 as f64 + v2))));
                } else if let Node::AstroString( AstroString{value:ref v2}) = *val_b {
                        return Ok( memory.alloc_rc(  Node::AstroString(AstroString::new(v1.to_string()+v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in +", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 + v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 + v2))));
                } else if let Node::AstroString( AstroString{value:ref v2}) = *val_b {
                    return Ok( memory.alloc_rc(  Node::AstroString(AstroString::new(v1.to_string()+v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in +", peek(ArenaRc::clone(&val_b))).to_string(), state, memory ));
                }

            } else if let Node::AstroList( AstroList{contents:ref c1}) = *val_a {
                if let Node::AstroList( AstroList{contents:ref c2}) = *val_b {
                    let mut c3 = (**c1).clone(); // we have to do a data-clone here otherwise we edit other nodes in place
                    c3.borrow_mut().append( &mut (*c2.borrow_mut())) ;
                    return Ok( memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(c3 )))));
                } 
                
            } else if let Node::AstroString( AstroString{value:ref v1}) = *val_a {
                if let Node::AstroString( AstroString{value:ref v2}) = *val_b {
                    return Ok( memory.alloc_rc(  Node::AstroString(AstroString::new(v1.to_owned()+v2))));
                } else if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(  Node::AstroString(AstroString::new(v1.to_owned()+&v2.to_string()))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(  Node::AstroString(AstroString::new(v1.to_owned()+&v2.to_string()))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in +", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else {
                return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in +", peek(ArenaRc::clone(&val_b))), state, memory ));
            }

        } else if builtin_type == "__minus__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroInteger( AstroInteger::new(v1 - v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 as f64 - v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in -", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 - v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 - v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in -", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else { // We can only subtract real/integers
                return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in -", peek(ArenaRc::clone(&val_a))), state, memory ));
            }

        } else if builtin_type == "__times__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroInteger( AstroInteger::new(v1 * v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 as f64 * v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in *", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 * v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 * v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in *", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else { // We can only multiply real/integers
                return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in *", peek(ArenaRc::clone(&val_b))), state, memory ));
            }    
        } else if builtin_type == "__divide__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    if v2 == 0 { // Divison by 0 check
                        return Err( new_exception("ArithmeticError".to_string(), "divison by zero".to_string(), state, memory ));
                    } else {
                        return Ok( memory.alloc_rc(Node::AstroInteger( AstroInteger::new(v1 / v2))));
                    }
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    if v2 == 0.0 { // Divison by 0 check
                        return Err( new_exception("ArithmeticError".to_string(), "divison by zero".to_string(), state, memory ));
                    } else {
                        return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 as f64 / v2))));
                    }
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in /", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    if v2 == 0 { // Divison by 0 check
                        return Err( new_exception("ArithmeticError".to_string(), "divison by zero".to_string(), state, memory ));
                    } else {
                        return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 / v2 as f64))));
                    }
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    if v2 == 0.0 { // Divison by 0 check
                        return Err( new_exception("ArithmeticError".to_string(), "divison by zero".to_string(), state, memory ));
                    } else {
                        return Ok( memory.alloc_rc(Node::AstroReal( AstroReal::new(v1 / v2))));
                    }
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in /", peek(ArenaRc::clone(&val_b))), state, memory));
                }

            } else { // We can only divide real/integers
                return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in /", peek(ArenaRc::clone(&val_b))), state, memory));
            }    
        } else if builtin_type == "__or__" {

            let b1 = map2boolean( &val_a);
            let b2 = map2boolean( &val_b);
            let Node::AstroBool( AstroBool{value:b1_val}) = b1
                else {return( Err(new_exception("VMError".to_string(), "handle_builtins: expected boolean.".to_string(), state, memory )))};
            let Node::AstroBool( AstroBool{value:b2_val}) = b2
                else {return( Err(new_exception("VMError".to_string(), "handle_builtins: expected boolean.".to_string(), state, memory )))};

            return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(b1_val || b2_val))));
        } else if builtin_type == "__and__" {

            let b1 = map2boolean( &val_a);
            let b2 = map2boolean( &val_b);
            let Node::AstroBool( AstroBool{value:b1_val}) = b1
                else {return( Err(new_exception("VMError".to_string(), "handle_builtins: expected boolean.".to_string(), state, memory )))};
            let Node::AstroBool( AstroBool{value:b2_val}) = b2
                else {return( Err(new_exception("VMError".to_string(), "handle_builtins: expected boolean.".to_string(), state, memory )))};

            return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(b1_val && b2_val))));
        } else if builtin_type == "__gt__" {
            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 > v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 as f64 > v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in >", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 > v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 > v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in >", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else { 
                return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in >", peek(ArenaRc::clone(&val_b))), state, memory ));
            }

        } else if builtin_type == "__lt__" {
         
            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 < v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new((v1 as f64) < v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in <", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 < v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 < v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in <", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else { 
                return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in <", peek(ArenaRc::clone(&val_b))), state, memory ));
            }

        } else if builtin_type == "__le__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 <= v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new((v1 as f64) <= v2))));
                } else {
                    return Err( new_exception(format!("Unsupported type {} in <=", peek(ArenaRc::clone(&val_b))), "message goes here".to_string(), state, memory  ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 <= v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 <= v2))));
                } else {
                    return Err( new_exception(format!("Unsupported type {} in <=", peek(ArenaRc::clone(&val_b))), "message goes here".to_string(), state, memory ));
                }

            } else { 
                return Err( new_exception(format!("Unsupported type {} in <=", peek(ArenaRc::clone(&val_b))), "message goes here".to_string(), state, memory  ));
            }

        } else if builtin_type == "__ge__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 >= v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 as f64 >= v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in >=", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 >= v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 >= v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in >=", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else { // We can only subtract real/integers
                return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in >=", peek(ArenaRc::clone(&val_b))), state, memory ));
            }
        } else if builtin_type == "__eq__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 == v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 as f64 == v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in ==", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 == v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 == v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in ==", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else { // TODO
                // false??
                return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new( false ))));
                //println!("PEEK: {}",peek(val_a.clone()));
                //return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in ==", peek(ArenaRc::clone(&val_b))), state, memory ));
            }
        } else if builtin_type == "__ne__" {

            if let Node::AstroInteger( AstroInteger{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 != v2))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 as f64 != v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in =/=", peek(ArenaRc::clone(&val_b))), state, memory ));
                }

            } else if let Node::AstroReal( AstroReal{value:v1}) = *val_a {
                if let Node::AstroInteger( AstroInteger{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 != v2 as f64))));
                } else if let Node::AstroReal( AstroReal{value:v2}) = *val_b {
                    return Ok( memory.alloc_rc(Node::AstroBool( AstroBool::new(v1 != v2))));
                } else {
                    return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in =/=", peek(ArenaRc::clone(&val_b))), state, memory ));
                }
            } else { // TODO
                return Err( new_exception("ValueError".to_string(), format!("Unsupported type {} in =/=", peek(ArenaRc::clone(&val_b))), state, memory ));
            }
        }
    

        
    }
    Ok(node)
}
/******************************************************************************/
pub fn index_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroIndex(AstroIndex{ref structure,ref index_exp}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected index expression in index_exp().".to_string(), state, memory )))
    };

    // look at the semantics of 'structure'
    let structure_val =  match walk(ArenaRc::clone(&structure),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };

    // indexing/slicing
    let result = match read_at_ix(structure_val,ArenaRc::clone(&index_exp),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };


    Ok(result)
}
/******************************************************************************/
pub fn escape_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{

    let Node::AstroEscape(AstroEscape{content:ref fname}) = *node
        else {return( Err(new_exception("VMError".to_string(), "escape_exp(): expected ID.".to_string(), state, memory )))};
    
    let old_lineinfo = state.lineinfo.clone();
    let return_value = state.dispatch_table[ fname.as_str() ]( memory.alloc_rc(Node::AstroNone(AstroNone::new())), state, memory );

    //  coming back from a function call - restore caller's lineinfo
    state.lineinfo = old_lineinfo;

    return_value
}
/******************************************************************************/
pub fn is_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    // unpack node
    let Node::AstroIs(AstroIs{ref pattern,ref term}) = *node else {
        return( Err(new_exception("VMError".to_string(), "walk: expected is expression in is_exp().".to_string(), state, memory )))
    };

    let term_val = match walk((*term).clone(), state, memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let unifiers = unify(term_val,(*pattern).clone(),state,memory,true);

    if let Err(_) = unifiers {
        Ok( memory.alloc_rc(  Node::AstroBool(AstroBool::new(false))))
    } else {
        let unifiers = match unifiers {
            Ok(val) => val,
            Err(e) => return Err(e),
        };
        declare_unifiers(&unifiers,state,memory);
        Ok( memory.alloc_rc(  Node::AstroBool(AstroBool::new(true))))
    }
}
/******************************************************************************/
pub fn in_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroIn(AstroIn{ref expression,ref expression_list}) = *node 
        else {return( Err(new_exception("VMError".to_string(), "walk: expected in expression in in_exp().".to_string(), state, memory )))};

    let exp_val = match walk((*expression).clone(),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let exp_list_val = match walk((*expression_list).clone(),state,memory) {
        Ok( val ) => val,
        Err( e ) => return Err(e),
    };
    let Node::AstroList(AstroList{ref contents}) = *exp_list_val
        else {return( Err(new_exception("VMError".to_string(), "right argument to in operator has to be a list.".to_string(), state, memory )))};

    // We simply map the in operator to Rust's contains function
    if (*contents).borrow().contains( &exp_val ) {
        Ok( memory.alloc_rc(  Node::AstroBool(AstroBool::new(true))))
    } else {
        Ok( memory.alloc_rc(  Node::AstroBool(AstroBool::new(false))))
    }
}
/******************************************************************************/
pub fn if_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroIf(AstroIf{ref cond_exp,ref then_exp,ref else_exp}) = *node 
        else {return( Err(new_exception("VMError".to_string(), "walk: expected if expression in if_exp().".to_string(), state, memory )))};

    let cond_val = match walk( ArenaRc::clone(&cond_exp), state, memory ) {
        Ok( val ) => map2boolean(&val),
        Err( e ) => return Err(e),
    };

    let Node::AstroBool(AstroBool{value}) = cond_val 
        else {return( Err(new_exception("VMError".to_string(), "walk: if_exp: expected boolean from map2boolean.".to_string(), state, memory )))};
    
    if value {
        walk(ArenaRc::clone(&then_exp),state,memory)
    } else {
        walk(ArenaRc::clone(&else_exp),state,memory)
    }
}
/*******************************************************************************
# Named patterns - when walking a named pattern we are interpreting a
# a pattern as a constructor - ignore the name                                */
pub fn named_pattern_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{
    let Node::AstroNamedPattern(AstroNamedPattern{ref name,ref pattern}) =* node 
        else {return( Err(new_exception("VMError".to_string(), "walk: expected named pattern expression in named_pattern_exp().".to_string(), state, memory )))};

    walk((*pattern).clone(),state,memory)
}
/******************************************************************************/
pub fn deref_exp<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{

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
fn check_repeated_symbols(unifiers: &Vec<(ArenaRc<Node>,ArenaRc<Node>)> ) -> bool {
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
pub fn declare_unifiers<'a>( unifiers: &Vec<(ArenaRc<Node>,ArenaRc<Node>)>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<(), ArenaRc<Node> >{
    // walk the unifiers and bind name-value pairs into the symtab

    for (lhs,value) in unifiers {

        if let Node::AstroID(AstroID{ref name}) = **lhs {
            if name == "this" {
                return Err( new_exception("ValueError".to_string(), "'this' is a reserved keyword.".to_string(), state, memory ));
            } else {
                state.enter_sym(&name,ArenaRc::clone(value));
            }
        } else if let Node::AstroIndex(AstroIndex{ref structure,ref index_exp}) = **lhs {
            // Note: structures have to be declared before index access
            // can be successful!!  They have to be declared so that there
            // is memory associated with the structure.

            // indexing/slicing
            // update the memory of the object.
            store_at_ix(ArenaRc::clone(structure),ArenaRc::clone(index_exp),ArenaRc::clone(value),state,memory);
        } else {
            return Err( new_exception("ValueError".to_string(), format!("unknown unifier type '{}'",peek(ArenaRc::clone(lhs))), state, memory ));
        }
    }
    Ok(())
}
/******************************************************************************/
pub fn declare_formal_args<'a>( unifiers: &Vec<(ArenaRc<Node>,ArenaRc<Node>)>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<(), ArenaRc<Node> >{
    // unfiers is of the format: [ (pattern, term), (pattern, term),...]

    for (pattern,term) in unifiers {
        if let Node::AstroID(AstroID{ref name}) = **pattern {
            if name == "this" {
                return Err( new_exception("ValueError".to_string(), "'this' is a reserved keyword.".to_string(), state, memory ));
            } else {
                state.enter_sym(&name,ArenaRc::clone(term));
            }
        } else {
            return Err( new_exception("ValueError".to_string(), format!("unknown unifier type '{}'",peek(ArenaRc::clone(pattern))), state, memory ));
        }
    }
    Ok(())
}
/******************************************************************************/
pub fn store_at_ix<'a>( structure: ArenaRc<Node>, ix: ArenaRc<Node>, value: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<(), ArenaRc<Node>>{

    let mut structure_val = memory.alloc_rc(Node::AstroNone(AstroNone::new()));
    
     
    // Handle recurive application 
    if let Node::AstroIndex(AstroIndex{structure:ref s,index_exp:ref idx}) = *structure {

        let mut inner_mem = match read_at_ix(ArenaRc::clone(&s),ArenaRc::clone(&idx),state,memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };

        inner_mem = match *inner_mem {
            Node::AstroID( AstroID{name:ref id_tag} ) => state.symbol_table.lookup_sym(id_tag,true),
            _ => inner_mem,
        };

        if let Node::AstroList(AstroList{ref contents}) = *inner_mem {

            let ix_val = match walk(ArenaRc::clone(&ix), state, memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
            let Node::AstroInteger(AstroInteger{value:int_val}) = *ix_val else {
                return( Err(new_exception("VMError".to_string(), "store_at_ix: expected integer.".to_string(), state, memory )))
            };
    
            contents.borrow_mut()[int_val as usize] = ArenaRc::clone(&value);
            return Ok(());
        } else if let Node::AstroTuple(AstroTuple{ref contents}) = *inner_mem {

            let ix_val = match walk(ArenaRc::clone(&ix), state, memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
            let Node::AstroInteger(AstroInteger{value:int_val}) = *ix_val else {
                return( Err(new_exception("VMError".to_string(), "store_at_ix: expected integer.".to_string(), state, memory )))
            };
    
            contents.borrow_mut()[int_val as usize] = ArenaRc::clone(&value);
            return Ok(());
        } else if let Node::AstroObject(AstroObject{ref struct_id, ref object_memory}) = *inner_mem {

            let Node::AstroID(AstroID{name:ref tag}) = *ix else {
                return( Err(new_exception("VMError".to_string(), "store_at_ix: expected id.".to_string(), state, memory )))
            };

            let AstroID{name:ref obj_type} = *struct_id;
            let object_data = match walk( memory.alloc_rc(Node::AstroID(struct_id.clone())), state, memory ) {
                Ok( val ) => val,
                Err( error ) => return Err( error ),
            };

            let Node::AstroStruct(AstroStruct{member_names:ref struct_tags, struct_memory:ref struct_mem}) = *object_data else {
                return( Err(new_exception("VMError".to_string(), "store_at_ix: expected struct.".to_string(), state, memory )))
            };

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

            (object_memory.borrow_mut())[ found_idx ] = ArenaRc::clone( &value );
            return Ok(());
        }
        


        return Ok(()); 
        // println!("yoz! {}",peek(idx.clone()));

        // if let Node::AstroID(AstroID{ref name}) = **s {
        //     println!("yo! {}",name);
        //     let s = match walk(ArenaRc::clone(&s), state, memory) {
        //         Ok( val ) => val,
        //         Err( e ) => return Err(e),
        //     };
        //     println!("yo! {}",peek(s.clone()));

        //     if let Node::AstroStruct(AstroStruct{ref member_names,ref struct_memory}) = *s {
        //         println!("BO!");
        //     }
        // }
        
        // // Construct a list of all of the indices
        // let ix_val = match walk(ArenaRc::clone(&ix), state, memory) {
        //     Ok( val ) => val,
        //     Err( e ) => return Err(e),
        // };
        
        // let Node::AstroInteger(AstroInteger{value:v}) = *ix_val else {
        //     return( Err(new_exception("VMError".to_string(), "store_at_ix: expected integer index.".to_string(), state, memory )))
        // };
        // println!("yo!");
        // let mut idx_list = vec![ v ];
        // while let Node::AstroIndex(AstroIndex{structure:ref s,index_exp:ref idx}) = **s {
        //     let Node::AstroInteger(AstroInteger{value:v}) = *ix_val else {
        //         return( Err(new_exception("VMError".to_string(), "store_at_ix: expected integer index.".to_string(), state, memory )))
        //     };
        //     idx_list.push(v);
        //     inner_mem = ArenaRc::clone(s);
        // }

        // // Walk through the index list accessing memory until we reach the intended interior memory.
        // let mut local_memory = match walk(ArenaRc::clone(&inner_mem),state,memory) {
        //     Ok( val ) => val,
        //     Err( e ) => return Err(e),
        // };
        
        // for val in idx_list {
        //     local_memory = match *local_memory {
        //         Node::AstroList( AstroList{contents:ref mem} ) => ArenaRc::clone(&(**mem).borrow()[ val as usize ]),
        //         Node::AstroTuple( AstroTuple{contents:ref mem} ) => ArenaRc::clone(&(**mem).borrow()[ val as usize ]),
        //         _ => return( Err(new_exception("VMError".to_string(), "store_at_ix: expected list or tuple.".to_string(), state, memory ))),
        //     };
        // }
        // structure_val = match walk(ArenaRc::clone(&local_memory),state,memory) {
        //     Ok( val ) => val,
        //     Err( e ) => return Err(e),
        // };
        
    } else {

        // look at the semantics of 'structure'
        structure_val = match walk(ArenaRc::clone(&structure),state,memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
    }

    if let Node::AstroList( AstroList{contents:ref mem} ) = *structure_val {

        let ix_val = match walk(ArenaRc::clone(&ix), state, memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        let Node::AstroInteger(AstroInteger{value:int_val}) = *ix_val // TODO error clean up
            else {return( Err(new_exception("VMError".to_string(), "store_at_ix: expected integer.".to_string(), state, memory )))};

        mem.borrow_mut()[int_val as usize] = ArenaRc::clone(&value);
    
        Ok(()) 
    } else if let Node::AstroObject(AstroObject{struct_id:ref id,object_memory:ref mem}) = *structure_val {
        
        //let ix_val = walk(ArenaRc::clone(&ix), state).unwrap();
        //println!("TYPE IS {}",peek(ArenaRc::clone(&ix)));
        let Node::AstroID(AstroID{name:ref tag}) = *ix
            else {return( Err(new_exception("VMError".to_string(), "store_at_ix: expected id.".to_string(), state, memory )))};

        let AstroID{name:ref obj_type} = *id;
        let object_data = match walk( memory.alloc_rc(Node::AstroID(id.clone())), state, memory ) {
            Ok( val ) => val,
            Err( error ) => return Err( error ),
        };

        let Node::AstroStruct(AstroStruct{member_names:ref struct_tags,struct_memory:ref struct_mem}) = *object_data
            else {return( Err(new_exception("VMError".to_string(), "store_at_ix: expected struct.".to_string(), state, memory )))};

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
        
        //(mem.borrow_mut())[ found_idx ] = memory.alloc_rc(  Node::AstroNone(AstroNone::new()) );
        (mem.borrow_mut())[ found_idx ] = ArenaRc::clone( &value );

        Ok(()) 
    } else {
        Err( new_exception("ValueError".to_string(), format!("Index op not supported for '{}'",peek(structure_val)), state, memory ))
    }
}
/******************************************************************************/
pub fn read_at_ix<'a>( mut structure_val: ArenaRc<Node>, mut ix: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result<ArenaRc<Node>, ArenaRc<Node>>{

    // find the actual memory we need to access
    let mut struct_type = peek(ArenaRc::clone(&structure_val));
    if struct_type == "id" {
        structure_val = match walk( ArenaRc::clone(&structure_val), state, memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        };
        struct_type = peek(ArenaRc::clone(&structure_val));
    }
    if struct_type != "object" {
        ix = match walk( ArenaRc::clone(&ix), state, memory) {
            Ok( val ) => val,
            Err( e ) => return Err(e),
        } ;
    }
    let ix_type = peek(ArenaRc::clone(&ix));
    
    if ["list","tuple"].contains( &struct_type ) {

        if ix_type == "integer" {

            let Node::AstroInteger(AstroInteger{value:ix_val}) = *ix else {
                return( Err(new_exception("VMError".to_string(), "read_at_ix: expected integer.".to_string(), state, memory )))
            };

            let content = match *structure_val {
                Node::AstroList( AstroList{contents:ref c}) => c,
                Node::AstroTuple( AstroTuple{contents:ref c}) => c,
                _ => return( Err(new_exception("VMError".to_string(), "read_at_ix: expected list or tuple.".to_string(), state, memory  ))),
            };
            
            return Ok( ArenaRc::clone( &content.borrow()[ix_val as usize] ) );
        }
    } else if struct_type == "object" {

        let Node::AstroObject(AstroObject{struct_id:ref id,object_memory:ref mem}) = *structure_val else {
            return( Err(new_exception("VMError".to_string(), "read_at_ix: expected object.".to_string(), state, memory )))
        };

        let Node::AstroID(AstroID{name:ref tag}) = *ix else {
            return( Err(new_exception("VMError".to_string(), "read_at_ix: expected id.".to_string(), state, memory )))
        };

        let AstroID{name:ref obj_type} = *id;
        let object_data = match walk( memory.alloc_rc(Node::AstroID(id.clone())), state, memory ) {
            Ok( val ) => val,
            Err( error ) => return Err( error ),
        };

        let Node::AstroStruct(AstroStruct{member_names:ref struct_tags,struct_memory:ref struct_mem}) = *object_data else {
            return( Err(new_exception("VMError".to_string(), "read_at_ix: expected struct.".to_string(), state, memory) ))
        };

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
        
        return Ok( ArenaRc::clone( &mem.borrow_mut()[ found_idx ]) );

    } else if struct_type == "string" {

        let Node::AstroInteger(AstroInteger{value:ix_val}) = *ix
                else {return( Err(new_exception("VMError".to_string(), "read_at_ix: expected integer.".to_string(), state, memory )))};

        let content = match *structure_val {
            Node::AstroString( AstroString{value:ref val}) => val,
            _ => return( Err(new_exception("VMError".to_string(), "read_at_ix: expected string.".to_string(), state, memory  )))
        };

        match content.chars().nth( ix_val as usize) {
            Some( character ) => return Ok(memory.alloc_rc(Node::AstroString(AstroString::new(character.to_string())))),
            _                 => return Err(new_exception("ValueError".to_string(), format!("String '{}' too short for index value {}",content,ix_val), state, memory )),
        }
    }

    Ok(structure_val.clone())
}
/******************************************************************************/
pub fn exit<'a>( error: ArenaRc<Node> , state: &'a mut State, memory: &'a mut Arena<Node> ) -> ! {
    println!("Asteroid encountered an error.");
    let error = match walk(error,state,memory) {
        Ok( val ) => val,
        Err( e ) => e,
    };
    match *error {
        Node::AstroObject(AstroObject{ struct_id:ref id, object_memory:ref mem  }) => {
            match id {
                AstroID{name:ref tag} if tag == "Exception" => {
                    match *mem.borrow()[0] {
                        Node::AstroString(AstroString{value:ref v}) if v == "ValueError"=> {
                            match *mem.borrow()[1] {
                                Node::AstroString(AstroString{value:ref msg}) => println!("ValueError: {}",msg),
                                _ => println!("Unknown Error Type"),
                            };
                        },
                        Node::AstroString(AstroString{value:ref v}) if v == "PatternMatchFailed"=> {
                            match *mem.borrow()[1] {
                                Node::AstroString(AstroString{value:ref msg}) => println!("PatternMatchFailed: {}",msg),
                                _ => println!("Unknown Error Type"),
                            };
                        },
                        Node::AstroString(AstroString{value:ref v}) if v == "NonLinearPattern"=> {
                            match *mem.borrow()[1] {
                                Node::AstroString(AstroString{value:ref msg}) => println!("NonLinearPattern: {}",msg),
                                _ => println!("Unknown Error Type"),
                            };
                        },
                        Node::AstroString(AstroString{value:ref v}) if v == "FileNotFound"=> {
                            match *mem.borrow()[1] {
                                Node::AstroString(AstroString{value:ref msg}) => println!("FileNotFound: {}",msg),
                                _ => println!("Unknown Error Type"),
                            };
                        },
                        Node::AstroString(AstroString{value:ref v}) if v == "ArithmeticError"=> {
                            match *mem.borrow()[1] {
                                Node::AstroString(AstroString{value:ref msg}) => println!("ArithmeticError: {}",msg),
                                _ => println!("Unknown Error Type"),
                            };
                        },
                        Node::AstroString(AstroString{value:ref v}) if v == "VMError"=> {
                            match *mem.borrow()[1] {
                                Node::AstroString(AstroString{value:ref msg}) => println!("An internal VM Error occurred.\nmessage: {}",msg),
                                _ => println!("Unknown Error Type"),
                            };
                        },
                        _ => println!("Unknown Error Type {}",peek(ArenaRc::clone(&mem.borrow()[0]))),
                    };
                },
                _ => println!("Unknown Error Type"),
            };
        },
        _ => println!("Unknown Error Type"),
    };
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
        let mut memory = Arena::new();

        let s1 = memory.alloc_rc(  Node::AstroString( AstroString::new(String::from("hello"))) );
        let s2 = memory.alloc_rc(  Node::AstroString( AstroString::new(String::from("hello"))) );
        let s3 = memory.alloc_rc(  Node::AstroString( AstroString::new(String::from("nothello"))) );

        let mut state = State::new().unwrap();
        let u = true;
        
        let out = match unify(s1.clone(),s2,&mut state,&mut memory, u) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
        assert_eq!(out.len(),0); //SHOULD PASS

        let out = unify(s1,s3,&mut state,&mut memory, u);
        match out {
            Err(x) => (), //SHOULD BE ERR
            _ => panic!("Regex text failed"),
        }
    }
    #[test]
    fn test_unify_primitives() {
        let mut memory = Arena::new();
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let i3 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));

        let b1 = memory.alloc_rc(  Node::AstroBool( AstroBool::new(true)));
        let b2 = memory.alloc_rc(  Node::AstroBool( AstroBool::new(false)));
        let b3 = memory.alloc_rc(  Node::AstroBool( AstroBool::new(true)));

        let r1 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(1.1)));
        let r2 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(1.2)));
        let r3 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(1.1)));

        let mut state = State::new().unwrap();
        let u_mode = true;

        let out1 = match unify(i1.clone(),i3,&mut state,&mut memory,u_mode){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
        let out2 = match unify(b1.clone(),b3,&mut state,&mut memory,u_mode){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
        let out3 = match unify(r1.clone(),r3,&mut state,&mut memory,u_mode){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        assert_eq!(out1.len(),0); //SHOULD PASS
        assert_eq!(out2.len(),0); //SHOULD PASS
        assert_eq!(out3.len(),0); //SHOULD PASS

        let out1 = unify(i1.clone(),i2,&mut state,&mut memory,u_mode);
        let out2 = unify(b1.clone(),b2,&mut state,&mut memory,u_mode);
        let out3 = unify(r1.clone(),r2,&mut state,&mut memory,u_mode);

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();
        let u_mode = true;

        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let i3 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(3)));

        let l1 = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));
        let l2 = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i2.clone(),i3.clone()])))));
        let l3 = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i3.clone(),i2.clone(),i1.clone()])))));
        let l4 = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));

        let out1 = match unify( ArenaRc::clone(&l1),ArenaRc::clone(&l4),&mut state,&mut memory,u_mode) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
        let out2 = unify( ArenaRc::clone(&l1),ArenaRc::clone(&l2),&mut state,&mut memory,u_mode );
        let out3 = unify( ArenaRc::clone(&l1),ArenaRc::clone(&l3),&mut state,&mut memory,u_mode );

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
        let mut memory = Arena::new();
        let newline = AstroLineInfo::new( String::from("test1"),123 );
        let mut state = State::new().unwrap();
        {
            let out1 = state.lineinfo.clone();
            assert_eq!(out1,(String::from("<input>"),1));
        }

        walk( memory.alloc_rc(  Node::AstroLineInfo(newline)),&mut state, &mut memory );

        {
            let out2 = state.lineinfo.clone();
            assert_eq!(out2,(String::from("test1"),123));
        }

        let newline = AstroLineInfo::new( String::from("math"), 987654321);
        walk( memory.alloc_rc(   Node::AstroLineInfo(newline)),&mut state, &mut memory );

        {
            let out3 = state.lineinfo.clone();
            assert_eq!(out3,(String::from("math"), 987654321));
        }
    }
    #[test]
    fn test_unify_var_to_int() {
        // let x = 123.
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let int = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(123)));

        let unifier = match unify(int,var,&mut state,&mut memory,true) {
            Ok( val ) => val,
            Err(e) => panic!("error!"),
        };

        let out = declare_unifiers( &unifier, &mut state, &mut memory );

        let check = state.lookup_sym("x",true);
        match *check {
            Node::AstroInteger(AstroInteger{value:123}) => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_real() {
        // let x = 1.23.
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let val = memory.alloc_rc(Node::AstroReal(AstroReal::new(1.23)));

        let unifier = match unify(val,var,&mut state,&mut memory,true) {
            Ok(val) => val,
            Err(e) => panic!("error!"),
        };

        let out = declare_unifiers( &unifier, &mut state, &mut memory );

        let check = state.lookup_sym("x",true);
        match *check {
            Node::AstroReal(AstroReal{value:val}) if val == 1.23 => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_string() {
        // let x = "hello123".
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let val = memory.alloc_rc(Node::AstroString(AstroString::new("hello123".to_string())));

        let unifiers = match unify(val,var,&mut state,&mut memory,true) {
            Ok(val) => val,
            Err(e) => panic!("error!"),
        };

        let out = declare_unifiers( &unifiers, &mut state, &mut memory );

        let check = state.lookup_sym("x",true);
        match *check {
            Node::AstroString(AstroString{value:ref val}) if val == "hello123" => (),
            _ => panic!("test failed"),
        };
    }
    #[test]
    fn test_unify_var_to_bool() {
        // let x = false.
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let val = memory.alloc_rc(Node::AstroBool(AstroBool::new(false)));

        let unifiers = match unify(val,var,&mut state,&mut memory,true){
            Ok(val) => val,
            Err(e) => panic!("Error"),
        };

        let out = declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let val2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(4)));
        let var3 = memory.alloc_rc(Node::AstroID(AstroID::new("z".to_string())));
        let val3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(8)));

        let unifiers = match unify(val1,var1,&mut state,&mut memory,true){
            Ok(val) => val,
            Err(e) => panic!("Error!")
        };
        declare_unifiers( &unifiers, &mut state, &mut memory );

        let unifiers = match unify(val2,var2,&mut state,&mut memory,true){
            Ok(val) => val,
            Err(e) => panic!("Error!")
        };
        declare_unifiers( &unifiers, &mut state, &mut memory );

        let unifiers = match unify(val3,var3,&mut state,&mut memory,true){
            Ok(val) => val,
            Err(e) => panic!("Error!")
        };
        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3)));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let val2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(4))); 
        let varlist = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&var1),ArenaRc::clone(&var2)])))));
        let vallist = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&val1),ArenaRc::clone(&val2)])))));

        let unifiers = match unify(vallist,varlist,&mut state,&mut memory,true) {
            Ok(val) => val,
            Err(e) => panic!("error!"),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = memory.alloc_rc(Node::AstroString(AstroString::new("string1".to_string())));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let val2 = memory.alloc_rc(Node::AstroReal(AstroReal::new(1.3334)));
        let int1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3))); 
        let int2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3)));
        let varlist = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&var1),ArenaRc::clone(&var2),ArenaRc::clone(&int1)])))));
        let vallist = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&val1),ArenaRc::clone(&val2),ArenaRc::clone(&int2)])))));

        let unifiers = match unify(vallist,varlist,&mut state,&mut memory,true){
            Ok(val) => val,
            Err(e) => panic!("error!")
        };

        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let val2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3))); 
        let var3 = memory.alloc_rc(Node::AstroID(AstroID::new("z".to_string())));
        let val3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(4))); 
        let varlist = memory.alloc_rc(  Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&var1),ArenaRc::clone(&var2),ArenaRc::clone(&var3)])))));
        let vallist = memory.alloc_rc(  Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&val1),ArenaRc::clone(&val2),ArenaRc::clone(&val3)])))));

        let unifiers = match unify(vallist,varlist,&mut state,&mut memory,true){
            Ok(val) => val,
            Err(e) => panic!("error!")
        };
        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let val1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(234)));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));

        let unifiers = match unify(val1,ArenaRc::clone(&var1),&mut state,&mut memory,true){
            Ok(val) => val,
            Err(e) => panic!("error!")
        };

        declare_unifiers( &unifiers, &mut state, &mut memory );

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:234}) => (),
            _ => panic!("test failed"),
        };

        let unifiers = match unify(ArenaRc::clone(&var1),var2,&mut state,&mut memory,true){
            Ok(val) => val,
            Err(e) => panic!("error!")
        };

        declare_unifiers( &unifiers, &mut state, &mut memory );

        let check2 = state.lookup_sym("y",true);
        match *check2 {
            Node::AstroInteger(AstroInteger{value:234}) => (),
            Node::AstroInteger(AstroInteger{value:v}) => println!("{}",v),
            _ =>    println!("DEBUG: {}", peek(ArenaRc::clone(&check2))),
        };

    }
    #[test]
    fn test_unify_int_to_namedpattern() {
        // let x:%integer = 17.
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var1 = AstroID::new("x".to_string());
        let pmatch_type = memory.alloc_rc(Node::AstroString( AstroString::new( "integer".to_string())));
        let pmatch = memory.alloc_rc(Node::AstroTypeMatch(AstroTypeMatch::new(pmatch_type)));
        let p = memory.alloc_rc(Node::AstroNamedPattern(AstroNamedPattern::new(var1,pmatch)));
        let val1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(17)));

        let unifiers = match unify(val1,p,&mut state,&mut memory,true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();

        let mut state = State::new().unwrap();
        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(0)));
        let i3 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(3)));
        let i4 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let l1 = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i1.clone(),i2.clone(),i3.clone()])))));
        let idx_exp = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));

        let unifiers = match unify(ArenaRc::clone(&l1),ArenaRc::clone(&var1),&mut state,&mut memory,true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory );

        let idx1 = memory.alloc_rc(  Node::AstroIndex( AstroIndex::new( ArenaRc::clone(&var1), ArenaRc::clone(&idx_exp) )));

        let unifiers = match unify(ArenaRc::clone(&i4),ArenaRc::clone(&idx1),&mut state,&mut memory,true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory );
        let check1 = state.lookup_sym("x",true);

        let Node::AstroList(AstroList{contents:ref c}) = *check1 else {
            panic!("test failed")};
        
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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('integer', 1), ('integer', 1)])))
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&i1),ArenaRc::clone(&i2))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));

        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple', [('integer', 1), ('real', 1.1)])))
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let r1 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(1.1)));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&i1),ArenaRc::clone(&r1))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple',  [('real', 1.35), ('integer', 1)])))
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let r1 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(1.35)));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&r1),ArenaRc::clone(&i1))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        // exp_val = walk(('apply', ('id', '__plus__'), ('tuple',  [('real', 1.35), ('real', 2.15)])))
        let r1 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(1.35)));
        let r2 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(2.15)));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&r1),ArenaRc::clone(&r2))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let i3 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(3)));
        let i4 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(4)));
        let l1 = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i1.clone(),i2.clone()])))));
        let l2 = memory.alloc_rc(  Node::AstroList( AstroList::new(Rc::new(RefCell::new(vec![i3.clone(),i4.clone()])))));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&l1),ArenaRc::clone(&l2))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        
        let s1 = memory.alloc_rc(  Node::AstroString( AstroString::new("Hello ".to_string())));
        let s2 = memory.alloc_rc(  Node::AstroString( AstroString::new("World!".to_string())));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&s1),ArenaRc::clone(&s2))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello World!",v);
    }
    #[test]
    fn test_prog_addition_string_to_int() {
        // rust compiler:
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        
        let s1 = memory.alloc_rc(  Node::AstroString( AstroString::new("Hello ".to_string())));
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(123)));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&s1),ArenaRc::clone(&i1))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello 123",v);
    }
    #[test]
    fn test_prog_addition_string_to_real() {
        // rust compiler:
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        
        let s1 = memory.alloc_rc(  Node::AstroString( AstroString::new("Hello ".to_string())));
        let r1 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(1.23)));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&s1),ArenaRc::clone(&r1))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("Hello 1.23",v);
    }
    #[test]
    fn test_prog_addition_int_to_string() {
        // rust compiler:
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        
        let s1 = memory.alloc_rc(  Node::AstroString( AstroString::new(" Hello".to_string())));
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(123)));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&i1),ArenaRc::clone(&s1))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

        let check1 = state.lookup_sym("a",true);
        let Node::AstroString(AstroString{value:ref v}) = *check1
            else {panic!("test failed")};
        assert_eq!("123 Hello",v);
    }
    #[test]
    fn test_prog_addition_real_to_string() {
        // rust compiler:
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        
        let s1 = memory.alloc_rc(  Node::AstroString( AstroString::new(" Hello".to_string())));
        let r1 = memory.alloc_rc(  Node::AstroReal( AstroReal::new(1.23)));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&r1),ArenaRc::clone(&s1))));
        let id1 = memory.alloc_rc(  Node::AstroID( AstroID::new( "__plus__".to_string() )));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&t1))));
        let exp_val = match walk( ArenaRc::clone( &apply1), &mut state, &mut memory){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // unifiers = unify(exp_val,('id', 'a'))
        let id2 = memory.alloc_rc(  Node::AstroID( AstroID::new( "a".to_string() )));
        let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true){
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        // declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory );

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
        
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.ast',1)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );
        
        //exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__gt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('id', 'x'), ('null',))))
        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("__gt__".to_string())));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let var3 = memory.alloc_rc(Node::AstroID(AstroID::new("POS_INT".to_string())));
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(0)));
        let null1 = memory.alloc_rc(  Node::AstroNone( AstroNone::new()));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&var2),ArenaRc::clone(&i1))));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&var1), ArenaRc::clone(&t1) )));
        let if1 = memory.alloc_rc(  Node::AstroIf( AstroIf::new( ArenaRc::clone(&apply1), ArenaRc::clone(&var2), ArenaRc::clone(&null1))));
        let quote1 = memory.alloc_rc(  Node::AstroQuote( AstroQuote::new( ArenaRc::clone( &if1))));
        let exp_val = walk( quote1, &mut state, &mut memory );

        //error handling
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        //unifiers = unify(exp_val,('id', 'POS_INT'))
        let unifiers = unify( exp_val, var3, &mut state, &mut memory, true);

        //error handling
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        //declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory);

        //set_lineinfo('prog.txt',2)
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:String::from("prog.ast"),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        //exp_val = walk(('integer', 2))
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let exp_val = walk( i2, &mut state, &mut memory );

        //error handling
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        //unifiers = unify(exp_val,('named-pattern', ('id', 'x'), ('deref', ('id', 'POS_INT'))))
        let var3 = AstroID::new("x".to_string());
        let var4 = memory.alloc_rc(Node::AstroID(AstroID::new("POS_INT".to_string())));
        let deref1 = memory.alloc_rc(  Node::AstroDeref(AstroDeref::new( ArenaRc::clone(&var4) )));
        let namedp1 = memory.alloc_rc(Node::AstroNamedPattern(AstroNamedPattern::new( var3, ArenaRc::clone(&deref1))));
        let unifiers = unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&namedp1), &mut state, &mut memory, true);

        //error handling
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        //declare_unifiers(unifiers)
        declare_unifiers( &unifiers, &mut state, &mut memory);

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('quote', ('if-exp', ('apply', ('id', '__gt__'), ('tuple', [('id', 'x'), ('integer', 0)])), ('id', 'x'), ('null',))))
        // unifiers = unify(exp_val,('id', 'POS_INT'))
        // declare_unifiers(unifiers)

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("__gt__".to_string())));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let var3 = memory.alloc_rc(Node::AstroID(AstroID::new("POS_INT".to_string())));
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(0)));
        let null1 = memory.alloc_rc(  Node::AstroNone( AstroNone::new()));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&var2),ArenaRc::clone(&i1))));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&var1), ArenaRc::clone(&t1) )));
        let if1 = memory.alloc_rc(  Node::AstroIf( AstroIf::new( ArenaRc::clone(&apply1), ArenaRc::clone(&var2), ArenaRc::clone(&null1))));
        let quote1 = memory.alloc_rc(  Node::AstroQuote( AstroQuote::new( ArenaRc::clone( &if1))));
        let exp_val = walk( quote1, &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, var3, &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        // set_lineinfo('prog.txt',2)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'sum'))
        // declare_unifiers(unifiers)

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id8 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast72".to_string())));
        let id9 = memory.alloc_rc(Node::AstroID(AstroID::new("sum".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new( ArenaRc::clone(&id8) )));
        let exp_val = walk( ArenaRc::clone(&func1), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id9), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        // set_lineinfo('prog.txt',8)
        // exp_val = walk(('apply', ('id', 'sum'), ('integer', 5)))
        // unifiers = unify(exp_val,('id', 'y'))
        // declare_unifiers(unifiers)

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id10 = memory.alloc_rc(Node::AstroID(AstroID::new("sum".to_string())));
        let id11 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(5)));
        let apply2 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id10), ArenaRc::clone(&i2) )));
        let exp_val = walk( ArenaRc::clone(&apply2), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id11), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);    
        
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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        // # structure def for A
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("a".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("b".to_string())));
        let d1 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id1))));
        let d2 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id2))));
        let member_list = vec![ ArenaRc::clone(&d1), ArenaRc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( ArenaRc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( memory.alloc_rc(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = memory.alloc_rc(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", ArenaRc::clone(&struct_type)  );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("A".to_string())));
        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("obj".to_string())));
        let t1 = memory.alloc_rc(  Node::AstroTuple( AstroTuple::new( Rc::new(RefCell::new(vec![ArenaRc::clone(&i1),ArenaRc::clone(&i2)])))));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id3), ArenaRc::clone(&t1) )));
        let exp_val = walk( ArenaRc::clone(&apply1), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id4), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);  
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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(0)));
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("ctr".to_string())));
        let exp_val = walk( ArenaRc::clone(&i1), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id1), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);  

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(100)));
        let i3 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("__lt__".to_string())));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("__plus__".to_string())));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&id1),ArenaRc::clone(&i2))));
        let t2 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&id1),ArenaRc::clone(&i3))));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id2), ArenaRc::clone(&t1) )));
        let apply2 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id3), ArenaRc::clone(&t2) )));

        let mut loop_val = match walk(ArenaRc::clone(&apply1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err(e) => panic!("Error"),
        };
        while let Node::AstroBool(AstroBool{value:true}) = map2boolean( &loop_val) {

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, &mut state, &mut memory );

            let exp_val = walk( ArenaRc::clone(&apply2), &mut state, &mut memory);
            let exp_val = match exp_val {
                Ok( val ) => val,
                Err( e ) => exit(e, &mut state, &mut memory),
            };

            let unifiers = unify( exp_val, ArenaRc::clone(&id1), &mut state, &mut memory, true);

            let unifiers = match unifiers {
                Ok( val ) => val,
                Err( e ) => exit(e, &mut state, &mut memory),
            };

            let check1 = state.lookup_sym("ctr",true);
            let Node::AstroInteger(AstroInteger{value:v}) = *check1 else {panic!("test failed.")};

            declare_unifiers( &unifiers, &mut state, &mut memory); 

            loop_val = match walk(ArenaRc::clone(&apply1), &mut state, &mut memory) {
                Ok( val ) => val,
                Err(e) => panic!("Error"),
            };
        }

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

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
    
        fn _ast72<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>> {
            
            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
            set_lineinfo(  new_lineinfo, state, memory );

            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("radius".to_string())));

            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&id1), state, memory, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state, memory),
                };

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
                set_lineinfo(  new_lineinfo, state, memory );

                let exp_val = walk( ArenaRc::clone(&id1), state, memory );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state, memory),
                };

                let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("this".to_string())));
                let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("diameter".to_string())));
                let index1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id2), ArenaRc::clone(&id1))));
                let index2 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id2), ArenaRc::clone(&id3))));

                let unifiers = unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&index1), state, memory, true);

                let unifiers = match unifiers {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state, memory),
                };

                declare_unifiers( &unifiers, state, memory);

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
                set_lineinfo(  new_lineinfo, state, memory );

                let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("__times__".to_string())));
                let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
                let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&i1),ArenaRc::clone(&id1))));
                let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id4), ArenaRc::clone(&t1))));

                let exp_val = walk( ArenaRc::clone(&apply1), state, memory );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state, memory),
                };

                let unifiers = unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&index2), state, memory, true);

                let unifiers = match unifiers {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state, memory),
                };

                declare_unifiers( &unifiers, state, memory);

                state.pop_scope();

                return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));
            } else {
                return  new_exception("ValueError".to_string(), "none of the function bodies unified with actual parameters".to_string(), state, memory  );
            }
        }
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("__init__") , _ast72 );
        
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );   

        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("radius".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("diameter".to_string())));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("__init__".to_string())));
        let data1 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id1))));
        let data2 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id2))));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new( ArenaRc::clone(&id3))));
        let unify1 = memory.alloc_rc(Node::AstroUnify(AstroUnify::new( ArenaRc::clone(&id3), ArenaRc::clone(&func1))));

        let member_list = vec![ ArenaRc::clone(&data1), ArenaRc::clone(&data2), ArenaRc::clone(&unify1) ];
        let mut struct_memory: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( ArenaRc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( memory.alloc_rc(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "unify" {
                let Node::AstroUnify(AstroUnify{term:ref id_node,pattern:ref function_exp}) = *member
                    else {panic!("ERROR: object construction: expection unify node.")};
                let function_val = match walk( ArenaRc::clone(&function_exp), &mut state, &mut memory) {
                    Ok( val ) => val,
                    Err ( e ) => panic!("error!"),
                };
                struct_memory.borrow_mut().push( ArenaRc::clone( &function_val ));
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "noop" {
                ;// pass
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }

        let struct_type = memory.alloc_rc(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "Circle", ArenaRc::clone(&struct_type)  );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo(AstroLineInfo{module:"prog.ast".to_string(),line_number:10}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );   

        // exp_val = walk(('apply', ('id', 'Circle'), ('integer', 2)))
        // unifiers = unify(exp_val,('id', 'a'))
        // declare_unifiers(unifiers)
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("Circle".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("a".to_string())));
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id1), ArenaRc::clone(&i1))));

        let exp_val = walk( ArenaRc::clone(&apply1), &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&id2), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();
        
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );   

        let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
        let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
        let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3)));
        let l1 = memory.alloc_rc(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&i1),ArenaRc::clone(&i2),ArenaRc::clone(&i3)])))));
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
     
        let exp_val = walk( ArenaRc::clone(&l1), &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id1), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );   

        let i4 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(4)));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let i5 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
        let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id2),ArenaRc::clone(&i5))));

        let exp_val = walk( ArenaRc::clone(&i4), &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&idx1), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );   

        let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
        let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
        let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3)));
        let i4 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(4)));
        let i5 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(5)));
        let l1 = memory.alloc_rc(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&i2),ArenaRc::clone(&i3),ArenaRc::clone(&i4)])))));
        let l2 = memory.alloc_rc(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&i1),ArenaRc::clone(&l1),ArenaRc::clone(&i5)])))));
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( ArenaRc::clone(&l2), &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id1), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );   

        let s1 = memory.alloc_rc(Node::AstroString(AstroString::new("hello".to_string())));
        let i6 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
        let i7 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id2),ArenaRc::clone(&i6))));
        let idx2 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&idx1),ArenaRc::clone(&i7))));

        let exp_val = walk( ArenaRc::clone(&s1), &mut state, &mut memory );
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&idx2), &mut state, &mut memory, true);
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

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
        fn _ast72<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>> {
            
            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, state, memory );

            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
            let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("tail".to_string())));
            let ht1 = memory.alloc_rc(Node::AstroHeadTail(AstroHeadTail::new(ArenaRc::clone(&id1),ArenaRc::clone(&id2))));

            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&ht1), state, memory, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state, memory),
                };

                let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
                set_lineinfo(  new_lineinfo, state, memory );

                let exp_val = walk( ArenaRc::clone(&id3), state, memory );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state, memory),
                };

                state.pop_scope();

                return Ok( exp_val )
            } else {
                return  new_exception("ValueError".to_string(), "none of the function bodies unified with actual parameters".to_string(), state, memory );
            }
        }

        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("_ast72") , _ast72 as fn( ArenaRc<Node>,&mut State, &mut Arena<Node> ) -> Result<ArenaRc<Node>,ArenaRc<Node>> );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );  

        let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
        let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
        let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3)));
        let l1 = memory.alloc_rc(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&i1),ArenaRc::clone(&i2),ArenaRc::clone(&i3)])))));
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( ArenaRc::clone(&l1), &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id1), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast72".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new(ArenaRc::clone(&id2))));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("f".to_string())));
        let exp_val = walk( ArenaRc::clone(&func1), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id3), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory ); 

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory ); 

        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("f".to_string())));
        let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id4),ArenaRc::clone(&id5))));

        let exp_val = walk( ArenaRc::clone(&apply1), &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id6), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

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
        fn _ast72<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>> {

            
            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, state, memory );

            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
            let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
            let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("tail".to_string())));
            let rht1 = memory.alloc_rc(Node::AstroRawHeadTail(AstroRawHeadTail::new(ArenaRc::clone(&id2),ArenaRc::clone(&id3))));
            let ht1 = memory.alloc_rc(Node::AstroHeadTail(AstroHeadTail::new(ArenaRc::clone(&id1),ArenaRc::clone(&rht1))));


            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&ht1), state, memory, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state, memory),
                };

                let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
                let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
                let tup1 = memory.alloc_rc(Node::AstroPair( AstroPair::new( ArenaRc::clone(&id4),ArenaRc::clone(&id5) )));
                let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("__plus__".to_string())));
                let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id6),ArenaRc::clone(&tup1))));

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
                set_lineinfo(  new_lineinfo, state, memory );

                let exp_val = walk( ArenaRc::clone(&apply1), state, memory );

                let exp_val = match exp_val {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state, memory),
                };

                state.pop_scope();

                return Ok( exp_val )
            } else {
                //return Err(Error::ValueError(memory.alloc_rc(Node::AstroString(AstroString::new(format!("none of the function bodies unified with actual parameters"))))));
                return  new_exception("ValueError".to_string(), "none of the function bodies unified with actual parameters".to_string(), state, memory );
            }
        }

        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );  

        let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
        let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
        let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3)));
        let l1 = memory.alloc_rc(Node::AstroList(AstroList::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&i1),ArenaRc::clone(&i2),ArenaRc::clone(&i3)])))));
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

        let exp_val = walk( ArenaRc::clone(&l1), &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id1), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast72".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new(ArenaRc::clone(&id2))));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("f".to_string())));
        let exp_val = walk( ArenaRc::clone(&func1), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id3), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory ); 

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory ); 

        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("f".to_string())));
        let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("z".to_string())));
        let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id4),ArenaRc::clone(&id5))));

        let exp_val = walk( ArenaRc::clone(&apply1), &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id6), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

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
        

     
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        // # structure def for A
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("a".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("b".to_string())));
        let d1 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id1))));
        let d2 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id2))));
        let member_list = vec![ ArenaRc::clone(&d1), ArenaRc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( ArenaRc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( memory.alloc_rc(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = memory.alloc_rc(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", ArenaRc::clone(&struct_type)  );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("A".to_string())));
        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("obj".to_string())));
        let t1 = memory.alloc_rc(  Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&i1),ArenaRc::clone(&i2)])))));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id3), ArenaRc::clone(&t1) )));
        let exp_val = walk( ArenaRc::clone(&apply1), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id4), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);  

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i3 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(4)));
        let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("obj".to_string())));
        let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("b".to_string())));
        let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id5), ArenaRc::clone(&id6))));

        let exp_val = match walk( ArenaRc::clone(&i3), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&idx1), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);  

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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        // # structure def for A
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("a".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("b".to_string())));
        let d1 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id1))));
        let d2 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id2))));
        let member_list = vec![ ArenaRc::clone(&d1), ArenaRc::clone(&d2) ];
        let mut struct_memory: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( ArenaRc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( memory.alloc_rc(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "unify" {
                ;
            } else if _type == "noop" {
                ;
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = memory.alloc_rc(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "A", ArenaRc::clone(&struct_type)  );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:7}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
        let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(2)));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("A".to_string())));
        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("obj".to_string())));
        let t1 = memory.alloc_rc(  Node::AstroTuple( AstroTuple::new( Rc::new(RefCell::new(vec![ArenaRc::clone(&i1),ArenaRc::clone(&i2)])))));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id3), ArenaRc::clone(&t1) )));
        let exp_val = walk( ArenaRc::clone(&apply1), &mut state, &mut memory);
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id4), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);  

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i3 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(4)));
        let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("obj".to_string())));
        let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("b".to_string())));
        let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("z".to_string())));
        let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id5), ArenaRc::clone(&id6))));

        let exp_val = match walk( ArenaRc::clone(&idx1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id7), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);  
     
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
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        // # structure def for A
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let s1 = memory.alloc_rc(Node::AstroString(AstroString::new("abcdefg".to_string())));

        let exp_val = match walk( ArenaRc::clone(&s1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id1), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory); 

        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
        let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id2), ArenaRc::clone(&i1) )));

        let exp_val = match walk( ArenaRc::clone(&idx1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id3), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory); 

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
        
        // return memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2*val)));
        // "
        // end
        // let y = times_two( 15 ).


        // Python
        // def _ast72():
            // let Node::AstroInteger(AstroInteger{value:val}) = *state.lookup_sym( "x" )
            //   else {return Error::ValueError("times_two() expected a single integer."))};
            // return Ok(memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2*val))));
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

        fn _ast72<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>> {
            let Node::AstroInteger(AstroInteger{value:val}) = *state.lookup_sym( "x", true )
              else {return  new_exception("ValueError".to_string(), "times_two() expected a single integer.".to_string(), state, memory )};
            return Ok(memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2*val))));
        }
        fn _ast73<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>> {

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
            set_lineinfo(  new_lineinfo, state, memory );

            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&id1), state, memory, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state, memory),
                };

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
                set_lineinfo(  new_lineinfo, state, memory );

                let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast72".to_string())));
                let esc1 = memory.alloc_rc(Node::AstroEscape(AstroEscape::new( "_ast72".to_string() )));

                let exp_val = match walk( ArenaRc::clone(&esc1), state, memory) {
                    Ok( val ) => val,
                    Err( e ) => exit(e, state, memory),
                };

                state.push_scope();

                return Ok( exp_val )
            } else {
                return  new_exception("ValueError".to_string(), "none of the function bodies unified with actual parameters".to_string(), state, memory );
            }
            
        }
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast72") , _ast72 );
        state.dispatch_table.insert( String::from("_ast73") , _ast73 );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast73".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("times_two".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new( ArenaRc::clone(&id1) )));
        let exp_val = walk( ArenaRc::clone(&func1), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id2), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:9}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("times_two".to_string())));
        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(15)));
        let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new( ArenaRc::clone(&id3), ArenaRc::clone(&i1) )));

        let exp_val = match  walk( ArenaRc::clone(&apply1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id4), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

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
        fn _ast72<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>> {

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
            set_lineinfo(  new_lineinfo, state, memory );

            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("a".to_string())));
            let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("b".to_string())));
            let t1 = memory.alloc_rc(  Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&id1),ArenaRc::clone(&id2)])))));

            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&t1), state, memory, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok(_) => (),
                    Err( e ) => exit(e, state, memory),
                };

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
                set_lineinfo(  new_lineinfo, state, memory );

                let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("a".to_string())));
                let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("b".to_string())));
                let t2 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&id3),ArenaRc::clone(&id4))));
                let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("__times__".to_string())));
                let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id5),ArenaRc::clone(&t2))));

                let val = walk(ArenaRc::clone(&apply1),state,memory);

                state.pop_scope();

                return val;
            } else {
                //return Err(Error::PatternMatchFailed(memory.alloc_rc(Node::AstroString(AstroString::new("None of the function bodies unified with actual parameters.".to_string());
                return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory  );
            }
        }
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast72".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("reduce".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new( ArenaRc::clone(&id1) )));

        let exp_val = match walk( ArenaRc::clone(&func1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id2), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("reduce".to_string()))); 
        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("r".to_string()))); 
        let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
        let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(4)));
        let t1 = memory.alloc_rc(  Node::AstroTuple( AstroTuple::new(Rc::new(RefCell::new(vec![ArenaRc::clone(&i1),ArenaRc::clone(&i2)])))));
        let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id3),ArenaRc::clone(&t1))));

        let exp_val = match walk( ArenaRc::clone(&apply1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id4), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(8)));
        let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("r".to_string()))); 
        let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("__eq__".to_string()))); 
        let t2 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&i3),ArenaRc::clone(&id5))));
        let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id6),ArenaRc::clone(&t2))));
        let exp_val = match walk( ArenaRc::clone(&apply1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
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
        fn _ast72<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
            set_lineinfo(  new_lineinfo, state, memory );
        
            let id1 = AstroID::new("n".to_string());
            let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("pos_int".to_string())));
            let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("neg_int".to_string())));  
            let deref1 = memory.alloc_rc(Node::AstroDeref(AstroDeref::new(ArenaRc::clone(&id2))));
            let deref2 = memory.alloc_rc(Node::AstroDeref(AstroDeref::new(ArenaRc::clone(&id3))));
            let namedp1 = memory.alloc_rc(Node::AstroNamedPattern(AstroNamedPattern::new(id1,ArenaRc::clone(&deref1))));

            let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(0)));
        
            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&i1), state, memory, true ) {

                state.push_scope();

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
                set_lineinfo(  new_lineinfo, state, memory );

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let i2 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
                let result = walk( ArenaRc::clone(&i2), state, memory );
                state.pop_scope();
                result

            } else if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&namedp1), state, memory, true ) {

                state.push_scope();

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
                set_lineinfo(  new_lineinfo, state, memory );

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("__times__".to_string())));
                let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("n".to_string())));
                let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("n".to_string())));
                let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("__minus__".to_string())));
                let id8 = memory.alloc_rc(Node::AstroID(AstroID::new("fact".to_string())));
                let i3 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(1)));
                let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new( ArenaRc::clone(&id5),ArenaRc::clone(&i3))));
                let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id7),ArenaRc::clone(&t1))));
                let apply2 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id8),ArenaRc::clone(&apply1))));
                let t2 = memory.alloc_rc(  Node::AstroPair( AstroPair::new( ArenaRc::clone(&id6),ArenaRc::clone(&apply2))));
                let apply3 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id4),ArenaRc::clone(&t2))));

                let result = walk( ArenaRc::clone(&apply3), state, memory );
                state.pop_scope();
                result

            } else if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&deref2), state, memory, true ) {
                state.push_scope();

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
                set_lineinfo(  new_lineinfo, state, memory );

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                state.pop_scope();
                 new_exception("ValueError".to_string(), "fact undefined for negative values".to_string(), state, memory  )

            } else {
                return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory  );
            }
        }
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast72") , _ast72 );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("__gt__".to_string())));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let var3 = memory.alloc_rc(Node::AstroID(AstroID::new("pos_int".to_string())));
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(0)));
        let null1 = memory.alloc_rc(  Node::AstroNone( AstroNone::new()));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&var2),ArenaRc::clone(&i1))));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&var1), ArenaRc::clone(&t1) )));
        let if1 = memory.alloc_rc(  Node::AstroIf( AstroIf::new( ArenaRc::clone(&apply1), ArenaRc::clone(&var2), ArenaRc::clone(&null1))));
        let quote1 = memory.alloc_rc(  Node::AstroQuote( AstroQuote::new( ArenaRc::clone( &if1))));
        let exp_val = walk( quote1, &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, var3, &mut state, &mut memory, true);
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let var1 = memory.alloc_rc(Node::AstroID(AstroID::new("__lt__".to_string())));
        let var2 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let var3 = memory.alloc_rc(Node::AstroID(AstroID::new("neg_int".to_string())));
        let i1 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(0)));
        let null1 = memory.alloc_rc(  Node::AstroNone( AstroNone::new()));
        let t1 = memory.alloc_rc(  Node::AstroPair( AstroPair::new(ArenaRc::clone(&var2),ArenaRc::clone(&i1))));
        let apply1 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&var1), ArenaRc::clone(&t1) )));
        let if1 = memory.alloc_rc(  Node::AstroIf( AstroIf::new( ArenaRc::clone(&apply1), ArenaRc::clone(&var2), ArenaRc::clone(&null1))));
        let quote1 = memory.alloc_rc(  Node::AstroQuote( AstroQuote::new( ArenaRc::clone( &if1))));
        let exp_val = walk( quote1, &mut state, &mut memory );

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, var3, &mut state, &mut memory, true);
        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);
        
        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let id8 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast72".to_string())));
        let id9 = memory.alloc_rc(Node::AstroID(AstroID::new("fact".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new( ArenaRc::clone(&id8) )));
        let exp_val = walk( ArenaRc::clone(&func1), &mut state, &mut memory);

        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = unify( exp_val, ArenaRc::clone(&id9), &mut state, &mut memory, true);

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:13}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        let i5 = memory.alloc_rc(  Node::AstroInteger( AstroInteger::new(10)));
        let id10 = memory.alloc_rc(Node::AstroID(AstroID::new("fact".to_string())));
        let id11 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let apply2 = memory.alloc_rc(  Node::AstroApply( AstroApply::new( ArenaRc::clone(&id10), ArenaRc::clone(&i5) )));
        let exp_val = walk( ArenaRc::clone(&apply2), &mut state, &mut memory);
 
        let exp_val = match exp_val {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
 
        let unifiers = unify( exp_val, ArenaRc::clone(&id11), &mut state, &mut memory, true);
 

        let unifiers = match unifiers {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let check1 = state.lookup_sym("x",true);
        match *check1 {
            Node::AstroInteger(AstroInteger{value:3628800}) => (),
            _ => panic!("test failed"),
        };

    }
    //#[test]
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
        fn _try1_catch1<'a>( state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
            set_lineinfo(  new_lineinfo, state, memory );

            let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3)));
            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

            let exp_val = match walk( ArenaRc::clone(&i1), state, memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };

            let unifiers = match unify( exp_val, ArenaRc::clone(&id1), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
    
            declare_unifiers( &unifiers, state, memory);

            return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));

        }
        fn _try1_catch2<'a>( state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
            set_lineinfo(  new_lineinfo, state, memory );

            let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(4)));
            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

            let exp_val = match walk( ArenaRc::clone(&i1), state, memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };

            let unifiers = match unify( exp_val, ArenaRc::clone(&id1), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
    
            declare_unifiers( &unifiers, state, memory);

            return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));

        };
        fn _try1_catch3<'a>( state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:10}));
            set_lineinfo(  new_lineinfo, state, memory );

            let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(5)));
            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

            let exp_val = match walk( ArenaRc::clone(&i1), state, memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };

            let unifiers = match unify( exp_val, ArenaRc::clone(&id1), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
    
            declare_unifiers( &unifiers, state, memory);

            return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));


        };
        fn _try1_catch4<'a>( state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:12}));
            set_lineinfo(  new_lineinfo, state, memory );

            let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(6)));
            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

            let exp_val = match walk( ArenaRc::clone(&i1), state, memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };

            let unifiers = match unify( exp_val, ArenaRc::clone(&id1), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
    
            declare_unifiers( &unifiers, state, memory);

            return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));

        };
        fn _try1_catch5<'a>( state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:14}));
            set_lineinfo(  new_lineinfo, state, memory );

            let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(7)));
            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

            let exp_val = match walk( ArenaRc::clone(&i1), state, memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };

            let unifiers = match unify( exp_val, ArenaRc::clone(&id1), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
    
            declare_unifiers( &unifiers, state, memory);

            return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));

        };
        fn _try1_catch6<'a>( state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:16}));
            set_lineinfo(  new_lineinfo, state, memory );

            let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(8)));
            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

            let exp_val = match walk( ArenaRc::clone(&i1), state, memory) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };

            let unifiers = match unify( exp_val, ArenaRc::clone(&id1), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return Err(e),
            };
    
            declare_unifiers( &unifiers, state, memory);

            return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));
        };
        fn _try1_catch<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node> >{

            let node = match walk(node,state,memory) {
                Ok(val) => val,
                Err(e) => return Err(e),
            };

            let Node::AstroObject(AstroObject{ struct_id:ref sid, object_memory:ref o }) = *node else {panic!("uh oh")};


            let mut catches = vec![];

            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("s".to_string())));
            let id2 = AstroID::new("Exception".to_string());
            let s1 = memory.alloc_rc(Node::AstroString(AstroString::new( "ValueError".to_string() )));
            let o1 = memory.alloc_rc(  Node::AstroObject(AstroObject::new( id2, Rc::new(RefCell::new(vec![ ArenaRc::clone(&s1), ArenaRc::clone(&id1) ])) )) );
            let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("s".to_string())));
            let id4 = AstroID::new("Exception".to_string());
            let s2 = memory.alloc_rc(Node::AstroString(AstroString::new( "FileNotFound".to_string() )));
            let o2 = memory.alloc_rc(  Node::AstroObject(AstroObject::new( id4, Rc::new(RefCell::new(vec![ ArenaRc::clone(&s2), ArenaRc::clone(&id3) ])) )) );
            let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("s".to_string())));
            let id6 = AstroID::new("Exception".to_string());
            let s3 = memory.alloc_rc(Node::AstroString(AstroString::new( "ArithmeticError".to_string() )));
            let o3 = memory.alloc_rc(  Node::AstroObject(AstroObject::new( id6, Rc::new(RefCell::new(vec![ ArenaRc::clone(&s3), ArenaRc::clone(&id5) ])) )) );
            let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("s".to_string())));
            let id8 = AstroID::new("Exception".to_string());
            let s4 = memory.alloc_rc(Node::AstroString(AstroString::new( "PatternMatchFailed".to_string() )));
            let o4 = memory.alloc_rc(  Node::AstroObject(AstroObject::new( id8, Rc::new(RefCell::new(vec![ ArenaRc::clone(&s4), ArenaRc::clone(&id7) ])) )) );
            let id9 = memory.alloc_rc(Node::AstroID(AstroID::new("s".to_string())));
            let id10 = AstroID::new("Exception".to_string());
            let s5 = memory.alloc_rc(Node::AstroString(AstroString::new( "RedundantPatternFound".to_string() )));
            let o5 = memory.alloc_rc(  Node::AstroObject(AstroObject::new( id10, Rc::new(RefCell::new(vec![ ArenaRc::clone(&s5), ArenaRc::clone(&id9) ])) )) );
            let id11 = memory.alloc_rc(Node::AstroID(AstroID::new("s".to_string())));
            let id12 = AstroID::new("Exception".to_string());
            let s6 = memory.alloc_rc(Node::AstroString(AstroString::new( "NonLinearPattern".to_string() )));
            let o6 = memory.alloc_rc(  Node::AstroObject(AstroObject::new( id12, Rc::new(RefCell::new(vec![ ArenaRc::clone(&s6), ArenaRc::clone(&id11) ])) )) );

            catches.push( (ArenaRc::clone(&o1), _try1_catch1 as fn( &'a mut State, &'a mut Arena<Node>) ->  Result< ArenaRc<Node>, ArenaRc<Node>>));
            catches.push( (ArenaRc::clone(&o2), _try1_catch2 as fn( &'a mut State, &'a mut Arena<Node>) ->  Result< ArenaRc<Node>, ArenaRc<Node>>));
            catches.push( (ArenaRc::clone(&o3), _try1_catch3 as fn( &'a mut State, &'a mut Arena<Node>) ->  Result< ArenaRc<Node>, ArenaRc<Node>>));
            catches.push( (ArenaRc::clone(&o4), _try1_catch4 as fn( &'a mut State, &'a mut Arena<Node>) ->  Result< ArenaRc<Node>, ArenaRc<Node>>));
            catches.push( (ArenaRc::clone(&o5), _try1_catch5 as fn( &'a mut State, &'a mut Arena<Node>) ->  Result< ArenaRc<Node>, ArenaRc<Node>>));
            catches.push( (ArenaRc::clone(&o6), _try1_catch6 as fn( &'a mut State, &'a mut Arena<Node>) ->  Result< ArenaRc<Node>, ArenaRc<Node>>));

            for (ptrn,f) in catches {

                if let Ok(unifiers) = unify( ArenaRc::clone(&node), ArenaRc::clone(&ptrn), state, memory, true ) {

                    let out1 = declare_formal_args( &unifiers, state, memory );
                    match out1 {
                        Ok(_) => (),
                        Err( e ) => return Err(e),
                    };

                    return f(state,memory);
                }
            }

            Err(node.clone()) // if it doesnt match any catch patterns, pass it back
        }
        fn _try1<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
            set_lineinfo(  new_lineinfo, state, memory );

            let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
            let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));

            // check for exceptions/errors
            let exp_val = match walk( ArenaRc::clone(&i1), state, memory) {
                Ok( val ) => val,
                Err( e ) => return _try1_catch( e, state, memory ),
            };

            // check for exceptions/errors
            let unifiers = match unify( exp_val, ArenaRc::clone(&id1), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return _try1_catch( e, state, memory ),
            };

            declare_unifiers( &unifiers, state, memory);

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
            set_lineinfo(  new_lineinfo, state, memory );

            let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
            let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(0)));
            let t1 = memory.alloc_rc(Node::AstroPair( AstroPair::new(ArenaRc::clone(&i2),ArenaRc::clone(&i3))));
            let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
            let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("__divide__".to_string())));
            let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new( ArenaRc::clone(&id3), ArenaRc::clone(&t1) )));

            // check for exceptions/errors
            let exp_val = match walk( ArenaRc::clone(&apply1), state, memory) {
                Ok( val ) => val,
                Err( e ) => return _try1_catch( e, state, memory ),
            };

            // check for exceptions/errors
            let unifiers = match unify( exp_val, ArenaRc::clone(&id2), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return _try1_catch( e, state, memory ),
            };

            declare_unifiers( &unifiers, state, memory);

            let i4 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
            let i5 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
            let t2 = memory.alloc_rc(Node::AstroPair( AstroPair::new(ArenaRc::clone(&i4),ArenaRc::clone(&i5))));
            let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("z".to_string())));
            let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("__divide__".to_string())));
            let apply2 = memory.alloc_rc(Node::AstroApply(AstroApply::new( ArenaRc::clone(&id3), ArenaRc::clone(&t1) )));

            // check for exceptions/errors
            let exp_val = match walk( ArenaRc::clone(&apply2), state, memory) {
                Ok( val ) => val,
                Err( e ) => return _try1_catch( e, state, memory ),
            };

            // check for exceptions/errors
            let unifiers = match unify( exp_val, ArenaRc::clone(&id2), state, memory, true) {
                Ok( val ) => val,
                Err( e ) => return _try1_catch( e, state, memory ),
            };
    
            declare_unifiers( &unifiers, state, memory);

            return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));
        }
        fn _ast1<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prologue.ast".to_string(),line_number:1}));
            set_lineinfo(  new_lineinfo, state, memory); 

            let id1 = AstroID::new("kind".to_string());
            let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
            let s1 = memory.alloc_rc(Node::AstroString(AstroString::new("string".to_string())));
            let tm1 = memory.alloc_rc(Node::AstroTypeMatch(AstroTypeMatch::new(ArenaRc::clone( &s1 ))));
            let np1 = memory.alloc_rc(Node::AstroNamedPattern(AstroNamedPattern::new( id1, ArenaRc::clone(&tm1))));

            let t1 = memory.alloc_rc(Node::AstroTuple(AstroTuple::new(Rc::new(RefCell::new(vec![ ArenaRc::clone(&np1), ArenaRc::clone(&id2) ])))));

            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&t1), state, memory, true ) {

                state.push_scope();

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prologue.ast".to_string(),line_number:2}));
                set_lineinfo(  new_lineinfo, state, memory); 

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("kind".to_string())));
                let exp_val = match walk( ArenaRc::clone(&id3), state, memory) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("this".to_string())));
                let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("kind".to_string())));
                let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id4), ArenaRc::clone(&id5))));
                let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&idx1), state, memory, true) {
                    Ok( val) => val,
                    Err( e ) => return Err(e),
                };

                declare_unifiers( &unifiers, state, memory);

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prologue.ast".to_string(),line_number:3}));
                set_lineinfo(  new_lineinfo, state, memory); 

                let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
                let exp_val = match walk( ArenaRc::clone(&id6), state, memory) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("this".to_string())));
                let id8 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
                let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id7), ArenaRc::clone(&id8))));
                let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&idx1), state, memory, true) {
                    Ok( val) => val,
                    Err( e ) => return Err(e),
                };

                declare_unifiers( &unifiers, state, memory);

                state.pop_scope();

                return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));

            } else {
                return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory  );
            }

        }
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast1") , _ast1 );

        // strucutre def for exception
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("kind".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("__init__".to_string())));
        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast1".to_string())));
        let f1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new( ArenaRc::clone(&id4))));
        let d1 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id1))));
        let d2 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id2))));
        let u1 = memory.alloc_rc(Node::AstroUnify(AstroUnify::new(ArenaRc::clone(&id3),ArenaRc::clone(&f1))));
        //let member_list = memory.alloc_rc(Node::AstroList(AstroList::new( Rc::new(RefCell::new(vec![ ArenaRc::clone(&d1), ArenaRc::clone(&d2), ArenaRc::clone(&u1) ])))));
        let member_list = vec![ ArenaRc::clone(&d1), ArenaRc::clone(&d2), ArenaRc::clone(&u1) ];
        let mut struct_memory: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( ArenaRc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( memory.alloc_rc(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "unify" {
                let Node::AstroUnify(AstroUnify{term:ref id_node,pattern:ref function_exp}) = *member
                    else {panic!("ERROR: object construction: expection unify node.")};
                let function_val = match walk( ArenaRc::clone(&function_exp), &mut state, &mut memory) {
                    Ok( val ) => val,
                    Err ( e ) => panic!("error!"),
                };
                struct_memory.borrow_mut().push( ArenaRc::clone( &function_val ));
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "noop" {
                ;// pass
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = memory.alloc_rc(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "Exception", ArenaRc::clone(&struct_type)  );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prologue.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory ); 

        // let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("kind".to_string())));
        // let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
        // let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("__init__".to_string())));
        // let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast1".to_string())));

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut state, &mut memory );

        match _try1( memory.alloc_rc(Node::AstroNone(AstroNone::new())), &mut state, &mut memory) {
            Ok(_) => (),
            Err(e) => exit( e, &mut state, &mut memory),
        };

        let check1 = state.lookup_sym("x",true);

        let Node::AstroInteger(AstroInteger{value:v}) = *check1
            else {panic!("test failed")}; 
        assert_eq!(5,v);
    }
    #[test]
    fn test_prog_fib() {
        // Asteroid
        // function fib
        //     with (x:%integer) if x <= 1 do
        //         return x.
        //     with (x:%integer) do
        //         return fib( x - 1 ) + fib( x - 2 ).
        //     end

        // let y = fib( 10 ).

        // Python

        // def _ast72(arg):
        // set_lineinfo('prog.txt',2)
        // try:
        //    unifiers = unify(arg,('if-exp', ('apply', ('id', '__le__'), ('tuple', [('id', 'x'), ('integer', 1)])), ('named-pattern', ('id', 'x'), ('typematch', 'integer')), ('null',)))
        //    state.symbol_table.push_scope({})
        //    declare_formal_args(unifiers)
        //    set_lineinfo('prog.txt',3)
        //    val = walk(('id', 'x'))
        //    state.symbol_table.pop_scope()
        //    return val
  
        //    state.symbol_table.pop_scope()
        // except PatternMatchFailed:
        //    set_lineinfo('prog.txt',4)
        //    try:
        //       unifiers = unify(arg,('named-pattern', ('id', 'x'), ('typematch', 'integer')))
        //       state.symbol_table.push_scope({})
        //       declare_formal_args(unifiers)
        //       set_lineinfo('prog.txt',5)
        //       val = walk(('apply', ('id', '__plus__'), ('tuple', [('apply', ('id', 'fib'), ('apply', ('id', '__minus__'), ('tuple', [('id', 'x'), ('integer', 1)]))), ('apply', ('id', 'fib'), ('apply', ('id', '__minus__'), ('tuple', [('id', 'x'), ('integer', 2)])))])))
        //       state.symbol_table.pop_scope()
        //       return val
  
        //       state.symbol_table.pop_scope()
        //    except PatternMatchFailed:
        //       raise ValueError('none of the function bodies unified with actual parameters')
  
        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'fib'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',8)
        // exp_val = walk(('apply', ('id', 'fib'), ('integer', 10)))
        // unifiers = unify(exp_val,('id', 'y'))
        // declare_unifiers(unifiers)
     

        // Rust
        
        // fib()
        fn _ast1<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
            set_lineinfo(  new_lineinfo, state, memory); 

            let Node::AstroInteger(AstroInteger{value}) = *node else {panic!("uh oh")};


            // first clause pattern
            let id1 = AstroID::new("x".to_string());
            let s1 = memory.alloc_rc(Node::AstroString(AstroString::new("integer".to_string())));
            let tm1 = memory.alloc_rc(Node::AstroTypeMatch(AstroTypeMatch::new(ArenaRc::clone(&s1))));
            let np1 = memory.alloc_rc(Node::AstroNamedPattern(AstroNamedPattern::new( id1, ArenaRc::clone(&tm1))));
            let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
            let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
            let t1 = memory.alloc_rc(Node::AstroPair( AstroPair::new( ArenaRc::clone(&id3), ArenaRc::clone(&i1) )));
            let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("__le__".to_string())));
            let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id4),ArenaRc::clone(&t1))));
            let if1 = memory.alloc_rc(Node::AstroIf(AstroIf::new( ArenaRc::clone(&apply1), ArenaRc::clone(&np1), memory.alloc_rc(Node::AstroNone(AstroNone::new())))));

            //second clause pattern
            let id5 = AstroID::new("x".to_string());
            let s2 = memory.alloc_rc(Node::AstroString(AstroString::new("integer".to_string())));
            let tm2 = memory.alloc_rc(Node::AstroTypeMatch(AstroTypeMatch::new(ArenaRc::clone(&s2))));
            let np2 = memory.alloc_rc(Node::AstroNamedPattern(AstroNamedPattern::new( id5, ArenaRc::clone(&tm2))));

            let Node::AstroInteger(AstroInteger{value}) = *node else {panic!("uh oh")};
 
            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&if1), state, memory, true ) {
                
                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
                set_lineinfo(  new_lineinfo, state, memory); 

                let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
                let val = walk( ArenaRc::clone(&id7),state,memory);

                let x = match val {
                    Ok(ref  val ) => val,
                    Err(ref e) => e,
                };
                state.pop_scope();
                return val;

            } else if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&np2), state, memory, true) {

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
                set_lineinfo(  new_lineinfo, state, memory); 

                let id8 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
                let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
                let t2 = memory.alloc_rc(Node::AstroPair( AstroPair::new( ArenaRc::clone(&id8), ArenaRc::clone(&i2) )));
                let id9 = memory.alloc_rc(Node::AstroID(AstroID::new("__minus__".to_string())));
                let apply2 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id9),ArenaRc::clone(&t2))));
                let id10 = memory.alloc_rc(Node::AstroID(AstroID::new("fib".to_string())));
                let apply3 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id10),ArenaRc::clone(&apply2))));

                let id11 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
                let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(2)));
                let t3 = memory.alloc_rc(Node::AstroPair( AstroPair::new( ArenaRc::clone(&id11), ArenaRc::clone(&i3) )));
                let id12 = memory.alloc_rc(Node::AstroID(AstroID::new("__minus__".to_string())));
                let apply4 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id12),ArenaRc::clone(&t3))));
                let id13 = memory.alloc_rc(Node::AstroID(AstroID::new("fib".to_string())));
                let apply5 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id10),ArenaRc::clone(&apply4))));

                let t3 = memory.alloc_rc(Node::AstroPair( AstroPair::new( ArenaRc::clone(&apply3), ArenaRc::clone(&apply5) )));
                let id14 = memory.alloc_rc(Node::AstroID(AstroID::new("__plus__".to_string())));
                let apply6 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id14),ArenaRc::clone(&t3))));

                let val = walk( ArenaRc::clone(&apply6),state,memory);
                state.pop_scope();

                return val;
            } else {
                return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory  );
            }
        }
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast1") , _ast1 );


        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut  state, &mut memory); 

        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast1".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("fib".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new(ArenaRc::clone(&id1))));
        let exp_val = match walk( ArenaRc::clone(&func1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id2), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
        set_lineinfo(  new_lineinfo, &mut  state, &mut memory); 

        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("fib".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(10)));
        let apply4 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id1),ArenaRc::clone(&i3))));

        let exp_val = match walk( ArenaRc::clone(&apply4), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
        
        let unifiers = match unify( exp_val, ArenaRc::clone(&id2), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let check1 = state.lookup_sym("y",true);

        let Node::AstroInteger(AstroInteger{value:v}) = *check1
            else {panic!("test failed")}; 
        assert_eq!(55,v);
    }
    #[test]
    fn test_prog_iterative_bubblesort() {
        //Asteroid
        // function bubblesort
        //     with l:%list do
        //     for i in 1 to (l@length()-1) do
        //         for j in 0 to (l@length()-i-1) do
        //         if l@j > l@(j+1) do
        //             let temp = l@(j+1).
        //             let l@(j+1) = l@j.
        //             let l@j = temp.
        //         end
        //         end
        //     end
        //     return l.
        // end.
      
        // let x = [2,8,3,7,4,6].
        // bubblesort(x).


        //Python
        // def _ast72(arg):
        // set_lineinfo('prog.txt',2)
        // try:
        //    unifiers = unify(arg,('named-pattern', ('id', 'l'), ('typematch', 'list')))
        //    state.symbol_table.push_scope({})
        //    declare_formal_args(unifiers)
        //    set_lineinfo('prog.txt',3)
        //    (LIST_TYPE, list_val) = walk(('raw-to-list', ('start', ('integer', 1)), ('stop', ('apply', ('id', '__minus__'), ('tuple', [('apply', ('index', ('id', 'l'), ('id', 'length')), ('none', None)), ('integer', 1)]))), ('stride', ('integer', '1'))))
        //    if LIST_TYPE not in ['list','string']:
        //        raise ValueError('only iteration over strings and lists is supported')
        //    if LIST_TYPE == 'string':
        //        new_list = []
        //        for c in list_val:
        //            new_list.append(('string',c))
        //        list_val = new_list
        //    for term in list_val:
        //       try:
        //          unifiers = unify(term,('id', 'i'))
        //       except PatternMatchFailed:
        //          pass
        //       else:
        //          declare_unifiers(unifiers)
        //          set_lineinfo('prog.txt',4)
        //          (LIST_TYPE, list_val) = walk(('raw-to-list', ('start', ('integer', 0)), ('stop', ('apply', ('id', '__minus__'), ('tuple', [('apply', ('id', '__minus__'), ('tuple', [('apply', ('index', ('id', 'l'), ('id', 'length')), ('none', None)), ('id', 'i')])), ('integer', 1)]))), ('stride', ('integer', '1'))))
        //          if LIST_TYPE not in ['list','string']:
        //              raise ValueError('only iteration over strings and lists is supported')
        //          if LIST_TYPE == 'string':
        //              new_list = []
        //              for c in list_val:
        //                  new_list.append(('string',c))
        //              list_val = new_list
        //          for term in list_val:
        //             try:
        //                unifiers = unify(term,('id', 'j'))
        //             except PatternMatchFailed:
        //                pass
        //             else:
        //                declare_unifiers(unifiers)
        //                set_lineinfo('prog.txt',5)
        //                if map2boolean(walk(('apply', ('id', '__gt__'), ('tuple', [('index', ('id', 'l'), ('id', 'j')), ('index', ('id', 'l'), ('apply', ('id', '__plus__'), ('tuple', [('id', 'j'), ('integer', 1)])))]))))[1]:
        //                   set_lineinfo('prog.txt',6)
        //                   exp_val = walk(('index', ('id', 'l'), ('apply', ('id', '__plus__'), ('tuple', [('id', 'j'), ('integer', 1)]))))
        //                   unifiers = unify(exp_val,('id', 'temp'))
        //                   declare_unifiers(unifiers)
  
        //                   set_lineinfo('prog.txt',7)
        //                   exp_val = walk(('index', ('id', 'l'), ('id', 'j')))
        //                   unifiers = unify(exp_val,('index', ('id', 'l'), ('apply', ('id', '__plus__'), ('tuple', [('id', 'j'), ('integer', 1)]))))
        //                   declare_unifiers(unifiers)
  
        //                   set_lineinfo('prog.txt',8)
        //                   exp_val = walk(('id', 'temp'))
        //                   unifiers = unify(exp_val,('index', ('id', 'l'), ('id', 'j')))
        //                   declare_unifiers(unifiers)
  
        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'bubblesort'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',13)
        // set_lineinfo('prog.txt',15)
        // exp_val = walk(('list', [('integer', 2), ('integer', 8), ('integer', 3), ('integer', 7), ('integer', 4), ('integer', 6)]))
        // unifiers = unify(exp_val,('id', 'x'))
        // declare_unifiers(unifiers)

        //Rust

        // list length function
        fn _ast1<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>>{
            let Node::AstroList(AstroList{contents:ref content}) = *node
                else {return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory ) };
            
            return Ok ( memory.alloc_rc(Node::AstroInteger(AstroInteger::new( content.borrow().len() as isize ))));
        }
        // iterative bubble sort function.
        fn _ast2<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>, ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:2}));
            set_lineinfo(  new_lineinfo, state, memory); 

            let id1 = AstroID::new("l".to_string());
            let s1 = memory.alloc_rc(Node::AstroString(AstroString::new("list".to_string())));
            let tm1 = memory.alloc_rc(Node::AstroTypeMatch(AstroTypeMatch::new(ArenaRc::clone(&s1))));
            let np1 = memory.alloc_rc(Node::AstroNamedPattern(AstroNamedPattern::new(id1,ArenaRc::clone(&tm1))));

            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&np1), state, memory, true ) {

                state.push_scope();

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:3}));
                set_lineinfo(  new_lineinfo, state, memory); 

                //walk(('raw-to-list', ('start', ('integer', 1)), ('stop', ('apply', ('id', '__minus__'), ('tuple', [('apply', ('index', ('id', 'l'), ('id', 'length')), ('none', None)), ('integer', 1)]))), ('stride', ('integer', '1'))))
                let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
                let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
                let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
                let i4 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(0)));
                let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));
                let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("i".to_string())));
                let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("__minus__".to_string())));
                let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("length".to_string())));
                let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id2),ArenaRc::clone(&id6))));
                let n1 = memory.alloc_rc(Node::AstroNone(AstroNone::new()));
                let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&idx1),ArenaRc::clone(&n1))));
                let t1 = memory.alloc_rc(Node::AstroPair(AstroPair::new(ArenaRc::clone(&apply1), ArenaRc::clone(&i2) )));
                let apply2 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id5),ArenaRc::clone(&t1))));
                let rtl1 = memory.alloc_rc(Node::AstroRawToList(AstroRawToList::new( ArenaRc::clone(&i3), ArenaRc::clone(&apply2), ArenaRc::clone(&i1) )));

                let val = match walk( ArenaRc::clone(&rtl1), state, memory) {
                    Ok( val ) => val,
                    Err(e) => return Err(e),
                };
                let content1 = match *val {
                    Node::AstroList(AstroList{ contents:ref content }) => content,
                    _ => return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory ),
                };

                for term1 in &*content1.borrow() {

                    let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:4}));
                    set_lineinfo(  new_lineinfo, state, memory); 

                    let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("i".to_string())));

                    let unifiers = match unify( ArenaRc::clone(&term1), ArenaRc::clone(&id7), state, memory, true) {
                        Ok( val ) => val,
                        Err( e ) => return Err(e),
                    };

                    declare_unifiers( &unifiers, state, memory);

                    //walk(('raw-to-list', ('start', ('integer', 0)), ('stop', ('apply', ('id', '__minus__'), ('tuple', [('apply', ('id', '__minus__'), ('tuple', [('apply', ('index', ('id', 'l'), ('id', 'length')), ('none', None)), ('id', 'i')])), ('integer', 1)]))), ('stride', ('integer', '1'))))
                    let i5 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
                    let i6 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(0)));
                    let i7 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(0)));
                    let id8 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));
                    let id9 = memory.alloc_rc(Node::AstroID(AstroID::new("i".to_string())));
                    let id10 = memory.alloc_rc(Node::AstroID(AstroID::new("__minus__".to_string())));
                    let id11 = memory.alloc_rc(Node::AstroID(AstroID::new("length".to_string())));
                    let id12 = memory.alloc_rc(Node::AstroID(AstroID::new("__minus__".to_string())));
                    let idx2 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id8),ArenaRc::clone(&id11))));
                    let n2 = memory.alloc_rc(Node::AstroNone(AstroNone::new()));
                    let apply9 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&idx2),ArenaRc::clone(&n2))));
                    let t2 = memory.alloc_rc(Node::AstroPair(AstroPair::new( ArenaRc::clone(&apply9), ArenaRc::clone(&id9) )));
                    let apply3 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id12),ArenaRc::clone(&t2))));
                    let t3 = memory.alloc_rc(Node::AstroPair(AstroPair::new(ArenaRc::clone(&apply3), ArenaRc::clone(&i6) )));
                    let apply4 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id10),ArenaRc::clone(&t3))));
                    let rtl2 = memory.alloc_rc(Node::AstroRawToList(AstroRawToList::new( ArenaRc::clone(&i7), ArenaRc::clone(&apply4), ArenaRc::clone(&i5) )));

                    let val = match walk( ArenaRc::clone(&rtl2), state, memory) {
                        Ok( val ) => val,
                        Err( e ) => return Err(e),
                    };
                    let content2 = match *val {
                        Node::AstroList(AstroList{ contents:ref content }) => content,
                        _ => return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory ),
                    };
    
    
                    for term2 in &*content2.borrow() {

                        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:5}));
                        set_lineinfo(  new_lineinfo, state, memory); 

                        let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("j".to_string())));
    
                        let unifiers = match unify( ArenaRc::clone(&term2), ArenaRc::clone(&id7), state, memory, true) {
                            Ok( val ) => val,
                            Err( e ) => return Err(e),
                        };

                        declare_unifiers( &unifiers, state, memory);

                        let check1 = state.lookup_sym("i",true);
                        let check2 = state.lookup_sym("j",true);
                        let Node::AstroInteger(AstroInteger{value:v1}) = *check1 else {panic!("Uh on!")};
                        let Node::AstroInteger(AstroInteger{value:v2}) = *check2 else {panic!("Uh on!")};
                        // println!("i:{}   j:{}",v1,v2);
                        let check3 = state.lookup_sym("l",true);
                        let Node::AstroList(AstroList{ contents:ref content }) = *check3 else {panic!("Uh on!")};
                        let Node::AstroInteger(AstroInteger{value:v3}) = *content.borrow()[v2 as usize] else {panic!("Uh on!")};
                        let Node::AstroInteger(AstroInteger{value:v4}) = *content.borrow()[v2 as usize + 1] else {panic!("Uh on!")};
                        // println!("Comparing {} > {}",v3,v4);
                        // println!("LENGTH IS {}",content2.borrow().len());

                        //if map2boolean(walk(('apply', ('id', '__gt__'), ('tuple', [('index', ('id', 'l'), ('id', 'j')), **('index', ('id', 'l'), ('apply', ('id', '__plus__'), ('tuple', [('id', 'j'), ('integer', 1)])))]))))[1]:
                        let id12 = memory.alloc_rc(Node::AstroID(AstroID::new("__gt__".to_string())));
                        let id13 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));
                        let id14 = memory.alloc_rc(Node::AstroID(AstroID::new("j".to_string())));
                        let id15 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));
                        let id16 = memory.alloc_rc(Node::AstroID(AstroID::new("__plus__".to_string())));
                        let id17 = memory.alloc_rc(Node::AstroID(AstroID::new("j".to_string())));
                        let idx3 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id13),ArenaRc::clone(&id14))));
                        let i9 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
                        let t5 = memory.alloc_rc(Node::AstroPair(AstroPair::new(ArenaRc::clone(&id17), ArenaRc::clone(&i9) )));
                        let apply6 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id16),ArenaRc::clone(&t5))));
                        let idx4 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id15),ArenaRc::clone(&apply6))));
                        let t4 = memory.alloc_rc(Node::AstroPair(AstroPair::new( ArenaRc::clone(&idx3), ArenaRc::clone(&idx4) )));
                        let apply5 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id12),ArenaRc::clone(&t4))));

                        let cond_val = match walk( ArenaRc::clone(&apply5), state, memory) {
                            Ok( val ) => val,
                            Err(e) => return Err(e),
                        };



                        if let Node::AstroBool(AstroBool{value:true}) = map2boolean( &cond_val  ) {
                            //swap
                            //                   set_lineinfo('prog.txt',6)
                            //                   exp_val = walk(('index', ('id', 'l'), ('apply', ('id', '__plus__'), ('tuple', [('id', 'j'), ('integer', 1)]))))
                            //                   unifiers = unify(exp_val,('id', 'temp'))
                            //                   declare_unifiers(unifiers)
                    
                            //                   set_lineinfo('prog.txt',7)
                            //                   exp_val = walk(('index', ('id', 'l'), ('id', 'j')))
                            //                   unifiers = unify(exp_val,('index', ('id', 'l'), ('apply', ('id', '__plus__'), ('tuple', [('id', 'j'), ('integer', 1)]))))
                            //                   declare_unifiers(unifiers)
                    
                            //                   set_lineinfo('prog.txt',8)
                            //                   exp_val = walk(('id', 'temp'))
                            //                   unifiers = unify(exp_val,('index', ('id', 'l'), ('id', 'j')))
                            //                   declare_unifiers(unifiers)

                            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:6}));
                            set_lineinfo(  new_lineinfo, state, memory); 

                            let id18 = memory.alloc_rc(Node::AstroID(AstroID::new("j".to_string())));
                            let id19 = memory.alloc_rc(Node::AstroID(AstroID::new("__plus__".to_string())));
                            let id20 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));
                            let id21 = memory.alloc_rc(Node::AstroID(AstroID::new("temp".to_string())));
                            let i10 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
                            let t6 = memory.alloc_rc(Node::AstroPair(AstroPair::new( ArenaRc::clone(&id18), ArenaRc::clone(&i10) )));
                            let apply7 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id19),ArenaRc::clone(&t6))));
                            let idx5 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id20),ArenaRc::clone(&apply7))));

                            let exp_val = match walk( ArenaRc::clone(&idx5), state, memory) {
                                Ok( val ) => val,
                                Err( e ) => exit(e, state, memory),
                            };

                            // let Node::AstroInteger(AstroInteger{value:v1}) = *exp_val else {panic!("error uo")};
                    
                            let unifiers = match unify( exp_val, ArenaRc::clone(&id21), state, memory, true) {
                                Ok( val ) => val,
                                Err( e ) => exit(e, state, memory),
                            };
                    
                            declare_unifiers( &unifiers, state, memory);

                            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:7}));
                            set_lineinfo(  new_lineinfo, state, memory); 

                            let id22 = memory.alloc_rc(Node::AstroID(AstroID::new("j".to_string())));
                            let id23 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));
                            let idx6 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id23),ArenaRc::clone(&id22))));
                            let id24 = memory.alloc_rc(Node::AstroID(AstroID::new("j".to_string())));
                            let id25 = memory.alloc_rc(Node::AstroID(AstroID::new("__plus__".to_string())));
                            let id26 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));
                            let i11 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)));
                            let t7 = memory.alloc_rc(Node::AstroPair(AstroPair::new(ArenaRc::clone(&id24), ArenaRc::clone(&i11) )));
                            let apply8 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id25),ArenaRc::clone(&t7))));
                            let idx7 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id26),ArenaRc::clone(&apply8))));

                            let exp_val = match walk( ArenaRc::clone(&idx6), state, memory) {
                                Ok( val ) => val,
                                Err( e ) => exit(e, state, memory),
                            };

                            // let Node::AstroInteger(AstroInteger{value:v2}) = *exp_val else {panic!("error uo")};
                            // println!("SWAPPING {} and {}",v1,v2);
                    
                            let unifiers = match unify( exp_val, ArenaRc::clone(&idx7), state, memory, true) {
                                Ok( val ) => val,
                                Err( e ) => exit(e, state, memory),
                            };
                    
                            declare_unifiers( &unifiers, state, memory);

                            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:8}));
                            set_lineinfo(  new_lineinfo, state, memory); 

                            let id27 = memory.alloc_rc(Node::AstroID(AstroID::new("temp".to_string())));
                            let id28 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));
                            let id29 = memory.alloc_rc(Node::AstroID(AstroID::new("j".to_string())));
                            let idx8 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new(ArenaRc::clone(&id28),ArenaRc::clone(&id29))));

                            let exp_val = match walk( ArenaRc::clone(&id27), state, memory) {
                                Ok( val ) => val,
                                Err( e ) => exit(e, state, memory),
                            };
                    
                            let unifiers = match unify( exp_val, ArenaRc::clone(&idx8), state, memory, true) {
                                Ok( val ) => val,
                                Err( e ) => exit(e, state, memory),
                            };
                    
                            declare_unifiers( &unifiers, state, memory);
                        }
                    }
                }

                let id30 = memory.alloc_rc(Node::AstroID(AstroID::new("l".to_string())));

                let result = match walk( ArenaRc::clone(&id30), state, memory) {
                    Ok( val ) => val,
                    Err(e) => return Err(e),
                };

                return Ok( result );

            } else {
                return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory );
            }
        }
        fn _ast3<'a>( node: ArenaRc<Node>, state: &'a mut State, memory: &'a mut Arena<Node> ) -> Result< ArenaRc<Node>,  ArenaRc<Node>>{

            let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prologue.ast".to_string(),line_number:1}));
            set_lineinfo(  new_lineinfo, state, memory); 

            let id1 = AstroID::new("kind".to_string());
            let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
            let s1 = memory.alloc_rc(Node::AstroString(AstroString::new("string".to_string())));
            let tm1 = memory.alloc_rc(Node::AstroTypeMatch(AstroTypeMatch::new(ArenaRc::clone( &s1 ))));
            let np1 = memory.alloc_rc(Node::AstroNamedPattern(AstroNamedPattern::new( id1, ArenaRc::clone(&tm1))));

            let t1 = memory.alloc_rc(Node::AstroTuple(AstroTuple::new(Rc::new(RefCell::new(vec![ ArenaRc::clone(&np1), ArenaRc::clone(&id2) ])))));

            if let Ok( unifiers ) = unify( ArenaRc::clone(&node), ArenaRc::clone(&t1), state, memory, true ) {

                state.push_scope();

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prologue.ast".to_string(),line_number:2}));
                set_lineinfo(  new_lineinfo, state, memory); 

                let out1 = declare_formal_args( &unifiers, state, memory );
                match out1 {
                    Ok( val ) => (),
                    Err( e ) => return Err( e ),
                };

                let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("kind".to_string())));
                let exp_val = match walk( ArenaRc::clone(&id3), state, memory) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("this".to_string())));
                let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("kind".to_string())));
                let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id4), ArenaRc::clone(&id5))));
                let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&idx1), state, memory, true) {
                    Ok( val) => val,
                    Err( e ) => return Err(e),
                };

                declare_unifiers( &unifiers, state, memory);

                let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prologue.ast".to_string(),line_number:3}));
                set_lineinfo(  new_lineinfo, state, memory); 

                let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
                let exp_val = match walk( ArenaRc::clone(&id6), state, memory) {
                    Ok( val ) => val,
                    Err( e ) => return Err(e),
                };

                let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("this".to_string())));
                let id8 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
                let idx1 = memory.alloc_rc(Node::AstroIndex(AstroIndex::new( ArenaRc::clone(&id7), ArenaRc::clone(&id8))));
                let unifiers = match unify( ArenaRc::clone(&exp_val), ArenaRc::clone(&idx1), state, memory, true) {
                    Ok( val) => val,
                    Err( e ) => return Err(e),
                };

                declare_unifiers( &unifiers, state, memory);

                state.pop_scope();

                return Ok(memory.alloc_rc(Node::AstroNone(AstroNone::new())));

            } else {
                return  new_exception("PatternMatchFailed".to_string(), "None of the function bodies unified with actual parameters.".to_string(), state, memory );
            }

        }

        // set_lineinfo('prog.txt',1)
        // exp_val = walk(('function-exp', ('implementation', '_ast72')))
        // unifiers = unify(exp_val,('id', 'bubblesort'))
        // declare_unifiers(unifiers)
     
        // set_lineinfo('prog.txt',13)
        // set_lineinfo('prog.txt',15)
        // exp_val = walk(('list', [('integer', 2), ('integer', 8), ('integer', 3), ('integer', 7), ('integer', 4), ('integer', 6)]))
        // unifiers = unify(exp_val,('id', 'x'))
        // declare_unifiers(unifiers)
        let mut memory = Arena::new();
        let mut state = State::new().unwrap();
        state.dispatch_table.insert( String::from("_ast1") , _ast1 );
        state.dispatch_table.insert( String::from("_ast2") , _ast2 );
        state.dispatch_table.insert( String::from("_ast3") , _ast3 );

        // strucutre def for exception
        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("kind".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())));
        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("__init__".to_string())));
        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast3".to_string())));
        let f1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new( ArenaRc::clone(&id4))));
        let d1 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id1))));
        let d2 = memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&id2))));
        let u1 = memory.alloc_rc(Node::AstroUnify(AstroUnify::new(ArenaRc::clone(&id3),ArenaRc::clone(&f1))));
        let member_list = vec![ ArenaRc::clone(&d1), ArenaRc::clone(&d2), ArenaRc::clone(&u1) ];
        let mut struct_memory: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        let mut member_names: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
        for member in member_list {
            let _type = peek( ArenaRc::clone(&member) );
            if _type == "data" {
                let Node::AstroData(AstroData{value:ref id_node}) = *member
                    else {panic!("ERROR: object construction: expected object data.")};
                let Node::AstroID(AstroID{name:ref val}) = ** id_node
                    else {panic!("ERROR: object construction: expected ID.")};
                struct_memory.borrow_mut().push( memory.alloc_rc(Node::AstroNone(AstroNone::new())) );
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "unify" {
                let Node::AstroUnify(AstroUnify{term:ref id_node,pattern:ref function_exp}) = *member
                    else {panic!("ERROR: object construction: expection unify node.")};
                let function_val = match walk( ArenaRc::clone(&function_exp), &mut state, &mut memory) {
                    Ok( val ) => val,
                    Err ( e ) => panic!("error!"),
                };
                struct_memory.borrow_mut().push( ArenaRc::clone( &function_val ));
                member_names.borrow_mut().push( ArenaRc::clone(&id_node));
            } else if _type == "noop" {
                ;// pass
            } else {
                panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
            }
        }
        let struct_type = memory.alloc_rc(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
        state.enter_sym( "Exception", ArenaRc::clone(&struct_type)  );

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prologue.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut  state, &mut memory); 

        let id1 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast1".to_string())));
        let id2 = memory.alloc_rc(Node::AstroID(AstroID::new("length".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new(ArenaRc::clone(&id1))));
        let exp_val = match walk( ArenaRc::clone(&func1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id2), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:1}));
        set_lineinfo(  new_lineinfo, &mut  state, &mut memory); 

        let id3 = memory.alloc_rc(Node::AstroID(AstroID::new("_ast2".to_string())));
        let id4 = memory.alloc_rc(Node::AstroID(AstroID::new("bubblesort".to_string())));
        let func1 = memory.alloc_rc(Node::AstroFunction(AstroFunction::new(ArenaRc::clone(&id3))));
        let exp_val = match walk( ArenaRc::clone(&func1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id4), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:15}));
        set_lineinfo(  new_lineinfo, &mut  state, &mut memory); 

        // exp_val = walk(('list', [('integer', 2), ('integer', 8), ('integer', 3), ('integer', 7), ('integer', 4), ('integer', 6)]))
        let i1 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(20)));
        let i2 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(8)));
        let i3 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(3)));
        let i4 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(7)));
        let i5 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(4)));
        let i6 = memory.alloc_rc(Node::AstroInteger(AstroInteger::new(6)));
        let l1 = memory.alloc_rc(Node::AstroList(AstroList::new( Rc::new(RefCell::new( vec![ ArenaRc::clone(&i1), ArenaRc::clone(&i2), ArenaRc::clone(&i3), ArenaRc::clone(&i4), ArenaRc::clone(&i5), ArenaRc::clone(&i6) ] )))));
        let id5 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));

        let exp_val = match walk( ArenaRc::clone(&l1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id5), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let new_lineinfo = memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"prog.ast".to_string(),line_number:16}));
        set_lineinfo(  new_lineinfo, &mut  state, &mut memory); 

        let id6 = memory.alloc_rc(Node::AstroID(AstroID::new("x".to_string())));
        let id7 = memory.alloc_rc(Node::AstroID(AstroID::new("bubblesort".to_string())));
        let id8 = memory.alloc_rc(Node::AstroID(AstroID::new("y".to_string())));
        let apply1 = memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&id7),ArenaRc::clone(&id6))));

        let exp_val = match walk( ArenaRc::clone(&apply1), &mut state, &mut memory) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&id8), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        let check1 = state.lookup_sym("y",true);

        let Node::AstroList(AstroList{ contents: ref content}) = *check1 else {panic!("test failed")};

        let Node::AstroInteger(AstroInteger{ value: v1 }) = *content.borrow()[0] else  {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{ value: v2 }) = *content.borrow()[1] else  {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{ value: v3 }) = *content.borrow()[2] else  {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{ value: v4 }) = *content.borrow()[3] else  {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{ value: v5 }) = *content.borrow()[4] else  {panic!("test failed")};
        let Node::AstroInteger(AstroInteger{ value: v6 }) = *content.borrow()[5] else  {panic!("test failed")};

        println!("Array values: {} {} {} {} {} {}",v1,v2,v3,v4,v5,v6);
    }
}

