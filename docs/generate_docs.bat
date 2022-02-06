#!/bin/bash

echo "generating User Guide.rst"
cpp -w -P "User Guide.txt" > "User Guide.rst"

echo "generating Reference Guide.rst"
cpp -w -P "Reference Guide.txt" > "Reference Guide.rst"
