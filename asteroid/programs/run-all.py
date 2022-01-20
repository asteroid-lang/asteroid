#!/usr/bin/env python
# coding: utf-8

# run all program


import sys
import os
import glob

# Get the path of this run-tests.py file
file_path = os.path.dirname(os.path.abspath( __file__ ))

# Temporarly change the working directory to that path
os.chdir(file_path)

# Set the path to the code directory
sys.path[0] = "../"

from asteroid.interp import interp

programs = glob.glob("*.ast")
programs.sort()

for pname in programs:
    f = open(pname,"r")
    p = f.read()
    print("**********"+pname+"************")
    print(p)
    print("**********output***********")
    interp(p)
    f.close()
