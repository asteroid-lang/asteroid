#!/usr/bin/env python3
# coding: utf-8

# # Regression Tests


import sys
import os

# Get the path of this run-tests.py file
file_path = os.path.dirname(os.path.abspath( __file__ ))

# Temporarly change the working directory to that path
os.chdir(file_path)

from asteroid.interp import interp

dirs = ['ug-programs']

for d in dirs:
    programs = os.listdir(d)
    programs.sort()

    for pname in programs:
        f = open(d+"/"+pname,"r")
        p = f.read()
        print("**********"+d+"/"+pname+"************")
        print(p)
        print("**********output***********")
        interp(p)
        f.close()
