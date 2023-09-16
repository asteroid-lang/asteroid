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
pub fn map2boolean<'a>(node: &'a Node) -> Node {
    match node {
        Node::AstroInteger( AstroInteger{value:0} ) => Node::AstroBool(AstroBool::new(false) ),
        Node::AstroReal( AstroReal{value} ) if *value == 0.0 => Node::AstroBool(AstroBool::new(false) ),
        Node::AstroNone(_) => Node::AstroBool(AstroBool::new(false)),
        Node::AstroNil(_) => Node::AstroBool(AstroBool::new(false)),
        Node::AstroBool( AstroBool{value:false} ) => Node::AstroBool(AstroBool::new(false)),
        Node::AstroString( AstroString{value} ) if value == "" => Node::AstroBool(AstroBool::new(false)),
        Node::AstroList( AstroList{contents}) if contents.borrow().len() == 0 => Node::AstroBool(AstroBool::new(false)),
        Node::AstroTuple( AstroTuple{contents}) => Node::AstroBool(AstroBool::new(false)),
        _ => Node::AstroBool( AstroBool::new(true)) 
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
pub fn term2string<'a>(node: &'a Node) -> Option<String> {
    match node {
        Node::AstroInteger(AstroInteger{value}) => Some(value.to_string()),
        Node::AstroReal(AstroReal{value}) => Some(value.to_string()),
        Node::AstroBool(AstroBool{value}) => Some(value.to_string()),
        Node::AstroString(AstroString{value}) => Some(value.clone()),
        Node::AstroLineInfo(AstroLineInfo{module,line_number}) => term2string_lineinfo(module,*line_number),
        Node::AstroNone(_) => Some(String::from("None")),
        Node::AstroNil(_) => Some(String::from("Nil")),
        Node::AstroList(_) => term2string_list(node),
        Node::AstroTuple(_) => term2string_tuple(node),
        Node::AstroToList(_) => term2string_tolist(node),
        Node::AstroHeadTail(_) => term2string_headtail(node),
        Node::AstroSequence(_) => term2string_sequence(node),
        Node::AstroFunction(_) => term2string_function(node),
        Node::AstroEval(_) => term2string_eval(node),
        Node::AstroQuote(_) => term2string_quote(node),
        Node::AstroConstraint(_) => term2string_constraint(node),
        Node::AstroTypeMatch(_) => term2string_typematch(node),
        Node::AstroForeign(_) => Some(String::from("Foriegn Object")),
        Node::AstroID(AstroID{name}) => Some(name.clone()),
        Node::AstroObject(_) => term2string_object(node),
        Node::AstroApply(_) => term2string_apply(node),
        Node::AstroIndex(_) => term2string_index(node),
        Node::AstroEscape(_) => term2string_escape(node),
        Node::AstroIs(_) => term2string_is(node),
        Node::AstroIn(_) => term2string_in(node),
        Node::AstroIf(_) => term2string_if(node),
        Node::AstroNamedPattern(_) => term2string_namedpattern(node),
        Node::AstroDeref(_) => term2string(node),
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
pub fn term2string_list<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroList(AstroList{contents}) = node
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
pub fn term2string_tuple<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroTuple(AstroTuple{contents}) = node
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
pub fn term2string_tolist<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroToList(AstroToList{start,stop,stride}) = node
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
pub fn term2string_headtail<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroHeadTail(AstroHeadTail{head,tail}) = node
        else {panic!("Expected head-tail in term2string_headtail")};

    let mut out = String::new();
    out += "[ ";
    out += term2string(head).unwrap().as_str();
    out += " | ";
    out += term2string(tail).unwrap().as_str();
    out += " ]";
    Some(out)
}
pub fn term2string_sequence<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroSequence(AstroSequence{first,second}) = node
        else {panic!("Expected sequence in term2string_sequence")};

    let mut out = String::new();
    out += "sequence: first=";
    out += term2string(first).unwrap().as_str();
    out += ", second=";
    out += term2string(second).unwrap().as_str();
    Some(out)
}
pub fn term2string_function<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroFunction(AstroFunction{body_list}) = node
        else {panic!("Expected function in term2string_function")};

    let mut out = String::new();
    out += "function{ ";
    out += term2string(body_list).unwrap().as_str();
    out += " }";
    Some(out)
}
pub fn term2string_eval<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroEval(AstroEval{expression}) = node
        else {panic!("Expected eval expression in term2string_eval")};

    let mut out = String::new();
    out += "eval( ";
    out += term2string(expression).unwrap().as_str();
    out += " )";
    Some(out)
}
pub fn term2string_quote<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroQuote(AstroQuote{expression}) = node
        else {panic!("Expected quote expression in term2string_quote")};

    let mut out = String::new();
    out += "*";
    out += term2string(expression).unwrap().as_str();
    Some(out)
}
pub fn term2string_constraint<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroConstraint(AstroConstraint{expression}) = node
        else {panic!("Expected constraint expression in term2string_constraint")};

    let mut out = String::new();
    out += "%[ ";
    out += term2string(expression).unwrap().as_str();
    out += " ]%";
    Some(out)
}
pub fn term2string_typematch<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroTypeMatch(AstroTypeMatch{expression}) = node
        else {panic!("Expected typematch expresssion in term2string_typematch")};

    let mut out = String::new();
    out += ":%";
    out += term2string(expression).unwrap().as_str();
    Some(out)
}
pub fn term2string_object<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroObject(AstroObject{struct_id,object_memory}) = node
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
pub fn term2string_apply<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroApply(AstroApply{argument,function}) = node
        else {panic!("Expected apply expression in term2string_apply")};

    let mut out = String::new();
    out += &term2string(function).unwrap();
    out += "( ";
    out += &term2string(argument).unwrap();
    out += " )";
    Some(out)
}
pub fn term2string_index<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroIndex(AstroIndex{structure,index_exp}) = node
        else {panic!("Expected index expression in term2string_index")};

    let mut out = String::new();
    out += &term2string(structure).unwrap();
    out += "@";
    out += &term2string(index_exp).unwrap();
    Some(out)
}
pub fn term2string_escape<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroEscape(AstroEscape{content}) = node
        else {panic!("Expected escape in term2string_escape")};

    let mut out = String::new();
    out += "Escape\"";
    out += &content;
    out += "\"";
    Some(out)
}
pub fn term2string_is<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroIs(AstroIs{pattern,term}) = node
        else {panic!("Expected is expression in term2string_is")};

    let mut out = String::new();
    out += &term2string(pattern).unwrap();
    out += " is ";
    out += &term2string(term).unwrap();
    Some(out)
}
pub fn term2string_in<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroIn(AstroIn{expression,expression_list}) = node
        else {panic!("Expected in expression in term2string_in")};

    let mut out = String::new();
    out += &term2string(expression).unwrap();
    out += " in ";
    out += &term2string( expression_list).unwrap();
    Some(out)
}
pub fn term2string_if<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroIf(AstroIf{cond_exp,then_exp,else_exp}) = node
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
pub fn term2string_namedpattern<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroNamedPattern(AstroNamedPattern{name,pattern}) = node
        else {panic!("Expected named pattern in term2string_namedpattern")};

    let mut out = String::new();
    out += &name.name;
    out += ":%";
    out += &term2string(pattern).unwrap();

    Some(out)
}
pub fn term2string_deref<'a>(node: &'a Node) -> Option<String> {
    let Node::AstroDeref(AstroDeref{expression}) = node
        else {panic!("Expected deref in term2string_deref")};

    let mut out = String::new();
    out += &term2string(expression).unwrap();

    Some(out)
}
/******************************************************************************/
pub fn data_only( memory: RefCell<Vec<Rc<Node>>> ) -> Rc<Vec<Rc<Node>>> {
    // filter an object memory and return a memory with only data values.

    let mut data_memory: Vec<Rc<Node>> = vec![];
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
pub fn data_ix_list( memory: RefCell<Vec<Rc<Node>>> ) -> Rc<Vec<usize>> {

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

        let out1 = map2boolean(& Node::AstroInteger(int1)).unwrap();
        let val1 = match out1 {
            Node::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            Node::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let out2 = map2boolean(& Node::AstroInteger(int2)).unwrap();
        let val2 = match out2 {
            Node::AstroBool( AstroBool{id,value:true}) => (),
            Node::AstroBool( AstroBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let int3 = AstroInteger::new(11).unwrap(); //false
        let int4 = AstroInteger::new(12).unwrap(); //true
        let list1 = AstroList::new(2,vec![Node::AstroInteger(int3),Node::AstroInteger(int4)]).unwrap();
        let out3 = map2boolean(& Node::AstroList(list1)).unwrap();
        let val3 = match out3 {
            Node::AstroBool( AstroBool{id,value:true}) => (),
            Node::AstroBool( AstroBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let list2 = AstroList::new(0,vec![]).unwrap();
        let out4 = map2boolean(& Node::AstroList(list2)).unwrap();
        let val4 = match out4 {
            Node::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            Node::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let none1 = AstroNone::new().unwrap();
        let nil1 = AstroNil::new().unwrap();
        let out5 = map2boolean(& Node::AstroNone(none1)).unwrap();
        let out6 = map2boolean(& Node::AstroNil(nil1)).unwrap();
        let val5 = match out5 {
            Node::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            Node::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };
        let val6 = match out6 {
            Node::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            Node::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let int5 = AstroInteger::new(11).unwrap(); //false
        let int6 = AstroInteger::new(12).unwrap(); //true
        let tuple1 = AstroTuple::new(2,vec![Node::AstroInteger(int5),Node::AstroInteger(int6)]).unwrap();
        let out7 = map2boolean(& Node::AstroTuple(tuple1)).unwrap();
        let val7 = match out7 {
            Node::AstroBool( AstroBool{id,value:true}) => (),
            Node::AstroBool( AstroBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let tuple2 = AstroTuple::new(0,vec![]).unwrap();
        let out8 = map2boolean(& Node::AstroTuple(tuple2)).unwrap();
        let val8 = match out8 {
            Node::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            Node::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let str1 = AstroString::new(String::from("abc")).unwrap();
        let str2 = AstroString::new(String::from("")).unwrap();
        let out9 = map2boolean(& Node::AstroString(str1)).unwrap();
        let out10 = map2boolean(& Node::AstroString(str2)).unwrap();
        let val9 = match out9 {
            Node::AstroBool( AstroBool{id,value:true}) => (),
            Node::AstroBool( AstroBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };
        let val10 = match out10 {
            Node::AstroBool( AstroBool{id,value:true}) => panic!("Error: test_map2boolean"),
            Node::AstroBool( AstroBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };
    }
}
