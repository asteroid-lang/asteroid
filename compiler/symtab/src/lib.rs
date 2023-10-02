/******************************************************************************/   
/* Asteroid                                                                   */ 
/* Symtab Module                                                              */
/*                                                                            */
/******************************************************************************/    
#![allow(unused)]

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
extern crate ast;
use ast::*;                   //Asteroid AST representation

/******************************************************************************/
const SCOPES_HINT: usize = 10;
const NAMESPACE_HINT: usize = 32;
const GLOBAL_LVL: usize = 0; 
/******************************************************************************/
#[derive( Clone,PartialEq)]
pub struct Symtab {
    pub scoped_symtab: Rc<RefCell<Vec<Rc<RefCell<HashMap<String, Rc<Node>>>>>>>,//A Vector of hashmaps,
                                // each hashmap represents a namespace/scope.
                                // Keys are strings which represent variable
                                // names and values are Nodes.
    pub globals: Rc<RefCell<Vec<Rc<RefCell<Vec<String>>>>>>,// Vector of vectors of strings. Each internal
                                // vector represents a global namespace/scope 
                                // and its contents indicate all of the 
                                // variables that have been declared global
                                // at that level.
    pub curr_scope: usize,      // Keeps track of the scope level that program
                                // execution is currently happening in.
}
/******************************************************************************/
impl Symtab {
    /**************************************************************************/
    // Constructor : Symtab
    // Returns a new Symtab struct with default field values.
    pub fn new() -> Option<Self> {

        let mut x: Symtab= Symtab {scoped_symtab: Rc::new( RefCell::new( Vec::with_capacity(SCOPES_HINT))),
                                   globals:       Rc::new( RefCell::new( Vec::with_capacity(SCOPES_HINT))),
                                   curr_scope: GLOBAL_LVL                        };
        x.scoped_symtab.borrow_mut().push(Rc::new(RefCell::new( HashMap::with_capacity(NAMESPACE_HINT))));
        x.globals.borrow_mut().push(Rc::new(RefCell::new( Vec::with_capacity(NAMESPACE_HINT))));
        Some(x)
    }
    /**************************************************************************/
    // Function enter_sym enters a id-value pair into the symbol table
    pub fn enter_sym( &mut self, id: &str, value: Rc<Node> ){

        // Check if it already exists in the global table If it does, update 
        // the variable in the global scope; else enter into std scope
        let mut namespace = &self.globals.borrow()[ self.globals.borrow().len() - 1 ];
        let index = namespace.borrow().iter().position(|r| r == id);
        
        match index {
            None => self.scoped_symtab.borrow()[ self.scoped_symtab.borrow().len()-1 ].borrow_mut().insert(String::from(id), value),
            _ => self.scoped_symtab.borrow()[ GLOBAL_LVL ].borrow_mut().insert(String::from(id), value),
        };
    }
    /**************************************************************************/
    // Function find_sym returns the scope level that an id/variable is stored
    // at in the vector of scopes. If multiple instances of an id exist, this 
    // function will return the HIGHEST level that an instance of the variable
    //  is stored at.
    pub fn find_sym( &self, id: &str) -> Option<usize> {
        let n_scopes = &self.scoped_symtab.borrow().len();
        for x in (0..*n_scopes).rev() {    
            match self.scoped_symtab.borrow()[x].borrow().get(id) {
                None => (),
                _ => return Some(x),
            }
        }
        None
    }
    /**************************************************************************/
    // Function lookup_sym returns the value paired with the passed in id in
    // the symbol table. The strict parameter is used to evaluate if this 
    // operation should be able to fail or if it should panic.
    pub fn lookup_sym( &self, id: &str, strict: bool) -> Rc<Node> {
        let scope = self.find_sym(id);
        if let None = scope {
            if strict {
                panic!("'{}' is not defined.",id);
            }
        }
        (self.scoped_symtab.borrow()[scope.unwrap()].borrow().get(id).unwrap()).clone() 
    }
    /**************************************************************************/
    // Function push_scope is used to push a new scope level onto the symbol
    // table.
    pub fn push_scope( &mut self) {

        // push a new dictionary and globals lookup onto the stacks
        self.scoped_symtab.borrow_mut().push( Rc::new( RefCell::new( HashMap::with_capacity(NAMESPACE_HINT))));
        self.globals.borrow_mut().push( Rc::new( RefCell::new( Vec::with_capacity(NAMESPACE_HINT))));
        self.curr_scope += 1;
    }
    /**************************************************************************/
    // Function pop_scope() is used to remove the highest scope level of the
    // symbol table. This funciton will panic if instructed to remove the 
    // global scope.
    pub fn pop_scope( &mut self) {
        if self.scoped_symtab.borrow().len() == 1 {
            panic!("Cannot pop global scope.");
        }
        self.scoped_symtab.borrow_mut().pop();
        self.globals.borrow_mut().pop();
        self.curr_scope -= 1;
    }
    /**************************************************************************/
    // Function update_sym updates a previously stored id-value entry with a 
    // new value to be paired with the id. It is an error to update a non-
    // existant key.
    pub fn update_sym( &mut self, id: &str, value: Rc<Node>) {
        let scope = self.find_sym(id);
        match scope {
            None => panic!("'{}' is not defined.",id),
            Some(v) => self.scoped_symtab.borrow()[v].borrow_mut().insert(String::from(id),value),
        };
    }
    /**************************************************************************/
    // Function is_global returns true is a variable is global in the current
    // scope. 
    pub fn is_global( &self, id: &str) -> bool {
        match self.globals.borrow()[self.curr_scope].borrow().iter().position(|r| r == id) {
            Some(_) => true,
            None => false,
        }
    }
    /**************************************************************************/
    // Function is_local returns true is a variable is global in the current
    // scope. 
    pub fn is_local( &self, id: &str) -> bool {
        match self.scoped_symtab.borrow()[self.curr_scope].borrow().get(id) {
            Some(_) => true,
            None => false,
        }
    }
    /**************************************************************************/
    // Debug function
    // Function dump dumps the complete contents of the symbol table to the 
    // console. TODO print values too.
    pub fn dump(&self) {
        println!("SYMBOL TABLE DUMP");
        let n_scopes = self.scoped_symtab.borrow().len();
        for i in (0..n_scopes).rev() {
            println!("SCOPE LEVEL: {}",i);
            for (key, value) in &*(self.scoped_symtab.borrow())[i].borrow() {
                println!("Found ID: {}: {}", key, peek(Rc::clone(&value)));
                match **value {
                    Node::AstroInteger(AstroInteger{value:a}) => println!("value is {}",a),
                    _ => (),
                };
            }
        }
        for i in (0..n_scopes).rev() {
            println!("GLOBAL SCOPE LEVEL: {}",i);
            for key in &*(self.globals).borrow()[i].borrow() {
                println!("Found ID: {}", key);
            }
        }
    }
    /**************************************************************************/
    // Function set_config is used to update a symbol table with a new set of
    // stacks and current scope flag.
    pub fn set_config(&mut self, local: Rc<RefCell<Vec<Rc<RefCell<HashMap<String, Rc<Node>>>>>>>,
                                 global:Rc<RefCell<Vec<Rc<RefCell<Vec<String>>>>>>,
                                 curr:  usize                                                   ) {
        self.scoped_symtab = local;
        self.globals = global;
        self.curr_scope = curr;
    }
    /**************************************************************************/
    // Function get_config returns a copy of the symbol tables stacks and 
    // current scope flag.
    pub fn get_config( &self)  -> (Rc<RefCell<Vec<Rc<RefCell<HashMap<String, Rc<Node>>>>>>>,
                                   Rc<RefCell<Vec<Rc<RefCell<Vec<String>>>>>>,
                                   usize                                                   ) {
        (Rc::clone(&self.scoped_symtab),Rc::clone(&self.globals),self.curr_scope)
    }
    /**************************************************************************/
    // Function enter_global enters an id unto the current scopes list of 
    // global variables.
    pub fn enter_global(&mut self, id: String) {
        self.globals.borrow()[self.curr_scope].borrow_mut().push(id);
        
    }
    /**************************************************************************/
    // Function inc_scope() increments the symtab's scope level by 1.
    pub fn inc_scope(&mut self) {
        self.curr_scope += 1;
    }
    /**************************************************************************/
    // Function dec_scope() decrements the symtab's scope level by 1.
    pub fn dec_scope(&mut self) {
        self.curr_scope -= 1;
    }
    /**************************************************************************/
    pub fn get_closure( &self)  -> (Rc<RefCell<Vec<Rc<RefCell<HashMap<String, Rc<Node>>>>>>>,
                                    Rc<RefCell<Vec<Rc<RefCell<Vec<String>>>>>>,
                                    usize                                                   ) {

            // Create shallow copies of symbol table and globals
            let mut scopes_copy = vec![];
            let mut globals_copy = vec![];
            for scope in &*self.scoped_symtab.borrow() {
                scopes_copy.push( Rc::clone(&scope) ) 
            }
            for global in &*self.globals.borrow() {
                globals_copy.push( Rc::clone(global) ) 
            }
            (Rc::new(RefCell::new(scopes_copy)),Rc::new(RefCell::new(globals_copy)),self.curr_scope.clone())
        }
    /**************************************************************************/
}
/******************************************************************************/
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut x = Symtab::new().unwrap();

        assert!(x.curr_scope == 0);
        assert!(x.scoped_symtab.len() == 1);
        assert!(x.globals.len() == 1);
    }
    #[test]
    fn test_enter() {
        let mut x = Symtab::new().unwrap();

        let data =  AstroNone::new().unwrap();
        let id = "sample";
        x.enter_sym(id, Node::AstroNone(data));
    }
    #[test]
    fn test_push_scope() {
        let mut x = Symtab::new().unwrap();

        x.push_scope();
        assert_eq!(x.scoped_symtab.len(),2);
        assert_eq!(x.globals.len(),2);
        x.push_scope();
        x.push_scope();
        x.push_scope();
        assert_eq!(x.scoped_symtab.len(),5);
        assert_eq!(x.globals.len(),5);
        x.push_scope();
        x.push_scope();
        x.push_scope();
        x.push_scope();
        x.push_scope();
        x.push_scope();
        assert_eq!(x.scoped_symtab.len(),11); //greater than capacity/must expand
        assert_eq!(x.globals.len(),11);
    }
    #[test]
    fn test_pop_scope() {
        let mut x = Symtab::new().unwrap();

        x.push_scope();
        assert_eq!(x.scoped_symtab.len(),2);
        assert_eq!(x.globals.len(),2);

        x.pop_scope();
        assert_eq!(x.scoped_symtab.len(),1);
        assert_eq!(x.globals.len(),1);
        
        for _ in 1..5 {
            x.push_scope();
        }
        for y in (1..5).rev() {
            x.pop_scope();
            assert_eq!(x.scoped_symtab.len(),y);
            assert_eq!(x.globals.len(),y);
        }
    }
    #[test]
    fn test_scoping() {
        let mut x = Symtab::new().unwrap();

        let data =  ASTNone::new().unwrap();
        let id = "1";
        x.enter_sym(id, ASTNode::ASTNone(data));

        let data =  ASTNone::new().unwrap();
        let id = "2";
        x.enter_sym(id, ASTNode::ASTNone(data));

        let y1 = x.find_sym("1").unwrap();
        let y2 = x.find_sym("2").unwrap();
        let y3 = x.find_sym("3");
        assert_eq!(y1,0);
        assert_eq!(y2,0);
        match y3 {
            None => (),
            _ => panic!("Var 3 should not exist in the table."),
        }

        x.push_scope();
        x.push_scope();
        x.push_scope();
        x.push_scope();
        x.curr_scope = x.curr_scope + 1;
        x.curr_scope = x.curr_scope + 1;
        let data =  ast::ASTNone::new().unwrap();
        let id = "5";
        x.enter_sym(id, ast::ASTNode::ASTNone(data));
        x.curr_scope = x.curr_scope + 1;
        x.curr_scope = x.curr_scope + 1;

        let data =  ast::ASTNone::new().unwrap();
        let id = "4";
        x.enter_sym(id, ast::ASTNode::ASTNone(data));

        let y4 = x.find_sym("4").unwrap();
        assert_eq!(y4,4);
        let y5 = x.find_sym("2").unwrap();
        assert_eq!(y5,0);

        x.pop_scope();
        x.curr_scope = x.curr_scope - 1;
        let y4 = x.find_sym("4");
        match y4 {
            None => (),
            _ => panic!("Var 4 should not exist in the table."),
        }
        let y5 = x.find_sym("2").unwrap();
        assert_eq!(y5,0);
        let y6 = x.find_sym("5").unwrap();
        assert_eq!(y6,2);

    }
    #[test]
    fn test_lookup() {
        let mut x = Symtab::new().unwrap();

        let data =  ast::ASTInteger::new( 654321 ).unwrap();
        let id = "val";
        x.enter_sym(id, ast::ASTNode::ASTInteger(data));

        let mut vout = 0i128;
        let out = x.lookup_sym(id,false).unwrap();
        match out {
            ast::ASTNode::ASTInteger(v) => vout = v.get().unwrap(),
            _ => panic!("test error"),
        };
        assert_eq!(vout,654321);

        x.push_scope();
        x.curr_scope += 1;

        let data =  ast::ASTInteger::new( -123 ).unwrap();
        let id = "val";
        x.enter_sym(id, ast::ASTNode::ASTInteger(data));

        let mut vout = 0i128;
        let out = x.lookup_sym(id,false).unwrap();
        match out {
            ast::ASTNode::ASTInteger(v) => vout = v.get().unwrap(),
            _ => panic!("test error"),
        };
        assert_eq!(vout,-123);

        x.pop_scope();
        x.curr_scope -= 1;

        let mut vout = 0i128;
        let out = x.lookup_sym(id,false).unwrap();
        match out {
            ast::ASTNode::ASTInteger(v) => vout = v.get().unwrap(),
            _ => panic!("test error"),
        };
        assert_eq!(vout,654321);
    }
    #[test]
    fn test_update() {
        let mut x = Symtab::new().unwrap();

        let data =  ast::ASTInteger::new( 654321 ).unwrap();
        let id = "val";
        x.enter_sym(id, ast::ASTNode::ASTInteger(data));

        let mut vout = 0i128;
        let out = x.lookup_sym(id,false).unwrap();
        match out {
            ast::ASTNode::ASTInteger(v) => vout = v.get().unwrap(),
            _ => panic!("test error"),
        };
        assert_eq!(vout,654321);

        let data =  ast::ASTNone::new().unwrap();
        x.update_sym(id, ast::ASTNode::ASTNone(data));

        let out = x.lookup_sym(id,false).unwrap();
        match out {
            ast::ASTNode::ASTInteger(v) => panic!("test error"),
            ast::ASTNode::ASTNone(v) => (),
            _ => panic!("test error"),
        };
    }
}
