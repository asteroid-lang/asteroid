#!/usr/bin/env python3
# coding: utf-8
################################
# general testsuite driver
################################

# the following is a list of directories of test cases

dirs = [
	    'action-tests',
        'programs',
        'redundant-pattern-test',
        'ref-programs',
        'regression-tests',
        'ug-programs',
       ]

# the exclusion list allows you to eliminate certain tests from the 
# current run
# TODO: make names test suite sensitive
#exclusion_list = ['test015.ast']
exclusion_list = []
if exclusion_list:
    print("Exclusion list: {}".format(exclusion_list))

# set the following to True if you encounter failed test cases. It will
# give you details and a stack dump.
verbose_failure = False

# control whether to do redundancy checks
redundancy = True

# if your test case needs input from stdin please provide
# a file named,
#
#    <testname>-io.txt
#
# For example, if your test case file is test-024.ast and it
# requires input on stdin then provide a file test-024-io.txt
# with one line of input for each require input.

# TODO: capture stdout and compare it to a given
# output file.

import sys
import os

# Get the path of this file and temporarily change
# the working directory to that path
file_path = os.path.dirname(os.path.abspath( __file__ ))
os.chdir(file_path)
# append the parent dir of our current dir to the PYTHONPATH
# so that we can easily find our Asteroid modules
(parent_dir,_) = os.path.split(file_path)
sys.path.append(parent_dir)
from interp import interp

n_tests = 0
for d in dirs:
    # assemble the test list
    test_list = os.listdir(d)
    test_list = list(set(test_list) - set(exclusion_list))
    test_list.sort()
    n_tests += len(test_list)
    # set the ASTEROIDPATH environment variable
    old_val = os.getenv('ASTEROIDPATH')
    os.environ['ASTEROIDPATH'] = d

    for testname in test_list:
        # check that we are actually looking at test case
        if testname[-3:] == "ast":
            # if a <testname>-io.txt file exists map stdin to it
            stdin_file = file_path+'/'+d+'/'+testname[0:-4]+"-io.txt"
            if os.path.exists(stdin_file):
                sys.stdin = open(stdin_file, "r")
            f = open(d+"/"+testname,"r")
            p = f.read()
            print("**********"+d+"/"+testname+"************")
            print(p)
            print("**********output***********")
            interp(p,
                   exceptions=verbose_failure,
                   redundancy=redundancy)
            f.close()
    if old_val:
        os.environ['ASTEROIDPATH'] = old_val
    else:
        del os.environ['ASTEROIDPATH']
        
print("Number of test cases:",n_tests)
