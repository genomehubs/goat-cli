#!/usr/bin/env bash

rm vars.json

# https://goat.genomehubs.org/api-docs/#/GoaT%20API/getResultFields
curl -X 'GET' \
'https://goat.genomehubs.org/api/v0.0.1/resultFields?result=taxon&taxonomy=ncbi' \
-H 'accept: application/json' > vars.json 2> /dev/null

python3 vars_to_string.py > goat_variable_data.txt
# add two tabs to start of each line
sed -i 's/^/\t\t/' goat_variable_data.txt

# remove in place the variables
sed '/\/\/ automated input start/,/\/\/ automated input end/{//!d}' ../src/utils/variable_data.rs > ./temp.rs

# insert the new ones
# get line first
LINE=$(grep -n '\/\/ automated input start' ./temp.rs | cut -d ":" -f 1)
# make new file
{ head -n $LINE ./temp.rs; cat goat_variable_data.txt; tail -n +$(($LINE+1)) ./temp.rs; } > temp2.rs

rm ../src/utils/variable_data.rs
mv temp2.rs ../src/utils/variable_data.rs
