#!/usr/bin/env python3
import json
import sys

# a heathen script to paste the output into Rust.
# does min/max work with assembly_vars.json??


def eprint(*args, **kwargs):
    """Print to stderr.
    Args:
        *args: arguments to print.
        **kwargs: keyword arguments to print.
    Notes:
        A thin wrapper of print() that prints to stderr.
        See https://stackoverflow.com/questions/5574702/how-to-print-to-stderr-in-python.
    """
    print(*args, file=sys.stderr, **kwargs)


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
    elif string == "None":
        eprint(f"{string} not found!")
        return "None"


cli_json_file = sys.argv[1]

with open(cli_json_file, "r") as json_file:
    data = json.load(json_file)
    for field in data["fields"]:
        name = data["fields"][field]["name"]
        # print(name)
        try:
            display_name = data["fields"][field]["display_name"]
        except KeyError:
            display_name = name
        # print(display_name)
        try:
            type_ = data["fields"][field]["type"]
        except KeyError:
            type_ = "None"
        type_of = type_to_type_of(type_)

        # more functions may be supported in the future
        # but only min/max for now.
        fun = "Function::None"
        try:
            functions = data["fields"][field]["summary"]
            if type(functions) is list:
                if all(el in functions for el in ["min", "max"]):
                    fun = 'Function::Some(vec!["min", "max"])'
        except KeyError:
            fun = "Function::None"

        # print format
        # "mitochondrion_assembly_span" => Variable {display_name: "mitochondrion span", type_of: TypeOf::Long},
        if type_ == "keyword":
            try:
                enum_list = data["fields"][field]["constraint"]["enum"]
            except KeyError:
                enum_list = []
            enum_list_str = '", "'.join(enum_list)
            print(
                f'"{name}" => Variable {{ display_name: "{display_name}", type_of: TypeOf::{type_of}(vec!["{enum_list_str}"]), functions: {fun} }},'
            )
        else:
            print(
                f'"{name}" => Variable {{ display_name: "{display_name}", type_of: TypeOf::{type_of}, functions: {fun} }},'
            )
