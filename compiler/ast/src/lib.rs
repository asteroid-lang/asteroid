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
    pub fn get(&self) -> Option<isize> {
        Some(self.value)
    }
    pub fn set(&mut self, v: isize) {
        self.value = v;
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
    pub fn get(&self) -> Option<f64> {
        Some(self.value)
    }
    pub fn set(&mut self, v: f64) {
        self.value = v;
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
    pub fn get(&self) -> Option<bool> {
        Some(self.value)
    }
    pub fn set(&mut self, v: bool) {
        self.value = v;
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
    pub fn get(&self) -> Option<String> {
        Some(self.value.clone())
    }
    pub fn set(&mut self, v: String) {
        self.value = v;
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
    pub fn get_module(&self) -> Option<String>{
        Some(self.module.clone())
    }
    pub fn set_module(&mut self, m: String) {
        self.module = m;        
    }
    pub fn get_line(&self) -> Option<usize>{
        Some(self.line_number)
    }
    pub fn set_line(&mut self, n: usize) {
        self.line_number = n;
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
    pub contents: Vec<ASTNode>,
}
impl ASTList {
    pub fn new(l: usize, c: Vec<ASTNode> ) -> Option<Self>{
        Some(ASTList { id: 7, length: l, contents: c })
    }
    pub fn get_length(&self) -> Option<usize> {
        Some(self.length)
    }
    pub fn set_length(&mut self, l: usize ) {
        self.length = l;    
    }
    pub fn get_element(&self, idx: usize) -> Option<ASTNode>{
        Some(self.contents[idx].clone())
    }
    pub fn set_element(&mut self, idx: usize, e: ASTNode) {
        self.contents[idx] = e;
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a tuple type node
#[derive( Clone,PartialEq)]
pub struct ASTTuple {
    pub id: u8,
    pub length: usize,
    pub contents: Vec<ASTNode>,
}
impl ASTTuple {
    pub fn new(l: usize, c: Vec<ASTNode> ) -> Option<Self>{
        Some(ASTTuple{ id: 8, length: l, contents: c })
    }
    pub fn get_length(&self) -> Option<usize> {
        Some(self.length)
    }
    pub fn set_length(&mut self, l: usize ) {
        self.length = l;    
    }
    pub fn get_element(&self, idx: usize) -> Option<ASTNode>{
        Some(self.contents[idx].clone())
    }
    pub fn set_element(&mut self, idx: usize, e: ASTNode) {
        self.contents[idx] = e;
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct ASTToList {
    pub id: u8,
    pub start: Vec<ASTNode>,
    pub stop: Vec<ASTNode>,
    pub stride: Vec<ASTNode>,
}
impl ASTToList {
    pub fn new(start: Vec<ASTNode>, stop: Vec<ASTNode>, stride: Vec<ASTNode>) -> Option<Self> {
        Some(ASTToList { id: 9, start: start, stop: stop, stride: stride })
    }
    pub fn set_start(&mut self, start: Vec<ASTNode>) {
        self.start = start;
    }
    pub fn  set_stop(&mut self, stop: Vec<ASTNode>) {
        self.stop = stop;
    }
    pub fn set_stride(&mut self, stride: Vec<ASTNode>) {
        self.stride = stride;
    }
    pub fn get_start(&self) -> Option<Vec<ASTNode>>{
        Some(self.start.clone())
    }
    pub fn get_stop(&self) -> Option<Vec<ASTNode>> {
        Some(self.stop.clone())
    }
    pub fn get_stride(&self) -> Option<Vec<ASTNode>> {
        Some(self.stride.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a head-tail type node
#[derive( Clone,PartialEq)]
pub struct ASTHeadTail {
    pub id: u8,
    pub head: Vec<ASTNode>,
    pub tail: Vec<ASTNode>,
}
impl ASTHeadTail {
    pub fn new(h: Vec<ASTNode>, t: Vec<ASTNode>) -> Option<Self> {
        Some(ASTHeadTail { id: 10, head: h, tail: t})
    }
    pub fn set_head(&mut self, h: Vec<ASTNode>) {
        self.head = h;
    }
    pub fn set_tail(&mut self, t: Vec<ASTNode>) {
        self.tail = t
    }
    pub fn get_head(&self) -> Option<Vec<ASTNode>> {
        Some(self.head.clone())
    }
    pub fn get_tail(&self) -> Option<Vec<ASTNode>> {
        Some(self.tail.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a taw to-list constructor type node
#[derive( Clone,PartialEq)]
pub struct ASTRawToList {
    pub id: u8,
    pub start: Vec<ASTNode>,
    pub stop: Vec<ASTNode>,
    pub stride: Vec<ASTNode>,
}
impl ASTRawToList {
    pub fn new(start: Vec<ASTNode>, stop: Vec<ASTNode>, stride: Vec<ASTNode>) -> Option<Self> {
        Some(ASTRawToList { id: 11, start: start, stop: stop, stride: stride })
    }
    pub fn set_start(&mut self, start: Vec<ASTNode>) {
        self.start = start;
    }
    pub fn  set_stop(&mut self, stop: Vec<ASTNode>) {
        self.stop = stop;
    }
    pub fn set_stride(&mut self, stride: Vec<ASTNode>) {
        self.stride = stride;
    }
    pub fn get_start(&self) -> Option<Vec<ASTNode>>{
        Some(self.start.clone())
    }
    pub fn get_stop(&self) -> Option<Vec<ASTNode>> {
        Some(self.stop.clone())
    }
    pub fn get_stride(&self) -> Option<Vec<ASTNode>> {
        Some(self.stride.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a raw head-tail type node
#[derive( Clone,PartialEq)]
pub struct ASTRawHeadTail {
    pub id: u8,
    pub head: Vec<ASTNode>,
    pub tail: Vec<ASTNode>,
}
impl ASTRawHeadTail {
    pub fn new(h: Vec<ASTNode>, t: Vec<ASTNode>) -> Option<Self> {
        Some(ASTRawHeadTail { id: 12, head: h, tail: t})
    }
    pub fn set_head(&mut self, h: Vec<ASTNode>) {
        self.head = h;
    }
    pub fn set_tail(&mut self, t: Vec<ASTNode>) {
        self.tail = t
    }
    pub fn get_head(&self) -> Option<Vec<ASTNode>> {
        Some(self.head.clone())
    }
    pub fn get_tail(&self) -> Option<Vec<ASTNode>> {
        Some(self.tail.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a sequence type node
#[derive( Clone,PartialEq)]
pub struct ASTSequence {
    pub id: u8,
    pub first: Vec<ASTNode>,
    pub second: Vec<ASTNode>,
}
impl ASTSequence {
    pub fn new(f: Vec<ASTNode>, s: Vec<ASTNode>) -> Option<Self>{
        Some(ASTSequence {id: 13, first: f, second: s})        
    }
    pub fn set_first(&mut self, f: Vec<ASTNode>) {
        self.first = f;
    }
    pub fn set_second(&mut self, s: Vec<ASTNode>) {
        self.second = s;
    }
    pub fn get_first(&self) -> Option<Vec<ASTNode>>{
        Some(self.first.clone())
    }
    pub fn get_second(&self) -> Option<Vec<ASTNode>> {
        Some(self.second.clone())
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
    pub fn set_body(&mut self, body: ASTList ) {
        self.body_list = body
    }
    pub fn get_bodu(&self) -> Option<ASTList> {
        Some(self.body_list.clone())
    }
}
/******************************************************************************/
// // Abstract Syntax Tree representation for a 'evaluate' type node
#[derive( Clone,PartialEq)]
pub struct ASTEval {
    pub id: u8,
    pub expression: Vec<ASTNode>,
}
impl ASTEval {
    pub fn new(expr: Vec<ASTNode>) -> Option<Self>{
        Some(ASTEval { id: 15, expression: expr})
    }
    pub fn set_expression(&mut self, expr: Vec<ASTNode>) {
        self.expression = expr;
    }
    pub fn get_expression(&self) -> Option<Vec<ASTNode>> {
        Some(self.expression.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a quote type node
#[derive( Clone,PartialEq)]
pub struct ASTQuote {
    pub id: u8,
    pub expression: Vec<ASTNode>,
}
impl ASTQuote {
    pub fn new(expr: Vec<ASTNode>) -> Option<Self>{
        Some(ASTQuote { id: 16, expression: expr})
    }
    pub fn set_expression(&mut self, expr: Vec<ASTNode>) {
        self.expression = expr;
    }
    pub fn get_expression(&self) -> Option<Vec<ASTNode>> {
        Some(self.expression.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a constraint type node
#[derive( Clone,PartialEq)]
pub struct ASTConstraint {
    pub id: u8,
    pub expression: Vec<ASTNode>,
}
impl ASTConstraint {
    pub fn new(expr: Vec<ASTNode>) -> Option<Self>{
        Some(ASTConstraint { id: 17, expression: expr})
    }
    pub fn set_expression(&mut self, expr: Vec<ASTNode>) {
        self.expression = expr;
    }
    pub fn get_expression(&self) -> Option<Vec<ASTNode>> {
        Some(self.expression.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a type-match type node
#[derive( Clone,PartialEq)]
pub struct ASTTypeMatch {
    pub id: u8,
    pub expression: Vec<ASTNode>,
}
impl ASTTypeMatch {
    pub fn new(expr: Vec<ASTNode>) -> Option<Self>{
        Some(ASTTypeMatch { id: 18, expression: expr})
    }
    pub fn set_expression(&mut self, expr: Vec<ASTNode>) {
        self.expression = expr;
    }
    pub fn get_expression(&self) -> Option<Vec<ASTNode>> {
        Some(self.expression.clone())
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
    pub fn set_content(&mut self, c: String) {
        self.content = c
    }
    pub fn  get_content(&self) -> Option<String>{
        Some( self.content.clone() )
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
    pub fn get_id(&self) -> Option<String> {
        Some( self.name.clone() )
    }
    pub fn set_id(&mut self,s: String) {
        self.name = s;
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
    pub fn set_id(&mut self, name: ASTID) {
        self.struct_id = name;
    }
    pub fn set_memory(&mut self, mem: ASTList) {
        self.object_memory = mem;
    }
    pub fn get_id(&self) -> Option<ASTID> {
        Some(self.struct_id.clone())
    }
    pub fn get_memory(&self) -> Option<ASTList> { 
        Some(self.object_memory.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a apply type node
#[derive( Clone,PartialEq)]
pub struct ASTApply {
    pub id: u8,
    pub function: Vec<ASTNode>,
    pub argument: Vec<ASTNode>,
}
impl ASTApply {
    pub fn new(f: Vec<ASTNode>, a: Vec<ASTNode>) -> Option<Self>{
        Some( ASTApply { id: 23, function: f, argument: a } )
    }
    pub fn set_function(&mut self, f: Vec<ASTNode>) {
        self.function = f;
    }
    pub fn set_argument(&mut self, a: Vec<ASTNode>) {
        self.argument = a;
    }
    pub fn get_function(&self) -> Option<Vec<ASTNode>> {
        Some(self.function.clone())
    }
    pub fn get_arguement(&self) -> Option<Vec<ASTNode>> {
        Some(self.argument.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a index type node
#[derive( Clone,PartialEq)]
pub struct ASTIndex {
    pub id: u8,
    pub structure: Vec<ASTNode>,
    pub index_exp: Vec<ASTNode>,
}
impl ASTIndex {
    pub fn new(s: Vec<ASTNode>, i: Vec<ASTNode>) -> Option<Self> {
        Some( ASTIndex { id: 24, structure: s, index_exp: i} )
    }
    pub fn set_structure(&mut self, s: Vec<ASTNode>) {
        self.structure = s;
    }
    pub fn set_index_exp(&mut self, i: Vec<ASTNode>) {
        self.index_exp = i;
    }
    pub fn get_structure(&self) -> Option<Vec<ASTNode>> {
        Some(self.structure.clone())
    }
    pub fn get_index_exp(&self) -> Option<Vec<ASTNode>> {
        Some(self.index_exp.clone())
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
    pub fn set_content(&mut self, c: String ) {
        self.content = c;
    }
    pub fn get_content(&self) -> Option<String> {
        Some( self.content.clone() )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'is' type node
#[derive( Clone,PartialEq)]
pub struct ASTIs {
    pub id: u8,
    pub pattern: Vec<ASTNode>,
    pub term: Vec<ASTNode>,
}
impl ASTIs {
    pub fn new(p: Vec<ASTNode>, t: Vec<ASTNode>) -> Option<Self> {
        Some( ASTIs { id: 26, pattern: p, term: t} )
    }
    pub fn set_pattern(&mut self, p: Vec<ASTNode>) {
        self.pattern = p;
    }
    pub fn set_term(&mut self, t: Vec<ASTNode>) {
        self.term = t;
    }
    pub fn get_pattern(&self) -> Option<Vec<ASTNode>> {
        Some( self.pattern.clone() )
    }
    pub fn get_term(&self) -> Option<Vec<ASTNode>>{
        Some( self.term.clone() )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'in' type node
#[derive( Clone,PartialEq)]
pub struct ASTIn {
    pub id: u8,
    pub expression: Vec<ASTNode>,
    pub expression_list: Vec<ASTNode>,
}
impl ASTIn {
    pub fn new(e: Vec<ASTNode>, l: Vec<ASTNode>) -> Option<Self> {
        Some( ASTIn { id: 27, expression: e, expression_list: l} )
    }
    pub fn set_expression(&mut self, e: Vec<ASTNode>) {
        self.expression = e;
    }
    pub fn set_expression_list(&mut self, l: Vec<ASTNode>) {
        self.expression_list = l;
    }
    pub fn get_expression(&self) -> Option<Vec<ASTNode>> {
        Some( self.expression.clone() )
    }
    pub fn get_expression_list(&self) -> Option<Vec<ASTNode>> {
        Some( self.expression_list.clone() )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a 'if' type node
#[derive( Clone,PartialEq)]
pub struct ASTIf {
    pub id: u8,
    pub cond_exp: Vec<ASTNode>,
    pub then_exp: Vec<ASTNode>,
    pub else_exp: Vec<ASTNode>,
}
impl ASTIf {
    pub fn new(c: Vec<ASTNode>, t: Vec<ASTNode>, e: Vec<ASTNode>) -> Option<Self> {
        Some(ASTIf { id: 28, cond_exp: c, then_exp: t, else_exp: e})
    }
    pub fn set_cond_exp(&mut self, c: Vec<ASTNode>) {
        self.cond_exp = c;
    }
    pub fn set_then_exp(&mut self, t: Vec<ASTNode>) {
        self.then_exp  = t;
    }
    pub fn set_else_exp(&mut self, e: Vec<ASTNode>) {
        self.else_exp = e;
    }
    pub fn get_cond_exp(&self) -> Option<Vec<ASTNode>>{
        Some( self.cond_exp.clone() )
    }
    pub fn get_then_exp(&self) -> Option<Vec<ASTNode>>{
        Some( self.then_exp.clone())
    }
    pub fn get_else_exp(&self) -> Option<Vec<ASTNode>>{
        Some( self.else_exp.clone())
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a named-pattern type node
#[derive( Clone,PartialEq)]
pub struct ASTNamedPattern {
    pub id: u8,
    pub name: ASTID,
    pub pattern: Vec<ASTNode>,
}
impl ASTNamedPattern {
    pub fn new(n: ASTID, p: Vec<ASTNode>) ->Option<Self>{
        Some( ASTNamedPattern { id: 29, name: n, pattern: p})
    }
    pub fn set_name(&mut self, n : ASTID) {
        self.name = n;
    }
    pub fn set_pattern(&mut self, p: Vec<ASTNode>) {
        self.pattern = p;
    }
    pub fn get_name(&self) -> Option<ASTID> {
        Some( self.name.clone() )
    }
    pub fn get_pattern(&self) -> Option<Vec<ASTNode>> {
        Some( self.pattern.clone() )
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a dereference type node
#[derive( Clone,PartialEq)]
pub struct ASTDeref {
    pub id: u8,
    pub expression: Vec<ASTNode>,
}
impl ASTDeref {
    pub fn new(e: Vec<ASTNode>) -> Option<Self> {
        Some( ASTDeref { id: 30, expression: e })
    }
    pub fn set_expr(&mut self, e: Vec<ASTNode>) {
        self.expression = e;
    }
    pub fn get_expr(&self) -> Option<Vec<ASTNode>> {
        Some( self.expression.clone() )
    }
}
/******************************************************************************/
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct ASTStruct {
    pub id: u8,
    pub member_names: Vec<ASTNode>,
    pub struct_memory: Vec<ASTNode>
}
impl ASTStruct {
    pub fn new(mn: Vec<ASTNode>,sm: Vec<ASTNode>) -> Option<Self> {
        Some( ASTStruct { id: 31, member_names: mn, struct_memory: sm})
    }
}
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct ASTMemberFunctionVal {
    pub id: u8,
    pub argument: Vec<ASTNode>,
    pub body: Vec<ASTNode>,
}
impl ASTMemberFunctionVal {
    pub fn new(arg: Vec<ASTNode>,body: Vec<ASTNode>) -> Option<Self> {
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
pub fn peek<'a>(node: &'a ASTNode ) -> Option<&'static str> {
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
