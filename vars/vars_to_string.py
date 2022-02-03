#!/usr/bin/env python3
import json

# a heathen script to paste the output into Rust.

def type_to_type_of(string):
    if string == "long":
        return "Long"
    elif string == "short":
        return "Short"
    elif string == "1dp":
        return "OneDP"
    elif string == "2dp":
        return "TwoDP"
    elif string == "integer":
        return "Integer"
    elif string == "date":
        return "Date"
    elif string == "half_float":
        return "HalfFloat"
    elif string == "keyword":
        return "Keyword"

with open("vars.json", "r") as json_file:
    data = json.load(json_file)
    for field in data["fields"]:
        try:
            display_name = data["fields"][field]["display_name"]
            name = data["fields"][field]["name"]
            type_ = data["fields"][field]["type"]
            type_of = type_to_type_of(type_)

            # more functions may be supported in the future
            # but only min/max for now.
            fun = "Function::None"
            
            functions = data["fields"][field]["summary"]
            if type(functions) is list:
                if all(el in functions for el in ['min', 'max']):
                    fun = "Function::Some(vec![\"min\", \"max\"])"
    
            # print format
            # "mitochondrion_assembly_span" => Variable {display_name: "mitochondrion span", type_of: TypeOf::Long}, 
            if type_ == "keyword":
                enum_list = data["fields"][field]["constraint"]["enum"]
                enum_list_str = "\", \"".join(enum_list)
                print(f"\"{name}\" => Variable {{ display_name: \"{display_name}\", type_of: TypeOf::{type_of}(vec![\"{enum_list_str}\"]), functions: {fun} }},")
            else:
                print(f"\"{name}\" => Variable {{ display_name: \"{display_name}\", type_of: TypeOf::{type_of}, functions: {fun} }},")
        except KeyError:
            pass