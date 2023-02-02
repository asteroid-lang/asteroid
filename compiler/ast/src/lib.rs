/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Abstract Syntax Tree Representation Module                                 */
/*                                                                            */
/******************************************************************************/   
#![allow(unused)]

/******************************************************************************/
// Abstract Syntax Tree representation for a integer type node
#[derive( Clone,PartialEq)]
pub struct ASTInteger {
    pub id: u8,         // Identifies node type
    pub value: isize,
}
impl ASTInteger {
    pub fn new(v: isize) -> Option<Self> {
        Some(ASTInteger { id: 0, value: v })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a real type node
#[derive( Clone,PartialEq)]
pub struct ASTReal {
    pub id: u8,         
    pub value: f64,
}
impl ASTReal {
    pub fn new(v: f64) -> Option<Self> {
        Some( ASTReal { id: 1, value: v} )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a real boolean node
#[derive( Clone,PartialEq)]
pub struct ASTBool {
    pub id: u8,         
    pub value: bool,   
}
impl ASTBool {
    pub fn new(v: bool) -> Option<Self>{
        Some(ASTBool{ id: 2, value: v})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a string type node
#[derive( Clone,PartialEq)]
pub struct ASTString {
    pub id: u8,
    pub value: String
}
impl ASTString {
    pub fn new(v: String) -> Option<Self>{
        Some(ASTString { id: 3, value: v})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a line information type node
#[derive( Clone,PartialEq)]
pub struct ASTLineInfo {
    pub id: u8,
    pub module: String,
    pub line_number: usize,
}
impl ASTLineInfo {
    pub fn new(m: String, n: usize) -> Option<Self>{
        Some(ASTLineInfo { id: 4, module: m, line_number: n})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a none type node
#[derive( Clone,PartialEq)]
pub struct ASTNone {
    id: u8,
}
impl ASTNone {
    pub fn new() -> Option<Self>{
        Some(ASTNone { id: 5 })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a nil type node
#[derive( Clone,PartialEq)]
pub struct ASTNil {
    id: u8,
}
impl ASTNil {
    pub fn new() -> Option<Self>{
        Some(ASTNil { id: 6 })        
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a list type node
#[derive( Clone,PartialEq)]
pub struct ASTList {
    pub id: u8,
    pub length: usize,
    pub contents: Vec<Box<ASTNode>>,
}
impl ASTList {
    pub fn new(l: usize, c: Vec<Box<ASTNode>> ) -> Option<Self>{
        Some(ASTList { id: 7, length: l, contents: c })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a tuple type node
#[derive( Clone,PartialEq)]
pub struct ASTTuple {
    pub id: u8,
    pub length: usize,
    pub contents: Vec<Box<ASTNode>>,
}
impl ASTTuple {
    pub fn new(l: usize, c: Vec<Box<ASTNode>> ) -> Option<Self>{
        Some(ASTTuple{ id: 8, length: l, contents: c })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct ASTToList {
    pub id: u8,
    pub start: Box<ASTNode>,
    pub stop: Box<ASTNode>,
    pub stride: Box<ASTNode>,
}
impl ASTToList {
    pub fn new(start: Box<ASTNode>, stop: Box<ASTNode>, stride: Box<ASTNode>) -> Option<Self> {
        Some(ASTToList { id: 9, start: start, stop: stop, stride: stride })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a head-tail type node
#[derive( Clone,PartialEq)]
pub struct ASTHeadTail {
    pub id: u8,
    pub head: Box<ASTNode>,
    pub tail: Box<ASTNode>,
}
impl ASTHeadTail {
    pub fn new(h: Box<ASTNode>, t: Box<ASTNode>) -> Option<Self> {
        Some(ASTHeadTail { id: 10, head: h, tail: t})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a taw to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct ASTRawToList {
    pub id: u8,
    pub start: Box<ASTNode>,
    pub stop: Box<ASTNode>,
    pub stride: Box<ASTNode>,
}
impl ASTRawToList {
    pub fn new(start: Box<ASTNode>, stop: Box<ASTNode>, stride: Box<ASTNode>) -> Option<Self> {
        Some(ASTRawToList { id: 11, start: start, stop: stop, stride: stride })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a raw head-tail type node
#[derive( Clone,PartialEq)]
pub struct ASTRawHeadTail {
    pub id: u8,
    pub head: Box<ASTNode>,
    pub tail: Box<ASTNode>,
}
impl ASTRawHeadTail {
    pub fn new(h: Box<ASTNode>, t: Box<ASTNode>) -> Option<Self> {
        Some(ASTRawHeadTail { id: 12, head: h, tail: t})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a sequence type node
#[derive( Clone,PartialEq)]
pub struct ASTSequence {
    pub id: u8,
    pub first: Box<ASTNode>,
    pub second: Box<ASTNode>,
}
impl ASTSequence {
    pub fn new(f: Box<ASTNode>, s: Box<ASTNode>) -> Option<Self>{
        Some(ASTSequence {id: 13, first: f, second: s})        
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a function type node
#[derive( Clone,PartialEq)]
pub struct ASTFunction {
    pub id: u8,
    pub body_list: ASTList
}
impl ASTFunction {
    pub fn new(body: ASTList) -> Option<Self>{
        Some( ASTFunction { id: 14, body_list: body} )
    }
}
/******************************************************************************/
// // Abstract Syntax Tree representation for a 'evaluate' type node
#[derive( Clone,PartialEq)]
pub struct ASTEval {
    pub id: u8,
    pub expression: Box<ASTNode>,
}
impl ASTEval {
    pub fn new(expr: Box<ASTNode>) -> Option<Self>{
        Some(ASTEval { id: 15, expression: expr})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a quote type node
#[derive( Clone,PartialEq)]
pub struct ASTQuote {
    pub id: u8,
    pub expression: Box<ASTNode>,
}
impl ASTQuote {
    pub fn new(expr: Box<ASTNode>) -> Option<Self>{
        Some(ASTQuote { id: 16, expression: expr})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a constraint type node
#[derive( Clone,PartialEq)]
pub struct ASTConstraint {
    pub id: u8,
    pub expression: Box<ASTNode>,
}
impl ASTConstraint {
    pub fn new(expr: Box<ASTNode>) -> Option<Self>{
        Some(ASTConstraint { id: 17, expression: expr})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a type-match type node
#[derive( Clone,PartialEq)]
pub struct ASTTypeMatch {
    pub id: u8,
    pub expression: Box<ASTNode>,
}
impl ASTTypeMatch {
    pub fn new(expr: Box<ASTNode>) -> Option<Self>{
        Some(ASTTypeMatch { id: 18, expression: expr})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a foriegn type node
#[derive( Clone,PartialEq)]
pub struct ASTForeign {
    id: u8,
    content: String,
}
impl ASTForeign {
    pub fn new(c: String) -> Option<Self> {
        Some(ASTForeign { id: 19, content: c })
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a ID/variable name type node
#[derive( Clone,PartialEq)]
pub struct ASTID {
    pub id: u8,
    pub name: String,
}
impl ASTID {
    pub fn new(s: String) -> Option<Self> {
        Some(ASTID {id: 20, name: s})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a object type node
#[derive( Clone,PartialEq)]
pub struct ASTObject {
    pub id: u8,
    pub struct_id: ASTID,
    pub object_memory: ASTList,
}
impl ASTObject {
    pub fn new(name: ASTID, mem: ASTList) -> Option<Self> {
        Some( ASTObject { id: 22, struct_id: name, object_memory: mem} )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a apply type node
#[derive( Clone,PartialEq)]
pub struct ASTApply {
    pub id: u8,
    pub function: Box<ASTNode>,
    pub argument: Box<ASTNode>,
}
impl ASTApply {
    pub fn new(f: Box<ASTNode>, a: Box<ASTNode>) -> Option<Self>{
        Some( ASTApply { id: 23, function: f, argument: a } )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a index type node
#[derive( Clone,PartialEq)]
pub struct ASTIndex {
    pub id: u8,
    pub structure: Box<ASTNode>,
    pub index_exp: Box<ASTNode>,
}
impl ASTIndex {
    pub fn new(s: Box<ASTNode>, i: Box<ASTNode>) -> Option<Self> {
        Some( ASTIndex { id: 24, structure: s, index_exp: i} )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a escape type node
#[derive( Clone,PartialEq)]
pub struct ASTEscape {
    pub id: u8,
    pub content: String,
}
impl ASTEscape {
    pub fn new(c: String) -> Option<Self>{
        Some(ASTEscape { id: 25, content: c})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'is' type node
#[derive( Clone,PartialEq)]
pub struct ASTIs {
    pub id: u8,
    pub pattern: Box<ASTNode>,
    pub term: Box<ASTNode>,
}
impl ASTIs {
    pub fn new(p: Box<ASTNode>, t: Box<ASTNode>) -> Option<Self> {
        Some( ASTIs { id: 26, pattern: p, term: t} )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'in' type node
#[derive( Clone,PartialEq)]
pub struct ASTIn {
    pub id: u8,
    pub expression: Box<ASTNode>,
    pub expression_list: Box<ASTNode>,
}
impl ASTIn {
    pub fn new(e: Box<ASTNode>, l: Box<ASTNode>) -> Option<Self> {
        Some( ASTIn { id: 27, expression: e, expression_list: l} )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'if' type node
#[derive( Clone,PartialEq)]
pub struct ASTIf {
    pub id: u8,
    pub cond_exp: Box<ASTNode>,
    pub then_exp: Box<ASTNode>,
    pub else_exp: Box<ASTNode>,
}
impl ASTIf {
    pub fn new(c: Box<ASTNode>, t: Box<ASTNode>, e: Box<ASTNode>) -> Option<Self> {
        Some(ASTIf { id: 28, cond_exp: c, then_exp: t, else_exp: e})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a named-pattern type node
#[derive( Clone,PartialEq)]
pub struct ASTNamedPattern {
    pub id: u8,
    pub name: ASTID,
    pub pattern: Box<ASTNode>,
}
impl ASTNamedPattern {
    pub fn new(n: ASTID, p: Box<ASTNode>) ->Option<Self>{
        Some( ASTNamedPattern { id: 29, name: n, pattern: p})
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a dereference type node
#[derive( Clone,PartialEq)]
pub struct ASTDeref {
    pub id: u8,
    pub expression: Box<ASTNode>,
}
impl ASTDeref {
    pub fn new(e: Box<ASTNode>) -> Option<Self> {
        Some( ASTDeref { id: 30, expression: e })
    }
}
/******************************************************************************/
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct ASTStruct {
    pub id: u8,
    pub member_names: Box<ASTNode>,
    pub struct_memory: Box<ASTNode>
}
impl ASTStruct {
    pub fn new(mn: Box<ASTNode>,sm: Box<ASTNode>) -> Option<Self> {
        Some( ASTStruct { id: 31, member_names: mn, struct_memory: sm})
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct ASTMemberFunctionVal {
    pub id: u8,
    pub argument: Box<ASTNode>,
    pub body: Box<ASTNode>,
}
impl ASTMemberFunctionVal {
    pub fn new(arg: Box<ASTNode>,body: Box<ASTNode>) -> Option<Self> {
        Some( ASTMemberFunctionVal{id:32,argument:arg,body:body})
    }
}
/******************************************************************************/



/******************************************************************************/
/******************************************************************************/
/******************************************************************************/
#[derive( Clone,PartialEq )]
pub enum ASTNode {
    ASTInteger(ASTInteger),
    ASTReal(ASTReal),
    ASTBool(ASTBool),
    ASTString(ASTString),
    ASTLineInfo(ASTLineInfo),
    ASTNone(ASTNone),
    ASTNil(ASTNil),
    ASTList(ASTList),
    ASTTuple(ASTTuple),
    ASTToList(ASTToList),
    ASTHeadTail(ASTHeadTail),
    ASTRawToList(ASTRawToList),
    ASTRawHeadTail(ASTRawHeadTail),
    ASTSequence(ASTSequence),
    ASTFunction(ASTFunction),
    ASTEval(ASTEval),
    ASTQuote(ASTQuote),
    ASTConstraint(ASTConstraint),
    ASTTypeMatch(ASTTypeMatch),
    ASTForeign(ASTForeign),
    ASTID(ASTID),
    ASTObject(ASTObject),
    ASTApply(ASTApply),
    ASTIndex(ASTIndex),
    ASTEscape(ASTEscape),
    ASTIs(ASTIs),
    ASTIn(ASTIn),
    ASTIf(ASTIf),
    ASTNamedPattern(ASTNamedPattern),
    ASTDeref(ASTDeref),
    ASTStruct(ASTStruct),
    ASTMemberFunctionVal(ASTMemberFunctionVal),
}
/******************************************************************************/
pub fn peek<'a>(node: &'a ASTNode ) -> Option<&'a str> {
    match node {
        ASTNode::ASTInteger(_) => Some("integer"),
        ASTNode::ASTReal(_) => Some("real"),
        ASTNode::ASTBool(_) => Some("bool"),
        ASTNode::ASTString(_) => Some("string"),
        ASTNode::ASTLineInfo(_) => Some("lineinfo"),
        ASTNode::ASTNone(_) => Some("none"),
        ASTNode::ASTNil(_) => Some("nil"),
        ASTNode::ASTList(_) => Some("list"),
        ASTNode::ASTTuple(_) => Some("tuple"),
        ASTNode::ASTToList(_) => Some("tolist"),
        ASTNode::ASTHeadTail(_) => Some("headtail"),
        ASTNode::ASTRawToList(_) => Some("rawtolist"),
        ASTNode::ASTRawHeadTail(_) => Some("rawheadtail"),
        ASTNode::ASTSequence(_) => Some("sequence"),
        ASTNode::ASTFunction(_) => Some("function"),
        ASTNode::ASTEval(_) => Some("eval"),
        ASTNode::ASTQuote(_) => Some("quote"),
        ASTNode::ASTConstraint(_) => Some("constraint"),
        ASTNode::ASTTypeMatch(_) => Some("typematch"),
        ASTNode::ASTForeign(_) => Some("foreign"),
        ASTNode::ASTID(_) => Some("id"),
        ASTNode::ASTObject(_) => Some("object"),
        ASTNode::ASTApply(_) => Some("apply"),
        ASTNode::ASTIndex(_) => Some("index"),
        ASTNode::ASTEscape(_) => Some("escape"),
        ASTNode::ASTIs(_) => Some("is"),
        ASTNode::ASTIn(_) => Some("in"),
        ASTNode::ASTIf(_) => Some("if"),
        ASTNode::ASTNamedPattern(_) => Some("namedpattern"),
        ASTNode::ASTDeref(_) => Some("deref"),
        ASTNode::ASTStruct(_) => Some("struct"),
        ASTNode::ASTMemberFunctionVal(_) => Some("memberfunctionval"),
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
