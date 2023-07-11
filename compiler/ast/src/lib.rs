/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Abstract Syntax Tree Representation Module                                 */
/*                                                                            */
/******************************************************************************/   
#![allow(unused)]

use std::rc::Rc;  // used for astronodes; an astronode may have up to two owners
                  // at a time: The state object and whatever function(s) is/are 
                  // processing the astronode

/******************************************************************************/
// Abstract Syntax Tree representation for a integer type node
#[derive( Clone,PartialEq)]
pub struct AstroInteger {
    pub id: u8,         // Identifies node type
    pub value: isize,
}
impl AstroInteger {
    pub fn new(v: isize) -> Option<Self> {
        Some(AstroInteger { id: 0, value: v })
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
    pub fn new(v: f64) -> Option<Self> {
        Some( AstroReal { id: 1, value: v} )
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
    pub fn new(v: bool) -> Option<Self>{
        Some(AstroBool{ id: 2, value: v})
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
    pub fn new(v: String) -> Option<Self>{
        Some(AstroString { id: 3, value: v})
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
    pub fn new(m: String, n: usize) -> Option<Self>{
        Some(AstroLineInfo { id: 4, module: m, line_number: n})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a none type node
#[derive( Clone,PartialEq)]
pub struct AstroNone {
    pub id: u8,
}
impl AstroNone {
    pub fn new() -> Option<Self>{
        Some(AstroNone { id: 5 })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a nil type node
#[derive( Clone,PartialEq)]
pub struct AstroNil {
    pub id: u8,
}
impl AstroNil {
    pub fn new() -> Option<Self>{
        Some(AstroNil { id: 6 })        
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a list type node
#[derive( Clone,PartialEq)]
pub struct AstroList {
    pub id: u8,
    pub length: usize,
    pub contents: Vec<Rc<AstroNode>>,
}
impl AstroList {
    pub fn new(l: usize, c: Vec<Rc<AstroNode>> ) -> Option<Self>{
        Some(AstroList { id: 7, length: l, contents: c })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a tuple type node
#[derive( Clone,PartialEq)]
pub struct AstroTuple {
    pub id: u8,
    pub length: usize,
    pub contents: Vec<Rc<AstroNode>>,
}
impl AstroTuple {
    pub fn new(l: usize, c: Vec<Rc<AstroNode>> ) -> Option<Self>{
        Some(AstroTuple{ id: 8, length: l, contents: c })
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
    pub fn new(start: Rc<AstroNode>, stop: Rc<AstroNode>, stride: Rc<AstroNode>) -> Option<Self> {
        Some(AstroToList { id: 9, start: start, stop: stop, stride: stride })
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
    pub fn new(h: Rc<AstroNode>, t: Rc<AstroNode>) -> Option<Self> {
        Some(AstroHeadTail { id: 10, head: h, tail: t})
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
    pub fn new(start: Rc<AstroNode>, stop: Rc<AstroNode>, stride: Rc<AstroNode>) -> Option<Self> {
        Some(AstroRawToList { id: 11, start: start, stop: stop, stride: stride })
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
    pub fn new(h: Rc<AstroNode>, t: Rc<AstroNode>) -> Option<Self> {
        Some(AstroRawHeadTail { id: 12, head: h, tail: t})
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
    pub fn new(f: Rc<AstroNode>, s: Rc<AstroNode>) -> Option<Self>{
        Some(AstroSequence {id: 13, first: f, second: s})        
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a function type node
#[derive( Clone,PartialEq)]
pub struct AstroFunction {
    pub id: u8,
    pub body_list: AstroList
}
impl AstroFunction {
    pub fn new(body: AstroList) -> Option<Self>{
        Some( AstroFunction { id: 14, body_list: body} )
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
    pub fn new(expr: Rc<AstroNode>) -> Option<Self>{
        Some(AstroEval { id: 15, expression: expr})
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
    pub fn new(expr: Rc<AstroNode>) -> Option<Self>{
        Some(AstroQuote { id: 16, expression: expr})
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
    pub fn new(expr: Rc<AstroNode>) -> Option<Self>{
        Some(AstroConstraint { id: 17, expression: expr})
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
    pub fn new(expr: Rc<AstroNode>) -> Option<Self>{
        Some(AstroTypeMatch { id: 18, expression: expr})
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
    pub fn new(c: String) -> Option<Self> {
        Some(AstroForeign { id: 19, content: c })
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
    pub fn new(s: String) -> Option<Self> {
        Some(AstroID {id: 20, name: s})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a object type node
#[derive( Clone,PartialEq)]
pub struct AstroObject {
    pub id: u8,
    pub struct_id: AstroID,
    pub object_memory: Rc<AstroNode>,
}
impl AstroObject {
    pub fn new(name: AstroID, mem: Rc<AstroNode>) -> Option<Self> {
        Some( AstroObject { id: 22, struct_id: name, object_memory: mem} )
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
    pub fn new(f: Rc<AstroNode>, a: Rc<AstroNode>) -> Option<Self>{
        Some( AstroApply { id: 23, function: f, argument: a } )
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
    pub fn new(s: Rc<AstroNode>, i: Rc<AstroNode>) -> Option<Self> {
        Some( AstroIndex { id: 24, structure: s, index_exp: i} )
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
    pub fn new(c: String) -> Option<Self>{
        Some(AstroEscape { id: 25, content: c})
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
    pub fn new(p: Rc<AstroNode>, t: Rc<AstroNode>) -> Option<Self> {
        Some( AstroIs { id: 26, pattern: p, term: t} )
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
    pub fn new(e: Rc<AstroNode>, l: Rc<AstroNode>) -> Option<Self> {
        Some( AstroIn { id: 27, expression: e, expression_list: l} )
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
    pub fn new(c: Rc<AstroNode>, t: Rc<AstroNode>, e: Rc<AstroNode>) -> Option<Self> {
        Some(AstroIf { id: 28, cond_exp: c, then_exp: t, else_exp: e})
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
    pub fn new(n: AstroID, p: Rc<AstroNode>) ->Option<Self>{
        Some( AstroNamedPattern { id: 29, name: n, pattern: p})
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
    pub fn new(e: Rc<AstroNode>) -> Option<Self> {
        Some( AstroDeref { id: 30, expression: e })
    }
}
/******************************************************************************/
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct AstroStruct {
    pub id: u8,
    pub member_names: Rc<AstroNode>,
    pub struct_memory: Rc<AstroNode>
}
impl AstroStruct {
    pub fn new(mn: Rc<AstroNode>,sm: Rc<AstroNode>) -> Option<Self> {
        Some( AstroStruct { id: 31, member_names: mn, struct_memory: sm})
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
    pub fn new(arg: Rc<AstroNode>,body: Rc<AstroNode>) -> Option<Self> {
        Some( AstroMemberFunctionVal{id:32,argument:arg,body:body})
    }
}
/******************************************************************************/



/******************************************************************************/
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
}
/******************************************************************************/
pub fn peek<'a>(node: Rc<AstroNode> ) -> Option<&'a str> {
    match *node {
        AstroNode::AstroInteger(_) => Some("integer"),
        AstroNode::AstroReal(_) => Some("real"),
        AstroNode::AstroBool(_) => Some("bool"),
        AstroNode::AstroString(_) => Some("string"),
        AstroNode::AstroLineInfo(_) => Some("lineinfo"),
        AstroNode::AstroNone(_) => Some("none"),
        AstroNode::AstroNil(_) => Some("nil"),
        AstroNode::AstroList(_) => Some("list"),
        AstroNode::AstroTuple(_) => Some("tuple"),
        AstroNode::AstroToList(_) => Some("tolist"),
        AstroNode::AstroHeadTail(_) => Some("headtail"),
        AstroNode::AstroRawToList(_) => Some("rawtolist"),
        AstroNode::AstroRawHeadTail(_) => Some("rawheadtail"),
        AstroNode::AstroSequence(_) => Some("sequence"),
        AstroNode::AstroFunction(_) => Some("function"),
        AstroNode::AstroEval(_) => Some("eval"),
        AstroNode::AstroQuote(_) => Some("quote"),
        AstroNode::AstroConstraint(_) => Some("constraint"),
        AstroNode::AstroTypeMatch(_) => Some("typematch"),
        AstroNode::AstroForeign(_) => Some("foreign"),
        AstroNode::AstroID(_) => Some("id"),
        AstroNode::AstroObject(_) => Some("object"),
        AstroNode::AstroApply(_) => Some("apply"),
        AstroNode::AstroIndex(_) => Some("index"),
        AstroNode::AstroEscape(_) => Some("escape"),
        AstroNode::AstroIs(_) => Some("is"),
        AstroNode::AstroIn(_) => Some("in"),
        AstroNode::AstroIf(_) => Some("if"),
        AstroNode::AstroNamedPattern(_) => Some("namedpattern"),
        AstroNode::AstroDeref(_) => Some("deref"),
        AstroNode::AstroStruct(_) => Some("struct"),
        AstroNode::AstroMemberFunctionVal(_) => Some("memberfunctionval"),
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
