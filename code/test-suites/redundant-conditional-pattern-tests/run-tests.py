#!/usr/bin/env python
# coding: utf-8

# # Regression Tests


import sys
import os

# uncomment if tim
#sys.path[0] ='/Users/Timothy/Documents/GitHub/asteroid/code'

# uncomment if running on SageMaker
#sys.path[0] = '/home/ec2-user/SageMaker/asteroid/code'
sys.path[0] = '/Users/lutz/Dropbox/URI/Projects/Asteroid/asteroid-git/code'
from asteroid_interp import interp

programs = os.listdir("programs_w_redundancies")
programs.sort()
print("******************************************************")
print("******************************************************")
print("Programs with redundant patterns:")
for pname in programs:
    f = open("programs_w_redundancies/"+pname,"r")
    p = f.read()
    print("**********"+pname+"************")
    print(p)
    print("**********output***********")
    interp(p)
    f.close()

programs = os.listdir("programs_wo_redundancies")
programs.sort()
print("******************************************************")
print("******************************************************")
print("Programs withOUT redundant patterns:")
for pname in programs:
    f = open("programs_wo_redundancies/"+pname,"r")
    p = f.read()
    print("**********"+pname+"************")
    print(p)
    print("**********output***********")
    interp(p)
    f.close()
