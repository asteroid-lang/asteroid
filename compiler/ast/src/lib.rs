/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Abstract Syntax Tree Representation Module                                 */
/*                                                                            */
/******************************************************************************/    
#![allow(unused)]

use std::rc::Rc;  // used for nodes; an node may have up to two owners
                  // at a time: The state object and whatever function(s) is/are 
                  // processing the node
use std::cell::RefCell;
use std::collections::HashMap;//Symbol Table


/******************************************************************************/
// Abstract Syntax Tree representation for a integer type node
#[derive( Clone,PartialEq)]
pub struct AstroInteger {
    pub value: isize,
}
impl AstroInteger {
    pub fn new(v: isize) -> Self {
        AstroInteger { value: v }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a real type node
#[derive( Clone,PartialEq)]
pub struct AstroReal {        
    pub value: f64,
}
impl AstroReal {
    pub fn new(v: f64) -> Self {
       AstroReal { value: v}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a real boolean node
#[derive( Clone,PartialEq)]
pub struct AstroBool {    
    pub value: bool,   
}
impl AstroBool {
    pub fn new(v: bool) -> Self {
        AstroBool{ value: v}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a string type node
#[derive( Clone,PartialEq)]
pub struct AstroString {
    pub value: String
}
impl AstroString {
    pub fn new(v: String) -> Self {
        AstroString { value: v}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a line information type node
#[derive( Clone,PartialEq)]
pub struct AstroLineInfo {
    pub module: String,
    pub line_number: usize,
}
impl AstroLineInfo {
    pub fn new(m: String, n: usize) -> Self {
        AstroLineInfo { module: m, line_number: n}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a none type node
#[derive( Clone,PartialEq)]
pub struct AstroNone {}
impl AstroNone {
    pub fn new() -> Self {
        AstroNone {  }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a nil type node
#[derive( Clone,PartialEq)]
pub struct AstroNil {}
impl AstroNil {
    pub fn new() -> Self {
        AstroNil {  }     
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a list type node
#[derive( Clone,PartialEq)]
pub struct AstroList {
    pub contents: Rc<RefCell<Vec<Rc<Node>>>>,
}
impl AstroList {
    pub fn new( c: Rc<RefCell<Vec<Rc<Node>>>>) -> Self {
        AstroList { contents: c }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a tuple type node
#[derive( Clone,PartialEq)]
pub struct AstroTuple {
    pub contents: Rc<RefCell<Vec<Rc<Node>>>>,
}
impl AstroTuple {
    pub fn new( c: Rc<RefCell<Vec<Rc<Node>>>> ) -> Self {
        AstroTuple{ contents: c }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct AstroToList {
    pub start: Rc<Node>,
    pub stop: Rc<Node>,
    pub stride: Rc<Node>,
}
impl AstroToList {
    pub fn new(start: Rc<Node>, stop: Rc<Node>, stride: Rc<Node>) -> Self {
        AstroToList { start: start, stop: stop, stride: stride }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a head-tail type node
#[derive( Clone,PartialEq)]
pub struct AstroHeadTail {
    pub head: Rc<Node>,
    pub tail: Rc<Node>,
}
impl AstroHeadTail {
    pub fn new(h: Rc<Node>, t: Rc<Node>) -> Self {
        AstroHeadTail { head: h, tail: t}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a taw to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct AstroRawToList {
    pub start: Rc<Node>,
    pub stop: Rc<Node>,
    pub stride: Rc<Node>,
}
impl AstroRawToList {
    pub fn new(start: Rc<Node>, stop: Rc<Node>, stride: Rc<Node>) -> Self {
        AstroRawToList { start: start, stop: stop, stride: stride }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a raw head-tail type node
#[derive( Clone,PartialEq)]
pub struct AstroRawHeadTail {
    pub head: Rc<Node>,
    pub tail: Rc<Node>,
}
impl AstroRawHeadTail {
    pub fn new(h: Rc<Node>, t: Rc<Node>) -> Self {
        AstroRawHeadTail { head: h, tail: t}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a sequence type node
#[derive( Clone,PartialEq)]
pub struct AstroSequence {
    pub first: Rc<Node>,
    pub second: Rc<Node>,
}
impl AstroSequence {
    pub fn new(f: Rc<Node>, s: Rc<Node>) -> Self{
        AstroSequence { first: f, second: s}       
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a function type node
#[derive( Clone,PartialEq)]
pub struct AstroFunction {
    pub body_list: Rc<Node>
}
impl AstroFunction {
    pub fn new(body: Rc<Node>) -> Self{
        AstroFunction { body_list: body}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a function type node
#[derive( Clone,PartialEq)]
pub struct AstroFunctionVal {
    pub body_list: Rc<Node>,
    pub closure: Rc< (Rc<RefCell<Vec<Rc<RefCell<HashMap<String, Rc<Node>>>>>>>,
                      Rc<RefCell<Vec<Rc<RefCell<Vec<String>>>>>>,
                      usize                                                   ) >
    // closure is a reference to a vector(scope levels; 0 is global) of 
    // hashmaps(namespace) mapping strings(tag) to nodes(value) along with a
    // vector of strings(global vars) and a usize(current scope level)
}
impl AstroFunctionVal {
    pub fn new(body_list: Rc<Node>, closure: Rc<(Rc<RefCell<Vec<Rc<RefCell<HashMap<String, Rc<Node>>>>>>>,
                                                Rc<RefCell<Vec<Rc<RefCell<Vec<String>>>>>>,
                                                usize                                                   )>) -> Self{
        AstroFunctionVal { body_list: body_list, closure: closure} 
    }
}
/******************************************************************************/
// // Abstract Syntax Tree representation for a 'evaluate' type node
#[derive( Clone,PartialEq)]
pub struct AstroEval {
    pub expression: Rc<Node>,
}
impl AstroEval {
    pub fn new(expr: Rc<Node>) -> Self{
        AstroEval { expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a quote type node
#[derive( Clone,PartialEq)]
pub struct AstroQuote {
    pub expression: Rc<Node>,
}
impl AstroQuote {
    pub fn new(expr: Rc<Node>) -> Self{
        AstroQuote { expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a constraint type node
#[derive( Clone,PartialEq)]
pub struct AstroConstraint {
    pub expression: Rc<Node>,
}
impl AstroConstraint {
    pub fn new(expr: Rc<Node>) -> Self{
        AstroConstraint { expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a type-match type node
#[derive( Clone,PartialEq)]
pub struct AstroTypeMatch {
    pub expression: Rc<Node>,
}
impl AstroTypeMatch {
    pub fn new(expr: Rc<Node>) -> Self{
        AstroTypeMatch { expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a foriegn type node
#[derive( Clone,PartialEq)]
pub struct AstroForeign {
    content: String,
}
impl AstroForeign {
    pub fn new(c: String) -> Self {
        AstroForeign { content: c }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a ID/variable name type node
#[derive( Clone,PartialEq)]
pub struct AstroID {
    pub name: String,
}
impl AstroID {
    pub fn new(s: String) -> Self {
        AstroID { name: s}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a object type node
#[derive( Clone,PartialEq)]
pub struct AstroObject {
    pub struct_id: AstroID,
    pub object_memory: Rc<RefCell<Vec<Rc<Node>>>>,
}
impl AstroObject {
    pub fn new(name: AstroID, mem: Rc<RefCell<Vec<Rc<Node>>>>) -> Self {
        AstroObject { struct_id: name, object_memory: mem}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a apply type node
#[derive( Clone,PartialEq)]
pub struct AstroApply {
    pub function: Rc<Node>,
    pub argument: Rc<Node>,
}
impl AstroApply {
    pub fn new(f: Rc<Node>, a: Rc<Node>) -> Self{
        AstroApply { function: f, argument: a }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a index type node
#[derive( Clone,PartialEq)]
pub struct AstroIndex {
    pub structure: Rc<Node>,
    pub index_exp: Rc<Node>,
}
impl AstroIndex {
    pub fn new(s: Rc<Node>, i: Rc<Node>) -> Self {
        AstroIndex { structure: s, index_exp: i}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a escape type node
#[derive( Clone,PartialEq)]
pub struct AstroEscape {
    pub content: String,
}
impl AstroEscape {
    pub fn new(c: String) -> Self{
        AstroEscape { content: c}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'is' type node
#[derive( Clone,PartialEq)]
pub struct AstroIs {
    pub pattern: Rc<Node>,
    pub term: Rc<Node>,
}
impl AstroIs {
    pub fn new(p: Rc<Node>, t: Rc<Node>) -> Self {
        AstroIs { pattern: p, term: t} 
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'in' type node
#[derive( Clone,PartialEq)]
pub struct AstroIn {
    pub expression: Rc<Node>,
    pub expression_list: Rc<Node>,
}
impl AstroIn {
    pub fn new(e: Rc<Node>, l: Rc<Node>) -> Self {
        AstroIn { expression: e, expression_list: l}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'if' type node
#[derive( Clone,PartialEq)]
pub struct AstroIf {
    pub cond_exp: Rc<Node>,
    pub then_exp: Rc<Node>,
    pub else_exp: Rc<Node>,
}
impl AstroIf {
    pub fn new(c: Rc<Node>, t: Rc<Node>, e: Rc<Node>) -> Self {
        AstroIf { cond_exp: c, then_exp: t, else_exp: e}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a named-pattern type node
#[derive( Clone,PartialEq)]
pub struct AstroNamedPattern {
    pub name: AstroID,
    pub pattern: Rc<Node>,
}
impl AstroNamedPattern {
    pub fn new(n: AstroID, p: Rc<Node>) -> Self{
        AstroNamedPattern { name: n, pattern: p}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a dereference type node
#[derive( Clone,PartialEq)]
pub struct AstroDeref {
    pub expression: Rc<Node>,
}
impl AstroDeref {
    pub fn new(e: Rc<Node>) -> Self {
        AstroDeref { expression: e }
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroStruct {
    pub member_names: RefCell<Vec<Rc<Node>>>,
    pub struct_memory: RefCell<Vec<Rc<Node>>>,
}
impl AstroStruct {
    pub fn new(mn: RefCell<Vec<Rc<Node>>>,sm: RefCell<Vec<Rc<Node>>>) -> Self {
        AstroStruct { member_names: mn, struct_memory: sm}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroMemberFunctionVal {
    pub argument: Rc<Node>,
    pub body: Rc<Node>,
}
impl AstroMemberFunctionVal {
    pub fn new(arg: Rc<Node>,body: Rc<Node>) -> Self {
        AstroMemberFunctionVal{argument:arg, body:body}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroData {
    pub value: Rc<Node>,
}
impl AstroData {
    pub fn new( value: Rc<Node> ) -> Self {
        AstroData{value:value}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroUnify {
    pub term: Rc<Node>,
    pub pattern: Rc<Node>,
}
impl AstroUnify {
    pub fn new( term: Rc<Node>, pattern: Rc<Node> ) -> Self {
        AstroUnify{term:term, pattern:pattern}
    }
}

/******************************************************************************/
/******************************************************************************/
#[derive( Clone,PartialEq )]
pub enum Node {
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
pub fn peek<'a>(node: Rc<Node> ) -> &'a str {
    match *node {
        Node::AstroInteger(_) => "integer",
        Node::AstroReal(_) => "real",
        Node::AstroBool(_) => "bool",
        Node::AstroString(_) => "string",
        Node::AstroLineInfo(_) => "lineinfo",
        Node::AstroNone(_) => "none",
        Node::AstroNil(_) => "nil",
        Node::AstroList(_) => "list",
        Node::AstroTuple(_) => "tuple",
        Node::AstroToList(_) => "tolist",
        Node::AstroHeadTail(_) => "headtail",
        Node::AstroRawToList(_) => "rawtolist",
        Node::AstroRawHeadTail(_) => "rawheadtail",
        Node::AstroSequence(_) => "sequence",
        Node::AstroFunction(_) => "function",
        Node::AstroFunctionVal(_) => "functionval",
        Node::AstroEval(_) => "eval",
        Node::AstroQuote(_) => "quote",
        Node::AstroConstraint(_) => "constraint",
        Node::AstroTypeMatch(_) => "typematch",
        Node::AstroForeign(_) => "foreign",
        Node::AstroID(_) => "id",
        Node::AstroObject(_) => "object",
        Node::AstroApply(_) => "apply",
        Node::AstroIndex(_) => "index",
        Node::AstroEscape(_) => "escape",
        Node::AstroIs(_) => "is",
        Node::AstroIn(_) => "in",
        Node::AstroIf(_) => "if",
        Node::AstroNamedPattern(_) => "namedpattern",
        Node::AstroDeref(_) => "deref",
        Node::AstroStruct(_) => "struct",
        Node::AstroMemberFunctionVal(_) => "memberfunctionval",
        Node::AstroData(_) => "data",
        Node::AstroUnify(_) => "unify",
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
