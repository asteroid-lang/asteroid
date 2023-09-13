/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Support Module                                                             */
/*                                                                            */
/******************************************************************************/  
#![allow(unused)]

use ast::*;
use std::rc::Rc;  
use std::cell::RefCell;

/*******************************************************************************
# Asteroid uses truth values similar to Python's Pythonic truth values:
#
# Any object can be tested for truth value, for use in an if or while condition or as
# operand of the Boolean operations.
#
# The following values are considered false:
#
#     none
#     false
#     zero of the numeric types: 0, 0.0.
#     the empty string
#     any empty list: (,), [].
#
#  All other values are considered true, in particular any object is considered
#  to be a true value. 
*******************************************************************************/
pub fn map2boolean<'a>(node: &'a AstroNode) -> Option<AstroNode> {
    match node {
        AstroNode::AstroInteger( AstroInteger{value:0} ) => Some( AstroNode::AstroBool(AstroBool::new(false) )),
        AstroNode::AstroReal( AstroReal{value} ) if *value == 0.0 => Some( AstroNode::AstroBool(AstroBool::new(false) )),
        AstroNode::AstroNone(_) => Some( AstroNode::AstroBool(AstroBool::new(false))),
        AstroNode::AstroNil(_) => Some( AstroNode::AstroBool(AstroBool::new(false))),
        AstroNode::AstroBool( AstroBool{value:false} ) => Some( AstroNode::AstroBool(AstroBool::new(false))),
        AstroNode::AstroString( AstroString{value} ) if value == "" => Some( AstroNode::AstroBool(AstroBool::new(false))),
        AstroNode::AstroList( AstroList{contents}) if contents.borrow().len() == 0 => Some( AstroNode::AstroBool(AstroBool::new(false))),
        AstroNode::AstroTuple( AstroTuple{contents}) => Some( AstroNode::AstroBool(AstroBool::new(false))),
        _ => Some( AstroNode::AstroBool( AstroBool::new(true)) )
    }
}
/******************************************************************************/
pub fn promote<'a>(type1:&'a str,type2:&'a str) ->  &'a str {
    if type1 == "string" && ["string","real","integer","list","tuple","boolean"].contains(&type2) {
        "string"
    } else if type2 == "string" &&  ["string","real","integer","list","tuple","boolean"].contains(&type1) {
        "string"
    } else if type1 == "real" && ["real","integer","boolean"].contains(&type2) {
        "real"
    } else if type2 == "real" && ["real","integer","boolean"].contains(&type1) {
        "real"
    } else if type1 == "integer" && ["integer","boolean"].contains(&type2) {
        "integer"
    } else if type2 == "integer" && ["integer","boolean"].contains(&type1) {
        "integer"
    } else if type1 == "boolean" && type2 == "boolean" {
        "boolean"
    } else if type1 == "list" && type2 == "list" {
        "list"
    } else if type1 == "tuple" && type2 == "tuple" {
        "tuple"
    } else if type1 == "none" && type2 == "none" {
        "none"
    } else {
        if type1 == type2 {
            panic!("binary operation on type '{}' not supported",type1);
        } else {
            panic!("type '{}' and type '{}' are incompatible", type1, type2);
        }
    }
}
/******************************************************************************/
pub fn term2string<'a>(node: &'a AstroNode) -> Option<String> {
    match node {
        AstroNode::AstroInteger(AstroInteger{value}) => Some(value.to_string()),
        AstroNode::AstroReal(AstroReal{value}) => Some(value.to_string()),
        AstroNode::AstroBool(AstroBool{value}) => Some(value.to_string()),
        AstroNode::AstroString(AstroString{value}) => Some(value.clone()),
        AstroNode::AstroLineInfo(AstroLineInfo{module,line_number}) => term2string_lineinfo(module,*line_number),
        AstroNode::AstroNone(_) => Some(String::from("None")),
        AstroNode::AstroNil(_) => Some(String::from("Nil")),
        AstroNode::AstroList(_) => term2string_list(node),
        AstroNode::AstroTuple(_) => term2string_tuple(node),
        AstroNode::AstroToList(_) => term2string_tolist(node),
        AstroNode::AstroHeadTail(_) => term2string_headtail(node),
        AstroNode::AstroSequence(_) => term2string_sequence(node),
        AstroNode::AstroFunction(_) => term2string_function(node),
        AstroNode::AstroEval(_) => term2string_eval(node),
        AstroNode::AstroQuote(_) => term2string_quote(node),
        AstroNode::AstroConstraint(_) => term2string_constraint(node),
        AstroNode::AstroTypeMatch(_) => term2string_typematch(node),
        AstroNode::AstroForeign(_) => Some(String::from("Foriegn Object")),
        AstroNode::AstroID(AstroID{name}) => Some(name.clone()),
        AstroNode::AstroObject(_) => term2string_object(node),
        AstroNode::AstroApply(_) => term2string_apply(node),
        AstroNode::AstroIndex(_) => term2string_index(node),
        AstroNode::AstroEscape(_) => term2string_escape(node),
        AstroNode::AstroIs(_) => term2string_is(node),
        AstroNode::AstroIn(_) => term2string_in(node),
        AstroNode::AstroIf(_) => term2string_if(node),
        AstroNode::AstroNamedPattern(_) => term2string_namedpattern(node),
        AstroNode::AstroDeref(_) => term2string(node),
        _ => Some(String::from("")),
    }
}
/******************************************************************************/
/*                          TERM2STRING HELPERS                               */
/******************************************************************************/
pub fn term2string_lineinfo(module: &str,line_number: usize) -> Option<String> {
    let mut out = String::new();
    out += "lineinfo: module=";
    out += module;
    out += ", line number=";
    out += line_number.to_string().as_str();
    Some(out)
}
pub fn term2string_list<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroList(AstroList{contents}) = node
        else {panic!("Expected list in term2string_list")};
    
    let mut out = String::new();
    out +="[ ";
    for i in 0..contents.borrow().len() {
        out += &term2string(&contents.borrow()[i]).unwrap();
        if i != (contents.borrow().len()-1) {
            out +=" , ";
        }
    }
    out += " ]";
    Some(out)
}
pub fn term2string_tuple<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroTuple(AstroTuple{contents}) = node
        else {panic!("Expected tuple in term2string_tuple")};

    let mut out = String::new();
    out +="( ";
    for i in 0..contents.borrow().len() {
        out += &term2string(&contents.borrow()[i]).unwrap();
        if i != (contents.borrow().len()-1) {
            out +=" , ";
        }
    }
    out += " )";
    Some(out)
}
pub fn term2string_tolist<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroToList(AstroToList{start,stop,stride}) = node
        else {panic!("Expected to-list in term2string_tolist")};

    let mut out = String::new();
    out += "to-list: start=";
    out += term2string(start).unwrap().as_str();
    out += " ,stop=";
    out += term2string(stop).unwrap().as_str();
    out += " ,stride=";
    out += term2string(stride).unwrap().as_str();
    Some(out)
}
pub fn term2string_headtail<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroHeadTail(AstroHeadTail{head,tail}) = node
        else {panic!("Expected head-tail in term2string_headtail")};

    let mut out = String::new();
    out += "[ ";
    out += term2string(head).unwrap().as_str();
    out += " | ";
    out += term2string(tail).unwrap().as_str();
    out += " ]";
    Some(out)
}
pub fn term2string_sequence<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroSequence(AstroSequence{first,second}) = node
        else {panic!("Expected sequence in term2string_sequence")};

    let mut out = String::new();
    out += "sequence: first=";
    out += term2string(first).unwrap().as_str();
    out += ", second=";
    out += term2string(second).unwrap().as_str();
    Some(out)
}
pub fn term2string_function<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroFunction(AstroFunction{body_list}) = node
        else {panic!("Expected function in term2string_function")};

    let mut out = String::new();
    out += "function{ ";
    out += term2string(body_list).unwrap().as_str();
    out += " }";
    Some(out)
}
pub fn term2string_eval<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroEval(AstroEval{expression}) = node
        else {panic!("Expected eval expression in term2string_eval")};

    let mut out = String::new();
    out += "eval( ";
    out += term2string(expression).unwrap().as_str();
    out += " )";
    Some(out)
}
pub fn term2string_quote<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroQuote(AstroQuote{expression}) = node
        else {panic!("Expected quote expression in term2string_quote")};

    let mut out = String::new();
    out += "*";
    out += term2string(expression).unwrap().as_str();
    Some(out)
}
pub fn term2string_constraint<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroConstraint(AstroConstraint{expression}) = node
        else {panic!("Expected constraint expression in term2string_constraint")};

    let mut out = String::new();
    out += "%[ ";
    out += term2string(expression).unwrap().as_str();
    out += " ]%";
    Some(out)
}
pub fn term2string_typematch<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroTypeMatch(AstroTypeMatch{expression}) = node
        else {panic!("Expected typematch expresssion in term2string_typematch")};

    let mut out = String::new();
    out += ":%";
    out += term2string(expression).unwrap().as_str();
    Some(out)
}
pub fn term2string_object<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroObject(AstroObject{struct_id,object_memory}) = node
        else {panic!("Expected object in term2string_object")};

    let mut out = String::new();
    out += &struct_id.name;
    out += "( ";

    for i in 0..object_memory.borrow().len() {
        if i != 0 {
            out += " , "
        }
        out += &term2string(&object_memory.borrow()[i]).unwrap();
    }
    out += " )";
    Some(out)
}
pub fn term2string_apply<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroApply(AstroApply{argument,function}) = node
        else {panic!("Expected apply expression in term2string_apply")};

    let mut out = String::new();
    out += &term2string(function).unwrap();
    out += "( ";
    out += &term2string(argument).unwrap();
    out += " )";
    Some(out)
}
pub fn term2string_index<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroIndex(AstroIndex{structure,index_exp}) = node
        else {panic!("Expected index expression in term2string_index")};

    let mut out = String::new();
    out += &term2string(structure).unwrap();
    out += "@";
    out += &term2string(index_exp).unwrap();
    Some(out)
}
pub fn term2string_escape<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroEscape(AstroEscape{content}) = node
        else {panic!("Expected escape in term2string_escape")};

    let mut out = String::new();
    out += "Escape\"";
    out += &content;
    out += "\"";
    Some(out)
}
pub fn term2string_is<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroIs(AstroIs{pattern,term}) = node
        else {panic!("Expected is expression in term2string_is")};

    let mut out = String::new();
    out += &term2string(pattern).unwrap();
    out += " is ";
    out += &term2string(term).unwrap();
    Some(out)
}
pub fn term2string_in<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroIn(AstroIn{expression,expression_list}) = node
        else {panic!("Expected in expression in term2string_in")};

    let mut out = String::new();
    out += &term2string(expression).unwrap();
    out += " in ";
    out += &term2string( expression_list).unwrap();
    Some(out)
}
pub fn term2string_if<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroIf(AstroIf{cond_exp,then_exp,else_exp}) = node
        else {panic!("Expected if expression in term2string_if")};

    let mut out = String::new();
    out += "If ";
    out += &term2string(cond_exp).unwrap();
    out += " then ";
    out += &term2string(then_exp).unwrap();
    out += " else ";
    out += &term2string(else_exp).unwrap();

    Some(out)
}
pub fn term2string_namedpattern<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroNamedPattern(AstroNamedPattern{name,pattern}) = node
        else {panic!("Expected named pattern in term2string_namedpattern")};

    let mut out = String::new();
    out += &name.name;
    out += ":%";
    out += &term2string(pattern).unwrap();

    Some(out)
}
pub fn term2string_deref<'a>(node: &'a AstroNode) -> Option<String> {
    let AstroNode::AstroDeref(AstroDeref{expression}) = node
        else {panic!("Expected deref in term2string_deref")};

    let mut out = String::new();
    out += &term2string(expression).unwrap();

    Some(out)
}
/******************************************************************************/
pub fn data_only( memory: RefCell<Vec<Rc<AstroNode>>> ) -> Rc<Vec<Rc<AstroNode>>> {
    // filter an object memory and return a memory with only data values.

    let mut data_memory: Vec<Rc<AstroNode>> = vec![];
    let mut _type = "";
    for item in memory.borrow().iter() {
        _type = peek( Rc::clone(item) );
        if _type != "function-val" {
            data_memory.push( Rc::clone(item));
        }
    };
    Rc::new( data_memory )
}
/******************************************************************************/
pub fn data_ix_list( memory: RefCell<Vec<Rc<AstroNode>>> ) -> Rc<Vec<usize>> {

    let mut idx_list: Vec<usize> = vec![];
    let mut counter = 0usize;
    let mut _type = "";
    for item in memory.borrow().iter() {
        _type = peek( Rc::clone(item) );
        if _type != "function-val" {
            idx_list.push( counter );
        }
        counter += 1;
    };
    Rc::new( idx_list )
}

/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map2boolean () {
        let int1 = AstroInteger::new(0).unwrap(); //false
        let int2 = AstroInteger::new(1).unwrap(); //true

        let out1 = map2boolean(& AstroNode::AstroInteger(int1)).unwrap();
        let val1 = match out1 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            AstroNode::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let out2 = map2boolean(& AstroNode::AstroInteger(int2)).unwrap();
        let val2 = match out2 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => (),
            AstroNode::AstroBool( AstroBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let int3 = AstroInteger::new(11).unwrap(); //false
        let int4 = AstroInteger::new(12).unwrap(); //true
        let list1 = AstroList::new(2,vec![AstroNode::AstroInteger(int3),AstroNode::AstroInteger(int4)]).unwrap();
        let out3 = map2boolean(& AstroNode::AstroList(list1)).unwrap();
        let val3 = match out3 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => (),
            AstroNode::AstroBool( AstroBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let list2 = AstroList::new(0,vec![]).unwrap();
        let out4 = map2boolean(& AstroNode::AstroList(list2)).unwrap();
        let val4 = match out4 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            AstroNode::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let none1 = AstroNone::new().unwrap();
        let nil1 = AstroNil::new().unwrap();
        let out5 = map2boolean(& AstroNode::AstroNone(none1)).unwrap();
        let out6 = map2boolean(& AstroNode::AstroNil(nil1)).unwrap();
        let val5 = match out5 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            AstroNode::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };
        let val6 = match out6 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            AstroNode::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let int5 = AstroInteger::new(11).unwrap(); //false
        let int6 = AstroInteger::new(12).unwrap(); //true
        let tuple1 = AstroTuple::new(2,vec![AstroNode::AstroInteger(int5),AstroNode::AstroInteger(int6)]).unwrap();
        let out7 = map2boolean(& AstroNode::AstroTuple(tuple1)).unwrap();
        let val7 = match out7 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => (),
            AstroNode::AstroBool( AstroBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let tuple2 = AstroTuple::new(0,vec![]).unwrap();
        let out8 = map2boolean(& AstroNode::AstroTuple(tuple2)).unwrap();
        let val8 = match out8 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            AstroNode::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let str1 = AstroString::new(String::from("abc")).unwrap();
        let str2 = AstroString::new(String::from("")).unwrap();
        let out9 = map2boolean(& AstroNode::AstroString(str1)).unwrap();
        let out10 = map2boolean(& AstroNode::AstroString(str2)).unwrap();
        let val9 = match out9 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => (),
            AstroNode::AstroBool( AstroBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };
        let val10 = match out10 {
            AstroNode::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            AstroNode::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };
    }
}
