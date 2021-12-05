#!/usr/bin/env python3
# coding: utf-8

# # Regression Tests


import sys
import os

# uncomment if running on SageMaker
#sys.path[0] = '/home/ec2-user/SageMaker/asteroid/code'
sys.path[0] = '/Users/lutz/Dropbox/URI/Projects/Asteroid/asteroid-git/code'
from asteroid.asteroid_interp import interp

programs = os.listdir("programs")
programs.sort()

for pname in programs:
    f = open("programs/"+pname,"r")
    p = f.read()
    print("**********"+pname+"************")
    print(p)
    print("**********output***********")
    interp(p, exceptions=True)
    f.close()
