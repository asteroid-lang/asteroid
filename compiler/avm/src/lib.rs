/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Abstract Virtual Machine                                                   */
/*                                                                            */
/******************************************************************************/
#![allow(unused)]

use state::*;     //Asteroid state representation
use symtab::*;    //Asteroid symbol table
use ast::*;       //Asteroid AST representation
use support::*;   //Asteroid support functions

use std::rc::Rc;  //Multiple ownership(astronodes)
use regex::Regex; //Regular expressions

/******************************************************************************/
pub fn unify<'a>( term: Rc<AstroNode>, pattern: Rc<AstroNode>, state: &'a mut State, unifying: bool) -> Result<Vec<(Rc<AstroNode>,Rc<AstroNode>)>, (&'static str,String) >{
   
    let term_type = peek( Rc::clone(&term) ).unwrap();
    let pattern_type = peek( Rc::clone(&pattern) ).unwrap();


    if term_type == "string" { // Apply regular expression pattern match
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
    } else if term_type == "integer" || term_type == "bool" || term_type == "real"  {

        if term_type == pattern_type && term == pattern {
            Ok( vec![] ) // Return an empty unifier
        } else {
            Err( ("PatternMatchFailed", format!("{} is not the same as {}",term2string(&pattern).unwrap(),term2string(&term).unwrap())))
        }

    } else { /********** PLACEHOLDER UNITL UNIFY IS FINISHED ***/
        Ok(vec![])
    }
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
    fn test_unify_regex() {
        let s1 = Rc::new( AstroNode::AstroString( AstroString::new(String::from("hello")).unwrap()) );
        let s2 = Rc::new( AstroNode::AstroString( AstroString::new(String::from("hello")).unwrap()) );
        let s3 = Rc::new( AstroNode::AstroString( AstroString::new(String::from("nothello")).unwrap()) );

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
        let i1 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1).unwrap()));
        let i2 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(2).unwrap()));
        let i3 = Rc::new( AstroNode::AstroInteger( AstroInteger::new(1).unwrap()));

        let b1 = Rc::new( AstroNode::AstroBool( AstroBool::new(true).unwrap()));
        let b2 = Rc::new( AstroNode::AstroBool( AstroBool::new(false).unwrap()));
        let b3 = Rc::new( AstroNode::AstroBool( AstroBool::new(true).unwrap()));

        let r1 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.1).unwrap()));
        let r2 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.2).unwrap()));
        let r3 = Rc::new( AstroNode::AstroReal( AstroReal::new(1.1).unwrap()));

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
    fn test_walk_lineinfo() {
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
    
}