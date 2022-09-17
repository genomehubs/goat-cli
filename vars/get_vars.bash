#!/usr/bin/env bash

# fetch data from goat api
rm taxon_vars.json
rm assembly_vars.json

# TAXON INDEX DATA

# https://goat.genomehubs.org/api-docs/#/GoaT%20API/getResultFields
curl -X 'GET' \
'https://goat.genomehubs.org/api/v2/resultFields?result=taxon&taxonomy=ncbi' \
-H 'accept: application/json' | python3 -m json.tool > taxon_vars.json 2> /dev/null

# parse this data to Rust
python3 vars_to_string.py taxon_vars.json > goat_taxon_variable_data.txt

# replace the variable_data.rs
# add two tabs to start of each line
sed -i -e 's/^/\t\t/' goat_taxon_variable_data.txt

# remove in place the variables
sed '/\/\/ automated input start taxon/,/\/\/ automated input end taxon/{//!d;}' ../src/utils/variable_data.rs > ./temp.rs

# insert the new ones
# get line first
LINE=$(grep -n '\/\/ automated input start taxon' ./temp.rs | cut -d ":" -f 1)
# make new file
{ head -n $LINE ./temp.rs; cat goat_taxon_variable_data.txt; tail -n +$(($LINE+1)) ./temp.rs; } > temp2.rs

# clean up
rm ../src/utils/variable_data.rs
mv temp2.rs ../src/utils/variable_data.rs
rm ./temp.rs

# ASSEMBLY INDEX DATA

curl -X 'GET' \
'https://goat.genomehubs.org/api/v2/resultFields?result=assembly&taxonomy=ncbi' \
-H 'accept: application/json' | python3 -m json.tool > assembly_vars.json 2> /dev/null

python3 vars_to_string.py assembly_vars.json > goat_assembly_variable_data.txt

sed -i -e 's/^/\t\t/' goat_assembly_variable_data.txt

sed '/\/\/ automated input start assembly/,/\/\/ automated input end assembly/{//!d;}' ../src/utils/variable_data.rs > ./temp.rs

LINE=$(grep -n '\/\/ automated input start assembly' ./temp.rs | cut -d ":" -f 1)
# make new file
{ head -n $LINE ./temp.rs; cat goat_assembly_variable_data.txt; tail -n +$(($LINE+1)) ./temp.rs; } > temp2.rs

# clean up
rm ../src/utils/variable_data.rs
mv temp2.rs ../src/utils/variable_data.rs
rm ./temp.rs

# some sed strangeness here is generating extra files with -e as extensions?
rm *-e