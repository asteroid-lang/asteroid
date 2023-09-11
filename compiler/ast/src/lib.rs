/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Abstract Syntax Tree Representation Module                                 */
/*                                                                            */
/******************************************************************************/   
#![allow(unused)]

use std::rc::Rc;  // used for astronodes; an astronode may have up to two owners
                  // at a time: The state object and whatever function(s) is/are 
                  // processing the astronode
use std::cell::RefCell;
use std::collections::HashMap;//Symbol Table


/******************************************************************************/
// Abstract Syntax Tree representation for a integer type node
#[derive( Clone,PartialEq)]
pub struct AstroInteger {
    pub id: u8,         // Identifies node type
    pub value: isize,
}
impl AstroInteger {
    pub fn new(v: isize) -> Self {
        AstroInteger { id: 0, value: v }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a real type node
#[derive( Clone,PartialEq)]
pub struct AstroReal {
    pub id: u8,         
    pub value: f64,
}
impl AstroReal {
    pub fn new(v: f64) -> Self {
       AstroReal { id: 1, value: v}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a real boolean node
#[derive( Clone,PartialEq)]
pub struct AstroBool {
    pub id: u8,         
    pub value: bool,   
}
impl AstroBool {
    pub fn new(v: bool) -> Self {
        AstroBool{ id: 2, value: v}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a string type node
#[derive( Clone,PartialEq)]
pub struct AstroString {
    pub id: u8,
    pub value: String
}
impl AstroString {
    pub fn new(v: String) -> Self {
        AstroString { id: 3, value: v}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a line information type node
#[derive( Clone,PartialEq)]
pub struct AstroLineInfo {
    pub id: u8,
    pub module: String,
    pub line_number: usize,
}
impl AstroLineInfo {
    pub fn new(m: String, n: usize) -> Self {
        AstroLineInfo { id: 4, module: m, line_number: n}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a none type node
#[derive( Clone,PartialEq)]
pub struct AstroNone {
    pub id: u8,
}
impl AstroNone {
    pub fn new() -> Self {
        AstroNone { id: 5 }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a nil type node
#[derive( Clone,PartialEq)]
pub struct AstroNil {
    pub id: u8,
}
impl AstroNil {
    pub fn new() -> Self {
        AstroNil { id: 6 }     
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a list type node
#[derive( Clone,PartialEq)]
pub struct AstroList {
    pub id: u8,
    pub length: usize,
    pub contents: Rc<RefCell<Vec<Rc<AstroNode>>>>,
}
impl AstroList {
    pub fn new(l: usize, c: Rc<RefCell<Vec<Rc<AstroNode>>>>) -> Self {
        AstroList { id: 7, length: l, contents: c }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a tuple type node
#[derive( Clone,PartialEq)]
pub struct AstroTuple {
    pub id: u8,
    pub length: usize,
    pub contents: Rc<RefCell<Vec<Rc<AstroNode>>>>,
}
impl AstroTuple {
    pub fn new(l: usize, c: Rc<RefCell<Vec<Rc<AstroNode>>>> ) -> Self {
        AstroTuple{ id: 8, length: l, contents: c }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct AstroToList {
    pub id: u8,
    pub start: Rc<AstroNode>,
    pub stop: Rc<AstroNode>,
    pub stride: Rc<AstroNode>,
}
impl AstroToList {
    pub fn new(start: Rc<AstroNode>, stop: Rc<AstroNode>, stride: Rc<AstroNode>) -> Self {
        AstroToList { id: 9, start: start, stop: stop, stride: stride }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a head-tail type node
#[derive( Clone,PartialEq)]
pub struct AstroHeadTail {
    pub id: u8,
    pub head: Rc<AstroNode>,
    pub tail: Rc<AstroNode>,
}
impl AstroHeadTail {
    pub fn new(h: Rc<AstroNode>, t: Rc<AstroNode>) -> Self {
        AstroHeadTail { id: 10, head: h, tail: t}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a taw to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct AstroRawToList {
    pub id: u8,
    pub start: Rc<AstroNode>,
    pub stop: Rc<AstroNode>,
    pub stride: Rc<AstroNode>,
}
impl AstroRawToList {
    pub fn new(start: Rc<AstroNode>, stop: Rc<AstroNode>, stride: Rc<AstroNode>) -> Self {
        AstroRawToList { id: 11, start: start, stop: stop, stride: stride }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a raw head-tail type node
#[derive( Clone,PartialEq)]
pub struct AstroRawHeadTail {
    pub id: u8,
    pub head: Rc<AstroNode>,
    pub tail: Rc<AstroNode>,
}
impl AstroRawHeadTail {
    pub fn new(h: Rc<AstroNode>, t: Rc<AstroNode>) -> Self {
        AstroRawHeadTail { id: 12, head: h, tail: t}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a sequence type node
#[derive( Clone,PartialEq)]
pub struct AstroSequence {
    pub id: u8,
    pub first: Rc<AstroNode>,
    pub second: Rc<AstroNode>,
}
impl AstroSequence {
    pub fn new(f: Rc<AstroNode>, s: Rc<AstroNode>) -> Self{
        AstroSequence {id: 13, first: f, second: s}       
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a function type node
#[derive( Clone,PartialEq)]
pub struct AstroFunction {
    pub id: u8,
    pub body_list: Rc<AstroNode>
}
impl AstroFunction {
    pub fn new(body: Rc<AstroNode>) -> Self{
        AstroFunction { id: 14, body_list: body}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a function type node
#[derive( Clone,PartialEq)]
pub struct AstroFunctionVal {
    pub id: u8,
    pub body_list: Rc<AstroNode>,
    pub closure: Rc<(Vec<HashMap<String, Rc<AstroNode>>>,Vec<Vec<String>>,usize)>
    // closure is a reference to a vector(scope levels; 0 is global) of 
    // hashmaps(namespace) mapping strings(tag) to astronodes(value) along with a
    // vector of strings(global vars) and a usize(current scope level)
}
impl AstroFunctionVal {
    pub fn new(body_list: Rc<AstroNode>, closure: Rc<(Vec<HashMap<String, Rc<AstroNode>>>,Vec<Vec<String>>,usize)>) -> Self{
        AstroFunctionVal { id: 21, body_list: body_list, closure: closure} 
    }
}
/******************************************************************************/
// // Abstract Syntax Tree representation for a 'evaluate' type node
#[derive( Clone,PartialEq)]
pub struct AstroEval {
    pub id: u8,
    pub expression: Rc<AstroNode>,
}
impl AstroEval {
    pub fn new(expr: Rc<AstroNode>) -> Self{
        AstroEval { id: 15, expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a quote type node
#[derive( Clone,PartialEq)]
pub struct AstroQuote {
    pub id: u8,
    pub expression: Rc<AstroNode>,
}
impl AstroQuote {
    pub fn new(expr: Rc<AstroNode>) -> Self{
        AstroQuote { id: 16, expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a constraint type node
#[derive( Clone,PartialEq)]
pub struct AstroConstraint {
    pub id: u8,
    pub expression: Rc<AstroNode>,
}
impl AstroConstraint {
    pub fn new(expr: Rc<AstroNode>) -> Self{
        AstroConstraint { id: 17, expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a type-match type node
#[derive( Clone,PartialEq)]
pub struct AstroTypeMatch {
    pub id: u8,
    pub expression: Rc<AstroNode>,
}
impl AstroTypeMatch {
    pub fn new(expr: Rc<AstroNode>) -> Self{
        AstroTypeMatch { id: 18, expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a foriegn type node
#[derive( Clone,PartialEq)]
pub struct AstroForeign {
    id: u8,
    content: String,
}
impl AstroForeign {
    pub fn new(c: String) -> Self {
        AstroForeign { id: 19, content: c }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a ID/variable name type node
#[derive( Clone,PartialEq)]
pub struct AstroID {
    pub id: u8,
    pub name: String,
}
impl AstroID {
    pub fn new(s: String) -> Self {
        AstroID {id: 20, name: s}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a object type node
#[derive( Clone,PartialEq)]
pub struct AstroObject {
    pub id: u8,
    pub struct_id: AstroID,
    pub object_memory: Rc<RefCell<Vec<Rc<AstroNode>>>>,
}
impl AstroObject {
    pub fn new(name: AstroID, mem: Rc<RefCell<Vec<Rc<AstroNode>>>>) -> Self {
        AstroObject { id: 22, struct_id: name, object_memory: mem}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a apply type node
#[derive( Clone,PartialEq)]
pub struct AstroApply {
    pub id: u8,
    pub function: Rc<AstroNode>,
    pub argument: Rc<AstroNode>,
}
impl AstroApply {
    pub fn new(f: Rc<AstroNode>, a: Rc<AstroNode>) -> Self{
        AstroApply { id: 23, function: f, argument: a }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a index type node
#[derive( Clone,PartialEq)]
pub struct AstroIndex {
    pub id: u8,
    pub structure: Rc<AstroNode>,
    pub index_exp: Rc<AstroNode>,
}
impl AstroIndex {
    pub fn new(s: Rc<AstroNode>, i: Rc<AstroNode>) -> Self {
        AstroIndex { id: 24, structure: s, index_exp: i}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a escape type node
#[derive( Clone,PartialEq)]
pub struct AstroEscape {
    pub id: u8,
    pub content: String,
}
impl AstroEscape {
    pub fn new(c: String) -> Self{
        AstroEscape { id: 25, content: c}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'is' type node
#[derive( Clone,PartialEq)]
pub struct AstroIs {
    pub id: u8,
    pub pattern: Rc<AstroNode>,
    pub term: Rc<AstroNode>,
}
impl AstroIs {
    pub fn new(p: Rc<AstroNode>, t: Rc<AstroNode>) -> Self {
        AstroIs { id: 26, pattern: p, term: t} 
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'in' type node
#[derive( Clone,PartialEq)]
pub struct AstroIn {
    pub id: u8,
    pub expression: Rc<AstroNode>,
    pub expression_list: Rc<AstroNode>,
}
impl AstroIn {
    pub fn new(e: Rc<AstroNode>, l: Rc<AstroNode>) -> Self {
        AstroIn { id: 27, expression: e, expression_list: l}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'if' type node
#[derive( Clone,PartialEq)]
pub struct AstroIf {
    pub id: u8,
    pub cond_exp: Rc<AstroNode>,
    pub then_exp: Rc<AstroNode>,
    pub else_exp: Rc<AstroNode>,
}
impl AstroIf {
    pub fn new(c: Rc<AstroNode>, t: Rc<AstroNode>, e: Rc<AstroNode>) -> Self {
        AstroIf { id: 28, cond_exp: c, then_exp: t, else_exp: e}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a named-pattern type node
#[derive( Clone,PartialEq)]
pub struct AstroNamedPattern {
    pub id: u8,
    pub name: AstroID,
    pub pattern: Rc<AstroNode>,
}
impl AstroNamedPattern {
    pub fn new(n: AstroID, p: Rc<AstroNode>) -> Self{
        AstroNamedPattern { id: 29, name: n, pattern: p}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a dereference type node
#[derive( Clone,PartialEq)]
pub struct AstroDeref {
    pub id: u8,
    pub expression: Rc<AstroNode>,
}
impl AstroDeref {
    pub fn new(e: Rc<AstroNode>) -> Self {
        AstroDeref { id: 30, expression: e }
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroStruct {
    pub id: u8,
    pub member_names: RefCell<Vec<Rc<AstroNode>>>,
    pub struct_memory: RefCell<Vec<Rc<AstroNode>>>,
}
impl AstroStruct {
    pub fn new(mn: RefCell<Vec<Rc<AstroNode>>>,sm: RefCell<Vec<Rc<AstroNode>>>) -> Self {
        AstroStruct { id: 31, member_names: mn, struct_memory: sm}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroMemberFunctionVal {
    pub id: u8,
    pub argument: Rc<AstroNode>,
    pub body: Rc<AstroNode>,
}
impl AstroMemberFunctionVal {
    pub fn new(arg: Rc<AstroNode>,body: Rc<AstroNode>) -> Self {
        AstroMemberFunctionVal{id:32,argument:arg,body:body}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroData {
    pub id: u8,
    pub value: Rc<AstroNode>,
}
impl AstroData {
    pub fn new( value: Rc<AstroNode> ) -> Self {
        AstroData{id:33,value:value}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroUnify {
    pub id: u8,
    pub term: Rc<AstroNode>,
    pub pattern: Rc<AstroNode>,
}
impl AstroUnify {
    pub fn new( term: Rc<AstroNode>, pattern: Rc<AstroNode> ) -> Self {
        AstroUnify{id:34,term:term,pattern:pattern}
    }
}

/******************************************************************************/
/******************************************************************************/
#[derive( Clone,PartialEq )]
pub enum AstroNode {
    AstroInteger(AstroInteger),
    AstroReal(AstroReal),
    AstroBool(AstroBool),
    AstroString(AstroString),
    AstroLineInfo(AstroLineInfo),
    AstroNone(AstroNone),
    AstroNil(AstroNil),
    AstroList(AstroList),
    AstroTuple(AstroTuple),
    AstroToList(AstroToList),
    AstroHeadTail(AstroHeadTail),
    AstroRawToList(AstroRawToList),
    AstroRawHeadTail(AstroRawHeadTail),
    AstroSequence(AstroSequence),
    AstroFunction(AstroFunction),
    AstroFunctionVal(AstroFunctionVal),
    AstroEval(AstroEval),
    AstroQuote(AstroQuote),
    AstroConstraint(AstroConstraint),
    AstroTypeMatch(AstroTypeMatch),
    AstroForeign(AstroForeign),
    AstroID(AstroID),
    AstroObject(AstroObject),
    AstroApply(AstroApply),
    AstroIndex(AstroIndex),
    AstroEscape(AstroEscape),
    AstroIs(AstroIs),
    AstroIn(AstroIn),
    AstroIf(AstroIf),
    AstroNamedPattern(AstroNamedPattern),
    AstroDeref(AstroDeref),
    AstroStruct(AstroStruct),
    AstroMemberFunctionVal(AstroMemberFunctionVal),
    AstroData(AstroData),
    AstroUnify(AstroUnify),
}
/******************************************************************************/
pub fn peek<'a>(node: Rc<AstroNode> ) -> &'a str {
    match *node {
        AstroNode::AstroInteger(_) => "integer",
        AstroNode::AstroReal(_) => "real",
        AstroNode::AstroBool(_) => "bool",
        AstroNode::AstroString(_) => "string",
        AstroNode::AstroLineInfo(_) => "lineinfo",
        AstroNode::AstroNone(_) => "none",
        AstroNode::AstroNil(_) => "nil",
        AstroNode::AstroList(_) => "list",
        AstroNode::AstroTuple(_) => "tuple",
        AstroNode::AstroToList(_) => "tolist",
        AstroNode::AstroHeadTail(_) => "headtail",
        AstroNode::AstroRawToList(_) => "rawtolist",
        AstroNode::AstroRawHeadTail(_) => "rawheadtail",
        AstroNode::AstroSequence(_) => "sequence",
        AstroNode::AstroFunction(_) => "function",
        AstroNode::AstroFunctionVal(_) => "functionval",
        AstroNode::AstroEval(_) => "eval",
        AstroNode::AstroQuote(_) => "quote",
        AstroNode::AstroConstraint(_) => "constraint",
        AstroNode::AstroTypeMatch(_) => "typematch",
        AstroNode::AstroForeign(_) => "foreign",
        AstroNode::AstroID(_) => "id",
        AstroNode::AstroObject(_) => "object",
        AstroNode::AstroApply(_) => "apply",
        AstroNode::AstroIndex(_) => "index",
        AstroNode::AstroEscape(_) => "escape",
        AstroNode::AstroIs(_) => "is",
        AstroNode::AstroIn(_) => "in",
        AstroNode::AstroIf(_) => "if",
        AstroNode::AstroNamedPattern(_) => "namedpattern",
        AstroNode::AstroDeref(_) => "deref",
        AstroNode::AstroStruct(_) => "struct",
        AstroNode::AstroMemberFunctionVal(_) => "memberfunctionval",
        AstroNode::AstroData(_) => "data",
        AstroNode::AstroUnify(_) => "unify",
    }
}
/******************************************************************************/

#[cfg(test)]
mod tests {
    use super::*;
/*
    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
    */
}
