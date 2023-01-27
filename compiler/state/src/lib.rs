/******************************************************************************/   
/* Asteroid                                                                   */ 
/* State Module                                                               */
/*                                                                            */
/******************************************************************************/ 
#![allow(unused)]

use symtab::*;  //Asteroid symbol table
use ast::*;     //Asteroid AST representation

// guesstimate for the number of modules an Asteroid program will have.
const MODULES_HINT: usize = 8; 
/******************************************************************************/  
#[derive( Clone,PartialEq)]                         
pub struct State {
    pub symbol_table: symtab::Symtab,   // Symbol table
    modules: Vec<String>,           // List of currently loaded modules
    ast: ast::ASTNode,              // Abstrat syntax tree
    ignore_quote: bool,             // flags when to ignore quoted vars
    constraint_lvl: usize,          // indicated current constraint bracket
                                    // depth level.
    cond_warning: bool,             // Flags when a conditional pattern warning
                                    // has already been displayed. Used to 
                                    // prevent repeating the same warning.
    eval_redundancy: bool,          // Flags is we should evaluate overlapping
                                    // patterns. This turns the redundant 
                                    // pattern detector on or off.
    lineinfo: (String,usize),       // Used to know what module/line number is
                                    // currently being executed.
    
}
impl State {
    /**************************************************************************/
    // New object constructor, returns a new state struct with default values
    pub fn new() -> Option<Self> {
        Some( State { symbol_table: symtab::Symtab::new().unwrap(),
                      modules: Vec::with_capacity(MODULES_HINT),
                      ast: ast::ASTNode::ASTNone(ast::ASTNone::new().unwrap()),
                      ignore_quote: false,
                      constraint_lvl: 0,
                      cond_warning: false,
                      eval_redundancy: true,
                      lineinfo: (String::from("<input>"),1),                   } )
    }
    /**************************************************************************/
    // Getter : &symbol_table
    // Retrieves a reference to the states symbol table. Variable/value pairs are
    // directly stored in the symbol table. 
    pub fn get_symbol_table(&self) -> Option<&Symtab> {
        Some( &self.symbol_table )
    }
    /**************************************************************************/
    // Getter : &self.modules
    // Retrieves a reference to the modules vector. A name:&String entry for
    // every module loaded in the state is stored in this vector.  
    pub fn get_modules(&self) -> Option<&Vec<String>> {
        Some( &self.modules)
    }  
    /**************************************************************************/
    // Function add_module() is used to add a new module name to the list 
    // of loaded modules.
    pub fn add_module(&mut self, new: &str) {
        self.modules.push( String::from(new) );
    }
    /**************************************************************************/
    // Setter : ast
    // Sets the ast field with a new ASTNode.
    pub fn set_ast(&mut self, ast: ast::ASTNode) {
        self.ast = ast;
    }
    /**************************************************************************/
    // Getter : ast
    // Retrieves a reference to the ast field. This is a representation of the
    // state's program's Abstract Syntax Tree. details in 'ast' module.
    pub fn get_ast(&mut self) -> Option<&ASTNode>{
        Some( &self.ast )
    }
    /**************************************************************************/
    // Setter : ignore_quote -> true
    // Function ignore_quote_on sets the states ignore_quote field to true. This
    // lets a program know when to ignore quoted/dereferenced pattern variables.
    pub fn ignore_quote_on(&mut self) {
        self.ignore_quote = true;
    }
    /**************************************************************************/
    // Setter : ignore_quote -> false
    // Function ignore_quote_on sets the states ignore_quote field to false. 
    // This lets a program know when to ignore quoted/dereferenced pattern 
    // variables.
    pub fn ignore_quote_off(&mut self) {
        self.ignore_quote = false;
    }
    /**************************************************************************/
    // Getter : ignore_quote
    // Retrieves the current value of the ignore_quote field.
    pub fn get_ignore_quote(&self) -> Option<bool> {
        Some( self.ignore_quote )
    }
    /**************************************************************************/
    // Function inc_constraint_lvl increments the state's constraint level 
    // field by 1. 
    pub fn inc_constraint_lvl(&mut self) {
        self.constraint_lvl += 1;
    }
    /**************************************************************************/
    // Function inc_constraint_lvl deccrements the state's constraint level 
    // field by 1. 
    pub fn dec_constraint_lvl(&mut self) {
        if self.constraint_lvl == 0 {
            panic!("STATE ERROR: cannot go below consrtaint level 0.");
        }
        self.constraint_lvl -= 1;
    }
    /**************************************************************************/
    // Getter : constraint_lvl
    // Retrieves the current value of the state's constraint_lvl field.
    pub fn get_constraint_lvl(&self) -> Option<usize> {
        Some(self.constraint_lvl)
    }
    /**************************************************************************/
    // Setter : cond_warning -> true
    // Sets the value of the state's cond_warning field to be true.
    pub fn cond_warning_on(&mut self) {
        self.cond_warning = true;
    }
    /**************************************************************************/
    // Setter : cond_warning -> false
    // Sets the value of the state's cond_warning field to be false.
    pub fn cond_warning_off(&mut self) {
        self.cond_warning = false;
    }
    /**************************************************************************/
    // Getter : cond_warning
    // Retrieves the current value of the states cond_warning field.
    pub fn get_cond_warning(&self) -> Option<bool> {
        Some( self.cond_warning )
    }
    /**************************************************************************/
    // Setter : eval_redundancy -> true
    // Sets the state's eval_redundancy field to true.
    pub fn eval_redundancy_on(&mut self) {
        self.eval_redundancy = true;
    }
    /**************************************************************************/
    // Setter : eval_redundancy -> false
    // Sets the state's eval_redundancy field to false. 
    pub fn eval_redundancy_off(&mut self) {
        self.eval_redundancy = false;
    }
    /**************************************************************************/
    // Getter : eval_redundancy
    // Gets the current value of the state's eval_redundancy field.
    pub fn get_eval_redundancy(&self) -> Option<bool> {
        Some( self.eval_redundancy )
    }
    /**************************************************************************/
    // Getter : lineinfo
    // Retrieves the current value of the states lineinfo field.
    pub fn get_lineinfo(&self) -> Option<&(String,usize)> {
        Some( &self.lineinfo )
    }
    /**************************************************************************/
    // Setter : lineinfo
    // Sets the state's lineinfo field to the passed in lineinfo tuple.
    pub fn set_lineinfo(&mut self, li: (String,usize)) {
        self.lineinfo = li;
    }
    /**************************************************************************/
    // Function warning used to print warning message to console.
    pub fn warning( &self, msg:&str ){
        let (module,lineno) = self.get_lineinfo().unwrap();
        println!("Warning: {}: {}: {}",module,lineno,msg);
    }
    /**************************************************************************/
    pub fn lookup_sym( &self, id: &str, strict: bool) -> Option<&ASTNode> {
        self.symbol_table.lookup_sym(id,strict)
    }
    /**************************************************************************/
    pub fn enter_sym( &mut self, id: &str, value: ASTNode ){
        self.symbol_table.enter_sym(id,value);
    }
    /**************************************************************************/
    pub fn find_sym( &self, id: &str) -> Option<usize> {
        self.symbol_table.find_sym(id)
    }
    /**************************************************************************/
    pub fn push_scope( &mut self ){
        self.symbol_table.push_scope();
    }
    /**************************************************************************/
    pub fn pop_scope( &mut self ){
        self.symbol_table.pop_scope();
    }
    /**************************************************************************/
} 
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_modules() {
        let mut x = State::new().unwrap();

        let s1 = "module1";
        let s2 = "module2";
        x.add_module(s1);
        x.add_module(s2);

        let y1 = x.get_modules().unwrap();
        assert_eq!(y1[0],s1);
        assert_eq!(y1[1],s2);
    }
    #[test]
    fn test_quote() {
        let mut x = State::new().unwrap();

        assert_eq!(x.ignore_quote,false);
        x.ignore_quote_on();
        assert_eq!(x.ignore_quote,true);
        x.ignore_quote_off();
        assert_eq!(x.ignore_quote,false);
    }
    #[test]
    fn test_constraint() {
        let mut x = State::new().unwrap();

        assert_eq!(x.constraint_lvl,0);
        x.inc_constraint_lvl();
        assert_eq!(x.constraint_lvl,1);
        x.inc_constraint_lvl();
        assert_eq!(x.constraint_lvl,2);
        x.inc_constraint_lvl();
        assert_eq!(x.constraint_lvl,3);
        x.dec_constraint_lvl();
        assert_eq!(x.constraint_lvl,2);
        x.inc_constraint_lvl();
        assert_eq!(x.constraint_lvl,3);
        x.dec_constraint_lvl();
        assert_eq!(x.constraint_lvl,2);
        x.dec_constraint_lvl();
        assert_eq!(x.constraint_lvl,1);
        x.dec_constraint_lvl();
        assert_eq!(x.constraint_lvl,0);
    }
    #[test]
    fn test_cond() {
        let mut x = State::new().unwrap();

        assert_eq!(x.get_cond_warning().unwrap(),false);
        x.cond_warning_on();
        assert_eq!(x.get_cond_warning().unwrap(),true);
        x.cond_warning_off();
        assert_eq!(x.get_cond_warning().unwrap(),false);
    }
    #[test]
    fn test_eval() {
        let mut x = State::new().unwrap();

        assert_eq!(x.get_eval_redundancy().unwrap(),true);
        x.eval_redundancy_off();
        assert_eq!(x.get_eval_redundancy().unwrap(),false);
        x.eval_redundancy_on();
        assert_eq!(x.get_eval_redundancy().unwrap(),true);
    }
    #[test]
    fn test_lineinfo() {
        let mut x = State::new().unwrap();

        let y1 = (String::from("<input1>"),123);
        let y2 = (String::from("<file2>"),321);
        assert_eq!(*x.get_lineinfo().unwrap(),(String::from("<input>"),1usize));

        x.set_lineinfo(y1.clone());
        assert_eq!(*x.get_lineinfo().unwrap(),y1);
        x.set_lineinfo(y2.clone());
        assert_eq!(*x.get_lineinfo().unwrap(),y2);

        x.warning("THIS IS A TEST!.");
    }
}
