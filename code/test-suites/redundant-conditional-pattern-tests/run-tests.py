#!/usr/bin/env python
# coding: utf-8

# # Regression Tests


import sys
import os

sys.path[0] = '/Users/Timothy/Documents/GitHub/asteroid/code'


# uncomment if running on SageMaker
#sys.path[0] = '/home/ec2-user/SageMaker/asteroid/code'
#sys.path[0] = '/Users/lutz/Dropbox/URI/Projects/Asteroid/asteroid-git/code'
from asteroid_interp import interp
print("*******************************************************")
print("************PROGRAMS*WITH*REDUNDANT*PATTERNS***********")
print("*******************************************************")
programs = os.listdir("programs_w_redundancies")
programs.sort()

for pname in programs:
    f = open("programs_w_redundancies/"+pname,"r")
    p = f.read()
    print("**********"+pname+"************")
    print(p)
    print("**********output***********")
    try:
        interp(p)
    except:
        pass
    f.close()
print("*******************************************************")
print("************PROGRAMS*WITHOUT*REDUNDANT*PATTERNS********")
print("*******************************************************")
programs = os.listdir("programs_wo_redundancies")
programs.sort()

for pname in programs:
    f = open("programs_wo_redundancies/"+pname,"r")
    p = f.read()
    print("**********"+pname+"************")
    print(p)
    print("**********output***********")
    try:
        interp(p)
    except:
        pass
    f.close()
