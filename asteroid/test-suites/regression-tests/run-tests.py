#!/usr/bin/env python3
# coding: utf-8

# # Regression Tests


import sys
import os

# Get the path of this run-tests.py file
file_path = os.path.dirname(os.path.abspath( __file__ ))

# Temporarly change the working directory to that path
os.chdir(file_path)

# Set the path to two levels above (the code directory)
sys.path[0] = "../../"

from interp import interp

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
