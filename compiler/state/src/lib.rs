/******************************************************************************/   
/* Asteroid                                                                   */ 
/* State Module                                                               */
/*                                                                            */
/******************************************************************************/ 
#![allow(unused)]

use std::collections::HashMap;
extern crate symtab;
use symtab::*;   //Asteroid symbol table
extern crate ast;
use ast::*;      //Asteroid AST representation
use std::rc::Rc; 
/******************************************************************************/
// All of the AVM error and exception types 
pub enum Error {
    ValueError( Rc<Node> ),
    PatternMatchFailed( Rc<Node> ),
    RedundantPatternFound( Rc<Node> ),
    NonLinearPattern( Rc<Node> ),
    ArithmeticError( Rc<Node> ),
    FileNotFound( Rc<Node> ),
    VMError( Rc<Node> ),
}

// guesstimate for the number of modules an Asteroid program will have.
const MODULES_HINT: usize = 8; 
/******************************************************************************/  
#[derive( Clone )]                         
pub struct State {
    pub symbol_table: symtab::Symtab,// Symbol table
    pub modules: Vec<String>,        // List of currently loaded modules
    pub ast: Node,              // Abstrat syntax tree
    pub ignore_quote: bool,          // flags when to ignore quoted vars
    pub constraint_lvl: usize,       // indicated current constraint bracket
                                     // depth level.
    pub cond_warning: bool,          // Flags when a conditional pattern warning
                                     // has already been displayed. Used to 
                                     // prevent repeating the same warning.
    pub eval_redundancy: bool,       // Flag for evaluating overlapping
                                     // patterns. This turns the redundant 
                                     // pattern detector on or off.
    pub lineinfo: (String,usize),    // Used to know what module/line number is
                                     // currently being executed.
    pub dispatch_table: HashMap<String, fn( node: Rc<Node>, state: &mut State ) -> Result<Rc<Node>, Rc<Node>>>,
                                     // Dispatch table for function calls. Maps 
                                     // strings to functions. 
}
impl State {
    /**************************************************************************/
    // New object constructor, returns a new state struct with default values
    pub fn new() -> Option<Self> {
        Some( State { symbol_table: symtab::Symtab::new().unwrap(),
                      modules: Vec::with_capacity(MODULES_HINT),
                      ast: Node::AstroNone(ast::AstroNone::new()),
                      ignore_quote: false,
                      constraint_lvl: 0,
                      cond_warning: false,
                      eval_redundancy: true,
                      lineinfo: (String::from("<input>"),1),
                      dispatch_table: HashMap::new(),                  })
    }
    /**************************************************************************/
    // Function add_module() is used to add a new module name to the list 
    // of loaded modules.
    pub fn add_module(&mut self, new: &str) {
        self.modules.push( String::from(new) );
    }
    /**************************************************************************/
    // Function ignore_quote_on sets the states ignore_quote field to true. This
    // lets a program know when to ignore quoted/dereferenced pattern variables.
    pub fn ignore_quote_on(&mut self) {
        self.ignore_quote = true;
    }
    /**************************************************************************/
    // Function ignore_quote_on sets the states ignore_quote field to false. 
    // This lets a program know when to ignore quoted/dereferenced pattern 
    // variables.
    pub fn ignore_quote_off(&mut self) {
        self.ignore_quote = false;
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
            panic!("STATE ERROR: cannot go below constraint level 0.");
        }
        self.constraint_lvl -= 1;
    }
    /**************************************************************************/
    /**************************************************************************/
    /**************************************************************************/
    /**************************************************************************/
    // Function warning used to print warning message to console.
    pub fn warning( &self, msg:&str ){
        let (module,lineno) = &self.lineinfo;
        println!("Warning: {}: {}: {}",module,lineno,msg);
    }
    /**************************************************************************/
    pub fn lookup_sym( &self, id: &str, strict: bool) -> Rc<Node> {
        self.symbol_table.lookup_sym(id,strict)
    }
    /**************************************************************************/
    pub fn enter_sym( &mut self, id: &str, value: Rc<Node> ){
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
