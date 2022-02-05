#!/bin/bash

echo "generating Asteroid User Guide.rst"
cpp -w -P "Asteroid User Guide.txt" > "Asteroid User Guide.rst"
