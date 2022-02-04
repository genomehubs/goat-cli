# GoaT variables

This is mainly a developer note. Run `bash get_vars.bash` to retrieve latest GoaT variables.

As variables are being continually added to GoaT, they need to be updated. The CLI has to add these in manually, as they will need additional CLI flags when the time comes. The expression option in the CLI however needs a database to compare against, to prevent against no-hits from the GoaT API.

Therefore the script in this dir fetches the latest variables and parses them into a Rust formatted structure. This is currently pasted into the code, but in future there might be a better way of handling this.

The current implementation uses the following data structures:

```Rust
// data type associated with each variable
enum TypeOf<'a> {
    Long,
    Short,
    OneDP,
    TwoDP,
    Integer,
    Date,
    HalfFloat,
    Keyword(Vec<&'a str>),
}

// An alias for option
// this is for detecting min()/max()
// and any other expression functions in the future
enum Function<'a> {
    None,
    Some(Vec<&'a str>),
}

// each entry in the database.
struct Variable<'a> {
    display_name: &'a str,
    type_of: TypeOf<'a>,
    functions: Function<'a>,
}

```

The file `goat_variable_data.txt` formats the text from the JSON into the above structure.