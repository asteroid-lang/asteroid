#![allow(unused)]

// Jemalloc project / alternate malloc implementation
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use std::rc::Rc;                // Rust's Refernce counted smart pointers; used by state
use std::cell::RefCell;         // Rust's mutable reference counted smart pointers; used by state
use std::collections::HashMap;  // Rust's hashmap; used by state
use std::ptr;                   // standard pointer

use shared_arena::*; //Arena_rc is a arena which utilizes reference counting pointers
// **NOTE** ArenaRC::clone(&data[]) DOES NOT DEEP COPY but increases reference count

use state::*;     //Asteroid state representation
use symtab::*;    //Asteroid symbol table
use ast::*;       //Asteroid AST representation
use support::*;   //Asteroid support functions
use avm::*;       //Asteroid virtual machine

static mut POOL: *mut Vec<ArenaRc<Node>> = ptr::null_mut(); // global vector to hold ASTs

fn main() {

    // initialize memory and state
    let mut memory: Arena<Node> = Arena::new(); 
    let mut state = State::new().unwrap();

    let mut data; // master data pointer

    /**********************************************************************************************************************
    **********************************************************************************************************************/
    // unsafe: construction of a global list
    unsafe { 
    
        if POOL.is_null() {
            POOL = Box::into_raw(Box::new(Vec::new()));
        }

        data = &mut *POOL;

        data.push(memory.alloc_rc(Node::AstroInteger(AstroInteger::new(100)) )); //integer: 100 / index: 0
        data.push(memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)) )); //integer: 1 / index: 1
        data.push(memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)) )); //integer: 1 / index: 2
        data.push(memory.alloc_rc(Node::AstroReal(AstroReal::new(0.0)) )); //real: 0.0 / index: 3
        data.push(memory.alloc_rc(Node::AstroReal(AstroReal::new(-1.0)) )); //real: 1.0 / index: 4
        data.push(memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"toosimple.ast".to_string(),line_number:4}))); //lineinfo / index: 5
        data.push(memory.alloc_rc(Node::AstroRawToList(AstroRawToList::new( ArenaRc::clone(&data[1]), ArenaRc::clone(&data[0]), ArenaRc::clone(&data[2]))))); //rawtolist / index: 6
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"n".to_string()}))); // id: n / index: 7
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"sum".to_string()}))); // id: sum / index: 8
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"flip".to_string()}))); // id: flip / index: 9
        data.push(memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"toosimple.ast".to_string(),line_number:5}))); //lineinfo / index: 10
        data.push(memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"toosimple.ast".to_string(),line_number:6}))); //lineinfo / index: 11
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"n".to_string()}))); // id: n  / index: 12
        data.push(memory.alloc_rc(Node::AstroInteger(AstroInteger::new(1)) )); //integer: 1 / index: 13
        data.push(memory.alloc_rc(Node::AstroPair( AstroPair{first:  ArenaRc::clone(&data[12]),second: ArenaRc::clone(&data[13])}))); //pair n,1 / index: 14
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"__minus__".to_string()}))); // id: minus  / index: 15
        data.push(memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&data[15]),ArenaRc::clone(&data[14]))))); // apply: n-1 / index: 16
        data.push(memory.alloc_rc(Node::AstroRawToList(AstroRawToList::new( ArenaRc::clone(&data[1]), ArenaRc::clone(&data[16]), ArenaRc::clone(&data[2]))))); //rawtolist / index: 17
        data.push(memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"toosimple.ast".to_string(),line_number:8}))); //lineinfo / index: 18
        data.push(memory.alloc_rc(Node::AstroReal(AstroReal::new(-1.0)) )); //real: -1.0 / index: 19
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"__times__".to_string()}))); // id: multiply  / index: 20
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"flip".to_string()}))); // id: flip / index: 21
        data.push(memory.alloc_rc(Node::AstroPair( AstroPair{first:  ArenaRc::clone(&data[19]),second: ArenaRc::clone(&data[21])}))); //pair flip,-1.0 / index: 22
        data.push(memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&data[20]),ArenaRc::clone(&data[22]))))); // apply: flip*-1.0 / index: 23
        data.push(memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"toosimple.ast".to_string(),line_number:10}))); //lineinfo / index: 24
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"i".to_string()}))); // id: i / index: 25
        data.push(memory.alloc_rc(Node::AstroReal(AstroReal::new(1.0)) )); //real: 1.0 / index: 26
        data.push(memory.alloc_rc(Node::AstroReal(AstroReal::new(2.0)) )); //real: 2.0 / index: 27
        data.push(memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"toosimple.ast".to_string(),line_number:9}))); //lineinfo / index: 28
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"flip".to_string()}))); // id: flip / index: 29
        data.push(memory.alloc_rc(Node::AstroPair( AstroPair{first:  ArenaRc::clone(&data[27]),second: ArenaRc::clone(&data[25])}))); //pair i,2.0 / index: 30
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"__times__".to_string()}))); // id: multiply  / index: 31
        data.push(memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&data[31]),ArenaRc::clone(&data[30]))))); // apply: i*2.0 / index: 32
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"__minus__".to_string()}))); // id: minus  / index: 33
        data.push(memory.alloc_rc(Node::AstroPair( AstroPair{first: ArenaRc::clone(&data[32]),second: ArenaRc::clone(&data[26])}))); //pair 2.0*i,1.0 / index: 34
        data.push(memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&data[33]),ArenaRc::clone(&data[34]))))); // apply: i*2.0-1.0 / index: 35
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"flip".to_string()}))); // id: flip / index: 36
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"__divide__".to_string()}))); // id: divide  / index: 37
        data.push(memory.alloc_rc(Node::AstroPair( AstroPair{first: ArenaRc::clone(&data[36]),second: ArenaRc::clone(&data[35])}))); //pair flip, (2.0 * i - 1.0) / index: 38
        data.push(memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&data[37]),ArenaRc::clone(&data[38]))))); // apply: flip / (2.0 * i - 1.0) / index: 39
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"sum".to_string()}))); // id: sum / index: 40
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"__plus__".to_string()}))); // id: plus  / index: 41
        data.push(memory.alloc_rc(Node::AstroPair( AstroPair{first: ArenaRc::clone(&data[40]),second: ArenaRc::clone(&data[39])}))); //pair sum, flip / (2.0 * i - 1.0) / index: 42
        data.push(memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&data[41]),ArenaRc::clone(&data[42]))))); // apply: sum + flip / (2.0 * i - 1.0) / index: 43
        data.push(memory.alloc_rc(Node::AstroLineInfo( AstroLineInfo{module:"toosimple.ast".to_string(),line_number:11}))); //lineinfo / index: 44
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"sum".to_string()}))); // id: sum / index: 45
        data.push(memory.alloc_rc(Node::AstroReal(AstroReal::new(4.0)) )); //real: 4.0 / index: 46
        data.push(memory.alloc_rc(Node::AstroID(AstroID{name:"__times__".to_string()}))); // id: plus  / index: 47
        data.push(memory.alloc_rc(Node::AstroPair( AstroPair{first: ArenaRc::clone(&data[45]),second: ArenaRc::clone(&data[46])}))); //pair sum, 4.0 / index: 48
        data.push(memory.alloc_rc(Node::AstroApply(AstroApply::new(ArenaRc::clone(&data[47]),ArenaRc::clone(&data[48]))))); // apply: sum * 4.0 / index: 49
        data.push(memory.alloc_rc(Node::AstroID(AstroID::new("kind".to_string())))); // id: kind / index 50
        data.push(memory.alloc_rc(Node::AstroID(AstroID::new("val".to_string())))); // id: val / index 51
        data.push(memory.alloc_rc(Node::AstroID(AstroID::new("__init__".to_string())))); // id: __init__ / index 52
        data.push(memory.alloc_rc(Node::AstroID(AstroID::new("_ast3".to_string())))); // id: _ast / index 53
        data.push(memory.alloc_rc(Node::AstroFunction(AstroFunction::new(ArenaRc::clone(&data[53]))))); // func // index 54
        data.push(memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&data[50]))))); // data / index 55
        data.push(memory.alloc_rc(Node::AstroData(AstroData::new(ArenaRc::clone(&data[51]))))); // data / index 56
        data.push(memory.alloc_rc(Node::AstroUnify(AstroUnify::new(ArenaRc::clone(&data[52]),ArenaRc::clone(&data[54]))))); // unify __init__ to function body / index 57
    }

    /**********************************************************************************************************************
    **********************************************************************************************************************/
    // PROLOGUE
    // strucutre def for exception
    let member_list = vec![ ArenaRc::clone(&data[55]), ArenaRc::clone(&data[56]), ArenaRc::clone(&data[57]) ];
    let mut struct_memory: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
    let mut member_names: RefCell<Vec<ArenaRc<Node>>> = RefCell::new(vec![]);
    for member in member_list {
        let _type = peek( ArenaRc::clone(&member) );
        if _type == "data" {
            let Node::AstroData(AstroData{value:ref id_node}) = *member
                else {panic!("ERROR: object construction: expected object data.")};
            let Node::AstroID(AstroID{name:ref val}) = ** id_node
                else {panic!("ERROR: object construction: expected ID.")};
            struct_memory.borrow_mut().push( memory.alloc_rc(Node::AstroNone(AstroNone::new())) );
            member_names.borrow_mut().push( ArenaRc::clone(&id_node));
        } else if _type == "unify" {
            let Node::AstroUnify(AstroUnify{term:ref id_node,pattern:ref function_exp}) = *member
                else {panic!("ERROR: object construction: expection unify node.")};
            let function_val = match walk( ArenaRc::clone(&function_exp), &mut state, &mut memory ) {
                Ok( val ) => val,
                Err ( e ) => panic!("error!"),
            };
            struct_memory.borrow_mut().push( ArenaRc::clone( &function_val ));
            member_names.borrow_mut().push( ArenaRc::clone(&id_node));
        } else if _type == "noop" {
            ;// pass
        } else {
            panic!("{}: {}: {}: {}","ValueError",state.lineinfo.0,state.lineinfo.1,format!("unsupported struct member {}",_type));
        }
    }

    let struct_type = memory.alloc_rc(Node::AstroStruct(AstroStruct::new(RefCell::clone(&member_names),RefCell::clone(&struct_memory))));
    state.enter_sym( "Exception", ArenaRc::clone(&struct_type)  );

    /**********************************************************************************************************************
    **********************************************************************************************************************/
    // let n = tointeger(os@argv@1).
    // this one I have to lie because we have not implemented the os module yet/ i set n to 100
    set_lineinfo(  ArenaRc::clone(&data[5]) ,  &mut state, &mut memory ); 

    let exp_val = match walk( ArenaRc::clone(&data[0]), &mut state, &mut memory ) {
        Ok( val ) => val,
        Err( e ) => exit(e, &mut state, &mut memory),
    };

    let unifiers = match unify( exp_val, ArenaRc::clone(&data[7]), &mut state, &mut memory, true) {
        Ok( val ) => val,
        Err( e ) => exit(e, &mut state, &mut memory),
    };

    declare_unifiers( &unifiers, &mut state, &mut memory);

    /**********************************************************************************************************************
    **********************************************************************************************************************/
    // let sum = 0.0.
    set_lineinfo(  ArenaRc::clone(&data[6]) ,  &mut state, &mut memory ); 

    let exp_val = match walk( ArenaRc::clone(&data[3]), &mut state, &mut memory ) {
        Ok( val ) => val,
        Err( e ) => exit(e, &mut state, &mut memory),
    };

    let unifiers = match unify( exp_val, ArenaRc::clone(&data[8]), &mut state, &mut memory, true) {
        Ok( val ) => val,
        Err( e ) => exit(e, &mut state, &mut memory),
    };

    declare_unifiers( &unifiers, &mut state, &mut memory);

    /**********************************************************************************************************************
    **********************************************************************************************************************/
    // let flip = 1.0.
    set_lineinfo(  ArenaRc::clone(&data[7]) ,  &mut state, &mut memory ); 

    let exp_val = match walk( ArenaRc::clone(&data[4]), &mut state, &mut memory ) {
        Ok( val ) => val,
        Err( e ) => exit(e, &mut state, &mut memory),
    };

    let unifiers = match unify( exp_val, ArenaRc::clone(&data[9]), &mut state, &mut memory, true) {
        Ok( val ) => val,
        Err( e ) => exit(e, &mut state, &mut memory),
    };

    declare_unifiers( &unifiers, &mut state, &mut memory);

    /**********************************************************************************************************************
    **********************************************************************************************************************/
    // for i in 1 to n-1 do
    set_lineinfo(  ArenaRc::clone(&data[18]) ,  &mut state, &mut memory ); 

    let exp_val = match walk( ArenaRc::clone(&data[17]), &mut state, &mut memory ) {
        Ok( val ) => val,
        Err( e ) => exit(e, &mut state, &mut memory),
    };

    let content = match *exp_val {
        Node::AstroList(AstroList{ contents:ref content }) => content,
        _ => return (),
    };

    for term in &*content.borrow() {   

        // init i inside for loop
        set_lineinfo(  ArenaRc::clone(&data[24]),  &mut state, &mut memory ); 

        let unifiers = match unify( ArenaRc::clone(&term), ArenaRc::clone(&data[25]),  &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => return (),
        };

        declare_unifiers( &unifiers,  &mut state, &mut memory);

        /**********************************************************************************************************************
        **********************************************************************************************************************/
        // let flip = flip * -1.0.
        set_lineinfo(  ArenaRc::clone(&data[28]) ,  &mut state, &mut memory ); 

        let exp_val = match walk( ArenaRc::clone(&data[23]), &mut state, &mut memory ) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&data[21]), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        declare_unifiers( &unifiers, &mut state, &mut memory);

        /**********************************************************************************************************************
        **********************************************************************************************************************/
        // let sum = sum + flip / (2.0 * toreal(i) - 1.0).
        set_lineinfo(  ArenaRc::clone(&data[44]) ,  &mut state, &mut memory ); 

        let exp_val = match walk( ArenaRc::clone(&data[43]), &mut state, &mut memory ) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };

        let unifiers = match unify( exp_val, ArenaRc::clone(&data[45]), &mut state, &mut memory, true) {
            Ok( val ) => val,
            Err( e ) => exit(e, &mut state, &mut memory),
        };
 
        declare_unifiers( &unifiers, &mut state, &mut memory);

    } // end for loop

    /**********************************************************************************************************************
    **********************************************************************************************************************/
    // sum * 4.0
    let exp_val = match walk( ArenaRc::clone(&data[49]), &mut state, &mut memory ) {
        Ok( val ) => val,
        Err( e ) => exit(e, &mut state, &mut memory),
    };

    // extract value to print
    let Node::AstroReal(AstroReal{value:v}) = *exp_val
        else {panic!("test failed")};

    // io: print to console out
    println!("{:.9}",v);
    
    return (); // return none / successful Rust program exit.
}
