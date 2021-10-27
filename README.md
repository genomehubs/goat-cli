# GoaT

GoaT's on a Terminal. Genomes on a Terminal?

Concurrently query the GoaT API, and receive tabular data back. In progress and unstable. Documentation to come.

## Usage

```
goat-search 
Query the search API.

USAGE:
    goat search [FLAGS] [OPTIONS] --file <file> --tax-id <tax-id>

FLAGS:
    -z, --all            This flag indicates all data should be printed.
    -a, --assembly       This flag indicates assembly data should be printed.
    -b, --busco          Include BUSCO estimates?
    -c, --c-values       This flag indicates C-value data should be printed.
    -g, --genome-size    This flag indicates genome size data should be printed.
    -h, --help           Prints help information
    -k, --karyotype      This flag indicates karyotype data should be printed.
    -r, --raw            This flag indicates raw values should be all listed out. So you can do your own aggregations
                         for example.
    -p, --tree           Get information for all descendents of a common ancestor.
    -u, --url            Print the underlying GoaT API URL. Nice to know, and useful for debugging.
    -V, --version        Prints version information

OPTIONS:
    -f, --file <file>        A file of NCBI taxonomy ID's (tips) and/or binomial names.
                             Each line should contain a single entry.
        --size <size>        The number of results to return. [default: 50]
    -t, --tax-id <tax-id>    The tax-id. Can be NCBI taxonomy ID, or a binomial name.
```

### Examples

To get all of the (basic) raw assembly statistics for <i>Arabidopsis thaliana</i>:

`goat search -ra -t "Arabidopsis thaliana"`

To get aggregated assembly statistics across all <i>Arabidopsis</i> in NCBI:

`goat search -pa -t "Arabidopsis"`