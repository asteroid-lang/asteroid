from asteroid.interp import interp
import lex
from state import state

def test_get_identifiers():
    print(lex.get_indentifiers())
    interp("load io.", "test")
    print(lex.get_indentifiers())
    interp("let aaa = 1.", "test")
    interp("let aab = 2.", "test", initialize_state=False)
    interp("let abc = 3.", "test", initialize_state=False)
    interp("let bc = 4.", "test", initialize_state=False)
    interp("let baa = 5.", "test", initialize_state=False)
    print(lex.get_indentifiers())
    
def test_get_identifiers_by_prefix():
    print(lex.get_indentifiers())
    interp("load io.", "test")
    print(lex.get_indentifiers())
    interp("let aaa = 1.", "test")
    interp("let aab = 2.", "test", initialize_state=False)
    interp("let abc = 3.", "test", initialize_state=False)
    interp("let bc = 4.", "test", initialize_state=False)
    interp("let baa = 5.", "test", initialize_state=False)
    print(lex.get_indentifiers())
    print(lex.get_indentifiers_by_prefix("a"))
    print(lex.get_indentifiers_by_prefix("aa"))
    

if __name__ == "__main__":
    print("Testing get_identifiers:")
    test_get_identifiers()
    print("\n\n\n")
    print("Testing get_identifiers_by_prefix")
    test_get_identifiers_by_prefix()