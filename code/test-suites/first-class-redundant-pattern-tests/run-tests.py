#!/usr/bin/env python
# coding: utf-8

# # Regression Tests


import sys
import os

# uncomment if tim
# sys.path[0] ='/Users/Timothy/Documents/GitHub/asteroid/code'

# uncomment if running on SageMaker
#sys.path[0] = '/home/ec2-user/SageMaker/asteroid/code'
#sys.path[0] = '/Users/lutz/Dropbox/URI/Projects/Asteroid/asteroid-git/code'

# Get the path of this run-tests.py file
file_path = os.path.dirname(os.path.abspath( __file__ ))

# Temporarly change the working directory to that path
os.chdir(file_path)

# Set the path to two levels above (the code directory)
sys.path[0] = "../../"

from asteroid_interp import interp
from asteroid_globals import eval_redundancy

eval_redundancy[0] = True

programs = os.listdir("programs")
programs.sort()

for pname in programs:
    f = open("programs/"+pname,"r")
    p = f.read()
    print("**********"+pname+"*************")
    #print(p)
    #print("********************output*********************")
    try:
        interp(p,exceptions=True)
    except:
        pass

    f.close()
