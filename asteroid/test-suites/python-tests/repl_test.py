import sys
import os

#Code from run-tests.py
file_path = os.path.dirname(os.path.abspath( __file__ ))
os.chdir(file_path)
(parent_dir,_) = os.path.split(file_path)
#We need the granparent dir, since we're 2 layers down from asteroid
(grandparent_dir,_) = os.path.split(parent_dir)
sys.path.append(grandparent_dir)

from asteroid.interp import interp
import asteroid.lex as lex
from asteroid.state import state

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
    print(lex.get_indentifiers_by_prefix("__"))
    

if __name__ == "__main__":
    print("Testing get_identifiers:")
    test_get_identifiers()
    print("\n\n\n")
    print("Testing get_identifiers_by_prefix")
    test_get_identifiers_by_prefix()