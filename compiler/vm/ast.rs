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
use shared_arena::*;
use std::hash::BuildHasherDefault;
use fnv::FnvHasher;

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
    pub contents: Rc<RefCell<Vec<ArenaRc<Node>>>>,
}
impl AstroList {
    pub fn new( c: Rc<RefCell<Vec<ArenaRc<Node>>>>) -> Self {
        AstroList { contents: c }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a tuple type node
#[derive( Clone,PartialEq)]
pub struct AstroTuple {
    pub contents: Rc<RefCell<Vec<ArenaRc<Node>>>>,
}
impl AstroTuple {
    pub fn new( c: Rc<RefCell<Vec<ArenaRc<Node>>>> ) -> Self {
        AstroTuple{ contents: c }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct AstroToList {
    pub start: ArenaRc<Node>,
    pub stop: ArenaRc<Node>,
    pub stride: ArenaRc<Node>,
}
impl AstroToList {
    pub fn new(start: ArenaRc<Node>, stop: ArenaRc<Node>, stride: ArenaRc<Node>) -> Self {
        AstroToList { start: start, stop: stop, stride: stride }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a head-tail type node
#[derive( Clone,PartialEq)]
pub struct AstroHeadTail {
    pub head: ArenaRc<Node>,
    pub tail: ArenaRc<Node>,
}
impl AstroHeadTail {
    pub fn new(h: ArenaRc<Node>, t: ArenaRc<Node>) -> Self {
        AstroHeadTail { head: h, tail: t}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a taw to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct AstroRawToList {
    pub start: ArenaRc<Node>,
    pub stop: ArenaRc<Node>,
    pub stride: ArenaRc<Node>,
}
impl AstroRawToList {
    pub fn new(start: ArenaRc<Node>, stop: ArenaRc<Node>, stride: ArenaRc<Node>) -> Self {
        AstroRawToList { start: start, stop: stop, stride: stride }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a raw head-tail type node
#[derive( Clone,PartialEq)]
pub struct AstroRawHeadTail {
    pub head: ArenaRc<Node>,
    pub tail: ArenaRc<Node>,
}
impl AstroRawHeadTail {
    pub fn new(h: ArenaRc<Node>, t: ArenaRc<Node>) -> Self {
        AstroRawHeadTail { head: h, tail: t}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a sequence type node
#[derive( Clone,PartialEq)]
pub struct AstroSequence {
    pub first: ArenaRc<Node>,
    pub second: ArenaRc<Node>,
}
impl AstroSequence {
    pub fn new(f: ArenaRc<Node>, s: ArenaRc<Node>) -> Self{
        AstroSequence { first: f, second: s}       
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a function type node
#[derive( Clone,PartialEq)]
pub struct AstroFunction {
    pub body_list: ArenaRc<Node>
}
impl AstroFunction {
    pub fn new(body: ArenaRc<Node>) -> Self{
        AstroFunction { body_list: body}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a function type node
#[derive( Clone,PartialEq)]
pub struct AstroFunctionVal {
    pub body_list: ArenaRc<Node>,
    pub closure: Rc< (Rc<RefCell<Vec<Rc<RefCell<HashMap<String, ArenaRc<Node>, BuildHasherDefault<FnvHasher>>>>>>>,
                      Rc<RefCell<Vec<Rc<RefCell<Vec<String>>>>>>,
                      usize                                                   ) >
    // closure is a reference to a vector(scope levels; 0 is global) of 
    // hashmaps(namespace) mapping strings(tag) to nodes(value) along with a
    // vector of strings(global vars) and a usize(current scope level)
}
impl AstroFunctionVal {
    pub fn new(body_list: ArenaRc<Node>, closure: Rc<(Rc<RefCell<Vec<Rc<RefCell<HashMap<String, ArenaRc<Node>, BuildHasherDefault<FnvHasher>>>>>>>,
                                                Rc<RefCell<Vec<Rc<RefCell<Vec<String>>>>>>,
                                                usize                                                   )>) -> Self{
        AstroFunctionVal { body_list: body_list, closure: closure} 
    }
}
/******************************************************************************/
// // Abstract Syntax Tree representation for a 'evaluate' type node
#[derive( Clone,PartialEq)]
pub struct AstroEval {
    pub expression: ArenaRc<Node>,
}
impl AstroEval {
    pub fn new(expr: ArenaRc<Node>) -> Self{
        AstroEval { expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a quote type node
#[derive( Clone,PartialEq)]
pub struct AstroQuote {
    pub expression: ArenaRc<Node>,
}
impl AstroQuote {
    pub fn new(expr: ArenaRc<Node>) -> Self{
        AstroQuote { expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a constraint type node
#[derive( Clone,PartialEq)]
pub struct AstroConstraint {
    pub expression: ArenaRc<Node>,
}
impl AstroConstraint {
    pub fn new(expr: ArenaRc<Node>) -> Self{
        AstroConstraint { expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a type-match type node
#[derive( Clone,PartialEq)]
pub struct AstroTypeMatch {
    pub expression: ArenaRc<Node>,
}
impl AstroTypeMatch {
    pub fn new(expr: ArenaRc<Node>) -> Self{
        AstroTypeMatch { expression: expr}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a foriegn type node
#[derive( Clone,PartialEq)]
pub struct AstroForeign {
    pub content: String,
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
    pub object_memory: Rc<RefCell<Vec<ArenaRc<Node>>>>,
}
impl AstroObject {
    pub fn new(name: AstroID, mem: Rc<RefCell<Vec<ArenaRc<Node>>>>) -> Self {
        AstroObject { struct_id: name, object_memory: mem}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a apply type node
#[derive( Clone,PartialEq)]
pub struct AstroApply {
    pub function: ArenaRc<Node>,
    pub argument: ArenaRc<Node>,
}
impl AstroApply {
    pub fn new(f: ArenaRc<Node>, a: ArenaRc<Node>) -> Self{
        AstroApply { function: f, argument: a }
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a index type node
#[derive( Clone,PartialEq)]
pub struct AstroIndex {
    pub structure: ArenaRc<Node>,
    pub index_exp: ArenaRc<Node>,
}
impl AstroIndex {
    pub fn new(s: ArenaRc<Node>, i: ArenaRc<Node>) -> Self {
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
    pub pattern: ArenaRc<Node>,
    pub term: ArenaRc<Node>,
}
impl AstroIs {
    pub fn new(p: ArenaRc<Node>, t: ArenaRc<Node>) -> Self {
        AstroIs { pattern: p, term: t} 
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'in' type node
#[derive( Clone,PartialEq)]
pub struct AstroIn {
    pub expression: ArenaRc<Node>,
    pub expression_list: ArenaRc<Node>,
}
impl AstroIn {
    pub fn new(e: ArenaRc<Node>, l: ArenaRc<Node>) -> Self {
        AstroIn { expression: e, expression_list: l}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'if' type node
#[derive( Clone,PartialEq)]
pub struct AstroIf {
    pub cond_exp: ArenaRc<Node>,
    pub then_exp: ArenaRc<Node>,
    pub else_exp: ArenaRc<Node>,
}
impl AstroIf {
    pub fn new(c: ArenaRc<Node>, t: ArenaRc<Node>, e: ArenaRc<Node>) -> Self {
        AstroIf { cond_exp: c, then_exp: t, else_exp: e}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a named-pattern type node
#[derive( Clone,PartialEq)]
pub struct AstroNamedPattern {
    pub name: AstroID,
    pub pattern: ArenaRc<Node>,
}
impl AstroNamedPattern {
    pub fn new(n: AstroID, p: ArenaRc<Node>) -> Self{
        AstroNamedPattern { name: n, pattern: p}
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a dereference type node
#[derive( Clone,PartialEq)]
pub struct AstroDeref {
    pub expression: ArenaRc<Node>,
}
impl AstroDeref {
    pub fn new(e: ArenaRc<Node>) -> Self {
        AstroDeref { expression: e }
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroStruct {
    pub member_names: RefCell<Vec<ArenaRc<Node>>>,
    pub struct_memory: RefCell<Vec<ArenaRc<Node>>>,
}
impl AstroStruct {
    pub fn new(mn: RefCell<Vec<ArenaRc<Node>>>,sm: RefCell<Vec<ArenaRc<Node>>>) -> Self {
        AstroStruct { member_names: mn, struct_memory: sm}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroMemberFunctionVal {
    pub argument: ArenaRc<Node>,
    pub body: ArenaRc<Node>,
}
impl AstroMemberFunctionVal {
    pub fn new(arg: ArenaRc<Node>,body: ArenaRc<Node>) -> Self {
        AstroMemberFunctionVal{argument:arg, body:body}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroData {
    pub value: ArenaRc<Node>,
}
impl AstroData {
    pub fn new( value: ArenaRc<Node> ) -> Self {
        AstroData{value:value}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroUnify {
    pub term: ArenaRc<Node>,
    pub pattern: ArenaRc<Node>,
}
impl AstroUnify {
    pub fn new( term: ArenaRc<Node>, pattern: ArenaRc<Node> ) -> Self {
        AstroUnify{term:term, pattern:pattern}
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroPair {
    pub first: ArenaRc<Node>,
    pub second: ArenaRc<Node>,
}
impl AstroPair {
    pub fn new( first: ArenaRc<Node>, second: ArenaRc<Node> ) -> Self {
        AstroPair{first:first, second:second}
    }
}/******************************************************************************/


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
    AstroPair(AstroPair),
}
/******************************************************************************/
pub fn peek<'a>(node: ArenaRc<Node> ) -> &'a str {
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
        Node::AstroPair(_) => "pair",
    }
}
/******************************************************************************/
pub fn encode_node<'a>(node: &Node ) -> u8 {
    match *node {
        Node::AstroInteger(_) => 0,
        Node::AstroReal(_) => 1,
        Node::AstroBool(_) => 2,
        Node::AstroString(_) => 3,
        Node::AstroLineInfo(_) => 4,
        Node::AstroNone(_) => 5,
        Node::AstroNil(_) => 6,
        Node::AstroList(_) => 7,
        Node::AstroTuple(_) => 8,
        Node::AstroToList(_) => 9,
        Node::AstroHeadTail(_) => 10,
        Node::AstroRawToList(_) => 11,
        Node::AstroRawHeadTail(_) => 12,
        Node::AstroSequence(_) => 13,
        Node::AstroFunction(_) => 14,
        Node::AstroFunctionVal(_) => 14,
        Node::AstroEval(_) => 15,
        Node::AstroQuote(_) => 16,
        Node::AstroConstraint(_) => 17,
        Node::AstroTypeMatch(_) => 18,
        Node::AstroForeign(_) => 19,
        Node::AstroID(_) => 20,
        Node::AstroObject(_) => 21,
        Node::AstroApply(_) => 22,
        Node::AstroIndex(_) => 23,
        Node::AstroEscape(_) => 24,
        Node::AstroIs(_) => 25,
        Node::AstroIn(_) => 26,
        Node::AstroIf(_) => 27,
        Node::AstroNamedPattern(_) => 28,
        Node::AstroDeref(_) => 29,
        Node::AstroStruct(_) => 30,
        Node::AstroMemberFunctionVal(_) => 31,
        Node::AstroData(_) => 32,
        Node::AstroUnify(_) => 33,
        Node::AstroPair(_) => 34,
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
