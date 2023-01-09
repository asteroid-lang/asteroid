/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Abstract Syntax Tree Representation Module                                 */
/*                                                                            */
/******************************************************************************/   

/******************************************************************************/
// Abstract Syntax Tree representation for a integer type node
#[derive( Clone)]
pub struct ASTInteger {
    id: u8,         // Identifies node type
    value: i128,
}
impl ASTInteger {
    pub fn new(v: i128) -> Option<Self> {
        Some(ASTInteger { id: 0, value: v })
    }
    pub fn get(&self) -> Option<i128> {
        Some(self.value)
    }
    pub fn set(&mut self, v: i128) {
        self.value = v;
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a real type node
#[derive( Clone)]
pub struct ASTReal {
    id: u8,         
    value: f64,
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
#[derive( Clone)]
pub struct ASTBool {
    id: u8,         
    value: bool,   
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
#[derive( Clone)]
pub struct ASTString {
    id: u8,
    value: String
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
#[derive( Clone)]
pub struct ASTLineInfo {
    id: u8,
    module: String,
    line_number: u128,
}
impl ASTLineInfo {
    pub fn new(m: String, n: u128) -> Option<Self>{
        Some(ASTLineInfo { id: 4, module: m, line_number: n})
    }
    pub fn get_module(&self) -> Option<String>{
        Some(self.module.clone())
    }
    pub fn set_module(&mut self, m: String) {
        self.module = m;        
    }
    pub fn get_line(&self) -> Option<u128>{
        Some(self.line_number)
    }
    pub fn set_line(&mut self, n: u128) {
        self.line_number = n;
    }
}
/******************************************************************************/
// Abstract Syntax Tree representation for a none type node
#[derive( Clone)]
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
#[derive( Clone)]
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
#[derive( Clone)]
pub struct ASTList {
    id: u8,
    length: u128,
    contents: Vec<ASTNode>,
}
impl ASTList {
    pub fn new(l: u128, c: Vec<ASTNode> ) -> Option<Self>{
        Some(ASTList { id: 7, length: l, contents: c })
    }
    pub fn get_length(&self) -> Option<u128> {
        Some(self.length)
    }
    pub fn set_length(&mut self, l: u128 ) {
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
#[derive( Clone)]
pub struct ASTTuple {
    id: u8,
    length: u128,
    contents: Vec<ASTNode>,
}
impl ASTTuple {
    pub fn new(l: u128, c: Vec<ASTNode> ) -> Option<Self>{
        Some(ASTTuple{ id: 8, length: l, contents: c })
    }
    pub fn get_length(&self) -> Option<u128> {
        Some(self.length)
    }
    pub fn set_length(&mut self, l: u128 ) {
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
#[derive( Clone)]
pub struct ASTToList {
    id: u8,
    start: Vec<ASTNode>,
    stop: Vec<ASTNode>,
    stride: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTHeadTail {
    id: u8,
    head: Vec<ASTNode>,
    tail: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTRawToList {
    id: u8,
    start: Vec<ASTNode>,
    stop: Vec<ASTNode>,
    stride: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTRawHeadTail {
    id: u8,
    head: Vec<ASTNode>,
    tail: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTSequence {
    id: u8,
    first: Vec<ASTNode>,
    second: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTFunction {
    id: u8,
    body_list: ASTList
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
#[derive( Clone)]
pub struct ASTEval {
    id: u8,
    expression: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTQuote {
    id: u8,
    expression: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTConstraint {
    id: u8,
    expression: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTTypeMatch {
    id: u8,
    expression: Vec<ASTNode>,
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
#[derive( Clone)]
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
#[derive( Clone)]
pub struct ASTID {
    id: u8,
    name: String,
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
#[derive( Clone)]
pub struct ASTObject {
    id: u8,
    struct_id: ASTID,
    object_memory: ASTList,
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
#[derive( Clone)]
pub struct ASTApply {
    id: u8,
    function: Vec<ASTNode>,
    argument: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTIndex {
    id: u8,
    structure: Vec<ASTNode>,
    index_exp: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTEscape {
    id: u8,
    content: String,
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
#[derive( Clone)]
pub struct ASTIs {
    id: u8,
    pattern: Vec<ASTNode>,
    term: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTIn {
    id: u8,
    expression: Vec<ASTNode>,
    expression_list: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTIf {
    id: u8,
    cond_exp: Vec<ASTNode>,
    then_exp: Vec<ASTNode>,
    else_exp: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTNamedPattern {
    id: u8,
    name: ASTID,
    pattern: Vec<ASTNode>,
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
#[derive( Clone)]
pub struct ASTDeref {
    id: u8,
    expression: Vec<ASTNode>,
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
/******************************************************************************/
#[derive( Clone)]
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
}
/******************************************************************************/
pub fn peek(node: ASTNode ) -> Option<&'static str> {
    match node {
        ASTInteger => Some("integer"),
        ASTReal => Some("real"),
        ASTBool => Some("bool"),
        ASTString => Some("string"),
        ASTLineInfo => Some("lineinfo"),
        ASTNone => Some("none"),
        ASTNil => Some("nil"),
        ASTList => Some("list"),
        ASTTuple => Some("tuple"),
        ASTToList => Some("tolist"),
        ASTHeadTail => Some("headtail"),
        ASTRawToList => Some("rawtolist"),
        ASTRawHeadTail => Some("rawheadtail"),
        ASTSequence => Some("sequence"),
        ASTFunction => Some("function"),
        ASTEval => Some("eval"),
        ASTQuote => Some("quote"),
        ASTConstraint => Some("constraint"),
        ASTTypeMatch => Some("typematch"),
        ASTForeign => Some("foreign"),
        ASTID => Some("id"),
        ASTObject => Some("object"),
        ASTApply => Some("apply"),
        ASTIndex => Some("index"),
        ASTEscape => Some("escape"),
        ASTIs => Some("is"),
        ASTIn => Some("in"),
        ASTIf => Some("if"),
        ASTNamedPattern => Some("namedpattern"),
        ASTDeref => Some("deref"),
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
