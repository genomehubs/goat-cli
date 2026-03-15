# GoaT variables

This directory holds snapshots of the GoaT `resultFields` API endpoint, used at
compile time to generate the field validation map in `src/utils/variable_data.rs`.

## Updating

Run `get_vars.bash` from anywhere in the repo, then rebuild:

```bash
./vars/get_vars.bash
cargo build
```

`get_vars.bash` fetches fresh JSON from:
- `GET /api/v2/resultFields?result=taxon&taxonomy=ncbi` → `taxon_vars.json`
- `GET /api/v2/resultFields?result=assembly&taxonomy=ncbi` → `assembly_vars.json`

`cargo build` then runs `build.rs`, which reads those JSON files and generates
the `GOAT_TAXON_VARIABLE_DATA` and `GOAT_ASSEMBLY_VARIABLE_DATA` maps. No manual
editing of Rust code is required.

## Runtime supplement

Fields added to GoaT after the last `get_vars.bash` run are still accepted at
runtime via the dynamic field registry (`src/utils/field_registry.rs`), which
fetches live field names from the same endpoint at startup.

## Data structures

Each field in the JSON is mapped to:

```rust
struct Variable<'a> {
    display_name: &'a str,
    type_of: TypeOf<'a>,   // Long | Short | Integer | Date | HalfFloat | OneDP | TwoDP | Keyword(Vec<&'a str>)
    functions: Function<'a>, // None | Some(Vec<&'a str>)  e.g. Some(vec!["min", "max"])
}
```

Type mapping from JSON `type` field:

| JSON type       | `TypeOf` variant        |
|-----------------|-------------------------|
| `long`          | `Long`                  |
| `integer`       | `Integer`               |
| `short`         | `Short`                 |
| `date`          | `Date`                  |
| `half_float`    | `HalfFloat`             |
| `1dp`           | `OneDP`                 |
| `2dp` / `4dp`   | `TwoDP`                 |
| `keyword`       | `Keyword(enum values)`  |
| null / other    | `Keyword(vec![""])`     |
