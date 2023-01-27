/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Support Module                                                             */
/*                                                                            */
/******************************************************************************/  
#![allow(unused)]

use ast::*;
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
pub fn map2boolean<'a>(node: &'a ASTNode) -> Option<ASTNode> {
    match node {
        ASTNode::ASTInteger( ASTInteger{id:0,value:0} ) => Some( ASTNode::ASTBool(ASTBool::new(false).unwrap() )),
        ASTNode::ASTReal( ASTReal{id:1,value:0.0} ) => Some( ASTNode::ASTBool(ASTBool::new(false).unwrap() )),
        ASTNode::ASTNone(_) => Some( ASTNode::ASTBool(ASTBool::new(false).unwrap() )),
        ASTNode::ASTNil(_) => Some( ASTNode::ASTBool(ASTBool::new(false).unwrap() )),
        ASTNode::ASTBool( ASTBool{id:2,value:false} ) => Some( ASTNode::ASTBool(ASTBool::new(false).unwrap() )),
        ASTNode::ASTString( ASTString{id:3,value} ) if value == "" => Some( ASTNode::ASTBool(ASTBool::new(false).unwrap() )),
        ASTNode::ASTList( ASTList{id:7,length:0,contents}) => Some( ASTNode::ASTBool(ASTBool::new(false).unwrap() )),
        ASTNode::ASTTuple( ASTTuple{id:8,length:0,contents}) => Some( ASTNode::ASTBool(ASTBool::new(false).unwrap() )),
        _ => Some( ASTNode::ASTBool( ASTBool::new(true).unwrap()) )
    }
}
/******************************************************************************/
pub fn promote(type1:&str,type2:&str) -> Option<String> {
    if type1 == "string" && ["string","real","integer","list","tuple","boolean"].contains(&type2) {
        Some(String::from("string"))
    } else if type2 == "string" &&  ["string","real","integer","list","tuple","boolean"].contains(&type1) {
        Some(String::from("string"))
    } else if type1 == "real" && ["real","integer","boolean"].contains(&type2) {
        Some(String::from("real"))
    } else if type2 == "real" && ["real","integer","boolean"].contains(&type1) {
        Some(String::from("real"))
    } else if type1 == "integer" && ["integer","boolean"].contains(&type2) {
        Some(String::from("integer"))
    } else if type2 == "integer" && ["integer","boolean"].contains(&type1) {
        Some(String::from("integer"))
    } else if type1 == "boolean" && type2 == "boolean" {
        Some(String::from("boolean"))
    } else if type1 == "list" && type2 == "list" {
        Some(String::from("list"))
    } else if type1 == "tuple" && type2 == "tuple" {
        Some(String::from("tuple"))
    } else if type1 == "none" && type2 == "none" {
        Some(String::from("none"))
    } else {
        if type1 == type2 {
            panic!("binary operation on type '{}' not supported",type1);
        } else {
            panic!("type '{}' and type '{}' are incompatible", type1, type2);
        }
    }
}
/******************************************************************************/
pub fn term2string<'a>(node: &'a ASTNode) -> Option<String> {
    match node {
        ASTNode::ASTInteger(ASTInteger{id,value}) => Some(value.to_string()),
        ASTNode::ASTReal(ASTReal{id,value}) => Some(value.to_string()),
        ASTNode::ASTBool(ASTBool{id,value}) => Some(value.to_string()),
        ASTNode::ASTString(ASTString{id,value}) => Some(value.clone()),
        ASTNode::ASTLineInfo(ASTLineInfo{id,module,line_number}) => term2string_lineinfo(module,*line_number),
        ASTNode::ASTNone(_) => Some(String::from("None")),
        ASTNode::ASTNil(_) => Some(String::from("Nil")),
        ASTNode::ASTList(_) => term2string_list(node),
        ASTNode::ASTTuple(_) => term2string_tuple(node),
        ASTNode::ASTToList(_) => term2string_tolist(node),
        ASTNode::ASTHeadTail(_) => term2string_headtail(node),
        ASTNode::ASTSequence(_) => term2string_sequence(node),
        ASTNode::ASTFunction(_) => term2string_function(node),
        ASTNode::ASTEval(_) => term2string_eval(node),
        ASTNode::ASTQuote(_) => term2string_quote(node),
        ASTNode::ASTConstraint(_) => term2string_constraint(node),
        ASTNode::ASTTypeMatch(_) => term2string_typematch(node),
        ASTNode::ASTForeign(_) => Some(String::from("Foriegn Object")),
        ASTNode::ASTID(ASTID{id,name}) => Some(name.clone()),
        ASTNode::ASTObject(_) => term2string_object(node),
        ASTNode::ASTApply(_) => term2string_apply(node),
        ASTNode::ASTIndex(_) => term2string_index(node),
        ASTNode::ASTEscape(_) => term2string_escape(node),
        ASTNode::ASTIs(_) => term2string_is(node),
        ASTNode::ASTIn(_) => term2string_in(node),
        ASTNode::ASTIf(_) => term2string_if(node),
        ASTNode::ASTNamedPattern(_) => term2string_namedpattern(node),
        ASTNode::ASTDeref(_) => term2string(node),
        _ => Some(String::from("")),
    }
}
/******************************************************************************/
/*                          TERM2STRING HELPERS                               */
pub fn term2string_lineinfo(module: &str,line_number: usize) -> Option<String> {
    let mut out = String::new();
    out += "lineinfo: module=";
    out += module;
    out += ", line number=";
    out += line_number.to_string().as_str();
    Some(out)
}
pub fn term2string_list<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTList(ASTList{id,length,contents}) = node
        else {panic!("Expected list in term2string_list")};
    
    let mut out = String::new();
    out +="[ ";
    for i in 0..*length {
        out += &term2string(&contents[i]).unwrap();
        if i != (*length-1) {
            out +=" , ";
        }
    }
    out += " ]";
    Some(out)
}
pub fn term2string_tuple<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTTuple(ASTTuple{id,length,contents}) = node
        else {panic!("Expected tuple in term2string_tuple")};

    let mut out = String::new();
    out +="( ";
    for i in 0..*length {
        out += &term2string(&contents[i]).unwrap();
        if i != (*length-1) {
            out +=" , ";
        }
    }
    out += " )";
    Some(out)
}
pub fn term2string_tolist<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTToList(ASTToList{id,start,stop,stride}) = node
        else {panic!("Expected to-list in term2string_tolist")};

    let mut out = String::new();
    out += "to-list: start=";
    out += term2string(&start[0]).unwrap().as_str();
    out += " ,stop=";
    out += term2string(&stop[0]).unwrap().as_str();
    out += " ,stride=";
    out += term2string(&stride[0]).unwrap().as_str();
    Some(out)
}
pub fn term2string_headtail<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTHeadTail(ASTHeadTail{id,head,tail}) = node
        else {panic!("Expected head-tail in term2string_headtail")};

    let mut out = String::new();
    out += "[ ";
    out += term2string(&head[0]).unwrap().as_str();
    out += " | ";
    out += term2string(&tail[0]).unwrap().as_str();
    out += " ]";
    Some(out)
}
pub fn term2string_sequence<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTSequence(ASTSequence{id,first,second}) = node
        else {panic!("Expected sequence in term2string_sequence")};

    let mut out = String::new();
    out += "sequence: first=";
    out += term2string(&first[0]).unwrap().as_str();
    out += ", second=";
    out += term2string(&second[0]).unwrap().as_str();
    Some(out)
}
pub fn term2string_function<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTFunction(ASTFunction{id,body_list}) = node
        else {panic!("Expected function in term2string_function")};

    let mut out = String::new();
    out += "function{ ";
    out += term2string(&ASTNode::ASTList(body_list.clone())).unwrap().as_str();
    out += " }";
    Some(out)
}
pub fn term2string_eval<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTEval(ASTEval{id,expression}) = node
        else {panic!("Expected eval expression in term2string_eval")};

    let mut out = String::new();
    out += "eval( ";
    out += term2string(&expression[0]).unwrap().as_str();
    out += " )";
    Some(out)
}
pub fn term2string_quote<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTQuote(ASTQuote{id,expression}) = node
        else {panic!("Expected quote expression in term2string_quote")};

    let mut out = String::new();
    out += "*";
    out += term2string(&expression[0]).unwrap().as_str();
    Some(out)
}
pub fn term2string_constraint<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTConstraint(ASTConstraint{id,expression}) = node
        else {panic!("Expected constraint expression in term2string_constraint")};

    let mut out = String::new();
    out += "%[ ";
    out += term2string(&expression[0]).unwrap().as_str();
    out += " ]%";
    Some(out)
}
pub fn term2string_typematch<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTTypeMatch(ASTTypeMatch{id,expression}) = node
        else {panic!("Expected typematch expresssion in term2string_typematch")};

    let mut out = String::new();
    out += ":%";
    out += term2string(&expression[0]).unwrap().as_str();
    Some(out)
}
pub fn term2string_object<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTObject(ASTObject{id,struct_id,object_memory}) = node
        else {panic!("Expected object in term2string_object")};

    let mut out = String::new();
    out += &struct_id.get_id().unwrap();
    out += "( ";
    for i in 0..object_memory.get_length().unwrap() {
        if i != 0 {
            out += " , "
        }
        out += &term2string(&object_memory.get_element(i).unwrap()).unwrap();
    }
    out += " )";
    Some(out)
}
pub fn term2string_apply<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTApply(ASTApply{id,argument,function}) = node
        else {panic!("Expected apply expression in term2string_apply")};

    let mut out = String::new();
    out += &term2string(&function[0]).unwrap();
    out += "( ";
    out += &term2string(&argument[0]).unwrap();
    out += " )";
    Some(out)
}
pub fn term2string_index<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTIndex(ASTIndex{id,structure,index_exp}) = node
        else {panic!("Expected index expression in term2string_index")};

    let mut out = String::new();
    out += &term2string(&structure[0]).unwrap();
    out += "@";
    out += &term2string(&index_exp[0]).unwrap();
    Some(out)
}
pub fn term2string_escape<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTEscape(ASTEscape{id,content}) = node
        else {panic!("Expected escape in term2string_escape")};

    let mut out = String::new();
    out += "Escape\"";
    out += &content;
    out += "\"";
    Some(out)
}
pub fn term2string_is<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTIs(ASTIs{id,pattern,term}) = node
        else {panic!("Expected is expression in term2string_is")};

    let mut out = String::new();
    out += &term2string(&pattern[0]).unwrap();
    out += " is ";
    out += &term2string(&term[0]).unwrap();
    Some(out)
}
pub fn term2string_in<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTIn(ASTIn{id,expression,expression_list}) = node
        else {panic!("Expected in expression in term2string_in")};

    let mut out = String::new();
    out += &term2string(&expression[0]).unwrap();
    out += " in ";
    out += &term2string(&expression_list[0]).unwrap();
    Some(out)
}
pub fn term2string_if<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTIf(ASTIf{id,cond_exp,then_exp,else_exp}) = node
        else {panic!("Expected if expression in term2string_if")};

    let mut out = String::new();
    out += "If ";
    out += &term2string(&cond_exp[0]).unwrap();
    out += " then ";
    out += &term2string(&then_exp[0]).unwrap();
    out += " else ";
    out += &term2string(&else_exp[0]).unwrap();

    Some(out)
}
pub fn term2string_namedpattern<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTNamedPattern(ASTNamedPattern{id,name,pattern}) = node
        else {panic!("Expected named pattern in term2string_namedpattern")};

    let mut out = String::new();
    out += &name.get_id().unwrap();
    out += ":%";
    out += &term2string(&pattern[0]).unwrap();

    Some(out)
}
pub fn term2string_deref<'a>(node: &'a ASTNode) -> Option<String> {
    let ASTNode::ASTNamedPattern(ASTNamedPattern{id,name,pattern}) = node
        else {panic!("Expected deref in term2string_deref")};

    let mut out = String::new();
    out += &name.get_id().unwrap();
    out += ":%";
    out += &term2string(&pattern[0]).unwrap();

    Some(out)
}
/******************************************************************************/


/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map2boolean () {
        let int1 = ASTInteger::new(0).unwrap(); //false
        let int2 = ASTInteger::new(1).unwrap(); //true

        let out1 = map2boolean(& ASTNode::ASTInteger(int1)).unwrap();
        let val1 = match out1 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => panic!("Error: test_map2boolean"),
            ASTNode::ASTBool( ASTBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let out2 = map2boolean(& ASTNode::ASTInteger(int2)).unwrap();
        let val2 = match out2 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => (),
            ASTNode::ASTBool( ASTBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let int3 = ASTInteger::new(11).unwrap(); //false
        let int4 = ASTInteger::new(12).unwrap(); //true
        let list1 = ASTList::new(2,vec![ASTNode::ASTInteger(int3),ASTNode::ASTInteger(int4)]).unwrap();
        let out3 = map2boolean(& ASTNode::ASTList(list1)).unwrap();
        let val3 = match out3 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => (),
            ASTNode::ASTBool( ASTBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let list2 = ASTList::new(0,vec![]).unwrap();
        let out4 = map2boolean(& ASTNode::ASTList(list2)).unwrap();
        let val4 = match out4 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => panic!("Error: test_map2boolean"),
            ASTNode::ASTBool( ASTBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let none1 = ASTNone::new().unwrap();
        let nil1 = ASTNil::new().unwrap();
        let out5 = map2boolean(& ASTNode::ASTNone(none1)).unwrap();
        let out6 = map2boolean(& ASTNode::ASTNil(nil1)).unwrap();
        let val5 = match out5 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => panic!("Error: test_map2boolean"),
            ASTNode::ASTBool( ASTBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };
        let val6 = match out6 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => panic!("Error: test_map2boolean"),
            ASTNode::ASTBool( ASTBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let int5 = ASTInteger::new(11).unwrap(); //false
        let int6 = ASTInteger::new(12).unwrap(); //true
        let tuple1 = ASTTuple::new(2,vec![ASTNode::ASTInteger(int5),ASTNode::ASTInteger(int6)]).unwrap();
        let out7 = map2boolean(& ASTNode::ASTTuple(tuple1)).unwrap();
        let val7 = match out7 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => (),
            ASTNode::ASTBool( ASTBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };

        let tuple2 = ASTTuple::new(0,vec![]).unwrap();
        let out8 = map2boolean(& ASTNode::ASTTuple(tuple2)).unwrap();
        let val8 = match out8 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => panic!("Error: test_map2boolean"),
            ASTNode::ASTBool( ASTBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };

        let str1 = ASTString::new(String::from("abc")).unwrap();
        let str2 = ASTString::new(String::from("")).unwrap();
        let out9 = map2boolean(& ASTNode::ASTString(str1)).unwrap();
        let out10 = map2boolean(& ASTNode::ASTString(str2)).unwrap();
        let val9 = match out9 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => (),
            ASTNode::ASTBool( ASTBool{id,value:false}) => panic!("Error: test_map2boolean"),
            _ => panic!("Error: test_map2boolean"),
        };
        let val10 = match out10 {
            ASTNode::ASTBool( ASTBool{id,value:true}) => panic!("Error: test_map2boolean"),
            ASTNode::ASTBool( ASTBool{id,value:false}) => (),
            _ => panic!("Error: test_map2boolean"),
        };
    }
}
