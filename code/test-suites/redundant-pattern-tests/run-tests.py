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

programs = os.listdir("programs")
programs.sort()

for pname in programs:
    f = open("programs/"+pname,"r")
    p = f.read()
    print("**********"+pname+"************")
    print(p)
    print("**********output***********")
    interp(p)
    f.close()
