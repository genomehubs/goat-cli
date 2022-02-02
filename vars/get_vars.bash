#!/usr/bin/env bash

rm vars.json

# https://goat.genomehubs.org/api-docs/#/GoaT%20API/getResultFields
curl -X 'GET' \
'https://goat.genomehubs.org/api/v0.0.1/resultFields?result=taxon&taxonomy=ncbi' \
-H 'accept: application/json' > vars.json 2> /dev/null

python3 vars_to_string.py > goat_variable_data.txt