'''
This python script generates the RST files for our documentation.

NOTE: make sure that you have PCPP installed on your system before attempting
to generate documentation,

      pip install pcpp
'''

import subprocess

try:
   print("generating User Guide.rst")
   subprocess.run(["pcpp","--passthru-comments","--line-directive","nothing","-o","User Guide.rst","User Guide.txt"])

   print("generating Reference Guide.rst")
   subprocess.run(["pcpp","--passthru-comments","--line-directive","nothing","-o","Reference Guide.rst","Reference Guide.txt"])

   print("generating Asteroid in Action.rst")
   subprocess.run(["pcpp","--passthru-comments","--line-directive","nothing","-o","Asteroid in Action.rst","Asteroid in Action.txt"])

   print("generating Quickstart Tutorial.rst")
   subprocess.run(["pcpp","--passthru-comments","--line-directive","nothing","-o","Quickstart Tutorial.rst","Quickstart Tutorial.txt"])

   print("generating MAD.rst")
   subprocess.run(["pcpp","--passthru-comments","--line-directive","nothing","-o","MAD.rst","MAD.txt"])
except FileNotFoundError as e:
   print(str(e))
   exit(1)
